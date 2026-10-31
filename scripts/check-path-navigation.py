#!/usr/bin/env python3
"""Exercise native file completion and cancellation on owned RGB/NO_COLOR PTYs."""

import os
from pathlib import Path
import runpy
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
LIST = runpy.run_path(str(ROOT / "scripts/check-command-list.py"))
PTY = LIST["PTY"]
step = LIST["step"]
ordered = LIST["ordered"]


def fixtures(root):
    config = root / "user/rev/config.toml"
    config.parent.mkdir(parents=True)
    config.write_text("minibuffer.candidate_limit = 2\npalette.selection = '#112233'\npalette.selection_text = '#445566'\n")
    work = root / "work"
    (work / "child").mkdir(parents=True)
    for name in ["alpha.txt", "alpine.txt", "source.txt", "child/nested.txt"]:
        (work / name).write_bytes(b"retained\n")
    home = root / "home/space dir"
    home.mkdir(parents=True)
    (home / "界 name.txt").write_bytes(b"home\n")
    return work


def candidates(session, colored):
    frame = step(session, b"\x18\x06", b"Find file: ")
    ordered(frame, b"Find file: ", [b"child/", b"alpha.txt"])
    LIST["selected_style"](frame, b"child/", colored)
    frame = step(session, b"\x0e\x0e\t", b"Find file: alpine.txt")
    ordered(frame, b"Find file: alpine.txt", [b"alpine.txt"])
    LIST["selected_style"](frame, b"alpine.txt", colored)
    PTY["resize_surface"](session, 80, 4, True)
    assert not any(text.rstrip() == b"alpine.txt" for text in LIST["rows"](PTY["last_frame"](session)).values())
    PTY["resize_surface"](session, 80, 24, True)
    ordered(PTY["last_frame"](session), b"Find file: alpine.txt", [b"alpine.txt"])
    step(session, b"\x07", b"Quit")
    frame = step(session, b"\x18\x06al\t", b"Find file: alpha.txt  ")
    ordered(frame, b"Find file: alpha.txt", [b"alpha.txt"])
    step(session, b"\x07", b"Quit")


def directories(session, work):
    frame = step(session, b"\x18\x06ch\t", b"Find file: child/")
    ordered(frame, b"Find file: child/", [b"nested.txt"])
    frame = step(session, b"\x7f", b"Find file: child  ")
    ordered(frame, b"Find file: child", [b"child/"])
    step(session, b"\t../al\t", b"Find file: child/../alpha.txt")
    step(session, b"\x07", b"Quit")
    frame = step(session, b"\x18\x06~/space d\t", b"Find file: ~/space dir/")
    ordered(frame, b"Find file: ~/space dir/", ["界 name.txt".encode()])
    step(session, b"\t", "Find file: ~/space dir/界 name.txt".encode())
    step(session, b"\x07", b"Quit")
    step(session, b"\x18\x06" + PTY["paste_wire"](str(work / "child/nest").encode()) + b"\t", b"child/nested.txt")
    step(session, b"\x07", b"Quit")


def selected_directory(session, work, colored):
    choices = work / "choices"
    (choices / "alpha").mkdir(parents=True)
    (choices / "alpine").mkdir()
    for name, contents in [
        ("alpha/first.txt", b"first directory\n"),
        ("alpine/first.txt", b"first child\n"),
        ("alpine/z 界 name.txt", b"selected child\n"),
    ]:
        (choices / name).write_bytes(contents)
    before = {path.relative_to(work): path.read_bytes() for path in work.rglob("*") if path.is_file()}
    frame = step(session, b"\x18\x06choices/al", b"Find file: choices/al  ")
    ordered(frame, b"Find file: choices/al", [b"alpha/", b"alpine/"])
    LIST["selected_style"](frame, b"alpha/", colored)
    frame = step(session, b"\t", b"Find file: choices/alpha/")
    ordered(frame, b"Find file: choices/alpha/", [b"first.txt"])
    step(session, b"\x07", b"Quit")
    step(session, b"\x1bxfind-file\r", b"Find file: ")
    frame = step(session, b"choices/al\x0e\t", b"Find file: choices/alpine/")
    ordered(frame, b"Find file: choices/alpine/", [b"first.txt", "z 界 name.txt".encode()])
    frame = step(session, b"\x0e", b"Find file: choices/alpine/")
    LIST["selected_style"](frame, "z 界 name.txt".encode(), colored)
    PTY["resize_surface"](session, 80, 4, True)
    step(session, b"\t", "Find file: choices/alpine/z 界 name.txt".encode())
    PTY["resize_surface"](session, 80, 24, True)
    assert PTY['status_contains'](PTY["last_frame"](session), b"> source.txt -- RW -- L1 C1 Text")
    step(session, b"\r", "> z 界 name.txt -- RW -- L1 C1 Text".encode())
    assert PTY["frame_contains"](session.output, b"selected child", None)
    after = {path.relative_to(work): path.read_bytes() for path in work.rglob("*") if path.is_file()}
    assert after == before


def case(root, colored):
    work = fixtures(root)
    source = work / "source.txt"
    session = PTY["Session"]([str(ROOT / "target/debug/rev"), str(source)], env=LIST["environment"](root, colored))
    try:
        session.ready(b"> source.txt -- RW -- L1 C1 Text")
        candidates(session, colored)
        directories(session, work)
        assert PTY['status_contains'](PTY["last_frame"](session), b"> source.txt -- RW -- L1 C1 Text")
        selected_directory(session, work, colored)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert PTY["PASTE_OFF"] in session.output and PTY["LEAVE"] in session.output
        assert source.read_bytes() == b"retained\n"
        assert (root / "home/space dir/界 name.txt").read_bytes() == b"home\n"
    finally:
        session.close()
    print(f"PASS native highlighted completion, ambiguous directory continuation, explicit acceptance, parent/home/Unicode paths, resize/cancel and {'RGB' if colored else 'NO_COLOR'} selection")


def main():
    subprocess.run(["cargo", "build", "--locked", "--bin", "rev"], cwd=ROOT, check=True)
    for colored in [True, False]:
        with tempfile.TemporaryDirectory(prefix="rev-path-navigation-") as directory:
            case(Path(directory), colored)
    print("Path-navigation PTYs: 2 sessions passed, 0 skipped.")


if __name__ == "__main__":
    main()
