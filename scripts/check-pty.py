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
PASTE_ON = b"\x1b[?2004h"
PASTE_OFF = b"\x1b[?2004l"


def attach():
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


class Session:
    def __init__(self, command, colors=False):
        self.master, self.slave = pty.openpty()
        self.before = termios.tcgetattr(self.slave)
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
        self.output = bytearray()
        self.deadline = time.monotonic() + 8
        environment = dict(os.environ, TERM="xterm-256color")
        if colors:
            environment["NO_COLOR"] = ""
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
        assert PASTE_ON in session.output, bytes(session.output)
        os.write(session.master, b"\x18\x03")
        session.finish(0)
        assert SHOW in session.output and LEAVE in session.output, bytes(session.output)
        assert session.output.index(SHOW) < session.output.index(LEAVE)
        assert session.output.index(PASTE_OFF) < session.output.index(LEAVE)
        print("PASS startup and ordinary exit")
    finally:
        session.close()


def failures(root):
    binary = root / "target/debug/examples/terminal-probe"
    cases = [("raw", 1), ("screen", 1), ("paste", 1), ("paste-stop", 1), ("cursor-stop", 1), ("attributes-stop", 1), ("screen-stop", 1), ("restore", 1), ("entered", 1), ("io", 1), ("unwind", 101)]
    for mode, expected in cases:
        session = Session([str(binary), mode])
        try:
            session.ready(b"probe-ready")
            os.write(session.master, b"!")
            if mode == "restore":
                session.wait(lambda: b"probe-cleanup-pending" in session.output)
                assert termios.tcgetattr(session.slave) == session.before
                assert session.process.poll() is None
                os.write(session.master, b"!\n")
            session.finish(expected)
            if mode == "raw":
                assert all(code not in session.output for code in (ENTER, LEAVE, HIDE, SHOW))
            else:
                assert ENTER in session.output and LEAVE in session.output, bytes(session.output)
                assert SHOW in session.output, bytes(session.output)
                assert b"\x1b[0m" in session.output
                assert session.output.index(b"\x1b[0m") < session.output.index(LEAVE)
                if mode == "attributes-stop":
                    # Other obligations still run; Drop retries the failed reset.
                    assert session.output.count(b"\x1b[0m") >= 2
                    assert b"reset display attributes" in session.output
                else:
                    assert session.output.rfind(b"\x1b[0m") < session.output.rfind(LEAVE)
            assert b"injected" in session.output, bytes(session.output)
            if mode in ("paste", "paste-stop", "cursor-stop", "attributes-stop", "screen-stop", "restore", "entered", "io", "unwind"):
                assert PASTE_OFF in session.output, bytes(session.output)
            if mode == "restore":
                for operation in (b"disable bracketed paste", b"show cursor",
                                  b"reset display attributes", b"leave alternate screen"):
                    assert operation in session.output, bytes(session.output)
                assert session.output.rfind(PASTE_OFF) < session.output.rfind(SHOW)
                assert session.output.rfind(SHOW) < session.output.rfind(b"\x1b[0m")
                assert session.output.rfind(b"\x1b[0m") < session.output.rfind(LEAVE)
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
        refusal = b"y=save, n=discard, C-g=cancel"
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
        session.wait(lambda: b"y=save, n=discard, C-g=cancel"
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
        session.wait(lambda: b"y=save, n=discard, C-g=cancel"
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
            group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit")
            path.parent.mkdir()
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C3 Text")
            assert path.read_bytes() == b"ab"
            path.write_bytes(b"external")
            group_step(session, b"!", b"** RW L1 C4 Text")
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            group_step(session, b"n", b"Save canceled")
            assert path.read_bytes() == b"external"
            group_step(session, b"?", b"** RW L1 C5 Text", b"\x1b[1;1Hab!?")
            group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit")
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
            group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit")
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
                group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
                group_step(session, b"\x07", b"Quit")
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


def copy_rotation_lines(session, column):
    group_step(session, b"\x00\x05", b"-- RW L1 C2 Text")
    group_step(session, b"\x1bw", b"Region copied")
    group_step(session, b"\x0e\x01\x00\x05", f"-- RW L2 C{column} Text".encode())
    group_step(session, b"\x1bw", b"Region copied")
    group_step(session, b"\x07\x1b>", b"-- RW L3 C1 Text")


def yank_rotations(root):
    # Real M-y reports rotate unequal entries, undo a whole replacement and
    # refuse reuse after undo; a fresh yank captures only its own new insertion.
    with tempfile.TemporaryDirectory(prefix="rev-pty-yank-pop-") as directory:
        path = Path(directory) / "rotate.txt"
        source = "A\nlong 界\n".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> rotate.txt -- RW L1 C1 Text")
            group_step(session, b"\x1by", b"No current yank to replace")
            copy_rotation_lines(session, 8)
            group_step(session, b"<>\x02\x19", b"** RW L3 C9 Text", "<long 界>".encode())
            group_step(session, b"\x1by", b"** RW L3 C3 Text", b"<A>")
            group_step(session, b"\x1by", b"** RW L3 C9 Text", "<long 界>".encode())
            group_step(session, b"\x18u", b"** RW L3 C3 Text", b"<A>")
            group_step(session, b"\x1by", b"No current yank to replace", b"<A>")
            group_step(session, b"\x19", b"** RW L3 C10 Text", "<Along 界>".encode())
            group_step(session, b"\x1by", b"** RW L3 C4 Text", b"<AA>")
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == source + b"<AA>"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS M-y unequal Unicode rotation, whole undo, invalidated capture and fresh yank range")
        finally:
            session.close()


def yank_view_invalidation(root):
    # A shared-buffer edit from another pane invalidates the initiating pane's
    # capture; undoing that edit must not revive old replacement authority.
    with tempfile.TemporaryDirectory(prefix="rev-pty-yank-view-") as directory:
        path = Path(directory) / "shared.txt"
        source = b"A\nLONG\n"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> shared.txt -- RW L1 C1 Text")
            copy_rotation_lines(session, 5)
            group_step(session, b"\x183", b"> shared.txt -- RW L3 C1 Text", b"|A ")
            group_step(session, b"<>\x02\x19", b"> shared.txt ** RW L3 C6 Text", b"<LONG>")
            group_step(session, b"\x1by", b"> shared.txt ** RW L3 C3 Text", b"<A>")
            assert b"  shared.txt ** RW L3 C4 Text" in last_frame(session)
            group_step(session, b"\x18oX", b"|> shared.txt ** RW L3 C5 Text", b"<A>X")
            group_step(session, b"\x18o\x1by", b"No current yank to replace", b"<A>X")
            group_step(session, b"\x18u", b"> shared.txt ** RW L3 C3 Text", b"\x1b[3;1H<A> ")
            group_step(session, b"\x1by", b"No current yank to replace", b"<A>")
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == source + b"<A>"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS shared-view pop mapping, other-pane edit invalidation and refusal after undo")
        finally:
            session.close()


def terminal_key_reports(root):
    # Exercise the actual legacy and enhanced reports, including 0x1f's C-7
    # library alias, without requiring an emulator to infer punctuation keys.
    with tempfile.TemporaryDirectory(prefix="rev-pty-key-reports-") as directory:
        path = Path(directory) / "keys.txt"
        source = b"abc\nlast"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> keys.txt -- RW L1 C1 Text")
            for undo in [b"\x1f", b"\x1b[47;5u", b"\x1b[95;5u", b"\x18u"]:
                group_step(session, b"X", b"** RW L1 C2 Text", b"Xabc")
                group_step(session, undo, b"-- RW L1 C1 Text", b"\x1b[1;1Habc ")
            group_step(session, b"\x1b[32;5u", b"Mark set")
            group_step(session, b"\x06", b"L1 C2 Text", b"\x1b[7ma\x1b[27mbc")
            group_step(session, b"\x07", b"Quit")
            group_step(session, b"\x1b>", b"L2 C5 Text")
            group_step(session, b"\x1b<", b"L1 C1 Text")
            group_step(session, b"\x18r\x1bw", b"Rectangle is not active")
            group_step(session, b"\x18r\x07", b"Quit")
            group_step(session, b"\x1b[999~\x0c", b"-- RW L1 C1 Text", b"\x1b[1;1Habc ")
            group_step(session, b"\x1bq", b"Unbound key")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS legacy/enhanced undo, C-SPC, M-</M->, nested chords and unknown escape isolation")
        finally:
            session.close()


def escape_deadlines(root):
    # Separate writes expose an actual pending Escape event. Observe its frame
    # before the suffix, resize or cancellation; idle expiry needs no sleep.
    with tempfile.TemporaryDirectory(prefix="rev-pty-escape-") as directory:
        path = Path(directory) / "escape.txt"
        source = b"abc\nlast"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> escape.txt -- RW L1 C1 Text")
            group_step(session, b"\x1b", b"ESC")
            group_step(session, b">", b"L2 C5 Text")
            group_step(session, b"\x1b", b"ESC")
            group_step(session, b"<", b"L1 C1 Text")
            group_step(session, b"\x1b", b"ESC")
            resize_surface(session, 7, 5, True)
            group_step(session, b"\x07", b"Quit")
            resize_surface(session, 80, 24, True)
            group_step(session, b"\x1b", b"ESC")
            session.output.clear()
            session.wait(lambda: frame_contains(session.output, b"Escape", None))
            group_step(session, b"f", b"** RW L1 C2 Text", b"fabc")
            group_step(session, b"\x18u", b"-- RW L1 C1 Text")
            group_step(session, b"\x18r", b"C-x r")
            group_step(session, b"\x1b", b"ESC")
            group_step(session, b"w", b"Rectangle is not active")
            group_step(session, b"\x18\x06", b"Find file:")
            group_step(session, b"\x1b", b"ESC")
            group_step(session, b"q", b"Unbound prompt key")
            group_step(session, b"\x07", b"Quit")
            group_step(session, b"X", b"** RW L1 C2 Text", b"Xabc")
            group_step(session, b"\x1b", b"ESC")
            group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit")
            group_step(session, b"\x18u", b"-- RW L1 C1 Text")
            group_step(session, b"\x1b", b"ESC")
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert path.read_bytes() == source
            print("PASS separate Escape Meta, finite idle expiry, resize/cancel, focused input and guarded exit")
        finally:
            session.close()


def paste_wire(payload):
    return b"\x1b[200~" + payload + b"\x1b[201~"


def bracketed_paste(root):
    # Real terminal framing must deliver one large literal edit, even with
    # command-looking bytes, mixed separators and a pending nested prefix.
    with tempfile.TemporaryDirectory(prefix="rev-pty-paste-") as directory:
        path = Path(directory) / "paste.txt"
        source = b"\xef\xbb\xbf<>\r\nlast"
        path.write_bytes(source)
        payload = ("αe\u0301界👩\u200d💻\t\r\nline\n" * 256).encode()
        payload += b"PASTE-END\x18\x03\x1b[31m\x07"
        assert len(payload) > 4096
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> paste.txt -- RW L1 C1 Text")
            assert PASTE_ON in session.output
            group_step(session, b"\x06", b"-- RW L1 C2 Text")
            group_step(session, paste_wire(b"ok\0bad"), b"Paste contains NUL", b"-- RW L1 C2 Text")
            group_step(session, b"\x18r", b"C-x r")
            group_step(session, paste_wire(payload), b"** RW L513 ", b"PASTE-END^X^C^[[31m^G>")
            assert b"\x1b[31m" not in last_frame(session)
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L513 ")
            assert path.read_bytes() == b"\xef\xbb\xbf<" + payload + b">\r\nlast"
            group_step(session, b"\x18u", b"** RW L1 C2 Text", b"\x1b[1;1H<> ")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C2 Text")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert session.output.rfind(PASTE_OFF) < session.output.rfind(LEAVE)
            assert PASTE_OFF in session.output
            print("PASS large bracketed Unicode/control paste, exact BOM/mixed-newline save and one-step undo")
        finally:
            session.close()


def shared_paste(root):
    # Prompt paste retains focus and can be canceled. Document paste maps both
    # pane points, ignores embedded C-x/C-g, and undoes from the passive pane.
    with tempfile.TemporaryDirectory(prefix="rev-pty-paste-views-") as directory:
        path = Path(directory) / "panes.txt"
        source = b"abc\nlast"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> panes.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x183", b"> panes.txt -- RW L1 C2 Text", b"|")
            group_step(session, b"\x18o\x1b>", b"|> panes.txt -- RW L2 C5 Text")
            group_step(session, b"\x18o", b"> panes.txt -- RW L1 C2 Text")
            group_step(session, b"\x18\x06", b"Find file:")
            group_step(session, paste_wire(str(path).encode()), b"Find file:", b"panes.txt")
            group_step(session, paste_wire(b"bad\ntext"), b"Control character", b"panes.txt")
            group_step(session, b"\x07", b"Quit", b"-- RW L1 C2 Text")
            group_step(session, b"\x18r", b"C-x r")
            group_step(session, paste_wire("x\r\n界\x18\x03\x07".encode()), b"> panes.txt ** RW L2 C9 Text", "界^X^C^Gbc".encode())
            assert b"  panes.txt ** RW L3 C5 Text" in last_frame(session)
            group_step(session, b"\x07", b"Quit", b"** RW L2 C9 Text")
            group_step(session, b"\x18o", b"|> panes.txt ** RW L3 C5 Text")
            group_step(session, b"\x18u", b"|> panes.txt -- RW L2 C5 Text", b"\x1b[1;1Habc ")
            assert b"  panes.txt -- RW L1 C2 Text" in last_frame(session)
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert PASTE_OFF in session.output and LEAVE in session.output
            print("PASS prompt paste refusal/cancel, literal command bytes and one undo across shared views")
        finally:
            session.close()


def named_step(session, name, marker, content=None):
    group_step(session, b"\x1bx", b"M-x ")
    group_step(session, name.encode(), b"M-x " + name.encode())
    group_step(session, b"\r", marker, content)


def command_palette(root):
    # Actual Meta-x, query keys, Tab cycling and history reach real shared
    # handlers; unknown/canceled names must never become underlying text.
    with tempfile.TemporaryDirectory(prefix="rev-pty-commands-") as directory:
        path = Path(directory) / "cmd.txt"
        source = b"abc\nlast"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> cmd.txt -- RW L1 C1 Text")
            group_step(session, b"\x1bx", b"M-x ", b"backward-delete-char")
            group_step(session, b"split-window", b"M-x split-window", b"split-window-right")
            resize_surface(session, 3, 1, True)
            resize_surface(session, 0, 0, False)
            resize_surface(session, 80, 24, True)
            group_step(session, b"\t", b"M-x split-window-below")
            group_step(session, b"\t", b"M-x split-window-right")
            group_step(session, b"\r", b"> cmd.txt -- RW L1 C1 Text", b"|")
            named_step(session, "other-window", b"|> cmd.txt -- RW L1 C1 Text")
            named_step(session, "newline", b"|> cmd.txt ** RW L2 C1 Text", b"abc")
            named_step(session, "undo", b"|> cmd.txt -- RW L1 C1 Text")
            named_step(session, "undo-redo", b"|> cmd.txt ** RW L2 C1 Text")
            named_step(session, "undo", b"|> cmd.txt -- RW L1 C1 Text")
            group_step(session, b"\x1bx", b"M-x ")
            group_step(session, b"absent-command", b"M-x absent-command", b"No matching commands")
            group_step(session, b"\t", b"No matching command")
            group_step(session, b"\r", b"Unknown command", b"-- RW L1 C1 Text")
            group_step(session, b"\x1bx", b"M-x ")
            group_step(session, b"\x1b[A", b"M-x undo  ")
            group_step(session, b"\x07", b"Quit")
            group_step(session, b"\x1bxkill-", b"M-x kill-")
            group_step(session, b"\x07", b"Quit", b"-- RW L1 C1 Text")
            named_step(session, "delete-other-windows", b"> cmd.txt -- RW L1 C1 Text")
            assert b"|" not in last_frame(session)
            assert path.read_bytes() == source
            group_step(session, b"\x1bxsave-buffers-kill-terminal", b"M-x save-buffers-kill-terminal")
            os.write(session.master, b"\r")
            session.finish(0)
            assert PASTE_OFF in session.output and LEAVE in session.output
            print("PASS M-x candidate cycling/history, resize/cancel, named splits/edit/undo/redo and exit")
        finally:
            session.close()


def command_arguments(root):
    # Named file commands continue through the real focused path handlers, save
    # exact BOM/CRLF bytes and reopen canonical buffers without leaking queries.
    with tempfile.TemporaryDirectory(prefix="rev-pty-command-args-") as directory:
        base = Path(directory)
        source = base / "source.txt"
        target = base / "copy.txt"
        canceled = base / "unused.txt"
        original = "\ufeffé\r\nlast".encode()
        source.write_bytes(original)
        session = Session([str(root / "target/debug/rev")])
        try:
            session.ready(b"C-x C-c: exit")
            named_step(session, "find-file", b"Find file:")
            group_step(session, paste_wire(str(source).encode()), b"Find file:", b"source.txt")
            group_step(session, b"\r", b"> source.txt -- RW L1 C1 Text", "é".encode())
            named_step(session, "write-file", b"Write file:")
            group_step(session, paste_wire(str(canceled).encode()), b"Write file:", b"unused.txt")
            group_step(session, b"\x07", b"Quit", b"> source.txt -- RW L1 C1 Text")
            assert not canceled.exists() and source.read_bytes() == original
            group_step(session, b"\x05!", b"** RW L1 C3 Text", "é!".encode())
            named_step(session, "save-buffer", b"Saved ", b"-- RW L1 C3 Text")
            expected = "\ufeffé!\r\nlast".encode()
            assert source.read_bytes() == expected
            named_step(session, "write-file", b"Write file:")
            group_step(session, paste_wire(str(target).encode()), b"Write file:", b"copy.txt")
            group_step(session, b"\r", b"Saved ", b"> copy.txt -- RW L1 C3 Text")
            assert target.read_bytes() == expected
            named_step(session, "find-file", b"Find file:")
            group_step(session, paste_wire(str(target).encode()), b"Find file:", b"copy.txt")
            group_step(session, b"\r", b"> copy.txt -- RW L1 C3 Text", "é!".encode())
            named_step(session, "list-buffers", b"Buffers: arrows move")
            group_step(session, b"\x07", b"Quit", b"> copy.txt -- RW L1 C3 Text")
            assert sorted(path.name for path in base.iterdir()) == ["copy.txt", "source.txt"]
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert PASTE_OFF in session.output and LEAVE in session.output
            print("PASS named find/write/save/buffer handlers, canceled path, exact saved bytes and canonical reopen")
        finally:
            session.close()


def focused_arguments(root):
    # Canceled M-x must retain the intended vertical column and recenter cycle.
    # Edited/canceled path drafts stay focused until a fresh argument accepts.
    with tempfile.TemporaryDirectory(prefix="rev-pty-focus-") as directory:
        base = Path(directory)
        path = base / "source.txt"
        other = base / "other.txt"
        unused = base / "unused.txt"
        original = b"0123456789\nx\n0123456789\nend"
        path.write_bytes(original)
        other.write_bytes("\ufeffé\r\nlast".encode())
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> source.txt -- RW L1 C1 Text")
            group_step(session, b"\x06" * 8, b"L1 C9 Text")
            group_step(session, b"\x0e", b"L2 C2 Text")
            group_step(session, b"\x0c", b"L2 C2 Text", b"\x1b[2;2H" + SHOW)
            group_step(session, b"\x1bxfind-", b"M-x find-")
            group_step(session, b"\x07", b"Quit", b"L2 C2 Text")
            group_step(session, b"\x0c", b"L2 C2 Text", b"\x1b[1;2H" + SHOW)
            group_step(session, b"\x0e", b"L3 C9 Text")
            group_step(session, b"\x18\x06", b"Find file:")
            group_step(session, b"\x1bq", b"Unbound prompt key")
            group_step(session, paste_wire(str(other).encode() + b"BAD"), b"Find file:", b"other.txtBAD")
            assert b"Unbound prompt key" not in last_frame(session)
            group_step(session, b"\t", b"Enter a path; C-g cancels", b"other.txtBAD")
            group_step(session, b"\x02" * 3 + b"\x04" * 3, b"other.txt  ")
            group_step(session, b"\x01\x06\x02\x05X\x08", b"other.txt  ")
            group_step(session, b"\x07", b"Quit", b"> source.txt -- RW L3 C9 Text")
            find_path(session, other, b"> other.txt -- RW L1 C1 Text", "é".encode())
            group_step(session, b"\x18\x17", b"Write file:")
            group_step(session, paste_wire(str(unused).encode()), b"unused.txt")
            group_step(session, b"\x07", b"Quit", b"> other.txt -- RW L1 C1 Text")
            assert not unused.exists()
            group_step(session, b"!", b"** RW L1 C2 Text", "!é".encode())
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C2 Text")
            assert path.read_bytes() == original
            assert other.read_bytes() == "\ufeff!é\r\nlast".encode()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert PASTE_OFF in session.output and LEAVE in session.output
            print("PASS canceled M-x navigation, Emacs path editing, feedback recovery and canceled find/write")
        finally:
            session.close()


def focused_close(root):
    # Each nested close stage can be canceled. A subsequent save must keep the
    # original shared buffer open, with independent pane points and no old consent.
    with tempfile.TemporaryDirectory(prefix="rev-pty-close-focus-") as directory:
        base = Path(directory)
        occupied = base / "occupied.txt"
        fresh = base / "fresh.txt"
        occupied.write_bytes(b"keep")
        session = Session([str(root / "target/debug/rev")])
        try:
            session.ready(b"C-x C-c: exit")
            group_step(session, b"draft", b"** RW L1 C6 Text", b"draft")
            group_step(session, b"\x183", b"> *scratch* ** RW L1 C6 Text", b"|")
            group_step(session, b"\x18o\x01", b"|> *scratch* ** RW L1 C1 Text")
            group_step(session, b"\x18o", b"> *scratch* ** RW L1 C6 Text")
            for stage in ["name", "question", "destination", "overwrite"]:
                group_step(session, b"\x18k", b"Kill buffer:")
                if stage != "name":
                    group_step(session, b"\r", b"y=save, n=discard")
                if stage in ("destination", "overwrite"):
                    group_step(session, b"y", b"Write file:")
                    group_step(session, paste_wire(str(occupied).encode()), b"occupied.txt")
                if stage == "overwrite":
                    group_step(session, b"\r", b"Overwrite occupied destination")
                    resize_surface(session, 3, 1, True)
                    resize_surface(session, 0, 0, False)
                    resize_surface(session, 80, 24, True)
                group_step(session, b"\x07", b"Quit", b"> *scratch* ** RW L1 C6 Text")
                assert occupied.read_bytes() == b"keep" and not fresh.exists()
                assert b"  *scratch* ** RW L1 C1 Text" in last_frame(session)
            group_step(session, b"y", b"** RW L1 C7 Text", b"drafty")
            group_step(session, b"\x18\x17", b"Write file:")
            group_step(session, paste_wire(str(fresh).encode()), b"fresh.txt")
            group_step(session, b"\r", b"Saved ", b"> fresh.txt -- RW L1 C7 Text")
            assert b"  fresh.txt -- RW L1 C1 Text" in last_frame(session)
            assert b"|drafty" in last_frame(session)
            assert fresh.read_bytes() == b"drafty" and occupied.read_bytes() == b"keep"
            assert sorted(path.name for path in base.iterdir()) == ["fresh.txt", "occupied.txt"]
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            assert PASTE_OFF in session.output and LEAVE in session.output
            print("PASS cancel at every close stage, resize during consent and later save without closing shared text")
        finally:
            session.close()


def forward_search(root):
    # Real incremental key reports and next-match search reveal a distant line;
    # failure keeps point, deletion recovers, and accepted search leaves disk clean.
    with tempfile.TemporaryDirectory(prefix="rev-pty-search-") as directory:
        path = Path(directory) / "search.txt"
        source = b"one cat\n" + b"filler\n" * 45 + b"cat end"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> search.txt -- RW L1 C1 Text")
            group_step(session, b"\x13", b"I-search: ")
            group_step(session, b"c", b"I-search: c", b"-- RW L1 C6 Text")
            group_step(session, b"at", b"I-search: cat", b"\x1b[4mcat\x1b[24m")
            group_step(session, b"\x13", b"-- RW L47 C4 Text", b"I-search: cat")
            assert b"\x1b[4mcat\x1b[24m" in last_frame(session)
            group_step(session, b"z", b"No match", b"-- RW L47 C4 Text")
            assert b"I-search: catz" in last_frame(session)
            assert b"\x1b[4m" not in last_frame(session)
            group_step(session, b"\x7f", b"I-search: cat", b"\x1b[4mcat\x1b[24m")
            assert b"No match" not in last_frame(session)
            group_step(session, b"\r", b"-- RW L47 C4 Text")
            assert b"I-search:" not in last_frame(session)
            assert b"\x1b[4m" not in last_frame(session)
            group_step(session, b"\x18u", b"Nothing to undo", b"-- RW L47 C4 Text")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS incremental forward search, distant continuation, no-match recovery and unchanged text/history")
        finally:
            session.close()


def multiline_search(root):
    # Named search accepts a bracketed CRLF/Unicode query across real rope chunks;
    # selected-pane styling and focused query ownership survive tiny geometry.
    with tempfile.TemporaryDirectory(prefix="rev-pty-search-lines-") as directory:
        path = Path(directory) / "unicode.txt"
        source = ("filler text\r\n" * 64 + "e\u0301\t界\r\nend\nlast").encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> unicode.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"|", b"> unicode.txt -- RW L1 C1 Text")
            group_step(session, b"\x18o", b"|> unicode.txt -- RW L1 C1 Text")
            named_step(session, "isearch-forward", b"I-search: ")
            group_step(session, paste_wire("界\r\nend".encode()), b"I-search:", b"|> unicode.txt -- RW L66 C4 Text")
            frame = last_frame(session)
            assert b"\x1b[4m" + "界".encode() in frame
            assert b"\x1b[4mend\x1b[24m" in frame
            assert b"  unicode.txt -- RW L1 C1 Text" in frame
            for width, height in [(0, 0), (1, 1), (3, 3), (80, 24)]:
                resize_surface(session, width, height, True)
            assert b"I-search:" in last_frame(session)
            assert b"|> unicode.txt -- RW L66 C4 Text" in last_frame(session)
            group_step(session, b"\x07", b"Quit", b"|> unicode.txt -- RW L1 C1 Text")
            assert b"I-search:" not in last_frame(session)
            assert b"\x1b[4m" not in last_frame(session)
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS named multiline Unicode search, pane isolation, resize/cancel and exact disk bytes")
        finally:
            session.close()


def backward_search(root):
    # A real C-r query crosses many lines, stops at BOF and reverses after a
    # failure; accepted search owns no stale cancellation origin or undo entry.
    with tempfile.TemporaryDirectory(prefix="rev-pty-backward-") as directory:
        path = Path(directory) / "back.txt"
        source = b"cat\n" + b"filler\n" * 45 + b"CAT cat"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> back.txt -- RW L1 C1 Text")
            group_step(session, b"\x1b>", b"-- RW L47 C8 Text")
            group_step(session, b"\x12", b"I-search backward:")
            group_step(session, b"cat", b"-- RW L47 C5 Text", b"I-search backward: cat")
            group_step(session, b"\x12", b"-- RW L47 C1 Text", b"\x1b[4mCAT\x1b[24m")
            group_step(session, b"\x12", b"-- RW L1 C1 Text", b"\x1b[4mcat\x1b[24m")
            group_step(session, b"\x12", b"No match", b"-- RW L1 C1 Text")
            assert b"\x1b[4m" not in last_frame(session)
            group_step(session, b"\x13", b"I-search: cat", b"-- RW L1 C4 Text")
            group_step(session, b"\x13", b"-- RW L47 C4 Text", b"\x1b[4mCAT\x1b[24m")
            group_step(session, b"\r", b"-- RW L47 C4 Text")
            group_step(session, b"\x07", b"Quit", b"-- RW L47 C4 Text")
            group_step(session, b"\x18u", b"Nothing to undo")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS backward search, smart-case continuation, BOF failure, direction recovery and acceptance")
        finally:
            session.close()


def smart_search(root):
    # Unicode expansions must highlight original text; cancel after vertical
    # reveal and resize restores the same document viewport, not just its point.
    with tempfile.TemporaryDirectory(prefix="rev-pty-smart-search-") as directory:
        path = Path(directory) / "smart.txt"
        source = ("İ\tStraße\n" + "filler\n" * 45 + "STRASSE\nend").encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> smart.txt -- RW L1 C1 Text")
            group_step(session, b"\x1b>", b"-- RW L48 C4 Text")
            original = last_frame(session).split(b"\x1b[23;1H")[0]
            named_step(session, "isearch-backward", b"I-search backward:")
            group_step(session, b"strasse", b"-- RW L47 C1 Text")
            group_step(session, b"\x12", b"-- RW L1 C5 Text", b"\x1b[4m" + "Straße".encode() + b"\x1b[24m")
            group_step(session, b"\x01" + b"\x04" * 7 + paste_wire("i\u0307".encode()), b"-- RW L1 C1 Text", b"\x1b[4m" + "İ".encode() + b"\x1b[24m")
            for width, height in [(0, 0), (1, 1), (80, 24)]:
                resize_surface(session, width, height, True)
            group_step(session, b"\x07", b"Quit", b"-- RW L48 C4 Text")
            assert last_frame(session).split(b"\x1b[23;1H")[0] == original
            group_step(session, b"\x1b<", b"-- RW L1 C1 Text")
            group_step(session, b"\x13", b"I-search:")
            group_step(session, b"STRASSE", b"-- RW L47 C8 Text", b"\x1b[4mSTRASSE\x1b[24m")
            group_step(session, b"\r", b"-- RW L47 C8 Text")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS Unicode fold/source ranges, named backward search, exact viewport cancel and uppercase policy")
        finally:
            session.close()


def query_replace(root):
    # Real M-%, literal arguments and fresh per-occurrence consent reach one
    # checked edit; skip/quit and undo/redo keep unaccepted text and disk intact.
    with tempfile.TemporaryDirectory(prefix="rev-pty-replace-") as directory:
        path = Path(directory) / "replace.txt"
        source = b"cat cat cat"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> replace.txt -- RW L1 C1 Text")
            group_step(session, b"\x1b%", b"Query replace: ")
            group_step(session, b"cat", b"Query replace: cat")
            group_step(session, b"\r", b"Replace with: ")
            group_step(session, paste_wire("界".encode()), b"Replace with: " + "界".encode())
            group_step(session, b"\r", b"Replace occurrence? [y/n/q]", b"\x1b[4mcat\x1b[24m")
            group_step(session, paste_wire(b"y"), b"Paste is unavailable", b"-- RW L1 C4 Text")
            group_step(session, b"\r", b"Answer y, n or q", b"-- RW L1 C4 Text")
            group_step(session, b"y", b"1 replaced", b"** RW L1 C7 Text")
            group_step(session, b"n", b"1 replaced", b"** RW L1 C11 Text")
            group_step(session, b"q", b"Quit", b"** RW L1 C11 Text")
            assert path.read_bytes() == source
            group_step(session, b"\x18u", b"-- RW L1 C4 Text", source)
            named_step(session, "undo-redo", b"** RW L1 C3 Text", "界 cat cat".encode())
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C3 Text")
            assert path.read_bytes() == "界 cat cat".encode()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS M-% fresh consent, skip/quit, literal replacement, independent undo/redo and exact save")
        finally:
            session.close()


def replace_lines(root):
    # Named replacement accepts multiline CRLF queries and literal LF/tab data;
    # resize and shared-view history preserve consent and source file format.
    with tempfile.TemporaryDirectory(prefix="rev-pty-replace-lines-") as directory:
        path = Path(directory) / "lines.txt"
        source = "\ufeff界\r\nend | 界\r\nend | 界\r\nend".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> lines.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"|", b"> lines.txt -- RW L1 C1 Text")
            named_step(session, "query-replace", b"Query replace: ")
            group_step(session, paste_wire("界\r\nend".encode()), b"Query replace:")
            group_step(session, b"\r", b"Replace with: ")
            group_step(session, paste_wire("é\n\t$1".encode()), b"Replace with:")
            group_step(session, b"\r", b"Replace occurrence? [y/n/q]", b"\x1b[4mend\x1b[24m")
            for width, height in [(0, 0), (1, 1), (80, 24)]:
                resize_surface(session, width, height, True)
            assert b"Replace occurrence? [y/n/q]" in last_frame(session)
            group_step(session, b"y", b"1 replaced", b"** RW")
            group_step(session, b"n", b"1 replaced")
            group_step(session, b"\x07", b"Quit")
            assert path.read_bytes() == source
            group_step(session, b"\x18o", b"|> lines.txt ** RW")
            group_step(session, b"\x18u", b"-- RW")
            named_step(session, "undo-redo", b"** RW")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW")
            assert path.read_bytes() == "\ufeffé\n\t$1 | 界\r\nend | 界\r\nend".encode()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS named multiline replacement, resize/cancel, shared-view undo/redo and BOM/CRLF fidelity")
        finally:
            session.close()


