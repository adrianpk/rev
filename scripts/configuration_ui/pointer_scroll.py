"""Exercise native wheel ownership through the ordinary binary in owned PTYs."""

from .gutter_geometry import idle
from .pointer_focus import report as press
from .support import (BINARY, PTY, close, cursor, environment, finish, passive,
                      rows, send, snapshot, startup_snapshot, step, visit, write)


def source(prefix):
    return ''.join(f'{prefix}{index:02}\r\n' for index in range(40)).encode()


def report(backward, column, row):
    code = 64 if backward else 65
    return f'\x1b[<{code};{column + 1};{row + 1}M'.encode()


def document(session, root):
    send(session, b'\x183', lambda frame: rows(frame)[23].count('L1 C1') == 2)
    send(session, b'\x18o', lambda frame: cursor(frame) == (1, 47))
    visit(session, str(root / 'b.txt'), b'b.txt')
    send(session, b'\x18o', lambda frame: cursor(frame) == (1, 7))
    frame = send(session, report(False, 46, 0), lambda frame:
                 cursor(frame) == (1, 7) and 'other-03' in rows(frame)[1])
    assert 'row-00' in rows(frame)[1] and b'L4 C1' in frame
    frame = send(session, report(True, 46, 0), lambda frame:
                 cursor(frame) == (1, 7) and 'other-00' in rows(frame)[1])
    assert b'L4 C1' in frame  # Still-visible peer point stays on logical row four.
    step(session, b'\x18', b'C-x')
    send(session, report(False, 46, 0) + b'o', lambda frame: cursor(frame) == (1, 47))
    send(session, b'\x18o', lambda frame: cursor(frame) == (1, 7))
    session.output.clear()
    idle(session)


def nested(session, root):
    send(session, b'\x18tt\x1b[C\t', lambda frame:
         b'Editing' in frame and 'file-00' in rows(frame)[2])
    send(session, b'\x183\x182', lambda frame:
         sum(row.count('L1 C1') for row in rows(frame).values()) == 3)
    frame = send(session, report(False, 31, 12), lambda frame:
                 cursor(frame) == (1, 32) and 'row-03' in rows(frame)[13])
    assert rows(frame)[1].count('row-00') == 2
    send(session, report(False, 1, 0), lambda frame:
         cursor(frame) == (1, 32) and 'file-02' in rows(frame)[1][:24])
    send(session, press(1, 0), lambda frame: cursor(frame) == (1, 1))
    frame = send(session, report(False, 59, 0), lambda frame:
                 cursor(frame) == (1, 1) and 'row-03' in rows(frame)[1])
    assert 'file-02' in rows(frame)[1][:24]
    PTY['resize_surface'](session, 120, 24, True)
    # The previous right-pane coordinate now belongs to the wider left leaf.
    frame = send(session, report(False, 59, 0), lambda frame:
                 cursor(frame) == (1, 1) and rows(frame)[1].count('row-03') == 2)
    assert 'row-03' in rows(frame)[13]
    frame = send(session, report(True, 1, 0), lambda frame:
                 cursor(frame) == (4, 1) and rows(frame)[1].startswith('v '))
    assert rows(frame)[1].count('row-03') == 2
    send(session, press(31, 12), lambda frame: cursor(frame) == (13, 32))
    session.output.clear()
    idle(session)


def case(root, journey):
    # XDG publication belongs outside the project tree whose row order is tested.
    project = root / 'work'
    path = write(project, 'a.txt' if journey == document else 'zz.txt', source('row-'))
    if journey == document:
        write(project, 'b.txt', ''.join(f'other-{index:02}\t界e\u0301\r\n' for index in range(40)).encode())
    else:
        for index in range(25):
            write(project, f'file-{index:02}', b'retained')
    before = startup_snapshot(root)
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, False))
    try:
        session.ready(b'-- RW -- L1 C1')
        journey(session, project)
        finish(session, False)
    finally:
        close(session)
    assert snapshot(root) == before
    passive(root)


def run(root):
    # Direct SGR input exercises the actual decoder and handlers while factory
    # terminal-mode activation remains with its recorded controls slice.
    for journey in [document, nested]:
        fixture = root / journey.__name__
        fixture.mkdir()
        case(fixture, journey)
        print(f'PASS pointer scroll {journey.__name__}: focus/viewport, source fidelity, idle and owned cleanup')
    print('Pointer scroll PTY suite: 2 owned sessions passed, 0 skipped.')
