"""Tree, nested shared panes and completion through one retained workspace."""

import re

from .support import BINARY, CONFIG, LIST, PTY, canvas, cells, close, cursor, environment, finish, fixtures, passive, rows, send, snapshot, step, visit, write


WORKSPACE = """
[tree]
visible = true
width = 12
max_fraction = 0.3
[windows]
split_ratio = 0.6
[scroll]
page_overlap = 1
[language.go]
tab_width = 4
"""


def states(frame, name='main.rs'):
    pattern = r'(>| ) ' + re.escape(name) + r' (--|\*\*) RW -- L(\d+) C(\d+)'
    return [(row, *match.groups()) for row, text in rows(frame).items() for match in re.finditer(pattern, text)]


def panes(frame, row):
    parts = rows(frame)[row].split('│')
    assert len(parts) == 3, parts
    assert tuple(cells(text) for text in parts) == (12, 39, 27)
    return parts[1:]


def nested(session):
    step(session, b'\x18tf', b'File tree focused')
    frame = step(session, b'\x1bl', b'File tree updated')
    assert 'v work/' in canvas(frame)[1]
    assert any('main.rs' in text[:12] for text in rows(frame).values())
    step(session, b'\t', b'Editing')
    send(session, b'\x183', lambda frame: len(states(frame)) == 2)
    frame = send(session, b'\x182', lambda frame: len(states(frame)) == 3)
    assert [state[0] for state in states(frame)] == [13, 23, 23]
    assert all(state[2:] == ('--', '1', '1') for state in states(frame))
    assert all(text.startswith('        row00 界e\u0301 ') for text in panes(frame, 1))
    assert panes(frame, 15)[0].startswith('        row00 界e\u0301 ')
    assert set(rows(frame)[14]).intersection('─├┤┼')


def boundaries(session):
    frame = step(session, b'\x05', b'> main.rs -- RW -- L1 C105 Rust')
    assert 'end00' in panes(frame, 1)[0]
    assert panes(frame, 1)[1].startswith('        row00 界e\u0301 ')
    frame = step(session, b'\x0e', b'> main.rs -- RW -- L2 C105 Rust')
    assert 'end01' in panes(frame, 2)[0]
    frame = step(session, b'\x16', b'> main.rs -- RW -- L13 C105 Rust')
    assert sum(state[3:] == ('1', '1') for state in states(frame)) == 2
    assert 'end11' in panes(frame, 1)[0]
    frame = step(session, b'\x1b>', b'> main.rs -- RW -- L41 C1 Rust')
    assert panes(frame, 1)[1].startswith('        row00 界e\u0301 ')
    frame = step(session, b'\x1b<', b'> main.rs -- RW -- L1 C1 Rust')
    assert panes(frame, 1)[0].startswith('        row00 界e\u0301 ')


def shared_edit(session):
    frame = step(session, b'!', b'> main.rs ** RW -- L1 C2 Rust')
    assert len(states(frame)) == 3 and all(state[2] == '**' for state in states(frame))
    assert all(text.startswith('!       row00') for text in panes(frame, 1))
    assert panes(frame, 15)[0].startswith('!       row00')
    frame = step(session, b'\x18u', b'> main.rs -- RW -- L1 C1 Rust')
    assert all(state[2] == '--' for state in states(frame))


def buffer_policy(session):
    frame = visit(session, 'alternate.go', b'> alternate.go -- RW -- L1 C1 Text')
    assert panes(frame, 1)[0].startswith('    go body')
    assert panes(frame, 1)[1].startswith('        row00')
    assert len(states(frame)) == 2 and len(states(frame, 'alternate.go')) == 1
    frame = step(session, b'\x06', b'> alternate.go -- RW -- L1 C5 Text')
    assert cursor(frame) == (1, 18)
    frame = visit(session, 'main.rs', b'> main.rs -- RW -- L1 C1 Rust')
    assert len(states(frame)) == 3
    assert all(text.startswith('        row00') for text in panes(frame, 1))


def resized_prompt(session):
    step(session, b'\x05\x0e', b'> main.rs -- RW -- L2 C105 Rust')
    frame = step(session, b'\x07', b'Quit')
    retained = canvas(frame)
    retained_cursor = cursor(frame)
    step(session, b'\x18\x06ma', b'Find file: ma')
    frame = step(session, b'\x1bk', b'Find file: ma')
    LIST['ordered'](frame, b'Find file: ma', [b'main.rs'])
    for width, height in [(0, 0), (1, 1), (20, 4), (80, 24)]:
        PTY['resize_surface'](session, width, height, True)
        if width and height:
            canvas(PTY['last_frame'](session), width, height)
    frame = PTY['last_frame'](session)
    LIST['ordered'](frame, b'Find file: ma', [b'main.rs'])
    frame = step(session, b'\x07', b'Quit')
    assert {row: text for row, text in canvas(frame).items() if row < 24} == {row: text for row, text in retained.items() if row < 24}
    assert cursor(frame) == retained_cursor
    for report in [b'\x1b[15;14H', b'\x1b[1;54H', b'\x1b[2;52H']:
        frame = step(session, b'\x18o', b'> main.rs -- RW', report + PTY['SHOW'])
    assert cursor(frame) == retained_cursor


def workspace(root, colored):
    source = fixtures(root)
    write(root, 'user/rev/config.toml', (CONFIG + WORKSPACE).encode())
    text = ''.join(f'\trow{line:02} 界e\u0301 ' + '0123456789' * 8 + f' end{line:02}\n' for line in range(40))
    source.write_text(text)
    write(root, 'work/alternate.go', b'\tgo body\n')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)], env=environment(root, colored))
    try:
        session.ready(b'> main.rs -- RW -- L1 C1 Rust')
        nested(session)
        boundaries(session)
        shared_edit(session)
        buffer_policy(session)
        resized_prompt(session)
        assert snapshot(root) == before, 'workspace operations changed disk'
        finish(session, colored)
    finally:
        close(session)
    assert snapshot(root) == before
    passive(root)
    print(f"PASS {'RGB' if colored else 'NO_COLOR'} tree/nested ratios, shared edit/undo, per-buffer tabs, horizontal/page/EOF boundaries, completion resize/cancel and editing-only cycle")
