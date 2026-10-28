"""One native session connects bootstrap, navigation, panes, edits and pointer IO."""

from .gutter_geometry import idle
from .pointer_focus import report as press
from .pointer_scroll import report as wheel
from .pointer_selection import report as drag
from .pointer_lifetime import SGR_ON, SGR_OFF, BUTTONS_ON, BUTTONS_OFF
from .support import (BINARY, PTY, canvas, cells, close, completed, cursor,
                      environment, finish, passive, rows, send, snapshot,
                      startup_snapshot, step, visit, write)
from .workspace import states

NAME = '界.rs'
BODY = '\r\n'.join(f'\t界e\u0301 row-{index:02}' for index in range(40)).encode()
SOURCE = b'\xef\xbb\xbf' + BODY
EXPANDED = SOURCE.replace(b'\t', b'    ')
PAYLOAD = BODY.split(b'\r\n', 1)[0] + '\r\n\t界e\u0301'.encode()
UPPER = (11, 25, 47)
LOWER = (23, 25, 47)
RIGHT = (23, 73, 47)


def span(text, start, width):
    result, column = '', 0
    for char in text:
        size = cells(char)
        if start <= column < start + width or size == 0 and start < column <= start + width:
            result += char
        column += size
    assert cells(result) == width, (start, width, result)
    return result


def status(frame, area, expected):
    row, left, width = area
    prefix, mode = expected.rsplit(' ', 1)
    gap = width - cells(prefix) - cells(mode)
    assert gap >= 1
    actual = span(rows(frame)[row], left, width)
    assert actual == prefix + ' ' * gap + mode, (actual, expected)


def named(session, name, marker):
    return step(session, b'\x1bx' + name.encode() + b'\r', marker)


def numbering(session, mode, expected):
    wire = b'\x1bxgutter-set-line-numbers-' + mode.encode() + b'\r'
    return send(session, wire, lambda f: b'M-x ' not in f and expected(f))


def save(session, path, expected):
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == expected, 'combined canonical saved bytes differ'


def bootstrap(session, root, expected):
    marker = f'> {NAME} -- RW -- L1 C1 Rust'.encode()
    session.wait(lambda: completed(session, lambda f: PTY['status_contains'](f, marker)))
    frame = PTY['last_frame'](session)
    content = canvas(frame)
    assert content[1].startswith('   1 │    界e\u0301 row-00')
    assert cursor(frame) == (1, 7)
    status(frame, (23, 0, 80), f'> {NAME} -- RW -- L1 C1 Rust')
    assert b'Configuration:' not in frame
    assert SGR_ON in session.output and BUTTONS_ON in session.output
    assert snapshot(root) == expected, 'bootstrap created unexpected entries or bytes'
    passive(root)


def navigation(session):
    # The typed edit is an ordering barrier after both arrow directions. Idle
    # observation catches queued movement after the final release without input.
    frame = step(session, b'\x1b[B' * 8 + b'\x1b[A' * 3 + b'X', b'** RW -- L6 C2')
    assert cursor(frame) == (6, 8)
    step(session, b'\x18u', b'-- RW -- L6 C1')
    idle(session)
    frame = step(session, b'\x1bgg17\r', b'-- RW -- L17 C1')
    assert cursor(frame) == (17, 7)
    frame = numbering(session, 'relative', lambda f: rows(f)[1].startswith('  16 │'))
    assert rows(frame)[17].startswith('  17 │')
    retained, point = canvas(frame), cursor(frame)
    step(session, b'\x1bg\x1bg39', b'Goto line: 39')
    frame = step(session, b'\x07', b'Quit')
    assert {r: t for r, t in canvas(frame).items() if r < 24} == {r: t for r, t in retained.items() if r < 24}
    assert cursor(frame) == point
    numbering(session, 'absolute', lambda f: rows(f)[1].startswith('   1 │'))
    step(session, b'\x1bgg1\r', b'-- RW -- L1 C1')


