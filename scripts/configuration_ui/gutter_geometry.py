"""Verify nested gutter geometry, source fidelity and event-driven idle in PTYs."""

from pathlib import Path
import select

from configuration_ui.gutter_controls import command
from configuration_ui.support import (BINARY, PTY, canvas, cells, close, completed,
                                      cursor, environment, finish, send, snapshot, startup_snapshot, step, write)

SOURCE = '\ufeff\t界e\u0301\r\nlast\n'.encode()


def field(frame, row, column, expected):
    # Number fields use cells, including when wide/combining source precedes a
    # right-side field. String/scalar offsets would move that assertion left.
    offsets = {}
    offset = 0
    for char in canvas(frame)[row]:
        width = cells(char)
        if width:
            offsets[offset] = char
        offset += width
    actual = ''.join(offsets.get(column + i, '') for i in range(len(expected)))
    assert actual == expected, (row, column, actual, expected)
    return True


def ticks(session):
    state = Path(f'/proc/{session.process.pid}/stat').read_text().rsplit(')', 1)[1].split()
    return int(state[11]) + int(state[12])


def idle(session):
    while select.select([session.master], [], [], 0)[0]:
        session.drain()
    before = ticks(session)
    assert not select.select([session.master], [], [], 0.12)[0], 'idle emitted terminal output'
    assert ticks(session) == before, 'idle consumed process CPU ticks'


def views(session, root):
    frame = send(session, b'\x18tt\x07\x183\x182', lambda f: f.count(b'RW -- L1 C1') == 3)
    field(frame, 1, 25, '   1 │')
    field(frame, 13, 25, '   1 │')
    field(frame, 1, 53, '   1 │')
    frame = step(session, b'\x1bgg2\r', b'RW -- L2 C1')
    assert cursor(frame) == (2, 32)
    command(session, 'set-line-numbers-relative', lambda f: field(f, 3, 25, '   1 │'))
    frame = command(session, 'toggle-line-number-side', lambda f: field(f, 2, 46, '│   2 '))
    assert cursor(frame) == (2, 26)
    field(frame, 3, 53, '   3 │')
    send(session, b'\x18o', lambda f: cursor(f)[0] >= 13)
    frame = step(session, b'\x1bgg3\r', b'RW -- L3 C1')
    assert cursor(frame) == (15, 32)
    frame = command(session, 'set-line-numbers-relative', lambda f: field(f, 13, 25, '   2 │'))
    field(frame, 2, 46, '│   2 ')
    step(session, b'\x18\x06', b'Find file: ')
    draft = str(root / 'al').encode()
    step(session, PTY['paste_wire'](draft), b'alpha.txt')
    for width, height in [(18,9), (0,0), (80,24)]:
        PTY['resize_surface'](session, width, height, True)
    frame = PTY['last_frame'](session)
    assert b'Find file: ' + draft in frame and b'alpha.txt' in frame and b'alpine.txt' in frame
    frame = step(session, b'\x07', b'Quit')
    assert cursor(frame) == (15, 32)
    field(frame, 2, 46, '│   2 ')
    field(frame, 13, 25, '   2 │')


def history(session, path):
    step(session, PTY['paste_wire'](b'Z'), b'RW -- L3 C2')
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == SOURCE + b'Z'
    step(session, b'\x18u', b'RW -- L3 C1')
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == SOURCE
    step(session, b'\x1bxundo-redo\r', b'RW -- L3 C2')
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == SOURCE + b'Z'
    step(session, b'\x18u', b'RW -- L3 C1')
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == SOURCE


def case(root, colored):
    path = write(root, 'g.txt', SOURCE)
    write(root, 'alpha.txt', b'alpha')
    write(root, 'alpine.txt', b'alpine')
    expected = startup_snapshot(root)
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, colored))
    try:
        session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'RW -- L1 C1 Text')))
        views(session, root)
        history(session, path)
        frame = command(session, 'set-line-numbers-absolute', lambda f: field(f, 13, 25, '   1 │'))
        field(frame, 2, 46, '│   2 ')
        frame = send(session, b'\x180', lambda f: cursor(f)[0] == 2)
        field(frame, 2, 46, '│   2 ')
        idle(session)
        assert path.read_bytes() == SOURCE
        assert snapshot(root) == expected
        finish(session, colored)
    finally:
        close(session)
    print(f'PASS gutter geometry {"RGB" if colored else "monochrome"}: independent nested origins, tree/path regions, zero resize, source save/history, close and idle output/CPU')


def run(root):
    for name, colored in [('colored', True), ('monochrome', False)]:
        fixture = root / name
        fixture.mkdir()
        case(fixture, colored)
    print('Gutter geometry PTY suite: 2 owned sessions passed, 0 skipped.')