def rectangle_copy(root):
    # Cell selection/copy preserves source bytes and the independent linear
    # entry. Reversed endpoints retain the same highlighted Unicode block.
    with tempfile.TemporaryDirectory(prefix="rev-pty-rectangle-") as directory:
        path = Path(directory) / "block.txt"
        source = "\ufeff012345\r\na\t界z\r\nQ👩\u200d💻e\u0301X\r\nx\r\n\r\n012345".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> block.txt -- RW L1 C1 Text")
            group_step(session, b"\x00\x06\x1bw", b"Region copied")
            group_step(session, b"\x07\x06\x18 ", b"Rectangle mark set", b"L1 C3 Text")
            group_step(session, b"\x1b>\x02", b"-- RW L6 C6 Text",
                       "a \x1b[7m  界\x1b[27mz".encode())
            frame = last_frame(session)
            assert b"01\x1b[7m234\x1b[27m5" in frame
            assert "Q\x1b[7m👩\u200d💻e\u0301X\x1b[27m".encode() in frame
            assert b"\x1b[4;1Hx " in frame and b"\x1b[5;1H " in frame
            group_step(session, b"\x18r\x1bw", b"Rectangle copied", b"-- RW L6 C6 Text")
            group_step(session, b"\x18\x18", b"-- RW L1 C3 Text",
                       b"01\x1b[7m234\x1b[27m5")
            assert path.read_bytes() == source
            group_step(session, b"\x07\x19", b"** RW L1 C4 Text", b"0102345")
            group_step(session, b"\x18u", b"-- RW L1 C3 Text", b"012345")
            group_step(session, b"\x18u", b"Nothing to undo")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS rectangular tab/Unicode highlighting, reversed copy, independent linear yank and exact source fidelity")
        finally:
            session.close()