def workspace(session):
    PTY['resize_surface'](session, 120, 24, True)
    step(session, b'\x18tt', b'File tree focused')
    step(session, b'\x1b[C\t', b'Editing')
    send(session, b'\x183\x182', lambda f: len(states(f, NAME)) == 3)
    frame = step(session, b'\x1bgg6\r', b'L6 C1')
    frame = numbering(session, 'relative', lambda f: span(rows(f)[1], 25, 6) == '   5 │')
    content = canvas(frame, 120)
    assert 'v work/' in content[1][:24]
    assert span(content[13], 25, 6) == span(content[1], 73, 6) == '   1 │'
    status(frame, UPPER, f'> {NAME} -- RW -- L6 C1 Rust')
    status(frame, LOWER, f'  {NAME} -- RW -- L1 C1 Rust')
    status(frame, RIGHT, f'  {NAME} -- RW -- L1 C1 Rust')
    frame = send(session, press(83, 1) + drag(83, 1, True), lambda f: cursor(f) == (2, 84))
    status(frame, RIGHT, f'> {NAME} -- RW -- L2 C5 Rust')
    status(frame, UPPER, f'  {NAME} -- RW -- L6 C1 Rust')
    frame = send(session, wheel(False, 31, 0), lambda f: cursor(f) == (2, 84) and 'row-03' in span(rows(f)[1], 31, 41))
    assert span(rows(frame)[1], 25, 6) == '   2 │'
    status(frame, RIGHT, f'> {NAME} -- RW -- L2 C5 Rust')
    status(frame, UPPER, f'  {NAME} -- RW -- L6 C1 Rust')
    frame = send(session, wheel(True, 31, 0), lambda f: cursor(f) == (2, 84) and 'row-00' in span(rows(f)[1], 31, 41))
    assert span(rows(frame)[1], 25, 6) == '   5 │'
    return frame


def selection(session, path, colored):
    step(session, press(79, 0), b'L1 C1')
    frame = step(session, drag(86, 1), b'L2 C8')
    assert cursor(frame) == (2, 87)
    if colored:
        assert b'\x1b[48;2;40;52;87m' in frame
    else:
        assert b'\x1b[7m' in frame
    frame = step(session, drag(86, 1, True) + drag(79, 0) + b'\x18', b'C-x')
    assert cursor(frame) == (2, 87), 'release left a live pointer capture'
    step(session, b'\x07', b'Quit')
    step(session, press(79, 0) + drag(86, 1) + drag(86, 1, True), b'L2 C8')
    step(session, b'\x1bw', b'Region copied')
    frame = step(session, b'\x17', b'** RW')
    assert len(states(frame, NAME)) == 3 and all(v[2] == '**' for v in states(frame, NAME))
    killed = b'\xef\xbb\xbf' + BODY[len(PAYLOAD):]
    save(session, path, killed)
    step(session, b'\x18u', b'RW')
    save(session, path, SOURCE)
    named(session, 'undo-redo', b'RW')
    save(session, path, killed)
    step(session, b'\x18u', b'RW')
    save(session, path, SOURCE)


def documents(frame):
    content = canvas(frame, 120)
    areas = [(25, 47, 1, 10), (25, 47, 13, 10), (73, 47, 1, 22)]
    return [span(content[row], left + 6, width - 6)
            for left, width, first, height in areas for row in range(first, first + height)]


def whitespace(session, path):
    select = b'\x1b<\x00\x1b>'
    frame = step(session, select, b'L40 C15')
    before = documents(frame)
    frame = named(session, 'untabify', b'** RW')
    assert documents(frame) == before, 'untabify shifted displayed source columns'
    assert len(states(frame, NAME)) == 3 and all(v[2] == '**' for v in states(frame, NAME))
    save(session, path, EXPANDED)
    step(session, b'\x18u', b'RW')
    save(session, path, SOURCE)
    named(session, 'undo-redo', b'RW')
    save(session, path, EXPANDED)
    step(session, select, b'L40 C15')
    frame = named(session, 'tabify', b'** RW')
    assert documents(frame) == before, 'tabify shifted displayed source columns'
    save(session, path, SOURCE)
    step(session, select, b'L40 C15')
    named(session, 'tabify', b'No whitespace changed')
    step(session, b'\x18u', b'RW')
    save(session, path, EXPANDED)
    named(session, 'undo-redo', b'RW')
    save(session, path, SOURCE)
    step(session, b'\x07\x1bgg1\r', b'L1 C1')


def retained(frame, width=120, height=24):
    content = canvas(frame, width, height)
    return {row: text for row, text in content.items() if row < height}, cursor(frame)


def failures(session, path):
    frame = step(session, b'\x07', b'Quit')
    before = retained(frame)
    named(session, 'untabify', b'Region is not active')
    named(session, 'goto-line', b'Goto line: ')
    draft = b'9223372036854775808'
    frame = step(session, draft + b'\r', b'Enter a signed 64-bit decimal line number')
    assert b'Goto line: ' + draft in frame
    status(frame, RIGHT, f'> {NAME} -- RW -- L1 C1 Rust')
    frame = step(session, b'\x07', b'Quit')
    assert retained(frame) == before, 'invalid/canceled navigation changed source or view'
    frame = visit(session, 'alpha.txt/child', b'Find file: alpha.txt/child  visit "')
    assert b'Find file: alpha.txt/child' in frame
    frame = step(session, b'\x07', b'Quit')
    assert retained(frame) == before, 'failed file ingress changed the retained workspace'
    assert path.read_bytes() == SOURCE


