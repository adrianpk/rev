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
    print("Terminal PTY suite: 11 cases passed, 0 skipped.")


if __name__ == "__main__":
    main()