def rectangle_scroll(root):
    # Named commands use the real block handler across hidden source rows.
    # Resize suspension and clipped edges preserve the active interaction.
    with tempfile.TemporaryDirectory(prefix="rev-pty-rectangle-scroll-") as directory:
        path = Path(directory) / "rows.txt"
        anchor = "0123456789" * 12
        middle = "0123456789" * 11 + "a\t界e\u0301👩\u200d💻z"
        source = "\n".join([anchor] + [middle] * 68 + [anchor]).encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> rows.txt -- RW L1 C1 Text")
            group_step(session, b"\x05" + b"\x02" * 5, b"-- RW L1 C116 Text")
            named_step(session, "rectangle-mark-mode", b"Rectangle mark set")
            group_step(session, b"\x1b>\x02\x02", b"-- RW L70 C119 Text",
                       "\x1b[7m👩\u200d💻z\x1b[27m".encode())
            named_step(session, "copy-rectangle-as-kill", b"Rectangle copied")
            for width, height in [(0, 0), (3, 3), (8, 5), (80, 24)]:
                resize_surface(session, width, height, True)
            assert b"-- RW L70 C119 Text" in last_frame(session)
            named_step(session, "copy-rectangle-as-kill", b"Rectangle copied")
            group_step(session, b"\x18rq", b"Undefined key sequence")
            group_step(session, b"\x07", b"Quit")
            assert b"\x1b[7m" not in last_frame(session)
            named_step(session, "copy-rectangle-as-kill", b"Rectangle is not active")
            group_step(session, b"\x18u", b"Nothing to undo")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS named rectangle copy across scrolled rows, zero/tiny resize, cancellation and unchanged history/file")
        finally:
            session.close()



