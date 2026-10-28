"""Exercise bounded drag deadlines through the native event loop in owned PTYs."""

import re

from .gutter_geometry import idle
from .pointer_focus import report as press
from .pointer_selection import report as drag
from .support import (BINARY, PTY, close, cursor, environment, finish, passive,
                      rows, send, snapshot, step, visit, write)

BODY = ''.join(f'row-{index:02}\r\n' for index in range(80)).encode()
SOURCE = b'\xef\xbb\xbf' + BODY


def top(frame, row):
    return int(re.search(r'row-(\d+)', rows(frame)[row])[1])


def stage(session, wire, predicate):
    frame = send(session, wire, predicate)
    assert b'\x1b]52;' not in session.output, 'pointer wrote a terminal clipboard'
    return frame


def save(session, path, expected):
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == expected, 'canonical saved bytes differ'


def lifecycle(session, root):
    # The first real deadline extends beyond the visible bottom. Exact clock
    # frequency/row ceilings are asserted by the controlled native tests.
    stage(session, press(7, 0) + drag(7, 22), lambda f: top(f, 1) >= 3)
    frame = stage(session, drag(7, 1), lambda f: cursor(f) == (2, 8))
    paused = top(frame, 1)
    idle(session)  # An inside captured drag has no pending tick or repaint.
    stage(session, drag(7, 23), lambda f: top(f, 1) >= paused + 3)
    frame = stage(session, drag(7, 1, True), lambda f: cursor(f) == (2, 8))
    endpoint = (top(frame, 1) + 1) * 8 + 1
    assert b'\x1b[7m' in frame, 'released canonical region was lost'
    idle(session)  # Release alone retires the timer and retains the region.
    step(session, b'\x19', b'Kill ring is empty')
    step(session, b'\x1bw', b'Region copied')
    step(session, b'\x17', b'** RW')
    save(session, root / 'source.txt', SOURCE[:4] + BODY[endpoint:])
    step(session, b'\x18u', b'RW')
    save(session, root / 'source.txt', SOURCE)
    step(session, b'\x1bxundo-redo\r', b'RW')
    save(session, root / 'source.txt', SOURCE[:4] + BODY[endpoint:])
    step(session, b'\x18u', b'RW')
    save(session, root / 'source.txt', SOURCE)


def ownership(session, root):
    stage(session, b'\x182\x18o', lambda f: cursor(f) == (13, 7))
    visit(session, str(root / 'peer.txt'), b'peer.txt')
    step(session, b'\x1bgg40\r', b'L40 C1')
    frame = stage(session, press(7, 12), lambda f: cursor(f) == (13, 8))
    initial = top(frame, 13)
    peer = rows(frame)[1]
    frame = stage(session, drag(7, 11), lambda f: top(f, 13) <= initial - 3)
    assert rows(frame)[1] == peer and cursor(frame) == (13, 8)
    frame = stage(session, drag(7, 13), lambda f: cursor(f) == (14, 8))
    paused = top(frame, 13)
    idle(session)
    # An outside report followed by focus loss in the same native batch cannot
    # leave a due step; delayed drag/release cannot revive it.
    frame = stage(session, drag(7, 11) + b'\x1b[O' + drag(7, 11) + b'\x18',
                  lambda f: b'C-x' in f)
    assert top(frame, 13) == paused and rows(frame)[1] == peer
    idle(session)
    step(session, b'\x07', b'Quit')
    stage(session, press(7, 12) + drag(7, 11), lambda f: cursor(f) == (13, 8))
    PTY['resize_surface'](session, 120, 24, True)
    frame = stage(session, drag(7, 11) + drag(7, 11, True) + b'\x18', lambda f: b'C-x' in f)
    assert rows(frame)[1].startswith(peer.rstrip())
    idle(session)
    step(session, b'\x07', b'Quit')


def case(root, journey):
    path = write(root, 'source.txt', SOURCE)
    if journey == ownership:
        write(root, 'peer.txt', BODY)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, False))
    try:
        session.ready(b'-- RW -- L1 C1')
        journey(session, root)
        assert b'\x1b]52;' not in session.output
        finish(session, False)
    finally:
        close(session)
    assert snapshot(root) == before
    passive(root)


def run(root):
    for journey in [lifecycle, ownership]:
        fixture = root / journey.__name__
        fixture.mkdir()
        case(fixture, journey)
        print(f'PASS pointer autoscroll {journey.__name__}: native deadlines, source ownership, idle and owned cleanup')
    print('Pointer autoscroll PTY suite: 2 owned sessions passed, 0 skipped.')
