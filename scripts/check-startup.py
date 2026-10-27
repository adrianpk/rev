#!/usr/bin/env python3
"""Verify startup configuration through isolated, owned controlling terminals."""

import os
from pathlib import Path
import runpy
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
PTY = runpy.run_path(str(ROOT / "scripts/check-pty.py"))
BINARY = ROOT / "target/debug/rev"


def environment(root):
    return {
        "HOME": str(root / "home"),
        "XDG_CONFIG_HOME": str(root / "user"),
        "XDG_DATA_HOME": str(root / "data"),
        "XDG_STATE_HOME": str(root / "state"),
        "XDG_CACHE_HOME": str(root / "cache"),
    }


def step(session, keys, marker):
    session.output.clear()
    os.write(session.master, keys)
    session.wait(lambda: PTY["frame_contains"](session.output, marker, None))
    assert session.process.poll() is None


def save(session):
    step(session, b"\x18\x13", b"Saved ")


def finish(session):
    os.write(session.master, b"\x18\x03")
    session.finish(0)
    wire = bytes(session.output)
    for code in [PTY["PASTE_OFF"], PTY["SHOW"], b"\x1b[0m", b"\x1b]112\x07", PTY["LEAVE"]]:
        assert code in wire, wire
    assert wire.rindex(b"\x1b]112\x07") < wire.rindex(PTY["LEAVE"])


def passive(root):
    for name in ["home", "data", "state", "cache"]:
        assert not (root / name).exists(), name


def configured(root):
    config = root / "user/rev/config.toml"
    config.parent.mkdir(parents=True)
    config.write_text("editor.indent_width = 2\neditor.tab_width = 8\nfiles.bom = true\nfiles.newline = 'crlf'\npalette.foreground = '#010203'\npalette.cursor = '#112233'\nkeys.editor.M-j = 'newline'\n")
    target = root / "capture.txt"
    session = PTY["Session"]([str(BINARY), str(target)], colors=True, env=environment(root))
    try:
        session.ready(b"capture.txt")
        assert b"\x1b]12;#112233\x07" in session.output
        assert b"\x1b[38;2;1;2;3m" in session.output
        os.write(session.master, b"\t" + "é".encode() + b"\x1bj")
        save(session)
        assert target.read_bytes() == b"\xef\xbb\xbf  \xc3\xa9\r\n"
        config.write_text("editor.indent_width = 6\neditor.tab_width = 2\npalette.foreground = '#040506'\npalette.cursor = '#445566'\nkeys.editor.M-j = 'insert-tab'\n")
        os.write(session.master, b"\x1bj")
        save(session)
        assert target.read_bytes() == b"\xef\xbb\xbf  \xc3\xa9\r\n\r\n"
        assert b"\x1b[38;2;1;2;3m" in session.output
        assert b"\x1b[38;2;4;5;6m" not in session.output
        finish(session)
    finally:
        session.close()
    restarted(root, target)
    passive(root)
    print("PASS configured palette/cursor, indentation, formats and keymaps; restart-only changes")


def restarted(root, target):
    before = target.read_bytes()
    session = PTY["Session"]([str(BINARY), str(target)], colors=True, env=environment(root))
    try:
        session.ready(b"capture.txt")
        assert b"\x1b]12;#445566\x07" in session.output
        assert b"\x1b[38;2;4;5;6m" in session.output
        os.write(session.master, b"\x1bj")
        save(session)
        assert target.read_bytes() == b"\xef\xbb\xbf      " + before[3:]
        finish(session)
    finally:
        session.close()


def missing(root):
    session = PTY["Session"]([str(BINARY)], colors=True, env=environment(root))
    try:
        session.ready(b"C-x C-c: exit")
        assert b"Configuration:" not in session.output
        finish(session)
    finally:
        session.close()
    passive(root)
    assert not (root / "user").exists()
    print("PASS silent missing configuration and passive XDG startup")


def rejected(root):
    config = root / "user/rev/config.toml"
    config.parent.mkdir(parents=True)
    source = "editor.tab_width = 0\neditor.indent_width = 2\nfiles.bom = true\npalette.foreground = '#010203'\nkeys.editor.M-j = 'newline'\n"
    config.write_text(source)
    target = root / "fallback.txt"
    session = PTY["Session"]([str(BINARY), str(target)], colors=True, env=environment(root))
    try:
        session.ready(b"Configuration:")
        os.write(session.master, b"\x1bj\tQ")
        save(session)
        assert target.read_bytes() == b"    Q"
        assert b"\x1b[38;2;1;2;3m" not in session.output
        finish(session)
    finally:
        session.close()
    assert config.read_text() == source
    passive(root)
    print("PASS nonfatal diagnostic and complete configured candidate rollback")


def main():
    subprocess.run(["cargo", "build", "--bin", "rev"], cwd=ROOT, check=True)
    for scenario in [configured, missing, rejected]:
        with tempfile.TemporaryDirectory(prefix="rev-startup-") as directory:
            scenario(Path(directory))
    print("Startup PTYs: 4 sessions passed, 0 skipped.")


if __name__ == "__main__":
    main()