def rectangle_kill(root):
    # Every row is one real transaction, including tab fragments and wide
    # clusters. Save/undo/redo preserve exact BOM and CRLF bytes and clean state.
    with tempfile.TemporaryDirectory(prefix="rev-pty-rectangle-kill-") as directory:
        path = Path(directory) / "kill.txt"
        source = "\ufeff012345\r\na\t界z\r\nQ👩\u200d💻e\u0301X\r\nx\r\n\r\n012345".encode()
        expected = "\ufeff015\r\na z\r\nQ\r\nx\r\n\r\n015".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> kill.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x06\x18 ", b"Rectangle mark set")
            group_step(session, b"\x1b>\x02", b"-- RW L6 C6 Text")
            group_step(session, b"\x18rk", b"Rectangle killed", b"** RW L1 C3 Text")
            frame = last_frame(session)
            assert b"\x1b[1;1H015 " in frame and b"\x1b[2;1Ha z " in frame
            assert b"\x1b[3;1HQ " in frame and b"\x1b[7m" not in frame
            assert path.read_bytes() == source
            group_step(session, b"\x18u", b"-- RW L6 C6 Text", "a   界z".encode())
            group_step(session, b"\x18rk", b"Rectangle is not active")
            named_step(session, "undo-redo", b"** RW L1 C3 Text", b"\x1b[2;1Ha z ")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW")
            assert path.read_bytes() == expected
            group_step(session, b"\x18u", b"** RW L6 C6 Text", "Q👩\u200d💻e\u0301X".encode())
            named_step(session, "undo-redo", b"-- RW L1 C3 Text", b"\x1b[1;1H015 ")
            assert path.read_bytes() == expected
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound rectangle kill, all-row undo/redo, partial tabs, whole Unicode and exact BOM/CRLF save state")
        finally:
            session.close()


def shared_rectangle_kill(root):
    # Named removal and passive-pane undo retain original cursor ownership.
    # The untouched later tab stays a tab on disk rather than drawn spaces.
    with tempfile.TemporaryDirectory(prefix="rev-pty-shared-rectangle-") as directory:
        path = Path(directory) / "tabs.txt"
        source = b"\txyz\n01234567\nend\tkeep"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> tabs.txt -- RW L1 C1 Text")
            group_step(session, b"\x183", b"|", b"> tabs.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x06\x06", b"> tabs.txt -- RW L1 C7 Text")
            named_step(session, "rectangle-mark-mode", b"Rectangle mark set")
            group_step(session, b"\x0e\x01\x06\x06", b"> tabs.txt -- RW L2 C3 Text")
            named_step(session, "kill-rectangle", b"Rectangle killed", b"> tabs.txt ** RW L1 C3 Text")
            assert b"  z" in last_frame(session) and b"0167" in last_frame(session)
            group_step(session, b"\x19", b"Kill ring is empty")
            group_step(session, b"\x18o\x18u", b"  tabs.txt -- RW L2 C3 Text",
                       b"|> tabs.txt -- RW L1 C7 Text")
            named_step(session, "undo-redo", b"> tabs.txt ** RW L1 C3 Text")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW")
            assert path.read_bytes() == b"  z\n0167\nend\tkeep"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS named rectangle kill, shared-view undo origin, partial-tab cursor, separate ring and untouched tab bytes")
        finally:
            session.close()



def rectangle_yank(root):
    # A cross-file retained block creates lines/prefixes as one transaction.
    # The destination controls new separators; source BOM/CRLF remains exact.
    with tempfile.TemporaryDirectory(prefix="rev-pty-rectangle-yank-") as directory:
        source = Path(directory) / "source.txt"
        target = Path(directory) / "target.txt"
        original = "\ufeff0AB9\r\nx界z\r\n\r\n0e\u0301X9".encode()
        source.write_bytes(original)
        target.write_bytes(b"\xef\xbb\xbf....")
        expected = "\ufeff...AB.\n   界\n\n   e\u0301X".encode()
        session = Session([str(root / "target/debug/rev"), str(source)])
        try:
            session.ready(b"> source.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x18 ", b"Rectangle mark set")
            group_step(session, b"\x1b>\x02\x18r\x1bw", b"Rectangle copied")
            find_path(session, target, b"> target.txt -- RW L1 C1 Text", b"....")
            group_step(session, b"\x06\x06\x06\x18ry", b"Rectangle yanked", b"** RW L4 C6 Text")
            frame = last_frame(session)
            assert b"\x1b[1;1H...AB. " in frame and "\x1b[2;1H   界".encode() in frame
            assert b"\x1b[3;1H    " in frame
            group_step(session, b"\x18u", b"-- RW L1 C4 Text", b"\x1b[1;1H.... ")
            named_step(session, "undo-redo", b"** RW L4 C6 Text")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW")
            assert target.read_bytes() == expected and source.read_bytes() == original
            group_step(session, b"\x18u", b"** RW L1 C4 Text", b"\x1b[1;1H.... ")
            named_step(session, "undo-redo", b"-- RW L4 C6 Text")
            assert target.read_bytes() == expected
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound cross-file rectangle yank, missing/empty rows, prefix padding, atomic history and exact BOM/newline saves")
        finally:
            session.close()


def shared_rectangle_yank(root):
    # Later target rows snap before touched wide/tab glyphs. Passive-pane undo
    # restores the original initiator and untouched tab bytes stay literal.
    with tempfile.TemporaryDirectory(prefix="rev-pty-shared-yank-") as directory:
        source = Path(directory) / "block.txt"
        target = Path(directory) / "views.txt"
        original = "0AB9\nx界z\n0CD9".encode()
        source.write_bytes(original)
        target.write_bytes("abcde\n界Z\n\tQ".encode())
        session = Session([str(root / "target/debug/rev"), str(source)])
        try:
            session.ready(b"> block.txt -- RW L1 C1 Text")
            group_step(session, b"\x06\x18 ", b"Rectangle mark set")
            group_step(session, b"\x1b>\x02\x18r\x1bw", b"Rectangle copied")
            find_path(session, target, b"> views.txt -- RW L1 C1 Text", b"abcde")
            group_step(session, b"\x183\x06", b"> views.txt -- RW L1 C2 Text", b"|")
            named_step(session, "yank-rectangle", b"Rectangle yanked", b"> views.txt ** RW L3 C3 Text")
            frame = last_frame(session)
            assert "界界Z".encode() in frame and b"CD  Q" in frame
            group_step(session, b"\x18o\x18u", b"  views.txt -- RW L1 C2 Text",
                       b"|> views.txt -- RW L1 C1 Text")
            named_step(session, "undo-redo", b"  views.txt ** RW L3 C3 Text")
            group_step(session, b"\x19", b"Kill ring is empty")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW")
            assert target.read_bytes() == "aABbcde\n界界Z\nCD\tQ".encode()
            assert source.read_bytes() == original
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS named rectangle yank, wide/tab target snapping, shared undo origin, separate ring and untouched tab bytes")
        finally:
            session.close()



def tree_pane(root):
    # The initial resolved parent is selected without a Git directory. Tree
    # focus rejects editing; M-x and bound routes preserve dirty text/history.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-") as directory:
        workspace = Path(directory) / "workspace"
        workspace.mkdir()
        path = workspace / "draft.txt"
        source = "ab界e\u0301\nlast".encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> draft.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused", b"> workspace/")
            frame = last_frame(session)
            assert b"\x1b[7m> workspace/" in frame and b"Tree *" in frame
            assert b"\x1b[1;1H" + SHOW in frame
            group_step(session, b"bad", b"File tree owns input focus", b"** RW L1 C2 Text")
            group_step(session, b"\t", b"Editing", b"** RW L1 C2 Text")
            assert b"\x1b[1;27H" + SHOW in last_frame(session)
            named_step(session, "focus-file-tree", b"File tree focused")
            named_step(session, "toggle-file-tree", b"File tree hidden", b"** RW L1 C2 Text")
            assert b"> workspace/" not in last_frame(session)
            group_step(session, b"\x18u", b"-- RW L1 C1 Text", "\x1b[1;1Hab界e\u0301".encode())
            assert path.read_bytes() == source and not (workspace / ".git").exists()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS collapsed startup root, bound/named tree focus, dirty-text admission and retained undo")
        finally:
            session.close()


