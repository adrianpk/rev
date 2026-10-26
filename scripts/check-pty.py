#!/usr/bin/env python3
"""Exercise terminal lifetime through real controlling PTYs on Linux."""

import argparse
import errno
import fcntl
import os
from pathlib import Path
import pty
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
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 3, 3, 0, 0))
        session.wait(lambda: b"\x1b[3;1HC-x" in session.output)
        session.output.clear()
        fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 1, 1, 0, 0))
        session.wait(lambda: b"\x1b[1;1HC" in session.output)
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
        refusal = b"Unsaved changes; cannot quit before saving is available"
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
        # This owned dirty fixture cannot quit normally before saving exists.
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
        session.wait(lambda: b"Unsaved changes; cannot quit before saving is available"
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
        session.wait(lambda: b"Unsaved changes; cannot quit before saving is available"
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
    session.wait(lambda: marker in session.output and SHOW in session.output
                 and (content is None or content in session.output))
    assert session.process.poll() is None and LEAVE not in session.output


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
            (invalid, b"UTF-8", b"valid\xff"),
            (binary, b"NUL", b"valid\0text"),
            (invalid / "child", b"Not a directory", None),
            (base, b"regular file", None),
            (denied, b"Permission denied", b"private"),
        ]
        for path, marker, source in cases:
            session = Session([str(root / "target/debug/rev"), str(path)])
            try:
                session.finish(1)
                assert marker in session.output and b"rev: load " in session.output
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
    print("Terminal PTY suite: 20 cases passed, 0 skipped.")


if __name__ == "__main__":
    main()
