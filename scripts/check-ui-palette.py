#!/usr/bin/env python3
"""Verify palette lifetime and capability fallback through owned Linux PTYs."""

import os
from pathlib import Path
import runpy
import subprocess
import tempfile
import termios


ROOT = Path(__file__).resolve().parent.parent
PTY = runpy.run_path(str(ROOT / "scripts/check-pty.py"))
SET = b"\x1b]12;#012345\x07"
RESET = b"\x1b]112\x07"


def restoration(mode, expected):
    session = PTY["Session"]([str(ROOT / "target/debug/examples/terminal-probe"), mode], colors=True)
    try:
        session.ready(b"probe-ready")
        os.write(session.master, b"!")
        if mode == "restore":
            session.wait(lambda: b"probe-cleanup-pending" in session.output)
            assert termios.tcgetattr(session.slave) == session.before
            os.write(session.master, b"!\n")
        session.finish(expected)
        wire = bytes(session.output)
        assert SET in wire and RESET in wire, wire
        if mode != "color-set":
            assert b"\x1b[38;2;1;35;69m" in wire and b"\x1b[48;2;67;89;101m" in wire
        assert b"\x1b[0m" in wire and PTY["PASTE_OFF"] in wire and PTY["SHOW"] in wire
        assert wire.index(SET) < wire.index(RESET) < wire.rfind(PTY["LEAVE"])
        if mode == "restore":
            assert b"reset cursor color" in wire
            assert b"reset display attributes" in wire
            assert b"disable bracketed paste" in wire
        if mode == "color-stop":
            assert b"reset cursor color" in wire and wire.count(RESET) >= 2
        print(f"PASS cursor/text restoration: {mode}")
    finally:
        session.close()


def fallback():
    for name, environment in [("unknown TERM", {"TERM": "dumb"}), ("NO_COLOR", {"NO_COLOR": "1"})]:
        session = PTY["Session"]([str(ROOT / "target/debug/examples/terminal-probe"), "normal"], colors=True, env=environment)
        try:
            session.ready(b"probe-ready")
            os.write(session.master, b"!")
            session.finish(0)
            assert SET not in session.output and RESET not in session.output
            print(f"PASS cursor fallback: {name}")
        finally:
            session.close()


def monochrome():
    with tempfile.TemporaryDirectory(prefix="rev-palette-pty-") as directory:
        path = Path(directory) / "main.rs"
        path.write_bytes(b"fn main() {}")
        session = PTY["Session"]([str(ROOT / "target/debug/rev"), str(path)], env={"NO_COLOR": "1"})
        try:
            session.ready(b"> main.rs -- RW L1 C1 Rust")
            PTY["group_step"](session, b"\x00\x06\x06", b"L1 C3 Rust", b"\x1b[7mfn")
            assert b"\x1b[38;2;" not in session.output and b"\x1b]12;" not in session.output
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert path.read_bytes() == b"fn main() {}"
            print("PASS monochrome selection and source retention")
        finally:
            session.close()


def application():
    with tempfile.TemporaryDirectory(prefix="rev-palette-app-") as directory:
        path = Path(directory) / "plain.txt"
        path.write_bytes(b"plain\n")
        session = PTY["Session"]([str(ROOT / "target/debug/rev"), str(path)], colors=True)
        try:
            session.ready(b"> plain.txt -- RW L1 C1 Text")
            wire = bytes(session.output)
            assert b"\x1b]12;#c0caf5\x07" in wire
            assert b"\x1b[48;2;26;27;38m" in wire and b"\x1b[38;2;192;202;245mplain" in wire
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            wire = bytes(session.output)
            assert wire.index(RESET) < wire.rfind(PTY["LEAVE"])
            assert path.read_bytes() == b"plain\n"
            print("PASS executable UI/cursor defaults and ordinary restoration")
        finally:
            session.close()


def main():
    subprocess.run(["cargo", "build", "--bin", "rev", "--example", "terminal-probe"], cwd=ROOT, check=True)
    for mode, expected in [("normal", 0), ("entered", 1), ("io", 1), ("unwind", 101), ("color-set", 1), ("color-stop", 1), ("attributes-stop", 1), ("restore", 1)]:
        restoration(mode, expected)
    fallback()
    monochrome()
    application()
    print("UI palette PTYs: 12 cases passed, 0 skipped.")


if __name__ == "__main__":
    main()