def tree_resize(root):
    # Auto-hide releases workspace focus, keeps the root/show preference and
    # restores the pane without stealing focus. Labels cannot emit terminal SGR.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-resize-") as directory:
        workspace = Path(directory) / "界e\u0301\x1b[31m"
        workspace.mkdir()
        path = workspace / "geometry.txt"
        path.write_bytes(b"original")
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> geometry.txt -- RW L1 C1 Text")
            group_step(session, b"\x18tt", b"File tree focused", "界e\u0301^[[31m/".encode())
            assert b"\x1b[31m" not in last_frame(session)
            resize_surface(session, 31, 8, True)
            assert b"Tree *" not in last_frame(session)
            group_step(session, b"X", b"** RW L1 C2 Text")
            resize_surface(session, 32, 8, True)
            assert "界e\u0301".encode() in last_frame(session)
            assert b"\x1b[1;13H" + SHOW in last_frame(session)
            resize_surface(session, 0, 0, False)
            resize_surface(session, 3, 3, True)
            resize_surface(session, 80, 24, True)
            assert b"\x1b[1;27H" + SHOW in last_frame(session)
            named_step(session, "focus-file-tree", b"File tree focused")
            group_step(session, b"\x1bx", b"M-x ")
            group_step(session, b"toggle-file-tree", b"M-x toggle-file-tree")
            group_step(session, b"\x07", b"Quit", b"Tree *")
            assert b"\x1b[1;1H" + SHOW in last_frame(session)
            named_step(session, "toggle-file-tree", b"File tree hidden")
            group_step(session, b"\x18u", b"-- RW L1 C1 Text", b"\x1b[1;1Horiginal")
            assert path.read_bytes() == b"original"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS Unicode/control root clipping, narrow/zero resize restoration and canceled tree M-x focus")
        finally:
            session.close()


def tree_navigation(root):
    # Real arrows/Enter traverse only expanded branches. Cycle links stay
    # leaves; explicit refresh reconciles external changes without editing text.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-nav-") as directory:
        workspace = Path(directory) / "workspace"
        (workspace / "a/inner").mkdir(parents=True)
        (workspace / "closed").mkdir()
        (workspace / "a/inner/deep.txt").write_bytes(b"unopened")
        (workspace / "a/child.txt").write_bytes(b"unopened")
        (workspace / "closed/invisible.txt").write_bytes(b"unopened")
        (workspace / "loop").symlink_to(workspace, target_is_directory=True)
        (workspace / "z.txt").write_bytes(b"old")
        path = workspace / "doc.txt"
        source = b"retained document"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> doc.txt -- RW L1 C1 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x1b[C", b"File tree updated", b"> a/")
            assert b"invisible.txt" not in last_frame(session) and b"deep.txt" not in last_frame(session)
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m  > a/")
            group_step(session, b"\r", b"File tree updated", b"> inner/")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m    > inner/")
            group_step(session, b"\x1b[C", b"File tree updated", b"deep.txt")
            group_step(session, b"\x1b[D", b"File tree updated", b"\x1b[7m    > inner/")
            assert b"deep.txt" not in last_frame(session)
            group_step(session, b"\x1b[D", b"File tree updated", b"\x1b[7m  v a/")
            group_step(session, b"\x1b[D", b"File tree updated", b"\x1b[7m  > a/")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m  > closed/")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m    doc.txt")
            group_step(session, b"\r", b"> doc.txt -- RW L1 C1 Text")
            assert b"Tree *" not in last_frame(session)
            named_step(session, "focus-file-tree", b"File tree focused", b"\x1b[7m    doc.txt")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m  @ loop")
            group_step(session, b"\x1b[C", b"File tree updated", b"\x1b[7m  @ loop")
            (workspace / "a").rename(workspace / "renamed")
            (workspace / "0-new.txt").write_bytes(b"new")
            (workspace / "z.txt").unlink()
            group_step(session, b"\x18tr", b"File tree updated", b"\x1b[7m  @ loop")
            frame = last_frame(session)
            assert b"> renamed/" in frame and b"0-new.txt" in frame and b"z.txt" not in frame
            assert b"invisible.txt" not in frame and b"deep.txt" not in frame
            assert b"> doc.txt -- RW L1 C1 Text" in frame and path.read_bytes() == source
            group_step(session, b"\t", b"Editing")
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound lazy tree traversal, directory toggle/parent transfer, symlink leaf and explicit external refresh")
        finally:
            session.close()


def tree_navigation_resize(root):
    # Selection scrolls above the footer and survives geometry/focus changes.
    # A denied read and deleted selection remain recoverable by real refresh.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-scroll-") as directory:
        workspace = Path(directory) / "long"
        workspace.mkdir()
        for index in range(30):
            (workspace / f"a{index:02}").mkdir()
        path = workspace / "doc.txt"
        path.write_bytes(b"retained")
        locked = workspace / "a29"
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> doc.txt -- RW L1 C1 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            resize_surface(session, 48, 8, True)
            group_step(session, b"\x1b[C", b"File tree updated", b"> a00/")
            group_step(session, b"\x1b[B" * 30, b"File tree updated", b"\x1b[7m  > a29/")
            assert b"\x1b[6;1H" + SHOW in last_frame(session)
            resize_surface(session, 48, 4, True)
            assert b"\x1b[2;1H" + SHOW in last_frame(session)
            resize_surface(session, 31, 8, True)
            resize_surface(session, 48, 8, True)
            named_step(session, "focus-file-tree", b"File tree focused", b"\x1b[7m  > a29/")
            locked.chmod(0)
            group_step(session, b"\x1b[C", b"read directory ", b"\x1b[7m  > a29/")
            locked.chmod(0o700)
            locked.rmdir()
            control = workspace / "界e\u0301\x1b[31m"
            control.write_bytes(b"exact")
            named_step(session, "tree-refresh", b"File tree updated", b"\x1b[7mv long/")
            group_step(session, b"\x1b[B" * 31, b"File tree updated", "\x1b[7m    界e\u0301^[[31m".encode())
            frame = last_frame(session)
            assert b"\x1b[31m" not in frame
            terminal_bounds(frame, 48, 8)
            resize_surface(session, 0, 0, False)
            resize_surface(session, 80, 24, True)
            group_step(session, b"\x07", b"Quit", b"-- RW L1 C1 Text")
            assert path.read_bytes() == b"retained" and control.read_bytes() == b"exact"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS tree selection scrolling/resizing, denied expansion, removed selection fallback and safe child labels")
        finally:
            if locked.exists():
                locked.chmod(0o700)
            session.close()


def tree_creation(root):
    # Creation stays in the tree. Opening uses real canonical ingress, so a
    # later symlink visit recovers dirty text/history rather than rereading disk.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-create-") as directory:
        workspace = Path(directory) / "workspace"
        workspace.mkdir()
        (workspace / "folder").mkdir()
        path = workspace / "work.txt"
        path.write_bytes(b"original")
        (workspace / "alias").symlink_to(path)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> work.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x18tn", b"New file name: ")
            group_step(session, b"new.xyz", b"New file name: new.xyz")
            group_step(session, b"\r", b"Created ", b"\x1b[7m    new.xyz")
            assert b"> work.txt ** RW L1 C2 Text" in last_frame(session)
            assert (workspace / "new.xyz").read_bytes() == b""
            group_step(session, b"\r", b"> new.xyz -- RW L1 C1 Text")
            group_step(session, "界".encode(), b"** RW L1 C3 Text")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C3 Text")
            group_step(session, b"\x18tf", b"File tree focused", b"\x1b[7m    new.xyz")
            group_step(session, b"\x1b[A", b"File tree updated", b"\x1b[7m  @ alias")
            named_step(session, "tree-open", b"> work.txt ** RW L1 C1 Text", b"|!original")
            group_step(session, b"\x18u", b"> work.txt -- RW L1 C1 Text", b"|original")
            group_step(session, b"\x18tf", b"File tree focused")
            group_step(session, b"\x1b[A", b"File tree updated", b"\x1b[7m  > folder/")
            group_step(session, b"\x1b[A", b"File tree updated", b"\x1b[7mv workspace/")
            named_step(session, "tree-create-directory", b"New directory name: ")
            group_step(session, b"made\r", b"Created ", b"\x1b[7m  > made/")
            group_step(session, b"\x18tn", b"New file name: ")
            group_step(session, b"inside.txt\r", b"Created ", b"\x1b[7m      inside.txt")
            group_step(session, b"\r", b"> inside.txt -- RW L1 C1 Text")
            assert (workspace / "new.xyz").read_bytes() == "界".encode()
            assert (workspace / "made/inside.txt").read_bytes() == b""
            assert path.read_bytes() == b"original" and (workspace / "alias").is_symlink()
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound/named tree creation, unsupported Text open/save and canonical dirty-buffer/history reuse")
        finally:
            session.close()


def tree_creation_failures(root):
    # Invalid/conflicting/canceled names preserve disk and text. Captured parent
    # paths reject symlink replacement and survive focused prompt resizing.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-name-") as directory:
        workspace = Path(directory) / "workspace"
        workspace.mkdir()
        blocked = workspace / "blocked"
        blocked.mkdir()
        path = workspace / "keep.txt"
        path.write_bytes(b"exact")
        (workspace / "occupied").symlink_to(workspace / "missing")
        moved = workspace / "moved"
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> keep.txt -- RW L1 C1 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x18tn", b"New file name: ")
            group_step(session, b"canceled\x07", b"Quit", b"Tree *")
            assert not (workspace / "canceled").exists()
            for name in [b"../escape", b"keep.txt", b"occupied"]:
                group_step(session, b"\x18tn", b"New file name: ")
                group_step(session, name + b"\r", b"create entry ", b"Tree *")
            assert path.read_bytes() == b"exact" and (workspace / "occupied").is_symlink()
            assert not (workspace / "missing").exists() and not (Path(directory) / "escape").exists()
            group_step(session, b"\x1b[C", b"File tree updated", b"> blocked/")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m  > blocked/")
            group_step(session, b"\x18td", b"New directory name: ")
            blocked.rename(moved)
            blocked.symlink_to(moved, target_is_directory=True)
            group_step(session, b"redirected\r", b"create entry ", b"Tree *")
            assert not (moved / "redirected").exists()
            named_step(session, "tree-refresh", b"File tree updated", b"\x1b[7m  @ blocked")
            group_step(session, b"\x1b[A", b"File tree updated", b"\x1b[7m  > moved/")
            moved.chmod(0o500)
            group_step(session, b"\x18tn", b"New file name: ")
            group_step(session, b"denied\r", b"create entry ", b"Tree *")
            moved.chmod(0o700)
            assert not (moved / "denied").exists()
            group_step(session, b"\x18td", b"New directory name: ")
            group_step(session, b"tiny", b"New directory name: tiny")
            resize_surface(session, 31, 8, True)
            resize_surface(session, 48, 8, True)
            group_step(session, b"\r", b"Created ", b"> tiny/")
            assert b"Tree *" not in last_frame(session)
            assert (moved / "tiny").is_dir() and not (workspace / "tiny").exists()
            assert path.read_bytes() == b"exact"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS tree name cancel/conflicts, captured-parent symlink refusal, permission failure and prompt resize publication")
        finally:
            if moved.exists():
                moved.chmod(0o700)
            session.close()