def overlays(session, path):
    frame = step(session, b'\x07', b'Quit')
    before = retained(frame)
    frame = step(session, b'\x13row-17', b'I-search: row-17')
    status(frame, RIGHT, f'> {NAME} -- RW -- L18 C15 Rust')
    assert span(rows(frame)[18], 73, 6) == '  18 │'
    assert 'row-17' in span(rows(frame)[18], 79, 41)
    frame = step(session, b'\x07', b'Quit')
    assert retained(frame) == before, 'search cancellation lost pane projection'
    step(session, b'\x18 ', b'Rectangle mark set')
    step(session, b'\x0e\x06', b'L2 C5')
    step(session, b'\x18r\x1bw', b'Rectangle copied')
    step(session, b'\x07', b'Quit')
    visit(session, 'alpha.txt', b'> alpha.txt -- RW -- L1 C1 Text')
    step(session, b'\x18ry', b'Rectangle yanked')
    target = path.parent / 'alpha.txt'
    save(session, target, b'    alpha\r\n    ')
    status(PTY['last_frame'](session), RIGHT, '> alpha.txt -- RW -- L2 C5 Text')
    step(session, b'\x18u', b'RW')
    save(session, target, b'alpha\r\n')
    visit(session, NAME, NAME.encode())
    step(session, b'\x1bgg1\r', b'L1 C1')
    assert path.read_bytes() == SOURCE


def capture_resize(session):
    # An actual source-edge deadline must fire before resize cancels ownership.
    frame = send(session, press(79, 0) + drag(79, 22), lambda f:
                 int(span(rows(f)[1], 73, 4)) >= 4)
    assert b'\x1b]52;' not in frame
    PTY['resize_surface'](session, 80, 12, True)
    frame = PTY['last_frame'](session)
    before = retained(frame, 80, 12)
    assert not span(rows(frame)[11], 53, 27).endswith('Rust'), 'narrow status did not omit mode first'
    frame = step(session, drag(79, 22) + drag(79, 22, True) + b'\x18', b'C-x')
    assert retained(frame, 80, 12) == before, 'late reports revived capture after resize'
    idle(session)
    step(session, b'\x07', b'Quit')
    PTY['resize_surface'](session, 120, 24, True)
    frame = PTY['last_frame'](session)
    content = canvas(frame, 120)
    assert span(content[11], 25, 47).endswith('Rust')
    assert span(content[23], 25, 47).endswith('Rust')
    assert span(content[23], 73, 47).endswith('Rust')
    step(session, b'\x1bgg1\r', b'L1 C1')
    # C-g also cancels a live captured region, independently of resize/release.
    step(session, press(79, 0) + drag(86, 1), b'L2 C8')
    frame = step(session, b'\x07', b'Quit')
    before = retained(frame)
    frame = step(session, drag(79, 22) + drag(79, 22, True) + b'\x18', b'C-x')
    assert retained(frame) == before, 'C-g left a live selection/capture'
    step(session, b'\x07', b'Quit')


def mouse_control(session):
    step(session, b'\x1bgg1\r', b'L1 C1')
    frame = named(session, 'mouse-mode', b'Mouse mode disabled')
    assert BUTTONS_OFF in session.output and SGR_OFF in session.output
    before = retained(frame)
    frame = step(session, wheel(False, 31, 0) + press(31, 0) + drag(39, 1) + drag(39, 1, True) + b'\x18', b'C-x')
    assert retained(frame) == before, 'disabled pointer reports changed native state'
    step(session, b'\x07', b'Quit')
    named(session, 'mouse-mode', b'Mouse mode enabled')
    assert SGR_ON in session.output and BUTTONS_ON in session.output


def combined(root, colored):
    path = write(root, 'work/' + NAME, SOURCE)
    write(root, 'work/alpha.txt', b'alpha\r\n')
    write(root, 'work/alpine.txt', b'alpine\n')
    expected = startup_snapshot(root)
    assert not (root / 'user').exists(), 'combined journey must enter native first startup'
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, colored))
    try:
        bootstrap(session, root, expected)
        navigation(session)
        workspace(session)
        selection(session, path, colored)
        whitespace(session, path)
        failures(session, path)
        overlays(session, path)
        capture_resize(session)
        mouse_control(session)
        assert snapshot(root) == expected, 'combined editing changed unrelated files/settings'
        session.output.clear()
        finish(session, colored)
        cleanup = bytes(session.output)
        assert cleanup.index(BUTTONS_OFF) < cleanup.index(SGR_OFF) < cleanup.index(PTY['PASTE_OFF'])
    finally:
        close(session)
    assert snapshot(root) == expected
    passive(root)
    print(f'PASS combined {"RGB" if colored else "NO_COLOR"}: first startup, ordered arrows, independent gutters/status, goto, pointed scrolling, canonical drag, exact conversion/history saves, failed/canceled prompts, overlays, resize capture cleanup and runtime mouse control')
