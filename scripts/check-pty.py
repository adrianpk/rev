#!/usr/bin/env python3
"""Exercise terminal lifetime through real controlling PTYs on Linux."""

import argparse
import errno
import fcntl
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import time


ENTER = b"\x1b[?1049h"
LEAVE = b"\x1b[?1049l"
HIDE = b"\x1b[?25l"
SHOW = b"\x1b[?25h"


def attach():
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


class Session:
    def __init__(self, command):
        self.master, self.slave = pty.openpty()
        self.before = termios.tcgetattr(self.slave)
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        self.output = bytearray()
        self.deadline = time.monotonic() + 8
        environment = dict(os.environ, TERM="xterm-256color")
        self.process = subprocess.Popen(
            command, stdin=self.slave, stdout=self.slave, stderr=self.slave,
            env=environment, preexec_fn=attach,
        )

    def drain(self):
        if select.select([self.master], [], [], 0.02)[0]:
            try:
                self.output.extend(os.read(self.master, 65536))
            except OSError as error:
                if error.errno != errno.EIO:
                    raise
        if len(self.output) > 1024 * 1024:
            raise AssertionError("terminal output exceeded 1 MiB")

    def wait(self, condition):
        while not condition():
            if time.monotonic() >= self.deadline:
                raise AssertionError(f"PTY deadline expired: {bytes(self.output)!r}")
            self.drain()

    def ready(self, marker):
        self.wait(lambda: marker in self.output or self.process.poll() is not None)
        assert marker in self.output, bytes(self.output)
        flags = termios.tcgetattr(self.slave)[3]
        assert not flags & (termios.ICANON | termios.ECHO), "raw mode was not acquired"

    def finish(self, expected):
        self.wait(lambda: self.process.poll() is not None)
        while select.select([self.master], [], [], 0)[0]:
            self.drain()
        assert self.process.returncode == expected, bytes(self.output)
        assert termios.tcgetattr(self.slave) == self.before, "terminal attributes were not restored"

    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=1)
        os.close(self.master)
        os.close(self.slave)


def lifecycle(root):
    binary = root / "target/debug/rev"
    session = Session([str(binary)])
    try:
        session.ready(b"C-x C-c: exit")
        assert ENTER in session.output and HIDE in session.output, bytes(session.output)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        assert session.output.index(SHOW) < session.output.index(LEAVE)
        print("PASS startup and ordinary exit")
    finally:
        session.close()


def failures(root):
    binary = root / "target/debug/examples/terminal-probe"
    cases = [("raw", 1), ("screen", 1), ("entered", 1), ("io", 1), ("unwind", 101)]
    for mode, expected in cases:
        session = Session([str(binary), mode])
        try:
            session.ready(b"probe-ready")
            os.write(session.master, b"!")
            session.finish(expected)
            if mode == "raw":
                assert all(code not in session.output for code in (ENTER, LEAVE, HIDE, SHOW))
            else:
                assert ENTER in session.output and LEAVE in session.output, bytes(session.output)
                assert SHOW in session.output, bytes(session.output)
                assert b"\x1b[0m" in session.output
                assert session.output.rfind(b"\x1b[0m") < session.output.rfind(LEAVE)
            assert b"injected" in session.output, bytes(session.output)
            print(f"PASS {mode} failure restores acquired state")
        finally:
            session.close()



