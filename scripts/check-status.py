#!/usr/bin/env python3
"""Verify two-group status geometry and fixture matching on owned terminals."""

from pathlib import Path
import re
import subprocess
import tempfile

from configuration_ui.gutter_controls import command
from configuration_ui.support import (BINARY, PTY, ROOT, canvas, cells, close,
                                      completed, environment, finish, step,
                                      write)


def matcher():
    marker = b'> main.rs ** RO -- L37 C3 Rust'
    good = b'prefix> main.rs ** RO -- L37 C3                 Rust\x1b[0m'
    assert PTY['status_contains'](good, marker)
    # The adapter retains every fact and cannot borrow mode from another pane,
    # row or redraw. Non-status markers still require exact bytes.
    bad = [good.replace(a, b) for a, b in [
        (b'main.rs', b'other.rs'), (b'**', b'--'), (b'RO', b'RW'),
        (b'L37', b'L38'), (b'C3', b'C4'), (b'Rust', b'Text'),
        (b'                 ', b' | '), (b'                 ', b'\r\n'),
        (b'                 ', b'\x1b[23;41H'),
    ]]
    assert all(not PTY['status_contains'](frame, marker) for frame in bad)
    assert not PTY['status_contains'](good.replace(b'Rust', b''), marker)
    assert not PTY['status_contains'](good.replace(b'                 ', b''), marker)
    assert PTY['status_contains'](b'Find file: alpha', b'Find file: alpha')
    assert not PTY['status_contains'](b'Find file:  alpha', b'Find file: alpha')
    print('PASS status matcher: all facts, exact non-status bytes and pane/frame boundaries')


def groups(text, left, mode, width):
    assert cells(text) == width, (text, width)
    assert text.startswith(left) and text.endswith(mode), text
    assert text[len(left):-len(mode)].strip() == '', text
    assert text[len(left):-len(mode)], 'groups have no separating space'


def statuses(frame, name):
    content = canvas(frame)
    result = []
    for text in content.values():
        # The fixture's status label contains no pane divider. Each complete
        # group is checked inside its own measured split, including mode edge.
        for part in text.split('│'):
            if name in part and re.search(r'(?:--|\*\*) RW -- L', part):
                result.append(part)
    return result


def split(session, name):
    step(session, b'\x183', b'-- RW -- L1 C1 Rust')
    frame = step(session, b'\x18o\x0e\x06', b'-- RW -- L2 C2 Rust')
    parts = statuses(frame, name)
    assert len(parts) == 2, parts
    groups(parts[0], f'  {name} -- RW -- L1 C1', 'Rust', 39)
    groups(parts[1], f'> {name} -- RW -- L2 C2', 'Rust', 40)
    step(session, b'!', b'** RW -- L2 C3 Rust')
    frame = step(session, b'\x18u', b'-- RW -- L2 C2 Rust')
    assert len(statuses(frame, name)) == 2
    # A completion prompt owns the bottom rows and does not inherit document
    # mode or point. Cancel restores both status rows and stored view points.
    frame = step(session, b'\x1bxgoto', b'M-x goto')
    assert b'goto-line' in frame
    assert any(text.startswith('M-x goto') for text in canvas(frame).values())
    frame = step(session, b'\x07', b'Quit')
    assert statuses(frame, name) == parts
    step(session, b'\x181', b'-- RW -- L2 C2 Rust')


def widths(session, name):
    # Full, language omission, compact position, clipped name, complete access,
    # and suspension all restore to the original complete status.
    for width, expected in [
        (31, f'> {name} -- RW -- L2 C2    Rust'),
        (27, f'> {name} -- RW -- L2 C2'),
        (22, f'> {name} -- RW -- L2:2'),
        (18, '> 界 -- RW -- L2:2'),
        (10, '> 界 -- RW'), (6, '>-- RW'), (3, '>--'),
    ]:
        PTY['resize_surface'](session, width, 24, True)
        content = canvas(PTY['last_frame'](session), width, 24)
        assert content[23].rstrip() == expected, (width, content[23], expected)
    for width, height in [(0, 0), (80, 24)]:
        PTY['resize_surface'](session, width, height, True)
    text = canvas(PTY['last_frame'](session))[23]
    groups(text, f'> {name} -- RW -- L2 C2', 'Rust', 80)


def case(root, colored):
    name = '界e\u0301.rs'
    source = b'a\tmain\nsecond\n'
    path = write(root, name, source)
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, colored))
    try:
        session.wait(lambda: completed(session, lambda f: PTY['status_contains'](f, b'-- RW -- L1 C1 Rust')))
        frame = PTY['last_frame'](session)
        original = canvas(frame)[23]
        groups(original, f'> {name} -- RW -- L1 C1', 'Rust', 80)
        frame = command(session, 'toggle-line-number-side', lambda f: canvas(f)[1].endswith('│   1 '))
        assert canvas(frame)[23] == original, 'gutter side moved status bounds'
        if colored:
            assert b'\x1b[48;2;22;22;30m' in frame
        else:
            assert b'\x1b[38;2;' not in frame and b'\x1b[48;2;' not in frame
        split(session, name)
        widths(session, name)
        assert path.read_bytes() == source
        finish(session, colored)
    finally:
        close(session)
    assert path.read_bytes() == source
    print(f'PASS status {"RGB" if colored else "NO_COLOR"}: exact pane edges, Unicode, shared points, gutter independence, prompt cancel, narrow/zero restoration and source bytes')


def main():
    matcher()
    subprocess.run(['cargo', 'build', '--bin', 'rev'], cwd=ROOT, check=True)
    for colored in [True, False]:
        with tempfile.TemporaryDirectory(prefix='rev-status-') as directory:
            case(Path(directory), colored)
    print('Status PTY suite: 2 owned sessions passed, 0 skipped.')


if __name__ == '__main__':
    main()