def tree_rename(root):
    # Root/directory rename maps all visited descendants while equal-basename
    # siblings stay distinct. Later normal saves must use the physical new path.
    with tempfile.TemporaryDirectory(prefix="rev-pty-tree-rename-") as directory:
        workspace = Path(directory)
        (workspace / "a/sub").mkdir(parents=True)
        (workspace / "b").mkdir()
        first = workspace / "a/same.txt"
        sibling = workspace / "b/same.txt"
        child = workspace / "a/sub/child.txt"
        first.write_bytes(b"first")
        sibling.write_bytes(b"second")
        child.write_bytes(b"child")
        session = Session([str(root / "target/debug/rev"), str(first)])
        try:
            session.ready(b"> same.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            find_path(session, sibling, b"> same.txt<2> -- RW L1 C1 Text", b"second")
            find_path(session, child, b"> child.txt -- RW L1 C1 Text", b"child")
            group_step(session, b"#", b"** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused", b"> a/")
            group_step(session, b"\x1b[C", b"File tree updated", b"> sub/")
            group_step(session, b"\x18tR", b"Rename to: ")
            group_step(session, b"moved\r", b"Renamed ", b"\x1b[7mv moved/")
            assert b"> child.txt ** RW L1 C2 Text" in last_frame(session)
            group_step(session, b"\t", b"Editing")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C2 Text")
            moved = workspace / "moved"
            assert (moved / "sub/child.txt").read_bytes() == b"#child"
            find_path(session, moved / "same.txt", b"> same.txt ** RW L1 C1 Text", b"!first")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C1 Text")
            group_step(session, b"\x18tf", b"File tree focused")
            group_step(session, b"\x1b[B" * 2, b"File tree updated", b"\x1b[7m    same.txt")
            named_step(session, "tree-rename", b"Rename to: ")
            group_step(session, b"final.txt\r", b"Renamed ", b"\x1b[7m    final.txt")
            assert b"> final.txt -- RW L1 C1 Text" in last_frame(session)
            group_step(session, b"\t", b"Editing")
            group_step(session, "界".encode(), b"** RW L1 C3 Text")
            group_step(session, b"\x18\x13", b"Saved ", b"-- RW L1 C3 Text")
            find_path(session, sibling, b"> same.txt<2> -- RW L1 C1 Text", b"second")
            assert (moved / "final.txt").read_bytes() == "界!first".encode()
            assert not (workspace / "a").exists() and not (moved / "same.txt").exists()
            assert sibling.read_bytes() == b"second"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound root/directory rename, visited descendants, equal-basename owners, named file rename and later new-path saves")
        finally:
            session.close()


def tree_rename_failures(root):
    # Canceled/conflicting/denied rename cannot alter the current association.
    # Renaming a link preserves its referent; stale source captures cannot move
    # a replacement entry under the old prompt's authority.
    with tempfile.TemporaryDirectory(prefix="rev-pty-rename-errors-") as directory:
        workspace = Path(directory) / "workspace"
        workspace.mkdir()
        path = workspace / "old.txt"
        path.write_bytes(b"original")
        (workspace / "taken.txt").write_bytes(b"keep")
        (workspace / "link").symlink_to("old.txt")
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> old.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x1b[C", b"File tree updated")
            group_step(session, b"\x1b[B" * 2, b"File tree updated", b"\x1b[7m    old.txt")
            group_step(session, b"\x18tR", b"Rename to: ")
            group_step(session, b"canceled", b"Rename to: canceled")
            resize_surface(session, 31, 8, True)
            resize_surface(session, 80, 24, True)
            group_step(session, b"\x07", b"Quit", b"> old.txt ** RW L1 C2 Text")
            named_step(session, "focus-file-tree", b"File tree focused")
            for name in [b"../escape", b"taken.txt"]:
                group_step(session, b"\x18tR", b"Rename to: ")
                group_step(session, name + b"\r", b"rename ", b"Tree *")
            group_step(session, b"\x18tR", b"Rename to: ")
            workspace.chmod(0o500)
            group_step(session, b"denied\r", b"rename ", b"Tree *")
            workspace.chmod(0o700)
            assert path.read_bytes() == b"original" and (workspace / "taken.txt").read_bytes() == b"keep"
            assert not (workspace / "denied").exists() and not (workspace / "canceled").exists()
            group_step(session, b"\x1b[A", b"File tree updated", b"\x1b[7m  @ link")
            named_step(session, "tree-rename", b"Rename to: ")
            group_step(session, b"alias\r", b"Renamed ", b"\x1b[7m  @ alias")
            assert b"> old.txt ** RW L1 C2 Text" in last_frame(session)
            group_step(session, b"\t", b"Editing")
            group_step(session, b"\x18\x13", b"Saved ", b"> old.txt -- RW L1 C2 Text")
            assert path.read_bytes() == b"!original" and (workspace / "alias").readlink() == Path("old.txt")
            named_step(session, "focus-file-tree", b"File tree focused")
            group_step(session, b"\x1b[B", b"File tree updated", b"\x1b[7m    old.txt")
            group_step(session, b"\x18tR", b"Rename to: ")
            path.rename(workspace / "held.txt")
            path.write_bytes(b"replacement")
            group_step(session, b"stale\r", b"rename ", b"Tree *")
            assert not (workspace / "stale").exists()
            assert path.read_bytes() == b"replacement" and (workspace / "held.txt").read_bytes() == b"!original"
            group_step(session, b"\t", b"Editing", b"> old.txt -- RW L1 C2 Text")
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS rename cancel/resize, invalid/occupied names, write denial, symlink entry semantics and stale-source rejection")
        finally:
            workspace.chmod(0o700)
            session.close()



def tree_delete(root):
    # Confirmed file removal preserves shared views/history and a canonical
    # absent visit. Only an explicit save recreates the retained document.
    with tempfile.TemporaryDirectory(prefix="rev-pty-delete-") as directory:
        path = Path(directory) / "live.txt"
        path.write_bytes(b"retained")
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> live.txt -- RW L1 C1 Text")
            group_step(session, b"!", b"** RW L1 C2 Text")
            group_step(session, b"\x182", b"> live.txt ** RW L1 C2 Text")
            group_step(session, b"\x18o\x01", b"> live.txt ** RW L1 C1 Text")
            group_step(session, b"\x18o", b"> live.txt ** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x1b[C\x1b[B", b"File tree updated", b"\x1b[7m    live.txt")
            group_step(session, b"\x18tD", b"Delete ", b"Tree *")
            group_step(session, b"z", b"Delete ", b"> live.txt ** RW L1 C2 Text")
            group_step(session, paste_wire(b"y"), b"Delete ")
            assert path.read_bytes() == b"retained"
            group_step(session, b"n", b"Delete canceled")
            named_step(session, "tree-delete", b"Delete ")
            group_step(session, b"y", b"Deleted ", b"> live.txt ** RW L1 C2 Text")
            assert not path.exists()
            assert b"  live.txt ** RW L1 C1 Text" in last_frame(session)
            group_step(session, b"\t", b"Editing")
            group_step(session, b"\x18\x03", b"y=save, n=discard, C-g=cancel")
            group_step(session, b"\x07", b"Quit")
            group_step(session, b"\x18u", b"** RW L1 C1 Text", b"retained")
            named_step(session, "undo-redo", b"** RW L1 C2 Text", b"!retained")
            find_path(session, path, b"> live.txt ** RW L1 C2 Text", b"!retained")
            assert not path.exists() and b"live.txt<2>" not in last_frame(session)
            group_step(session, b"\x18\x13", b"Saved ", b"> live.txt -- RW L1 C2 Text")
            assert path.read_bytes() == b"!retained"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bound/named delete, refused paste/cancel, retained shared views/history, absent revisit and explicit recreation")
        finally:
            session.close()


def tree_delete_decisions(root):
    # Nonempty removal needs two decisions. Cancellation, resize, denied
    # recursive removal and a child symlink cannot discard visiting text.
    with tempfile.TemporaryDirectory(prefix="rev-pty-delete-dir-") as directory:
        base = Path(directory)
        workspace = base / "dir"
        workspace.mkdir()
        path = workspace / "child.txt"
        path.write_bytes(b"child")
        external = base / "outside.txt"
        external.write_bytes(b"outside")
        (workspace / "link").symlink_to(external)
        session = Session([str(root / "target/debug/rev"), str(path)])
        try:
            session.ready(b"> child.txt -- RW L1 C1 Text")
            group_step(session, b"#", b"** RW L1 C2 Text")
            group_step(session, b"\x18tt", b"File tree focused")
            group_step(session, b"\x18tD", b"Delete ")
            group_step(session, b"n", b"Delete canceled")
            named_step(session, "tree-delete", b"Delete ")
            group_step(session, b"y", b"Directory contains children; recursively delete")
            resize_surface(session, 31, 8, True)
            resize_surface(session, 80, 24, True)
            assert b"Directory contains children" in last_frame(session)
            group_step(session, b"n", b"Delete canceled")
            assert path.read_bytes() == b"child" and external.read_bytes() == b"outside"
            named_step(session, "focus-file-tree", b"File tree focused")
            named_step(session, "tree-delete", b"Delete ")
            group_step(session, b"y", b"Directory contains children")
            group_step(session, paste_wire(b"y"), b"Directory contains children")
            group_step(session, b"\x07", b"Quit", b"Tree *")
            assert path.exists()
            group_step(session, b"\x18tD", b"Delete ")
            group_step(session, b"y", b"Directory contains children")
            workspace.chmod(0o500)
            group_step(session, b"y", b"Delete failed; may be partial", b"> child.txt ** RW L1 C2 Text")
            workspace.chmod(0o700)
            assert path.read_bytes() == b"child" and external.read_bytes() == b"outside"
            group_step(session, b"\x18tD", b"Delete ")
            group_step(session, b"y", b"Directory contains children")
            group_step(session, b"y", b"Deleted ", b"> child.txt ** RW L1 C2 Text")
            assert not workspace.exists() and external.read_bytes() == b"outside"
            group_step(session, b"\t", b"Editing")
            group_step(session, b"\x18\x13", b"save: not published", b"#child")
            assert not workspace.exists()
            workspace.mkdir()
            group_step(session, b"\x18\x13", b"Saved ", b"> child.txt -- RW L1 C2 Text")
            assert path.read_bytes() == b"#child" and external.read_bytes() == b"outside"
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS separate recursive consent, cancel/resize, write denial/refresh, symlink referent retention and missing-parent save")
        finally:
            if workspace.exists():
                workspace.chmod(0o700)
            session.close()


def rust_syntax(root):
    # The executable supplies grammar and query without a runtime setup step.
    # Actual foreground sequences survive editing/history without changing bytes.
    with tempfile.TemporaryDirectory(prefix="rev-pty-syntax-") as directory:
        path = Path(directory) / "main.rs"
        source = 'fn main() {\n    let text = "界";\n}\n// note'.encode()
        session = None
        path.write_bytes(source)
        try:
            session = Session([str(root / "target/debug/rev"), str(path)], colors=True)
            session.ready(b"> main.rs -- RW L1 C1 Rust")
            group_step(session, b"\x0c", b"> main.rs -- RW L1 C1 Rust")
            frame = last_frame(session)
            assert b"\x1b[38;5;176mfn\x1b[39m" in frame
            assert b"\x1b[38;5;75mmain\x1b[38;5;250m" in frame
            assert b"\x1b[38;5;114m" + '"界"'.encode() + b"\x1b[38;5;250m" in frame
            assert b"\x1b[38;5;244m// note\x1b[39m" in frame
            group_step(session, b"// ", b"> main.rs ** RW L1 C4 Rust", b"\x1b[38;5;244m// fn main() {")
            group_step(session, b"\x18\x13", b"Saved ", b"> main.rs -- RW L1 C4 Rust")
            assert path.read_bytes() == b"// " + source
            group_step(session, b"\x18u", b"> main.rs ** RW L1 C1 Rust", b"\x1b[38;5;176mfn")
            named_step(session, "undo-redo", b"> main.rs -- RW L1 C4 Rust")
            assert path.read_bytes() == b"// " + source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS bundled Rust foregrounds, Unicode literal, current edit/history styles, exact saves and terminal restoration")
        finally:
            if session is not None:
                session.close()


def syntax_fallback(root):
    # With foregrounds explicitly enabled, unsupported source remains plain.
    # Save-as recognition follows the actual destination in both directions.
    with tempfile.TemporaryDirectory(prefix="rev-pty-syntax-text-") as directory:
        base = Path(directory)
        path = base / "source.txt"
        rust = base / "new.rs"
        plain = base / "plain.txt"
        source = b"fn main() { let count = 42; }"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)], colors=True)
        try:
            session.ready(b"> source.txt -- RW L1 C1 Text")
            group_step(session, b"// ", b"> source.txt ** RW L1 C4 Text")
            assert b"\x1b[38;5;" not in last_frame(session)
            for target, mode in [(rust, b"Rust"), (plain, b"Text")]:
                group_step(session, b"\x18\x17", b"Write file: ")
                group_step(session, paste_wire(str(target).encode()), target.name.encode())
                group_step(session, b"\r", b"Saved ", b"> " + target.name.encode() + b" -- RW L1 C4 " + mode)
                assert target.read_bytes() == b"// " + source
                assert (b"\x1b[38;5;244m" in last_frame(session)) == (mode == b"Rust")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS editable unsupported Text fallback, extension-aware Rust/Text save-as and exact source preservation")
        finally:
            session.close()