def input_flow(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        os.write(session.master, b"\x18r")
        session.wait(lambda: b"C-x r" in session.output)
        os.write(session.master, b"\x07")
        session.wait(lambda: b"Quit" in session.output)
        os.write(session.master, b"\x03")
        session.wait(lambda: b"Unbound key" in session.output)
        assert session.process.poll() is None, bytes(session.output)
        os.write(session.master, b"\x18q")
        session.wait(lambda: b"Undefined key sequence" in session.output)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        print("PASS nested prefix cancellation and invalid-sequence recovery")
    finally:
        session.close()



def viewport(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        status = b"> *scratch* -- RW L1 C1 Text"
        cursor = b"\x1b[1;1H" + SHOW
        session.wait(lambda: cursor in session.output)
        assert status in session.output, bytes(session.output)
        # A rendered startup frame precedes event-reader registration. Observe
        # a handled key before the first SIGWINCH so the resize cannot be lost.
        session.output.clear()
        os.write(session.master, b"\x07")
        session.wait(lambda: b"Quit" in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 3, 3, 0, 0))
        session.wait(lambda: b"\x1b[3;1HQui" in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 1, 1, 0, 0))
        session.wait(lambda: b"\x1b[1;1HQ" in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 0, 0, 0, 0))
        session.wait(lambda: HIDE in session.output)
        assert b";1H" not in session.output, bytes(session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        session.wait(lambda: status in session.output and cursor in session.output)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        print("PASS rope viewport, status and cursor survive tiny/zero resize")
    finally:
        session.close()



def insertion(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        source = "e\u0301界👩\u200d💻".encode()
        os.write(session.master, source)
        session.wait(lambda: source in session.output and b"** RW L1 C6 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\x7f")
        prefix = "e\u0301界".encode()
        session.wait(lambda: prefix in session.output and b"** RW L1 C4 Text" in session.output)
        assert "👩".encode() not in session.output, bytes(session.output)
        session.output.clear()
        os.write(session.master, b"\t")
        session.wait(lambda: b"** RW L1 C5 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\rZ")
        session.wait(lambda: b"\x1b[2;1HZ" in session.output and b"** RW L2 C2 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\x04")
        session.wait(lambda: b"Buffer boundary" in session.output)
        session.output.clear()
        refusal = b"Unsaved changes; cannot quit"
        os.write(session.master, b"\x18\x03")
        session.wait(lambda: refusal in session.output)
        assert session.process.poll() is None, bytes(session.output)
        assert LEAVE not in session.output, bytes(session.output)
        assert prefix in session.output and b"** RW L2 C2 Text" in session.output
        session.output.clear()
        os.write(session.master, b"\x07!")
        session.wait(lambda: b"\x1b[2;1HZ!" in session.output and b"** RW L2 C3 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\x18\x03")
        session.wait(lambda: refusal in session.output)
        assert session.process.poll() is None and LEAVE not in session.output
        print("PASS Unicode insertion, grapheme deletion and dirty quit refusal")
    finally:
        # This owned unnamed dirty fixture has no save destination.
        # Terminate it only after checking refusal and continued editing.
        session.close()


def motion(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        first = "e\u0301界👩\u200d💻 first".encode()
        os.write(session.master, first + b"\rlast")
        session.wait(lambda: b"** RW L2 C5 Text" in session.output
                     and b"\x1b[2;5H" + SHOW in session.output)
        cases = [
            (b"\x01", 2, 1), (b"\x02", 1, 12), (b"\x02", 1, 11),
            (b"\x05", 1, 12), (b"\x01", 1, 1), (b"\x06", 1, 2),
            (b"\x1b[C", 1, 4), (b"\x06", 1, 6), (b"\x1bf", 1, 12),
            (b"\x1bb", 1, 7), (b"\x1b<", 1, 1), (b"\x1b>", 2, 5),
            (b"\x1b[H", 2, 1), (b"\x1b[F", 2, 5),
        ]
        for chord, row, column in cases:
            session.output.clear()
            os.write(session.master, chord)
            status = f"** RW L{row} C{column} Text".encode()
            cursor = f"\x1b[{row};{column}H".encode() + SHOW
            session.wait(lambda: status in session.output and cursor in session.output)
            assert b"\x1b[1;1H" + first in session.output, bytes(session.output)
            assert b"\x1b[2;1Hlast" in session.output, bytes(session.output)
        session.output.clear()
        os.write(session.master, b"\x18\x03")
        session.wait(lambda: b"Unsaved changes; cannot quit"
                     in session.output)
        assert session.process.poll() is None and LEAVE not in session.output
        print("PASS Emacs and arrow/Home/End motion preserves Unicode text and dirty state")
    finally:
        # Stop only this owned dirty fixture, after every motion/refusal check.
        session.close()


def scroll_step(session, chord, point, cursor):
    session.output.clear()
    os.write(session.master, chord)
    row, column = point
    status = f"** RW L{row} C{column} Text".encode()
    row, column = cursor
    position = f"\x1b[{row};{column}H".encode() + SHOW
    session.wait(lambda: status in session.output and position in session.output)
    assert session.process.poll() is None and LEAVE not in session.output


def scrolling(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        # Confirm event-reader registration before sending the first SIGWINCH.
        session.output.clear()
        os.write(session.master, b"\x07")
        session.wait(lambda: b"Quit" in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 7, 80, 0, 0))
        session.wait(lambda: b"\x1b[7;1HQuit" in session.output)
        lines = ["0123456789", "x", "a\t界e\u0301👩\u200d💻"] + ["0123456789"] * 12
        os.write(session.master, "\r".join(lines).encode())
        session.wait(lambda: b"** RW L15 C11 Text" in session.output)
        scroll_step(session, b"\x1b<" + b"\x06" * 8, (1, 9), (1, 9))
        steps = [
            (b"\x0e", (2, 2), (2, 2)), (b"\x0e", (3, 8), (3, 8)),
            (b"\x0e", (4, 9), (4, 9)), (b"\x1b[A", (3, 8), (3, 8)),
            (b"\x10", (2, 2), (2, 2)), (b"\x0e", (3, 8), (3, 8)),
            (b"\x16", (6, 9), (3, 9)), (b"\x0c", (6, 9), (3, 9)),
            (b"\x0c", (6, 9), (1, 9)), (b"\x0c", (6, 9), (5, 9)),
            (b"\x0c", (6, 9), (3, 9)), (b"\x1b[6~", (9, 9), (3, 9)),
            (b"\x1b[5~", (6, 9), (3, 9)), (b"\x1bv", (3, 8), (3, 8)),
            (b"\x10", (2, 2), (2, 2)), (b"\x1b[B", (3, 8), (3, 8)),
        ]
        for chord, point, cursor in steps:
            scroll_step(session, chord, point, cursor)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 3, 3, 0, 0))
        session.wait(lambda: b"\x1b[1;3H" + SHOW in session.output)
        session.output.clear()
        os.write(session.master, b"\x16")
        session.wait(lambda: b"\x1b[1;1H678" in session.output
                     and b"\x1b[1;3H" + SHOW in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 7, 80, 0, 0))
        session.wait(lambda: b"** RW L4 C9 Text" in session.output
                     and b"\x1b[1;3H" + SHOW in session.output)
        scroll_step(session, b"\x10", (3, 8), (1, 2))
        scroll_step(session, b"\x0e", (4, 9), (2, 3))
        session.output.clear()
        os.write(session.master, b"\x18\x03")
        session.wait(lambda: b"Unsaved changes; cannot quit"
                     in session.output)
        assert session.process.poll() is None and LEAVE not in session.output
        print("PASS retained columns, page/recenter keys and tiny-pane resize recovery")
    finally:
        # All navigation and dirty-exit assertions finish before owned cleanup.
        session.close()


