"""Verify canonical native drag selection and cancellation in owned PTYs."""

from .gutter_geometry import idle
from .pointer_focus import report as press
from .support import (BINARY, PTY, close, cursor, environment, finish, rows,
                      passive, snapshot, startup_snapshot, step, visit, write)

SOURCE = '\ufeffa\t界e\u0301👩‍💻\r\nlast\n'.encode()
PAYLOAD = '\t界e\u0301👩‍💻\r\nla'.encode()


def report(column, row, release=False):
    code, phase = (0, 'm') if release else (32, 'M')
    return f'\x1b[<{code};{column + 1};{row + 1}{phase}'.encode()


def stage(session, wire, marker):
    frame = step(session, wire, marker)
    assert b'\x1b]52;' not in session.output, 'gesture wrote a terminal clipboard'
    return frame


def save(session, path, expected):
    stage(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == expected, 'saved canonical bytes differ'


def editing(session, root):
    path = root / 'source.txt'
    stage(session, press(7, 0), b'L1 C2')
    frame = stage(session, report(8, 1), b'L2 C3')
    assert cursor(frame) == (2, 9) and b'\x1b[7m' in frame
    session.output.clear()
    idle(session)  # Capture itself adds no output or timer work.
    frame = stage(session, report(8, 1, True) + report(10, 0) + b'\x19', b'Kill ring is empty')
    assert cursor(frame) == (2, 9) and b'\x1b[7m' in frame
    assert path.read_bytes() == SOURCE
    stage(session, b'\x1bw', b'Region copied')
    stage(session, b'\x17', b'** RW -- L1 C2')
    save(session, path, b'\xef\xbb\xbfast\n')
    stage(session, b'\x18u', b'L2 C3')
    save(session, path, SOURCE)
    stage(session, b'\x1bxundo-redo\r', b'L1 C2')
    save(session, path, b'\xef\xbb\xbfast\n')
    stage(session, b'\x18u', b'L2 C3')
    save(session, path, SOURCE)
    stage(session, b'\x07\x1b>\x19', b'** RW')
    save(session, path, SOURCE + PAYLOAD)
    stage(session, b'\x18u', b'RW')
    save(session, path, SOURCE)


def cancellation(session, root):
    stage(session, b'\x183\x18o', b'L1 C1')
    visit(session, str(root / 'other.txt'), b'other.txt')
    stage(session, b'\x18o', b'L1 C1')
    stage(session, press(7, 0) + report(9, 1), b'L2 C4')
    frame = stage(session, report(47, 0) + report(8, 1), b'L2 C3')
    assert cursor(frame) == (2, 9) and rows(frame)[23].startswith('> source.txt')
    frame = stage(session, b'\x1b[O' + report(10, 0) + b'\x18', b'C-x')
    assert cursor(frame) == (2, 9) and b'\x1b[7m' in frame
    stage(session, b'\x07', b'Quit')
    stage(session, press(7, 0) + report(9, 1), b'L2 C4')
    PTY['resize_surface'](session, 120, 24, True)
    frame = stage(session, report(8, 0) + b'\x18', b'C-x')
    assert cursor(frame) == (2, 10)
    stage(session, b'\x07', b'Quit')
    stage(session, press(7, 0) + report(9, 1), b'L2 C4')
    frame = stage(session, b'\x07' + report(10, 0) + report(10, 0, True), b'Quit')
    assert cursor(frame) == (2, 10) and b'\x1b[7m' not in frame
    frame = stage(session, press(67, 0) + report(67, 0, True), b'L1 C2')
    assert cursor(frame) == (1, 68) and b'\x1b[7m' not in frame


def case(root, journey):
    write(root, 'source.txt', SOURCE if journey == editing else b'alpha\nbeta\n')
    if journey == cancellation:
        write(root, 'other.txt', b'other\r\n')
    before = startup_snapshot(root)
    session = PTY['Session']([str(BINARY), str(root / 'source.txt')], env=environment(root, False))
    try:
        session.ready(b'-- RW -- L1 C1')
        journey(session, root)
        finish(session, False)
    finally:
        close(session)
    assert snapshot(root) == before
    passive(root)


def run(root):
    for journey in [editing, cancellation]:
        fixture = root / journey.__name__
        fixture.mkdir()
        case(fixture, journey)
        print(f'PASS pointer selection {journey.__name__}: canonical region, fidelity and owned cleanup')
    print('Pointer selection PTY suite: 2 owned sessions passed, 0 skipped.')
