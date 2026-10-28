"""Inject native SGR presses through the ordinary binary in owned PTYs."""

import fcntl
import os
import struct
import termios

from .gutter_geometry import idle
from .support import (BINARY, PTY, close, cursor, environment, finish, passive,
                      rows, send, snapshot, startup_snapshot, step, write)

SOURCE = 'a\t界e\u0301\r\nlast\r\n'.encode()


def report(column, row):
    # SGR wire coordinates are one-based; the existing decoder owns conversion.
    return f'\x1b[<0;{column + 1};{row + 1}M'.encode()


def hit(session, column, row, expected, status):
    return send(session, report(column, row), lambda frame:
                cursor(frame) == expected and PTY['status_contains'](frame, status))


def split(session):
    send(session, b'\x183', lambda frame: rows(frame)[23].count('Text') == 2)


def ordinary(session):
    hit(session, 11, 0, (1, 11), b'L1 C5')  # Interior CJK cell -> cluster start.
    split(session)
    hit(session, 47, 1, (2, 48), b'L2 C2')
    hit(session, 0, 0, (1, 11), b'L1 C5')  # Number field focuses only.
    step(session, b'\x1bgg2', b'Goto line: 2')
    frame = step(session, report(47, 1) + b'3', b'Goto line: 23')
    assert b'L1 C5' in frame and b'L2 C2' in frame
    frame = step(session, b'\x07', b'Quit')
    assert cursor(frame) == (1, 11)
    step(session, b'\x18tt', b'File tree focused')
    hit(session, 60, 1, (2, 61), b'L2 C2')
    hit(session, 1, 0, (1, 1), b'L2 C2')
    hit(session, 59, 1, (2, 60), b'L2 C1')
    session.output.clear()
    idle(session)


def resized(session):
    split(session)
    hit(session, 47, 1, (2, 48), b'L2 C2')
    PTY['resize_surface'](session, 120, 24, True)
    # Old right-pane text coordinates now belong to the wider left pane's fill.
    hit(session, 46, 0, (1, 7), b'L1 C1')
    hit(session, 67, 1, (2, 68), b'L2 C2')
    session.output.clear()
    fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack('HHHH', 2, 2, 0, 0))
    hidden_frame(session)
    # No pane is projected. C-g provides a decoder barrier after the stale hit.
    session.output.clear()
    os.write(session.master, report(0, 0) + b'\x07')
    session.wait(lambda: PTY['status_contains'](session.output, b'Qu'))
    hidden_frame(session)
    PTY['resize_surface'](session, 80, 24, True)
    frame = PTY['last_frame'](session)
    assert cursor(frame) == (2, 48)
    assert b'L1 C1' in frame and b'L2 C2' in frame


def hidden_frame(session):
    # A hidden cursor has no SHOW terminator. Each of the two actual rows
    # completes with the renderer's reset; retain bounds and hidden assertions.
    reset = b'\x1b[0m'
    session.wait(lambda: session.output.endswith(reset) and session.output.count(reset) == 2)
    assert session.output.startswith(PTY['HIDE']) and PTY['SHOW'] not in session.output
    PTY['terminal_bounds'](session.output, 2, 2)


def case(root, journey):
    path = write(root, 'source.txt', SOURCE)
    before = startup_snapshot(root)
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, False))
    try:
        session.ready(b'-- RW -- L1 C1')
        journey(session)
        finish(session, False)
    finally:
        close(session)
    assert snapshot(root) == before
    assert path.read_bytes() == SOURCE
    passive(root)


def run(root):
    # Reporting activation remains with its recorded later slice. These tests
    # supply actual SGR reports directly to exercise decoder -> Session -> frame.
    for journey in [ordinary, resized]:
        fixture = root / journey.__name__
        fixture.mkdir()
        case(fixture, journey)
        print(f'PASS pointer {journey.__name__}: native focus/point, unchanged bytes and full PTY cleanup')
    print('Pointer focus PTY suite: 2 owned sessions passed, 0 skipped.')