def undo_history(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        os.write(session.master, b"\x18u")
        session.wait(lambda: b"Nothing to undo" in session.output)
        session.output.clear()
        source = "e\u0301界".encode()
        # Explicit command boundaries keep these three encoding checks separate
        # from the adjacent-typing group exercised by its own terminal journey.
        os.write(session.master, b"e\x07" + "\u0301".encode() + b"\x07" + "界".encode())
        session.wait(lambda: source in session.output and b"** RW L1 C4 Text" in session.output)
        session.output.clear()
        # CSI-u reports these exact Control combinations without the legacy
        # 0x1f ambiguity. C-x u remains the ordinary byte-sequence fallback.
        os.write(session.master, b"\x1b[47;5u")
        session.wait(lambda: b"** RW L1 C2 Text" in session.output
                     and "\x1b[1;1He\u0301 ".encode() in session.output)
        assert "界".encode() not in session.output, bytes(session.output)
        session.output.clear()
        os.write(session.master, b"\x1b[95;5u")
        session.wait(lambda: b"** RW L1 C2 Text" in session.output
                     and b"\x1b[1;1He " in session.output)
        assert "\u0301".encode() not in session.output, bytes(session.output)
        session.output.clear()
        os.write(session.master, b"\x18u")
        session.wait(lambda: b"-- RW L1 C1 Text" in session.output
                     and b"\x1b[1;1H" + SHOW in session.output)
        session.output.clear()
        os.write(session.master, b"!")
        session.wait(lambda: b"** RW L1 C2 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\x18u")
        session.wait(lambda: b"-- RW L1 C1 Text" in session.output)
        session.output.clear()
        os.write(session.master, b"\x18u")
        session.wait(lambda: b"Nothing to undo" in session.output)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        print("PASS reportable undo keys, clean-state restoration and ordinary exit")
    finally:
        session.close()


def group_step(session, keys, marker, content=None):
    session.output.clear()
    os.write(session.master, keys)
    session.wait(lambda: frame_contains(session.output, marker, content))
    assert session.process.poll() is None and LEAVE not in session.output


def frame_contains(output, marker, content):
    # Markers and cursor must belong to one finished redraw. A previous SHOW
    # cannot complete a new frame whose bytes arrive in a later PTY read.
    end = output.rfind(SHOW)
    if end < 0:
        return False
    start = output.rfind(HIDE, 0, end)
    if start < 0:
        return False
    frame = output[start:end + len(SHOW)]
    return marker in frame and (content is None or content in frame)


def typing_history(root):
    session = Session([str(root / "target/debug/rev")])
    try:
        session.ready(b"C-x C-c: exit")
        source = "ab e\u0301👩\u200d💻".encode()
        group_step(session, source, b"** RW L1 C7 Text", source)
        assert source in session.output, bytes(session.output)
        group_step(session, b"\x18u", b"-- RW L1 C1 Text")
        assert b"\x1b[1;1H " in session.output, bytes(session.output)
        group_step(session, b"ab", b"** RW L1 C3 Text")
        group_step(session, b"\x07", b"Quit")
        group_step(session, b"cd", b"** RW L1 C5 Text")
        group_step(session, b"\x18u", b"** RW L1 C3 Text")
        assert b"\x1b[1;1Hab " in session.output, bytes(session.output)
        group_step(session, b"\x18u", b"-- RW L1 C1 Text")
        group_step(session, b"ab\rcd", b"** RW L2 C3 Text")
        group_step(session, b"\x18u", b"** RW L2 C1 Text")
        group_step(session, b"\x18u", b"** RW L1 C3 Text")
        group_step(session, b"\x18u", b"-- RW L1 C1 Text")
        group_step(session, b"new", b"** RW L1 C4 Text")
        group_step(session, b"\x18u", b"-- RW L1 C1 Text")
        group_step(session, b"\x18u", b"Nothing to undo")
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        print("PASS whole Unicode typing groups, command/newline boundaries and clean exit")
    finally:
        session.close()


def file_ingress(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-files-") as directory:
        path = Path(directory) / "unicode.txt"
        source = "\ufeffe\u0301界\r\nlast".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"-- RW L1 C1 Text")
            first = "\x1b[1;1He\u0301界".encode()
            assert first in session.output and b"\x1b[2;1Hlast" in session.output
            assert b"\xef\xbb\xbf" not in session.output, bytes(session.output)
            group_step(session, b"\x18u", b"Nothing to undo")
            group_step(session, b"\x05", b"-- RW L1 C4 Text", first)
            group_step(session, b"!", b"** RW L1 C5 Text", "e\u0301界!".encode())
            group_step(session, b"\x18u", b"-- RW L1 C4 Text", first)
            group_step(session, b"\r", b"** RW L2 C1 Text", b"\x1b[3;1Hlast")
            group_step(session, b"\x18u", b"-- RW L1 C4 Text", b"\x1b[2;1Hlast")
            assert path.read_bytes() == source, "editing changed the visited file"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert SHOW in session.output and LEAVE in session.output
            assert path.read_bytes() == source, "exit changed the visited file"
            print("PASS UTF-8 startup, hidden BOM, CRLF editing and clean imported undo root")
        finally:
            session.close()


def ingress_failures(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-errors-") as directory:
        base = Path(directory)
        invalid = base / "invalid.txt"
        invalid.write_bytes(b"valid\xff")
        binary = base / "binary.txt"
        binary.write_bytes(b"valid\0text")
        denied = base / "denied.txt"
        denied.write_bytes(b"private")
        denied.chmod(0)
        cases = [
            (invalid, b"UTF-8", b"valid\xff", b"load"),
            (binary, b"NUL", b"valid\0text", b"load"),
            (invalid / "child", b"Not a directory", None, b"visit"),
            (base, b"regular file", None, b"visit"),
            (denied, b"Permission denied", b"private", b"load"),
        ]
        for path, marker, source, operation in cases:
            session = Session([str(root / "target/debug/rev"), str(path)])
            try:
                session.finish(1)
                assert marker in session.output, bytes(session.output)
                assert b"rev: " + operation + b" " in session.output, bytes(session.output)
                assert ENTER in session.output and LEAVE in session.output
                assert SHOW in session.output, bytes(session.output)
                assert session.output.index(LEAVE) < session.output.index(b"rev:")
                if source is not None:
                    path.chmod(0o600)
                    assert path.read_bytes() == source, "rejected ingress changed source bytes"
                print(f"PASS {path.name} ingress failure reports after terminal restoration")
            finally:
                session.close()


def argument_failure(root):
    session = Session([str(root / "target/debug/rev"), "one.txt", "two.txt"])
    try:
        session.finish(1)
        assert b"usage: rev [file]" in session.output, bytes(session.output)
        assert all(code not in session.output for code in (ENTER, LEAVE, HIDE, SHOW))
        print("PASS excess startup arguments fail before acquiring terminal state")
    finally:
        session.close()


def find_path(session, path, marker, content):
    group_step(session, b"\x18\x06", b"Find file: ")
    suffix = f"{path.parent.name}/{path.name}".encode()
    group_step(session, str(path).encode(), suffix)
    group_step(session, b"\r", marker, content)


def file_visits(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-visits-") as directory:
        base = Path(directory)
        (base / "one").mkdir()
        (base / "two").mkdir()
        first = base / "one/note.txt"
        second = base / "two/note.txt"
        first.write_bytes(b"alpha")
        second.write_bytes(b"beta")
        alias = base / "alias.txt"
        alias.symlink_to(first)
        session = Session([str(root / "target/debug/rev"), str(first)])
        try:
            session.ready(b"> note.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text", b"\x1b[1;1H!alpha")
            find_path(session, alias, b"> note.txt ** RW L1 C2 Text", b"\x1b[1;1H!alpha")
            group_step(session, b"\x18u", b"-- RW L1 C1 Text", b"\x1b[1;1Halpha")
            find_path(session, second, b"> note.txt<2> -- RW L1 C1 Text", b"\x1b[1;1Hbeta")
            find_path(session, alias, b"> note.txt -- RW L1 C1 Text", b"\x1b[1;1Halpha")
            group_step(session, b"\x18\x06", b"Find file: ")
            assert b"\x1b[24;12H" + SHOW in session.output, bytes(session.output)
            group_step(session, str(second).encode(), b"two/note.txt")
            group_step(session, b"\x07", b"Quit", b"> note.txt -- RW L1 C1 Text")
            assert b"\x1b[1;1Halpha" in session.output, bytes(session.output)
            assert first.read_bytes() == b"alpha" and second.read_bytes() == b"beta"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert SHOW in session.output and LEAVE in session.output
            print("PASS find-file focus, dirty symlink reuse, duplicate labels and cancellation")
        finally:
            session.close()


def new_visit(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-new-") as directory:
        base = Path(directory)
        path = base / "missing/deep/new.txt"
        alias = base / "unused/../missing/deep/./new.txt"
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> new.txt -- RW L1 C1 Text")
            assert not (base / "missing").exists()
            group_step(session, b"\x18\x06", b"Find file: ")
            group_step(session, b"\r", b"empty file path", b"> new.txt -- RW L1 C1 Text")
            group_step(session, b"ab", b"** RW L1 C3 Text", b"\x1b[1;1Hab")
            find_path(session, alias, b"> new.txt ** RW L1 C3 Text", b"\x1b[1;1Hab")
            assert b"\x1b[1;3H" + SHOW in session.output, bytes(session.output)
            group_step(session, b"\x18u", b"-- RW L1 C1 Text")
            assert not (base / "missing").exists() and not (base / "unused").exists()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert SHOW in session.output and LEAVE in session.output
            assert list(base.iterdir()) == [], "visiting created filesystem entries"
            print("PASS absent startup paths, consumed empty argument and normalized live reuse")
        finally:
            session.close()


def save_file(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-save-") as directory:
        path = Path(directory) / "unicode.txt"
        source = "\ufeffe\u0301界\r\nlast".encode()
        path.write_bytes(source)
        path.chmod(0o640)
        inode = path.stat().st_ino
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"-- RW L1 C1 Text")
            group_step(session, b"\x05!", b"** RW L1 C5 Text", "e\u0301界!".encode())
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C5 Text")
            assert path.read_bytes() == "\ufeffe\u0301界!\r\nlast".encode()
            assert path.stat().st_ino != inode
            assert path.stat().st_mode & 0o777 == 0o640
            group_step(session, b"?", b"** RW L1 C6 Text")
            group_step(session, b"\x18u", b"-- RW L1 C5 Text")
            group_step(session, b"\x18u", b"** RW L1 C4 Text")
            assert path.read_bytes() != source, "undo unexpectedly saved itself"
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C4 Text")
            assert path.read_bytes() == source
            assert list(Path(directory).iterdir()) == [path]
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert SHOW in session.output and LEAVE in session.output
            print("PASS real save-buffer, exact BOM/CRLF bytes, permissions and saved undo state")
        finally:
            session.close()


def save_failure(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-save-error-") as directory:
        path = Path(directory) / "missing/new.txt"
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> new.txt -- RW L1 C1 Text")
            group_step(session, b"ab", b"** RW L1 C3 Text")
            group_step(session, b"\x18\x13", b"save: not published", b"** RW L1 C3 Text")
            assert not path.parent.exists()
            group_step(session, b"\x18\x03", b"Unsaved changes; cannot quit")
            path.parent.mkdir()
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C3 Text")
            assert path.read_bytes() == b"ab"
            path.write_bytes(b"external")
            group_step(session, b"!", b"** RW L1 C4 Text")
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            group_step(session, b"n", b"Save canceled")
            assert path.read_bytes() == b"external"
            group_step(session, b"?", b"** RW L1 C5 Text", b"\x1b[1;1Hab!?")
            group_step(session, b"\x18\x03", b"Unsaved changes; cannot quit")
            assert list(path.parent.iterdir()) == [path]
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            path.write_bytes(b"changed while asking")
            group_step(session, b"y", b"save: not published", b"changed since last observation")
            assert path.read_bytes() == b"changed while asking"
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            group_step(session, b"y", b"Saved ", b"-- RW L1 C5 Text")
            assert path.read_bytes() == b"ab!?"
            path.unlink()
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            group_step(session, b"y", b"Saved ")
            assert path.read_bytes() == b"ab!?"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS failed save, missing-parent retry, stale consent and confirmed changed/removed paths")
        finally:
            session.close()


def write_path(session, path, marker):
    group_step(session, b"\x18\x17", b"Write file: ")
    group_step(session, str(path).encode(), path.name.encode())
    group_step(session, b"\r", marker)


def save_decisions(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-consent-") as directory:
        base = Path(directory)
        source = base / "first.txt"
        target = base / "referent.txt"
        target.write_bytes(b"keep")
        alias = base / "alias.txt"
        alias.symlink_to(target)
        session = Session([str(root / "target/debug/rev")])
        try:
            session.ready(b"C-x C-c: exit")
            group_step(session, b"new", b"** RW L1 C4 Text")
            group_step(session, b"\x18\x13", b"Write file: ")
            group_step(session, b"cancel", b"Write file: cancel")
            group_step(session, b"\x07", b"Quit", b"** RW L1 C4 Text")
            assert not source.exists()
            group_step(session, b"\x18\x13", b"Write file: ")
            group_step(session, str(source).encode(), source.name.encode())
            group_step(session, b"\r", b"Saved ", b"> first.txt -- RW L1 C4 Text")
            assert source.read_bytes() == b"new"
            write_path(session, alias, b"Overwrite occupied destination? [y/n]")
            assert target.read_bytes() == b"keep"
            group_step(session, b"n", b"Save canceled", b"> first.txt -- RW L1 C4 Text")
            write_path(session, alias, b"Overwrite occupied destination? [y/n]")
            group_step(session, b"y", b"Saved ", b"> referent.txt -- RW L1 C4 Text")
            assert target.read_bytes() == b"new" and source.read_bytes() == b"new"
            assert alias.is_symlink() and alias.readlink() == target
            group_step(session, b"\x18u", b"** RW L1 C1 Text")
            assert target.read_bytes() == b"new", "undo changed disk without save"
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C1 Text")
            assert target.read_bytes() == b""
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS unnamed save, argument cancellation, occupied save-as, symlink referent and dirty undo")
        finally:
            session.close()


def save_collision(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-collision-") as directory:
        base = Path(directory)
        source = base / "source.txt"
        source.write_bytes(b"source")
        other = base / "other.txt"
        other.write_bytes(b"other")
        alias = base / "alias.txt"
        alias.symlink_to(other)
        session = Session([str(root / "target/debug/rev"), str(source)])
        try:
            session.ready(b"> source.txt -- RW L1 C1 Text")
            find_path(session, other, b"> other.txt -- RW L1 C1 Text", b"other")
            find_path(session, source, b"> source.txt -- RW L1 C1 Text", b"source")
            group_step(session, b"!", b"** RW L1 C2 Text")
            write_path(session, alias, b"another live buffer")
            assert other.read_bytes() == b"other" and source.read_bytes() == b"source"
            group_step(session, b"?", b"** RW L1 C3 Text", b"\x1b[1;1H!?source")
            group_step(session, b"\x18\x13", b"Saved ")
            assert source.read_bytes() == b"!?source" and other.read_bytes() == b"other"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS canonical live-buffer collision refuses save-as and leaves editing available")
        finally:
            session.close()


def buffer_prompt(session, keys, label, name, marker):
    group_step(session, keys, label)
    if name:
        group_step(session, name, name)
    group_step(session, b"\r", marker)


def buffer_commands(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-buffers-") as directory:
        base = Path(directory)
        (base / "one").mkdir()
        (base / "two").mkdir()
        first = base / "one/note.txt"
        second = base / "two/note.txt"
        first.write_bytes(b"alpha")
        second.write_bytes(b"beta")
        session = Session([str(root / "target/debug/rev"), str(first)])
        try:
            session.ready(b"> note.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            find_path(session, second, b"> note.txt<2> -- RW L1 C1 Text", b"beta")
            group_step(session, b"\x18b", b"Switch to buffer: ")
            group_step(session, b"note.txt\t", b"2 matching buffer(s)")
            group_step(session, b"\r", b"> note.txt ** RW L1 C1 Text", b"!alpha")
            group_step(session, b"\x18u", b"> note.txt -- RW L1 C1 Text", b"alpha")
            group_step(session, b"\x18\x02", b"Buffers: arrows move")
            assert b"note.txt<2> -- RW" in session.output
            assert b"one/note.txt" in session.output and b"two/note.txt" in session.output
            group_step(session, b"\x1b[B", b"> note.txt<2> -- RW")
            group_step(session, b"\r", b"> note.txt<2> -- RW L1 C1 Text", b"beta")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"Buffer closed")
            assert b"> *scratch* -- RW L1 C1 Text" in session.output
            find_path(session, second, b"> note.txt<2> -- RW L1 C1 Text", b"beta")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"note.txt", b"Buffer closed")
            assert b"> note.txt<2> -- RW L1 C1 Text" in session.output
            assert first.read_bytes() == b"alpha" and second.read_bytes() == b"beta"
            find_path(session, first, b"> note.txt -- RW L1 C1 Text", b"alpha")
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS buffer completion, selectable duplicate-name inventory, retained history and inactive close")
        finally:
            session.close()


def close_buffers(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-close-") as directory:
        path = Path(directory) / "missing/new.txt"
        session = Session([str(root / "target/debug/rev")])
        try:
            session.ready(b"C-x C-c: exit")
            group_step(session, b"new", b"** RW L1 C4 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit", b"** RW L1 C4 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"y", b"Write file: ")
            group_step(session, str(path).encode(), path.name.encode())
            group_step(session, b"\r", b"save: not published", b"** RW L1 C4 Text")
            group_step(session, b"\x18\x03", b"Unsaved changes; cannot quit")
            assert not path.exists()
            path.parent.mkdir()
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"y", b"Write file: ")
            group_step(session, str(path).encode(), path.name.encode())
            group_step(session, b"\r", b"Buffer closed", b"> *scratch* -- RW L1 C1 Text")
            assert path.read_bytes() == b"new"
            find_path(session, path, b"> new.txt -- RW L1 C1 Text", b"new")
            group_step(session, b"!", b"** RW L1 C2 Text")
            path.write_bytes(b"external")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"y", b"Destination changed on disk")
            group_step(session, b"n", b"Save canceled", b"** RW L1 C2 Text")
            assert path.read_bytes() == b"external"
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"n", b"Buffer closed", b"> *scratch* -- RW L1 C1 Text")
            find_path(session, path, b"> new.txt -- RW L1 C1 Text", b"external")
            path.chmod(0o444)
            group_step(session, b"!", b"** RW L1 C2 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"y", b"Destination changed on disk")
            group_step(session, b"y", b"save: not published", b"Permission denied")
            assert path.read_bytes() == b"external"
            path.chmod(0o600)
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"y", b"Destination changed on disk")
            group_step(session, b"y", b"Buffer closed", b"> *scratch* -- RW L1 C1 Text")
            assert path.read_bytes() == b"!external"
            group_step(session, b"discard", b"** RW L1 C8 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"n", b"Buffer closed", b"> *scratch* -- RW L1 C1 Text")
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS close cancellation, failed required saves, nested consent, discard and final replacement")
        finally:
            session.close()


def nested_splits(root):
    for order in ["below-right", "right-below"]:
        with tempfile.TemporaryDirectory(prefix="rev-pty-splits-") as directory:
            path = Path(directory) / "split.txt"
            prefix = "a\t界e\u0301👩\u200d💻" + "0123456789" * 10
            source = "".join(f"{prefix}-{row:02d}\n" for row in range(60)).encode()
            path.write_bytes(source)
            session = Session([str(root / "target/debug/rev"), str(path)])
            try:
                session.ready(b"> split.txt -- RW L1 C1 Text")
                if order == "below-right":
                    group_step(session, b"\x182", b"\x1b[11;1H> split.txt", b"\x1b[23;1H  split.txt")
                    group_step(session, b"\x183", b"\x1b[11;1H> split.txt", b"|")
                else:
                    group_step(session, b"\x183", b"\x1b[23;1H> split.txt", b"|")
                    group_step(session, b"\x182", b"\x1b[11;1H> split.txt", b"\x1b[23;1H  split.txt")
                assert b"\x1b[1;1H" in session.output and b"\x1b[12;1H" in session.output
                assert "界e\u0301👩\u200d💻".encode() in session.output
                group_step(session, b"\x05", b"L1 C113 Text")
                group_step(session, b"\x0e", b"L2 C113 Text", b"-- RW L1 C1 Text")
                group_step(session, b"!", b"L2 C114 Text", b"** RW L1 C1 Text")
                assert path.read_bytes() == source, "shared editing unexpectedly saved disk"
                group_step(session, b"\x18u", b"L2 C113 Text", b"-- RW L1 C1 Text")
                # Selected pane width 39 fits three further right divisions;
                # the fourth must refuse without leaving a stale shared view.
                group_step(session, b"\x183\x183\x183\x183", b"Split does not fit")
                assert path.read_bytes() == source
                os.write(session.master, b"\x18\x03")
                session.finish(0)
                print(f"PASS {order} nested splits, Unicode panes, retained focus, shared edit/undo and refused split")
            finally:
                session.close()


def window_lifecycle(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-windows-") as directory:
        path = Path(directory) / "view.txt"
        source = b"alpha\nbeta\n"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> view.txt -- RW L1 C1 Text")
            group_step(session, b"\x182", b"\x1b[11;1H> view.txt", b"\x1b[23;1H  view.txt")
            group_step(session, b"\x183", b"\x1b[11;1H> view.txt", b"|")
            group_step(session, b"\x18o", b"|> view.txt -- RW L1 C1 Text", b"\x1b[1;41H" + SHOW)
            group_step(session, b"\x05", b"|> view.txt -- RW L1 C6 Text", b"\x1b[1;46H" + SHOW)
            group_step(session, b"\x18o", b"\x1b[23;1H> view.txt -- RW L1 C1 Text", b"\x1b[12;1H" + SHOW)
            group_step(session, b"\x18o", b"\x1b[11;1H> view.txt -- RW L1 C1 Text", b"\x1b[1;1H" + SHOW)
            assert b"|  view.txt -- RW L1 C6 Text" in session.output
            group_step(session, b"\x180", b"\x1b[11;1H> view.txt -- RW L1 C6 Text", b"\x1b[1;6H" + SHOW)
            end = session.output.rfind(SHOW)
            frame = session.output[session.output.rfind(HIDE, 0, end):end + len(SHOW)]
            assert b"|" not in frame, "deleted right division retained its separator"
            group_step(session, b"\x181", b"\x1b[23;1H> view.txt -- RW L1 C6 Text", b"\x1b[1;6H" + SHOW)
            group_step(session, b"\x180", b"Cannot delete the sole editing window", b"\x1b[1;6H" + SHOW)
            group_step(session, b"\x18o", b"\x1b[23;1H> view.txt -- RW L1 C6 Text", b"\x1b[1;6H" + SHOW)
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS editing-leaf cycle/wrap, independent point, sibling promotion, keep-selected and sole refusal")
        finally:
            session.close()


def hidden_window_buffers(root):
    for action in [b"0", b"1"]:
        with tempfile.TemporaryDirectory(prefix="rev-pty-hidden-buffer-") as directory:
            first = Path(directory) / "first.txt"
            hidden = Path(directory) / "hidden.txt"
            first.write_bytes(b"first")
            hidden.write_bytes(b"hidden")
            session = Session([str(root / "target/debug/rev"), str(first)])
            try:
                session.ready(b"> first.txt -- RW L1 C1 Text")
                group_step(session, b"\x183", b"> first.txt -- RW L1 C1 Text", b"|")
                group_step(session, b"\x18o", b"|> first.txt -- RW L1 C1 Text")
                find_path(session, hidden, b"|> hidden.txt -- RW L1 C1 Text", b"hidden")
                group_step(session, b"!", b"|> hidden.txt ** RW L1 C2 Text")
                if action == b"1":
                    group_step(session, b"\x18o", b"> first.txt -- RW L1 C1 Text", b"|  hidden.txt ** RW L1 C2 Text")
                group_step(session, b"\x18" + action, b"\x1b[23;1H> first.txt -- RW L1 C1 Text")
                group_step(session, b"\x18\x03", b"Unsaved changes; cannot quit")
                assert first.read_bytes() == b"first" and hidden.read_bytes() == b"hidden"
                buffer_prompt(session, b"\x18b", b"Switch to buffer: ", b"hidden.txt", b"> hidden.txt ** RW L1 C1 Text")
                assert b"\x1b[1;1H!hidden" in session.output
                group_step(session, b"\x18u", b"> hidden.txt -- RW L1 C1 Text", b"\x1b[1;1Hhidden")
                assert hidden.read_bytes() == b"hidden"
                os.write(session.master, b"\x18\x03")
                session.finish(0)
                print(f"PASS C-x {action.decode()} retains hidden modified file, quit protection, buffer reuse and undo")
            finally:
                session.close()



def shared_view_mapping(root):
    # The scrolled pane must keep the same source context when another pane
    # inserts above it. Undo from that passive pane restores the live origin.
    with tempfile.TemporaryDirectory(prefix="rev-pty-shared-") as directory:
        path = Path(directory) / "shared.txt"
        source = "".join(f"row-{row:02d} abcd\n" for row in range(60)).encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> shared.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"> shared.txt -- RW L1 C1 Text", b"|")
            group_step(session, b"\x18o", b"|> shared.txt -- RW L1 C1 Text")
            group_step(session, b"\x0e" * 30, b"|> shared.txt -- RW L31 C1 Text")
            group_step(session, b"\x0c", b"|> shared.txt -- RW L31 C1 Text", b"|row-19")
            group_step(session, b"\x18o", b"> shared.txt -- RW L1 C1 Text")
            first_row = b"\x1b[1;1H" + b" " * 39 + b"|row-19"
            group_step(session, b"\r", b"> shared.txt ** RW L2 C1 Text", first_row)
            assert b"|  shared.txt ** RW L32 C1 Text" in session.output
            group_step(session, b"\x18o", b"|> shared.txt ** RW L32 C1 Text")
            first_row = b"\x1b[1;1Hrow-00 abcd" + b" " * 28 + b"|row-19"
            group_step(session, b"\x18u", b"|> shared.txt -- RW L31 C1 Text", first_row)
            assert b"  shared.txt -- RW L1 C1 Text" in session.output
            assert b"\x1b[12;41H" + SHOW in session.output
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS shared line anchors, independent mapped cursors and undo through the passive pane")
        finally:
            session.close()


def closed_view_history(root):
    # A deleted editing origin cannot reappear during undo; a later split is
    # an independent live view and the invoking cursor receives the fallback.
    with tempfile.TemporaryDirectory(prefix="rev-pty-closed-origin-") as directory:
        path = Path(directory) / "history.txt"
        path.write_bytes(b"abcdef")
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> history.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"> history.txt -- RW L1 C1 Text", b"|")
            group_step(session, b"\x18o\x06\x06", b"|> history.txt -- RW L1 C3 Text")
            group_step(session, b"X", b"|> history.txt ** RW L1 C4 Text", b"abXcdef")
            group_step(session, b"\x180", b"\x1b[23;1H> history.txt ** RW L1 C1 Text")
            group_step(session, b"\x182\x18o", b"\x1b[23;1H> history.txt ** RW L1 C1 Text",
                       b"\x1b[11;1H  history.txt ** RW L1 C1 Text")
            group_step(session, b"\x18u", b"\x1b[23;1H> history.txt -- RW L1 C3 Text",
                       b"\x1b[11;1H  history.txt -- RW L1 C1 Text")
            assert b"\x1b[12;3H" + SHOW in session.output
            assert path.read_bytes() == b"abcdef"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS closed-origin undo fallback, later split and retained independent cursor")
        finally:
            session.close()


def kill_shared_buffer(root):
    # Cancel retains all three modified panes; explicit discard replaces every
    # reference, then revisiting reads unchanged file bytes into a fresh buffer.
    with tempfile.TemporaryDirectory(prefix="rev-pty-shared-kill-") as directory:
        path = Path(directory) / "kill.txt"
        path.write_bytes(b"original")
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> kill.txt -- RW L1 C1 Text")
            group_step(session, b"\x182\x183", b"\x1b[11;1H> kill.txt -- RW L1 C1 Text", b"|")
            group_step(session, b"!", b"> kill.txt ** RW L1 C2 Text", b"|  kill.txt ** RW L1 C2 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"\x07", b"Quit", b"\x1b[23;1H  kill.txt ** RW L1 C2 Text")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"y=save")
            group_step(session, b"n", b"Buffer closed", b"|  *scratch* -- RW L1 C1 Text")
            assert b"\x1b[23;1H  *scratch* -- RW L1 C1 Text" in session.output
            find_path(session, path, b"> kill.txt -- RW L1 C1 Text", b"original")
            assert b"|  *scratch* -- RW L1 C1 Text" in session.output
            assert path.read_bytes() == b"original"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS shared modified-buffer kill cancellation, discard, repaired panes and fresh visit")
        finally:
            session.close()



def terminal_bounds(output, width, height):
    moves = re.findall(rb"\x1b\[(\d+);(\d+)H", output)
    for row, column in moves:
        assert 1 <= int(row) <= height, (width, height, row)
        assert 1 <= int(column) <= width, (width, height, column)
    if width == 0 or height == 0:
        assert not moves and SHOW not in output


def resize_surface(session, width, height, cursor):
    session.output.clear()
    fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", height, width, 0, 0))
    if width == 0 or height == 0:
        session.wait(lambda: HIDE in session.output)
        assert session.output == HIDE
    elif cursor:
        session.wait(lambda: session.output.endswith(SHOW))
    else:
        echo = b"Quit"[:width].ljust(width, b" ")
        last = f"\x1b[{height};1H".encode() + echo
        session.wait(lambda: session.output.endswith(last))
        assert SHOW not in session.output
    terminal_bounds(session.output, width, height)
    assert session.process.poll() is None and LEAVE not in session.output


def resizing_windows(root):
    # Each report completes before the next one. The native endpoints use a
    # single skinny axis, keeping the real fixture below its 1 MiB output bound.
    for direction, minimum in [(b"3", (11, 3)), (b"2", (3, 7))]:
        with tempfile.TemporaryDirectory(prefix="rev-pty-resize-") as directory:
            path = Path(directory) / "r.txt"
            source = b"first\nsecond\n"
            path.write_bytes(source)
            session = Session([str(root / "target/debug/rev"), str(path)])
            try:
                session.ready(b"> r.txt -- RW L1 C1 Text")
                group_step(session, b"!", b"> r.txt ** RW L1 C2 Text")
                group_step(session, b"\x18" + direction + b"\x18" + direction,
                           b"> r.txt ** RW L1 C2")
                group_step(session, b"\x05\x07", b"Quit", b"> r.txt ** RW L1 C7")
                dimensions = [minimum, (0, 0), (0, 24), (80, 0), (1, 1),
                              (65535, 1), (1, 65535), minimum, (80, 24)]
                for width, height in dimensions:
                    fits = width >= minimum[0] and height >= minimum[1]
                    resize_surface(session, width, height, fits)
                assert b"> r.txt ** RW L1 C7" in session.output
                assert b"  r.txt ** RW L1 C2" in session.output
                resize_surface(session, *minimum, True)
                echo = f"\x1b[{minimum[1]};1H".encode() + b"Split does not fit"[:minimum[0]]
                group_step(session, b"\x18" + direction, echo)
                terminal_bounds(session.output, *minimum)
                group_step(session, b"\x07", b"Quit"[:minimum[0]])
                resize_surface(session, 80, 24, True)
                group_step(session, b"\x18u", b"> r.txt -- RW L1 C1", b"  r.txt -- RW L1 C1")
                assert path.read_bytes() == source
                os.write(session.master, b"\x18\x03")
                session.finish(0)
                print(f"PASS C-x {direction.decode()} nested minima, zero/tiny/native-axis resize, independent point and retained undo")
            finally:
                session.close()


def resizing_prompt(root):
    # Prompt ownership and its typed argument survive suspension and one-cell
    # clipping; Enter after expansion still visits the captured destination.
    with tempfile.TemporaryDirectory(prefix="rev-pty-resize-prompt-") as directory:
        first = Path(directory) / "first.txt"
        chosen = Path(directory) / "chosen.txt"
        first.write_bytes(b"first")
        chosen.write_bytes(b"destination")
        session = Session([str(root / "target/debug/rev"), str(first)])
        try:
            session.ready(b"> first.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"> first.txt -- RW L1 C1 Text", b"|")
            group_step(session, b"\x18\x06", b"Find file: ")
            group_step(session, str(chosen).encode(), b"Find file: " + str(chosen).encode())
            for width, height in [(0, 0), (1, 1), (3, 3), (80, 24)]:
                resize_surface(session, width, height, True)
            assert b"Find file: " + str(chosen).encode() in session.output
            group_step(session, b"\r", b"> chosen.txt -- RW L1 C1 Text", b"destination")
            assert b"|  first.txt -- RW L1 C1 Text" in session.output
            assert first.read_bytes() == b"first" and chosen.read_bytes() == b"destination"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS prompt argument and input ownership survive zero, one-cell and restored geometry")
        finally:
            session.close()



def last_frame(session):
    end = session.output.rfind(SHOW)
    start = session.output.rfind(HIDE, 0, end)
    assert start >= 0 and end > start
    return bytes(session.output[start:end + len(SHOW)])


def linear_regions(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-regions-") as directory:
        path = Path(directory) / "region.txt"
        source = "ab界e\u0301z\nlast".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> region.txt -- RW L1 C1 Text")
            group_step(session, b"\x00", b"Mark set")
            assert b"\x1b[7m" not in last_frame(session)
            group_step(session, b"\x06", b"L1 C2 Text", b"\x1b[7ma\x1b[27mb")
            group_step(session, b"\x05\x0e", b"L2 C5 Text", b"\x1b[7mlast\x1b[27m")
            group_step(session, b"\x18\x18", b"L1 C1 Text", b"\x1b[1;1H\x1b[7mab")
            group_step(session, b"\x07", b"Quit")
            assert b"\x1b[7m" not in last_frame(session)
            group_step(session, b"\x18\x18", b"L2 C5 Text", b"\x1b[7mlast\x1b[27m")
            group_step(session, b"\x00\x02", b"L2 C4 Text", b"las\x1b[7mt\x1b[27m")
            group_step(session, b"!", b"** RW L2 C5 Text", b"las!t")
            assert b"\x1b[7m" not in last_frame(session)
            group_step(session, b"\x18u", b"-- RW L2 C4 Text", b"last")
            assert b"\x1b[7m" not in last_frame(session)
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert b"\x1b[0m" in session.output
            assert session.output.rfind(b"\x1b[0m") < session.output.rfind(LEAVE)
            print("PASS NUL mark, Unicode multi-row region, exchange, cancel/reactivation and edit/undo deactivation")
        finally:
            session.close()


def shared_regions(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-shared-region-") as directory:
        path = Path(directory) / "shared.txt"
        source = b"abcdef\nsecond"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> shared.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x06\x00\x05", b"L1 C7 Text", b"ab\x1b[7mcdef\x1b[27m")
            group_step(session, b"\x183", b"> shared.txt -- RW L1 C7 Text", b"|abcdef")
            assert last_frame(session).count(b"\x1b[7m") == 1
            group_step(session, b"\x18o\x01", b"|> shared.txt -- RW L1 C1 Text",
                       b"|\x1b[7mab\x1b[27mcdef")
            assert last_frame(session).count(b"\x1b[7m") == 1
            group_step(session, b"\x18\x18", b"|> shared.txt -- RW L1 C3 Text")
            group_step(session, b"X", b"|> shared.txt ** RW L1 C4 Text", b"abXcdef")
            assert b"\x1b[7m" not in last_frame(session)
            assert b"  shared.txt ** RW L1 C8 Text" in session.output
            group_step(session, b"\x18u", b"|> shared.txt -- RW L1 C3 Text", b"abcdef")
            assert b"\x1b[7m" not in last_frame(session)
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS focused split selection, shared mark, independent mapped points and style clearing")
        finally:
            session.close()


def scrolling_regions(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-scrolled-region-") as directory:
        path = Path(directory) / "scroll.txt"
        source = "".join(f"row-{row:02d}\t界e\u0301\n" for row in range(60)).encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> scroll.txt -- RW L1 C1 Text")
            group_step(session, b"\x00\x1b>\x0c", b"L61 C1 Text", b"\x1b[1;1H\x1b[7mrow-49")
            assert last_frame(session).count(b"\x1b[7m") == 11
            resize_surface(session, 7, 5, True)
            assert last_frame(session).count(b"\x1b[7m") == 2
            resize_surface(session, 0, 0, False)
            resize_surface(session, 80, 24, True)
            group_step(session, b"\x07", b"Quit")
            assert b"\x1b[7m" not in last_frame(session)
            group_step(session, b"\x18\x18", b"L1 C1 Text", b"\x1b[7mrow-00")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS region spans through vertical scroll, clipped Unicode rows, suspension and restored mark")
        finally:
            session.close()


def line_kills(root):
    # The real C-k wire report removes CRLF atomically, with one undo per kill;
    # saving after replay preserves the visit's BOM and every surviving byte.
    with tempfile.TemporaryDirectory(prefix="rev-pty-line-kills-") as directory:
        path = Path(directory) / "line.txt"
        source = b"\xef\xbb\xbf" + "ab界\r\nlast".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> line.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x0b", b"** RW L1 C2 Text", b"\x1b[1;1Ha ")
            group_step(session, b"\x0b", b"** RW L1 C2 Text", b"\x1b[1;1Halast ")
            group_step(session, b"\x18u", b"** RW L1 C2 Text", b"\x1b[2;1Hlast ")
            group_step(session, b"\x18u", b"-- RW L1 C2 Text", "ab界".encode())
            assert path.read_bytes() == source
            group_step(session, b"\x0b\x0b", b"** RW L1 C2 Text", b"\x1b[1;1Halast ")
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == b"\xef\xbb\xbfalast"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS C-k line/whole CRLF kills, separate undo, exact BOM save and restored exit")
        finally:
            session.close()


def word_kills(root):
    # Actual ESC-d and ESC-DEL reports exercise Unicode word/punctuation ranges
    # in both directions, standalone undo and exact bytes across a CRLF save.
    with tempfile.TemporaryDirectory(prefix="rev-pty-word-kills-") as directory:
        path = Path(directory) / "word.txt"
        source = "e\u0301界_2...👩\u200d💻 last\r\nend".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> word.txt -- RW L1 C1 Text")
            group_step(session, b"\x1bd", b"** RW L1 C1 Text", "\x1b[1;1H...👩\u200d💻 last".encode())
            group_step(session, b"\x1bd", b"** RW L1 C1 Text", b"\x1b[1;1H ")
            group_step(session, b"\x18u", b"** RW L1 C1 Text", "...👩\u200d💻 last".encode())
            group_step(session, b"\x18u", b"-- RW L1 C1 Text", source.split(b"\r\n")[0])
            group_step(session, b"\x05", b"-- RW L1 C16 Text")
            group_step(session, b"\x1b\x7f", b"** RW L1 C12 Text", "👩\u200d💻 ".encode())
            group_step(session, b"\x1b\x7f", b"** RW L1 C1 Text", b"\x1b[1;1H ")
            group_step(session, b"\x18u", b"** RW L1 C12 Text", "e\u0301界_2...👩\u200d💻 ".encode())
            group_step(session, b"\x18u", b"-- RW L1 C16 Text", source.split(b"\r\n")[0])
            group_step(session, b"\x1b<\x1bd", b"** RW L1 C1 Text", "\x1b[1;1H...👩\u200d💻 last".encode())
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == "...👩\u200d💻 last\r\nend".encode()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS M-d/M-DEL Unicode word kills, directional boundaries, independent undo and CRLF save")
        finally:
            session.close()


def linear_ring(root):
    # Real mark/meta/control reports copy without dirtying, kill the region,
    # yank atomically, undo each edit and save exact multiline Unicode/BOM bytes.
    with tempfile.TemporaryDirectory(prefix="rev-pty-linear-ring-") as directory:
        path = Path(directory) / "region.txt"
        text = "αe\u0301\r\n界👩\u200d💻".encode()
        source = b"\xef\xbb\xbf" + text
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> region.txt -- RW L1 C1 Text")
            group_step(session, b"\x19", b"Kill ring is empty")
            group_step(session, b"\x00\x1b>", b"-- RW L2 C5 Text")
            group_step(session, b"\x1bw", b"Region copied")
            assert b"-- RW L2 C5 Text" in last_frame(session)
            assert b"\x1b[7m" in last_frame(session)
            assert path.read_bytes() == source
            group_step(session, b"\x17", b"** RW L1 C1 Text", b"\x1b[1;1H ")
            assert b"\x1b[7m" not in last_frame(session)
            group_step(session, b"\x19", b"** RW L2 C5 Text", "界👩\u200d💻".encode())
            group_step(session, b"\x18u", b"** RW L1 C1 Text", b"\x1b[1;1H ")
            group_step(session, b"\x18u", b"-- RW L2 C5 Text", "界👩\u200d💻".encode())
            assert path.read_bytes() == source
            group_step(session, b"\x19", b"** RW L3 C5 Text")
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == b"\xef\xbb\xbf" + text + text
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS C-w/M-w/C-y region kill/copy, empty ring, atomic yank/undo and exact Unicode/BOM save")
        finally:
            session.close()


def cross_buffer_yank(root):
    # The session-owned copy survives file focus changes and inserts once into
    # another buffer without modifying its source or normalizing CRLF bytes.
    with tempfile.TemporaryDirectory(prefix="rev-pty-cross-yank-") as directory:
        source_path = Path(directory) / "source.txt"
        target_path = Path(directory) / "target.txt"
        source = "keep α\r\nsecond".encode()
        source_path.write_bytes(source)
        target_path.write_bytes(b"TARGET")
        session = Session([str(root / "target/debug/rev"), str(source_path)])
        try:
            session.ready(b"> source.txt -- RW L1 C1 Text")
            group_step(session, b"\x00\x1b>", b"-- RW L2 C7 Text")
            group_step(session, b"\x1bw", b"Region copied")
            find_path(session, target_path, b"> target.txt -- RW L1 C1 Text", b"TARGET")
            group_step(session, b"\x19", b"> target.txt ** RW L2 C7 Text", b"secondTARGET")
            group_step(session, b"\x18u", b"> target.txt -- RW L1 C1 Text", b"\x1b[1;1HTARGET ")
            group_step(session, b"\x19", b"> target.txt ** RW L2 C7 Text", b"secondTARGET")
            group_step(session, b"\x18\x13", b"Saved ")
            assert target_path.read_bytes() == source + b"TARGET"
            assert source_path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS session ring across file visits, single-step yank undo, exact CRLF save and unchanged source")
        finally:
            session.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", choices=["terminal"], required=True)
    parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    subprocess.run(["cargo", "build", "--bin", "rev", "--example", "terminal-probe"],
                   cwd=root, check=True)
    lifecycle(root)
    failures(root)
    input_flow(root)
    viewport(root)
    insertion(root)
    motion(root)
    scrolling(root)
    undo_history(root)
    typing_history(root)
    file_ingress(root)
    ingress_failures(root)
    argument_failure(root)
    file_visits(root)
    new_visit(root)
    save_file(root)
    save_failure(root)
    save_decisions(root)
    save_collision(root)
    buffer_commands(root)
    close_buffers(root)
    nested_splits(root)
    window_lifecycle(root)
    hidden_window_buffers(root)
    shared_view_mapping(root)
    closed_view_history(root)
    kill_shared_buffer(root)
    resizing_windows(root)
    resizing_prompt(root)
    linear_regions(root)
    shared_regions(root)
    scrolling_regions(root)
    line_kills(root)
    word_kills(root)
    linear_ring(root)
    cross_buffer_yank(root)
    print("Terminal PTY suite: 46 cases passed, 0 skipped.")


if __name__ == "__main__":
    main()
