#!/usr/bin/env python3
"""Verify selected/typed file ingress and retained failures on owned PTYs."""

import os
from pathlib import Path
import runpy
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
NAV = runpy.run_path(str(ROOT / "scripts/check-path-navigation.py"))
PTY = NAV["PTY"]
step = NAV["step"]


def visit(session, draft, marker):
    step(session, b"\x18\x06", b"Find file: ")
    step(session, PTY["paste_wire"](draft.encode()), draft.encode())
    return step(session, b"\r", marker)


def directories(session):
    step(session, b"\x18\x06\x10\r", b"Find file: child/")
    step(session, b"\x10\r", b"> nested.txt -- RW L1 C1 Text")
    visit(session, "../source.txt", b"> source.txt -- RW L1 C1 Text")
    step(session, b"\x1bxfind-file\r", b"Find file: ")
    step(session, b"\x0e\x0e\r", b"> alpha.txt -- RW L1 C1 Text")
    visit(session, "source.txt", b"> source.txt -- RW L1 C1 Text")
    visit(session, "link", b"Find file: link/")
    step(session, b"\x10\r", b"> nested.txt -- RW L1 C1 Text")
    visit(session, "../source.txt", b"> source.txt -- RW L1 C1 Text")


def new_file(session, work):
    visit(session, "al", b"> al -- RW L1 C1 Text")
    assert not (work / "al").exists()
    step(session, b"x\r", b"> al ** RW L2 C1 Text")
    assert not (work / "al").exists()
    step(session, b"\x18u\x18u", b"> al -- RW L1 C1 Text")
    visit(session, "source.txt", b"> source.txt -- RW L1 C1 Text")


def failures(session):
    visit(session, "alpha.txt/child", b'Find file: alpha.txt/child  visit "')
    frame = step(session, b"x", b"Find file: alpha.txt/childx")
    assert b"> source.txt -- RW L1 C1 Text" in frame
    step(session, b"\x07", b"Quit")
    step(session, b"\x18\x06\r", b"empty file path")
    frame = step(session, b"z", b"Find file: z")
    assert b"> source.txt -- RW L1 C1 Text" in frame
    step(session, b"\x07", b"Quit")


def dirty_alias(session):
    step(session, b"!", b"> source.txt ** RW L1 C2 Text")
    frame = visit(session, "alias.txt", b"> source.txt ** RW L1 C2 Text")
    assert b"!retained" in frame
    step(session, b"\x18u", b"> source.txt -- RW L1 C1 Text")


def case(root, colored):
    work = NAV["fixtures"](root)
    (work / "alias.txt").symlink_to("source.txt")
    (work / "link").symlink_to("child")
    files = [work / name for name in ["source.txt", "alpha.txt", "alpine.txt", "child/nested.txt"]]
    files.append(root / "home/space dir/界 name.txt")
    original = {path: path.read_bytes() for path in files}
    entries = set(work.rglob("*"))
    session = PTY["Session"]([str(ROOT / "target/debug/rev"), str(work / "source.txt")], env=NAV["LIST"]["environment"](root, colored))
    try:
        session.ready(b"> source.txt -- RW L1 C1 Text")
        directories(session)
        new_file(session, work)
        failures(session)
        dirty_alias(session)
        visit(session, "~/space dir/界 name.txt", "name.txt -- RW L1 C1 Text".encode())
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert PTY["PASTE_OFF"] in session.output and PTY["LEAVE"] in session.output
        assert {path: path.read_bytes() for path in files} == original
        assert set(work.rglob("*")) == entries, "find-file created a filesystem entry"
    finally:
        session.close()
    print(f"PASS selected/typed Return, directory/symlink descent, new in-memory names, retained failures, dirty alias reuse and {'RGB' if colored else 'NO_COLOR'} restoration")


def main():
    subprocess.run(["cargo", "build", "--bin", "rev"], cwd=ROOT, check=True)
    for colored in [True, False]:
        with tempfile.TemporaryDirectory(prefix="rev-path-acceptance-") as directory:
            case(Path(directory), colored)
    print("Path-acceptance PTYs: 2 sessions passed, 0 skipped.")


if __name__ == "__main__":
    main()