def incremental_shared(root):
    # Multibyte CRLF paste, region/search layering and history share one current
    # syntax source across two editing panes; saved bytes remain exact.
    with tempfile.TemporaryDirectory(prefix="rev-pty-incremental-") as directory:
        path = Path(directory) / "main.rs"
        source = 'fn alpha() {\r\n let text = "界";\r\n}\r\n'.encode()
        prefix = '// 🙂\r\n'.encode()
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)], colors=True)
        try:
            session.ready(b"> main.rs -- RW L1 C1 Rust")
            group_step(session, b"\x183", b"> main.rs -- RW L1 C1 Rust", b"|")
            group_step(session, paste_wire(prefix), b"> main.rs ** RW L2 C1 Rust")
            assert last_frame(session).count(b"\x1b[38;5;176mfn") == 2
            group_step(session, b"\x00", b"Mark set")
            group_step(session, b"\x06\x06", b"> main.rs ** RW L2 C3 Rust", b"\x1b[7m")
            group_step(session, b"\x13alpha", b"I-search: alpha", b"> main.rs ** RW L2 C9 Rust")
            frame = last_frame(session)
            assert b"\x1b[38;5;176m" in frame and b"\x1b[38;5;75m" in frame
            assert b"\x1b[7m" in frame and b"\x1b[4m" in frame
            group_step(session, b"\r", b"> main.rs ** RW L2 C9 Rust")
            group_step(session, b"\x18u", b"> main.rs -- RW L1 C1 Rust")
            assert last_frame(session).count(b"\x1b[38;5;176mfn") == 2
            named_step(session, "undo-redo", b"> main.rs ** RW L2 C1 Rust")
            group_step(session, b"\x18\x13", b"Saved ", b"> main.rs -- RW L2 C1 Rust")
            assert path.read_bytes() == prefix + source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS incremental Unicode/CRLF paste, shared panes, region/search foreground layering, undo/redo and exact save")
        finally:
            session.close()


def incremental_rectangles(root):
    # Actual rectangle commands publish disjoint edits. Function colors disappear
    # with removed names and return only after current yank/history publication.
    with tempfile.TemporaryDirectory(prefix="rev-pty-syntax-rectangle-") as directory:
        path = Path(directory) / "rect.rs"
        source = b"fn ab(){}\nfn cd(){}"
        path.write_bytes(source)
        session = Session([str(root / "target/debug/rev"), str(path)], colors=True)
        try:
            session.ready(b"> rect.rs -- RW L1 C1 Rust")
            group_step(session, b"\x06" * 3, b"L1 C4 Rust")
            group_step(session, b"\x18 ", b"Rectangle mark set")
            group_step(session, b"\x0e\x06\x06", b"L2 C6 Rust")
            group_step(session, b"\x18rk", b"Rectangle killed", b"> rect.rs ** RW L1 C4 Rust")
            assert b"\x1b[38;5;75m" not in last_frame(session)
            group_step(session, b"\x18u", b"> rect.rs -- RW L2 C6 Rust")
            assert b"\x1b[38;5;75mab" in last_frame(session) and b"\x1b[38;5;75mcd" in last_frame(session)
            named_step(session, "undo-redo", b"> rect.rs ** RW L1 C4 Rust")
            assert b"\x1b[38;5;75m" not in last_frame(session)
            group_step(session, b"\x18ry", b"Rectangle yanked")
            assert b"\x1b[38;5;75mab" in last_frame(session) and b"\x1b[38;5;75mcd" in last_frame(session)
            group_step(session, b"\x18\x13", b"Saved ")
            assert path.read_bytes() == source
            os.write(session.master, b"\x18\x03")
            session.finish(0)
            print("PASS incremental multi-row rectangle kill/yank, current function captures, atomic undo/redo and exact save")
        finally:
            session.close()


