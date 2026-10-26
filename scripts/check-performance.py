#!/usr/bin/env python3
"""Measure deterministic large UTF-8 documents with native owners and real PTYs."""

import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import select
import subprocess
import sys
import tempfile
import time


WORKLOADS = {"load", "search", "insert", "delete", "scroll", "split_redraw", "highlighting", "syntax_work"}


def fixture(path, lines):
    digest = hashlib.sha256()
    total = 0
    with path.open("wb") as output:
        output.write(b"\xef\xbb\xbf")
        digest.update(b"\xef\xbb\xbf")
        total += 3
        for index in range(lines):
            tail = "x" * (512 if index % 251 == 0 else index % 43)
            ending = "" if index + 1 == lines else ("\r\n" if index % 7 == 0 else "\n")
            line = f'fn item_{index}() {{ let message = "界e\u0301🙂\t👩\u200d💻{tail}"; }}{ending}'.encode()
            output.write(line)
            digest.update(line)
            total += len(line)
    return {"lines": lines, "bytes": total, "sha256": digest.hexdigest()}


def hardware():
    model = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                  if line.startswith("model name")), "unknown")
    return {"system": platform.system(), "kernel": platform.release(), "architecture": platform.machine(),
            "cpu_model": model, "logical_cpus": os.cpu_count(),
            "memory_gib": round(os.sysconf("SC_PHYS_PAGES") * os.sysconf("SC_PAGE_SIZE") / 2**30, 2),
            "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
            "profile": "release", "terminal": {"columns": 80, "rows": 24}}


def native(root, path):
    result = subprocess.run([str(root / "target/release/examples/performance"), str(path)],
                            cwd=root, capture_output=True, text=True, timeout=120)
    if result.returncode:
        raise AssertionError(f"native measurement failed:\n{result.stderr[-4000:]}\n{result.stdout[-4000:]}")
    records = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(records) == len(WORKLOADS), "missing or duplicate native measurements"
    assert {record["workload"] for record in records} == WORKLOADS, "missing required workload"
    for record in records:
        assert record["samples"] > 0, record
        assert math.isfinite(record["median_us"]) and record["median_us"] > 0, record
        assert math.isfinite(record["max_us"]) and record["max_us"] >= record["median_us"], record
    return records


def cpu_ticks(process):
    fields = Path(f"/proc/{process.pid}/stat").read_text().rsplit(")", 1)[1].split()
    return int(fields[11]) + int(fields[12])


def idle(session):
    # This is a measured quiescent interval, not a fixed readiness sleep.
    session.output.clear()
    started = time.perf_counter()
    before = cpu_ticks(session.process)
    deadline = started + 0.35
    while time.perf_counter() < deadline:
        remaining = max(0, deadline - time.perf_counter())
        if select.select([session.master], [], [], remaining)[0]:
            session.drain()
    elapsed = time.perf_counter() - started
    ticks = cpu_ticks(session.process) - before
    frequency = os.sysconf("SC_CLK_TCK")
    assert session.process.poll() is None, "editor exited while idle"
    assert not session.output, "idle editor emitted repaint output"
    assert ticks <= max(2, math.ceil(elapsed * frequency * 0.1)), "sustained idle CPU work"
    return {"elapsed_ms": round(elapsed * 1000, 3), "output_bytes": len(session.output),
            "cpu_ms": round(ticks * 1000 / frequency, 3)}


def terminal(root, path, harness):
    start = time.perf_counter()
    session = harness.Session([str(root / "target/release/rev"), str(path)], colors=True)
    try:
        session.wait(lambda: harness.frame_contains(session.output, b"-- RW L1 C1 Rust", None))
        elapsed = (time.perf_counter() - start) * 1000
        flags = harness.termios.tcgetattr(session.slave)[3]
        assert not flags & (harness.termios.ICANON | harness.termios.ECHO), "raw mode missing"
        quiet = idle(session)
        harness.accepted_exit(session)
        return {"startup_ms": round(elapsed, 3), "idle": quiet}
    finally:
        session.close()


def main():
    root = Path(__file__).resolve().parent.parent
    subprocess.run(["cargo", "build", "--release", "--bin", "rev", "--example", "performance"],
                   cwd=root, check=True, timeout=180)
    sys.dont_write_bytecode = True
    specification = importlib.util.spec_from_file_location("rev_pty", root / "scripts/check-pty.py")
    harness = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(harness)
    result = {"hardware": hardware(), "fixtures": []}
    with tempfile.TemporaryDirectory(prefix="rev-performance-") as directory:
        for lines in [20_000, 40_000]:
            path = Path(directory) / f"mixed-{lines}.rs"
            record = fixture(path, lines)
            record["native"] = native(root, path)
            record["terminal"] = terminal(root, path, harness)
            assert path.stat().st_size == record["bytes"], "fixture size changed"
            assert hashlib.sha256(path.read_bytes()).hexdigest() == record["sha256"], "fixture bytes changed"
            result["fixtures"].append(record)
    print(json.dumps(result, indent=2))
    print("Performance workloads: 2 fixtures, 8 native workloads each, 2 real PTY startup/idle/restoration checks; passed.")


if __name__ == "__main__":
    main()
