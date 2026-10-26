#!/usr/bin/env python3
"""Measure deterministic large UTF-8 documents with native owners and real PTYs."""

import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time


WORKLOADS = {"load", "search", "insert", "delete", "scroll", "split_redraw", "highlighting"}


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
                            cwd=root, capture_output=True, text=True, timeout=120, check=True)
    records = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(records) == len(WORKLOADS), "missing or duplicate native measurements"
    assert {record["workload"] for record in records} == WORKLOADS, "missing required workload"
    for record in records:
        assert record["samples"] > 0, record
        assert math.isfinite(record["median_us"]) and record["median_us"] > 0, record
        assert math.isfinite(record["max_us"]) and record["max_us"] >= record["median_us"], record
    return records


def terminal(root, path, harness):
    start = time.perf_counter()
    session = harness.Session([str(root / "target/release/rev"), str(path)], colors=True)
    try:
        session.wait(lambda: harness.frame_contains(session.output, b"-- RW L1 C1 Rust", None))
        elapsed = (time.perf_counter() - start) * 1000
        flags = harness.termios.tcgetattr(session.slave)[3]
        assert not flags & (harness.termios.ICANON | harness.termios.ECHO), "raw mode missing"
        harness.accepted_exit(session)
        return {"startup_ms": round(elapsed, 3)}
    finally:
        session.close()


def main():
    root = Path(__file__).resolve().parent.parent
    subprocess.run(["cargo", "build", "--release", "--bin", "rev", "--example", "performance"],
                   cwd=root, check=True, timeout=180)
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
    print("Performance workloads: 2 fixtures, 7 native workloads each, 2 real PTY startups; passed.")


if __name__ == "__main__":
    main()