def guarded_session_exit(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-quit-") as directory:
        first = Path(directory) / "first.txt"
        second = Path(directory) / "second.txt"
        draft = Path(directory) / "draft.txt"
        first.write_bytes(b"first")
        second.write_bytes(b"second")
        session = Session([str(root / "target/debug/rev")])
        try:
            session.ready(b"C-x C-c: exit")
            source = "draft界\r\nlast".encode()
            group_step(session, paste_wire(source), b"** RW L2 C5 Text")
            find_path(session, first, b"> first.txt -- RW L1 C1 Text", b"first")
            group_step(session, b"!", b"> first.txt ** RW L1 C2 Text")
            group_step(session, b"\x183", b"|", b"first.txt ** RW L1 C2 Text")
            find_path(session, second, b"> second.txt -- RW L1 C1 Text", b"second")
            group_step(session, b"?", b"> second.txt ** RW L1 C2 Text")
            group_step(session, b"\x18\x03", b'Quit: "*scratch*"?')
            group_step(session, b"y", b"Write file: ")
            group_step(session, str(draft).encode(), b"draft.txt")
            group_step(session, b"\r", b'Quit: "first.txt"?')
            assert draft.read_bytes() == source
            group_step(session, b"y", b'Quit: "second.txt"?')
            assert first.read_bytes() == b"!first"
            group_step(session, b"\x07", b"Quit", b"> second.txt ** RW L1 C2 Text")
            assert b"|  first.txt -- RW L1 C2 Text" in last_frame(session)
            assert second.read_bytes() == b"second"
            group_step(session, b"!", b"> second.txt ** RW L1 C3 Text")
            group_step(session, b"\x18\x03", b'Quit: "second.txt"?')
            os.write(session.master, b"n")
            session.finish(0)
            assert PASTE_OFF in session.output and SHOW in session.output and LEAVE in session.output
            assert second.read_bytes() == b"second"
            print("PASS sequential hidden/unnamed saves, shared views, cancel after earlier saves, explicit discard and terminal restoration")
        finally:
            session.close()


def failed_session_exit(root):
    with tempfile.TemporaryDirectory(prefix="rev-pty-quit-failure-") as directory:
        parent = Path(directory) / "removed"
        parent.mkdir()
        target = parent / "target.txt"
        target.write_bytes(b"base")
        session = Session([str(root / "target/debug/rev"), str(target)])
        try:
            session.ready(b"> target.txt -- RW L1 C1 Text")
            group_step(session, b"new ", b"> target.txt ** RW L1 C5 Text")
            target.unlink()
            parent.rmdir()
            group_step(session, b"\x18\x03", b'Quit: "target.txt"?')
            group_step(session, b"y", b"Destination changed on disk")
            group_step(session, b"n", b"Save canceled", b"> target.txt ** RW L1 C5 Text")
            assert not target.exists()
            group_step(session, b"\x18\x03", b'Quit: "target.txt"?')
            group_step(session, b"y", b"Destination changed on disk")
            group_step(session, b"y", b"save: not published", b"> target.txt ** RW L1 C5 Text")
            assert not parent.exists() and LEAVE not in session.output
            parent.mkdir()
            group_step(session, b"\x18\x13", b"Destination changed on disk")
            group_step(session, b"y", b"Saved ", b"> target.txt -- RW L1 C5 Text")
            assert target.read_bytes() == b"new base"
            group_step(session, b"\x1bxsave-buffers-kill-terminal", b"M-x save-buffers-kill-terminal")
            os.write(session.master, b"\r")
            session.finish(0)
            assert PASTE_OFF in session.output and SHOW in session.output and LEAVE in session.output
            print("PASS removed destination consent, declined/failed required saves, independent retry and named clean exit with terminal restoration")
        finally:
            session.close()


def journey_step(session, keys, marker, content=None):
    session.output.clear()
    os.write(session.master, keys)
    prefixes = (b"\x1b[24;1HC-x", b"\x1b[24;1HESC")
    session.wait(lambda: frame_contains(session.output, marker, content)
                 and not any(prefix in last_frame(session) for prefix in prefixes))
    assert session.process.poll() is None and LEAVE not in session.output


def accepted_exit(session, keys=b"\x18\x03"):
    session.output.clear()
    os.write(session.master, keys)
    session.finish(0)
    for code in (PASTE_OFF, SHOW, b"\x1b[0m", LEAVE):
        assert code in session.output, bytes(session.output)
    assert session.output.count(LEAVE) == 1
    assert session.output.rindex(PASTE_OFF) < session.output.rindex(SHOW)
    assert session.output.rindex(SHOW) < session.output.rindex(b"\x1b[0m")
    assert session.output.rindex(b"\x1b[0m") < session.output.rindex(LEAVE)


def acceptance_document(root):
    # New text reaches a real file and another process through the same user
    # commands used for shared views, search, replacement and history aliases.
    with tempfile.TemporaryDirectory(prefix="rev-accept-document-") as directory:
        path = Path(directory) / "main.rs"
        source = 'fn main() {\r\n    let label = "e\u0301界🙂";\r\n}\r\n'.encode()
        expected = source.replace(b"label", b"name")
        session = Session([str(root / "target/debug/rev")], colors=True)
        try:
            session.ready(b"C-x C-c: exit")
            journey_step(session, paste_wire(source), b"** RW L4 C1 Text")
            journey_step(session, b"\x18\x13", b"Write file: ")
            journey_step(session, str(path).encode(), b"main.rs")
            journey_step(session, b"\r", b"Saved ", b"-- RW L4 C1 Rust")
            assert path.read_bytes() == source
            assert b"\x1b[38;5;176mfn" in last_frame(session)
            journey_step(session, b"\x1b<\x1bf", b"-- RW L1 C3 Rust")
            named_step(session, "backward-word", b"-- RW L1 C1 Rust")
            journey_step(session, b"\x183", b"|", b"main.rs -- RW")
            named_step(session, "split-window-below", b"main.rs -- RW")
            assert last_frame(session).count(b"main.rs -- RW") == 3
            journey_step(session, b"\x13label", b"I-search: label", b"\x1b[4m")
            journey_step(session, b"\r", b"-- RW L2 C14 Rust")
            journey_step(session, b"\x1b>\x12label", b"I-search backward: label", b"\x1b[4m")
            journey_step(session, b"\r", b"-- RW L2 C9 Rust")
            journey_step(session, b"\x1b<\x1b%", b"Query replace: ")
            journey_step(session, b"label\r", b"Replace with: ")
            journey_step(session, b"name\r", b"Replace occurrence? [y/n/q]")
            journey_step(session, b"y", b"1 replaced", b"main.rs ** RW")
            for undo in (b"\x1b[47;5u", b"\x1f", b"\x18u"):
                journey_step(session, undo, b"main.rs -- RW")
                assert b"main.rs ** RW" not in last_frame(session)
                named_step(session, "undo-redo", b"main.rs ** RW")
            journey_step(session, b"\x18\x13", b"Saved ", b"main.rs -- RW")
            assert path.read_bytes() == expected
            journey_step(session, b"\x18o", b"main.rs -- RW")
            named_step(session, "delete-window", b"main.rs -- RW")
            assert last_frame(session).count(b"main.rs -- RW") == 2
            journey_step(session, b"\x181", b"main.rs -- RW")
            assert last_frame(session).count(b"main.rs -- RW") == 1
            accepted_exit(session)
        finally:
            session.close()
        session = Session([str(root / "target/debug/rev"), str(path)], colors=True)
        try:
            session.ready(b"> main.rs -- RW L1 C1 Rust")
            assert b"\x1b[38;5;176mfn" in session.output
            journey_step(session, b"\x13name", b"I-search: name", b"\x1b[4m")
            journey_step(session, b"\r", b"-- RW L2 C13 Rust")
            journey_step(session, b"\x18u", b"Nothing to undo")
            journey_step(session, b"\x1bxsave-buffers-kill-terminal", b"M-x save-buffers-kill-terminal")
            accepted_exit(session, b"\r")
            assert path.read_bytes() == expected
            print("PASS acceptance document: new Unicode/CRLF text, named/bound motion, nested shared views, search/replace, undo reports, exact save and process reopen")
        finally:
            session.close()


def acceptance_blocks(root):
    # Linear copies/kill/yank-pop and rectangles stay separate while buffers
    # carry exact BOM/separator bytes and whole undo steps to their real files.
    with tempfile.TemporaryDirectory(prefix="rev-accept-blocks-") as directory:
        source = Path(directory) / "blocks.txt"
        target = Path(directory) / "target.txt"
        original = b"\xef\xbb\xbf" + "A\nlong 界\n".encode()
        source.write_bytes(original)
        target.write_bytes(b"\xef\xbb\xbf....\r\nq")
        expected = b"\xef\xbb\xbf...A.\r\nq  loA"
        session = Session([str(root / "target/debug/rev"), str(source)])
        try:
            session.ready(b"> blocks.txt -- RW L1 C1 Text")
            copy_rotation_lines(session, 8)
            journey_step(session, b"\x19", b"** RW L3 C8 Text", "long 界".encode())
            journey_step(session, b"\x1by", b"** RW L3 C2 Text")
            journey_step(session, b"\x18u", b"** RW L3 C8 Text")
            named_step(session, "undo-redo", b"** RW L3 C2 Text")
            journey_step(session, b"\x18\x13", b"Saved ")
            assert source.read_bytes() == original + b"A"
            journey_step(session, b"\x1b<\x00\x05\x17", b"** RW L1 C1 Text")
            journey_step(session, b"\x18u", b"blocks.txt -- RW")
            journey_step(session, b"\x1b<\x18 ", b"Rectangle mark set")
            journey_step(session, b"\x0e\x06\x06", b"-- RW L2 C3 Text")
            journey_step(session, b"\x18r\x1bw", b"Rectangle copied")
            find_path(session, target, b"> target.txt -- RW L1 C1 Text", b"....")
            journey_step(session, b"\x06\x06\x06\x18ry", b"Rectangle yanked", b"** RW L2 C6 Text")
            journey_step(session, b"\x18u", b"-- RW L1 C4 Text", b"....")
            named_step(session, "undo-redo", b"** RW L2 C6 Text")
            journey_step(session, b"\x19", b"** RW L2 C7 Text", b"q  loA")
            journey_step(session, b"\x18\x13", b"Saved ")
            assert target.read_bytes() == expected
            assert source.read_bytes() == original + b"A"
            buffer_prompt(session, b"\x18b", b"Switch to buffer: ", b"blocks.txt", b"> blocks.txt -- RW")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"Buffer closed")
            find_path(session, target, b"> target.txt -- RW", b"q  loA")
            accepted_exit(session)
            print("PASS acceptance blocks: linear region/kill ring and yank-pop, separate rectangles, cross-buffer padding, atomic undo/redo, exact BOM and CRLF files")
        finally:
            session.close()


def acceptance_workspace(root):
    # Tree publication and buffer/window commands share the same live owners;
    # deleted open text survives and requires an explicit save to reappear.
    with tempfile.TemporaryDirectory(prefix="rev-accept-workspace-") as directory:
        workspace = Path(directory) / "workspace"
        workspace.mkdir()
        original = workspace / "entry.rs"
        renamed = workspace / "renamed.rs"
        original.write_bytes(b"fn entry() {}\n")
        source = b"// saved\nfn entry() {}\n"
        draft = workspace / "notes/draft.txt"
        session = Session([str(root / "target/debug/rev"), str(original)], colors=True)
        try:
            session.ready(b"> entry.rs -- RW L1 C1 Rust")
            journey_step(session, paste_wire(b"// saved\n"), b"** RW L2 C1 Rust")
            journey_step(session, b"\x183", b"|", b"entry.rs ** RW")
            journey_step(session, b"\x18tt", b"File tree focused", b"> workspace/")
            journey_step(session, b"\x1b[C\x1b[B", b"File tree updated", b"entry.rs")
            named_step(session, "tree-rename", b"Rename to: ")
            journey_step(session, b"renamed.rs\r", b"Renamed ", b"renamed.rs ** RW")
            journey_step(session, b"\t", b"Editing")
            journey_step(session, b"\x18\x13", b"Saved ", b"renamed.rs -- RW")
            assert renamed.read_bytes() == source and not original.exists()
            journey_step(session, b"\x18tf", b"File tree focused")
            journey_step(session, b"\x1b[A", b"File tree updated", b"v workspace/")
            named_step(session, "tree-create-directory", b"New directory name: ")
            journey_step(session, b"notes\r", b"Created ")
            journey_step(session, b"\x18tn", b"New file name: ")
            journey_step(session, b"draft.txt\r", b"Created ")
            journey_step(session, b"\r", b"> draft.txt -- RW L1 C1")
            journey_step(session, "note界".encode(), b"** RW L1 C7")
            journey_step(session, b"\x18\x13", b"Saved ")
            assert draft.read_bytes() == "note界".encode()
            journey_step(session, b"\x18\x02", b"Buffers: arrows move")
            assert b"renamed.rs -- RW" in last_frame(session)
            assert b"draft.txt -- RW" in last_frame(session)
            journey_step(session, b"\x07", b"Quit")
            buffer_prompt(session, b"\x18b", b"Switch to buffer: ", b"renamed.rs", b"> renamed.rs -- RW")
            journey_step(session, b"!", b"renamed.rs ** RW")
            named_step(session, "kill-buffer", b"Kill buffer: ")
            journey_step(session, b"\r", b"y=save, n=discard, C-g=cancel")
            journey_step(session, b"\x07", b"Quit", b"renamed.rs ** RW")
            journey_step(session, b"\x18u", b"renamed.rs -- RW")
            journey_step(session, b"\x181", b"renamed.rs -- RW")
            buffer_prompt(session, b"\x18k", b"Kill buffer: ", b"", b"Buffer closed")
            buffer_prompt(session, b"\x18b", b"Switch to buffer: ", b"draft.txt", b"> draft.txt -- RW")
            journey_step(session, b"\x18tf", b"File tree focused")
            journey_step(session, b"\x18tD", b"Delete ")
            journey_step(session, b"n", b"Delete canceled")
            assert draft.exists()
            named_step(session, "tree-delete", b"Delete ")
            journey_step(session, b"y", b"Deleted ", b"draft.txt ** RW")
            assert not draft.exists()
            journey_step(session, b"\t", b"Editing")
            journey_step(session, b"\x18\x13", b"Saved ", b"draft.txt -- RW")
            assert draft.read_bytes() == "note界".encode()
            assert renamed.read_bytes() == source
            accepted_exit(session)
            print("PASS acceptance workspace: shared Rust views, tree rename/create/open/delete, buffer inventory/switch/guarded close and explicit recreation of retained text")
        finally:
            session.close()


def acceptance(root):
    journeys = [acceptance_document, acceptance_blocks, acceptance_workspace]
    for journey in journeys:
        journey(root)
    print(f"Acceptance PTY suite: {len(journeys)} journeys passed, 0 skipped.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", choices=["terminal", "acceptance"], required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    build = ["cargo", "build", "--bin", "rev"]
    if args.suite == "terminal":
        build.extend(["--example", "terminal-probe"])
    subprocess.run(build, cwd=root, check=True)
    if args.suite == "acceptance":
        acceptance(root)
        return
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
    yank_rotations(root)
    yank_view_invalidation(root)
    terminal_key_reports(root)
    escape_deadlines(root)
    bracketed_paste(root)
    shared_paste(root)
    command_palette(root)
    command_arguments(root)
    focused_arguments(root)
    focused_close(root)
    forward_search(root)
    multiline_search(root)
    backward_search(root)
    smart_search(root)
    query_replace(root)
    replace_lines(root)
    rectangle_copy(root)
    rectangle_scroll(root)
    rectangle_kill(root)
    shared_rectangle_kill(root)
    rectangle_yank(root)
    shared_rectangle_yank(root)
    tree_pane(root)
    tree_resize(root)
    tree_navigation(root)
    tree_navigation_resize(root)
    tree_creation(root)
    tree_creation_failures(root)
    tree_rename(root)
    tree_rename_failures(root)
    tree_delete(root)
    tree_delete_decisions(root)
    rust_syntax(root)
    syntax_fallback(root)
    incremental_shared(root)
    incremental_rectangles(root)
    guarded_session_exit(root)
    failed_session_exit(root)
    print("Terminal PTY suite: 90 cases passed, 0 skipped.")


if __name__ == "__main__":
    main()
