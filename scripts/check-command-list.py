#!/usr/bin/env python3
"""Verify M-x navigation, native arguments and retained geometry on owned PTYs."""

import os
from pathlib import Path
import re
import runpy
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
PTY = runpy.run_path(str(ROOT / "scripts/check-pty.py"))
SGR = re.compile(rb"\x1b\[[0-9;?]*[A-Za-z]")


def rows(frame):
    result = {}
    parts = re.split(rb"\x1b\[([0-9]+);([0-9]+)H", frame)
    for index in range(1, len(parts), 3):
        row, column, text = parts[index:index + 3]
        text = SGR.sub(b"", text)
        if column == b"1" and text:
            result[int(row)] = text
    return result


def step(session, keys, marker):
    session.output.clear()
    os.write(session.master, keys)
    session.wait(lambda: PTY["frame_contains"](session.output, marker, None))
    return PTY["last_frame"](session)


def ordered(frame, query, candidates):
    content = rows(frame)
    prompt = next(row for row, text in content.items() if text.startswith(query))
    for offset, candidate in enumerate(candidates, 1):
        assert content[prompt + offset].rstrip() == candidate, content
    return content


def environment(root, colored):
    return {
        "HOME": str(root / "home"),
        "XDG_CONFIG_HOME": str(root / "user"),
        "XDG_DATA_HOME": str(root / "data"),
        "XDG_STATE_HOME": str(root / "state"),
        "XDG_CACHE_HOME": str(root / "cache"),
        "NO_COLOR": "" if colored else "1",
    }


def selected_style(frame, name, colored):
    if colored:
        assert b"\x1b[48;2;17;34;51m" in frame
        assert b"\x1b[38;2;68;85;102m" + name in frame
    else:
        assert b"\x1b[7m" + name in frame
        assert b"\x1b[38;2;" not in frame


def selected_command(session, root):
    target = root / "selected-copy.txt"
    step(session, b"\x1bx", b"M-x ")
    step(session, b"\r", b"No command entered")
    step(session, b"\x1bxfile" + b"\x1b[B" * 4 + b"\r", b"Write file: ")
    step(session, PTY["paste_wire"](str(target).encode()), str(target).encode())
    step(session, b"\r", b"Saved ")
    assert target.read_bytes() == b"retained\n"
    step(session, b"\x1bxdraft", b"M-x draft")
    step(session, b"\x1bp", b"M-x write-file")
    step(session, b"\x1bn", b"M-x draft")
    step(session, b"\x07", b"Quit")


def case(root, colored):
    config = root / "user/rev/config.toml"
    config.parent.mkdir(parents=True)
    config.write_text("tree.visible = true\nminibuffer.candidate_limit = 2\npalette.selection = '#112233'\npalette.selection_text = '#445566'\n")
    path = root / "list.txt"
    path.write_bytes(b"retained\n")
    session = PTY["Session"]([str(ROOT / "target/debug/rev"), str(path)], env=environment(root, colored))
    try:
        session.ready(b"> list.txt -- RW -- L1 C1 Text")
        frame = step(session, b"\x1bxkill", b"M-x kill")
        ordered(frame, b"M-x kill", [b"kill-buffer", b"kill-line"])
        selected_style(frame, b"kill-buffer", colored)
        frame = step(session, b"\x1b[B" * 7 + b"\x10\x0e\t", b"M-x copy-rectangle-as-kill")
        ordered(frame, b"M-x copy-rectangle-as-kill", [b"backward-kill-word", b"copy-rectangle-as-kill"])
        selected_style(frame, b"copy-rectangle-as-kill", colored)
        PTY["resize_surface"](session, 80, 4, True)
        assert not any(text.rstrip() == b"copy-rectangle-as-kill" for text in rows(PTY["last_frame"](session)).values())
        PTY["resize_surface"](session, 80, 24, True)
        frame = PTY["last_frame"](session)
        ordered(frame, b"M-x copy-rectangle-as-kill", [b"backward-kill-word", b"copy-rectangle-as-kill"])
        selected_style(frame, b"copy-rectangle-as-kill", colored)
        step(session, b"\x07", b"Quit")
        selected_command(session, root)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert PTY["PASTE_OFF"] in session.output and PTY["LEAVE"] in session.output
        assert path.read_bytes() == b"retained\n"
    finally:
        session.close()
    print(f"PASS candidate/history navigation, selected native file arguments, empty-query guard, resize/cancel and {'RGB' if colored else 'NO_COLOR'} selection")


def main():
    subprocess.run(["cargo", "build", "--bin", "rev"], cwd=ROOT, check=True)
    for colored in [True, False]:
        with tempfile.TemporaryDirectory(prefix="rev-command-list-") as directory:
            case(Path(directory), colored)
    print("Command-list PTYs: 2 sessions passed, 0 skipped.")


if __name__ == "__main__":
    main()
