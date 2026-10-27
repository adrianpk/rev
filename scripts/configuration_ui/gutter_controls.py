"""Exercise native numbering controls and admitted startup settings in owned PTYs."""

from configuration_ui.support import (BINARY, PTY, canvas, close, completed,
                                      cursor, environment, finish, send, step,
                                      write)

SOURCE = '\ufeff\t界e\u0301\r\nlast\n'.encode()


def labels(frame, expected, right=False):
    content = canvas(frame)
    for row, label in enumerate(expected, 1):
        actual = content[row][-len(label):] if right else content[row][:len(label)]
        assert actual == label, (row, actual, label)
    return True


def hidden(frame):
    content = canvas(frame)
    assert all('│' not in content[row] for row in range(1, 23))
    assert cursor(frame) == (2, 1)
    return True


def command(session, name, predicate):
    return send(session, b'\x1bxgutter-' + name.encode() + b'\r',
                lambda frame: b'M-x ' not in frame and predicate(frame))


def factory(root):
    path = write(root, 'source.txt', SOURCE)
    config = root / 'user/rev/config.toml'
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, False))
    try:
        session.wait(lambda: completed(session, lambda frame: b'-- RW L1 C1 Text' in frame))
        step(session, b'\x1bgg2\r', b'-- RW L2 C1 Text')
        command(session, 'set-line-numbers-relative', lambda f: labels(f, ['   1 │', '   2 │', '   1 │']))
        command(session, 'toggle-line-numbers', hidden)
        command(session, 'cycle-line-number-mode', hidden)
        command(session, 'cycle-current-line', hidden)
        command(session, 'toggle-line-number-side', hidden)
        command(session, 'set-line-numbers-absolute', lambda f: labels(f, ['│   1 ', '│   0 ', '│   3 '], True))
        command(session, 'set-line-numbers-relative', lambda f: labels(f, ['│   1 ', '│   0 ', '│   1 '], True))
        command(session, 'cycle-current-line', lambda f: labels(f, ['│   1 ', '│     ', '│   1 '], True))
        frame = command(session, 'cycle-current-line', lambda f: labels(f, ['│   1 ', '│   2 ', '│   1 '], True))
        assert b'\x1b[1m' in frame and b'\x1b[22m' in frame
        assert b'\x1b[38;2;' not in frame
        command(session, 'inspect-numbering', lambda f: b'Gutter: visible=true(view) mode=Relative(view)' in f)
        step(session, b'\x1bxgutter-toggle', b'M-x gutter-toggle')
        frame = step(session, b'\x07', b'Quit')
        labels(frame, ['│   1 ', '│   2 ', '│   1 '], True)
        command(session, 'reset-numbering', lambda f: labels(f, ['   1 │', '   2 │', '   3 │']))
        step(session, PTY['paste_wire'](b'Z'), b'** RW L2 C2 Text')
        step(session, b'\x18\x13', b'Saved ')
        assert path.read_bytes() == SOURCE.replace(b'last', b'Zlast')
        step(session, b'\x18u', b'** RW L2 C1 Text')
        step(session, b'\x18\x13', b'Saved ')
        assert path.read_bytes() == SOURCE
        assert not config.exists(), 'runtime controls created configuration'
        finish(session, False)
    finally:
        close(session)
    print('PASS factory gutter controls: eight M-x handlers, all current forms, hidden cycling, side, inspect, reset, C-g, monochrome, edit/save/undo and exact bytes')


def configured(root):
    path = write(root, 'source.txt', SOURCE)
    base = write(root, 'user/rev/base.toml', b"[gutter]\nvisible = true\nmode = 'relative'\ncurrent_line = 'zero'\nside = 'right'\nminimum_digits = 2\nleft_padding = 1\nright_padding = 2\n[palette.gutter]\nbackground = '#010203'\nforeground = '#040506'\ncurrent_line = '#070809'\nseparator = '#0a0b0c'\n")
    config = write(root, 'user/rev/config.toml', b"include = ['base.toml']\ngutter.visible = false\n")
    original = base.read_bytes()
    session = PTY['Session']([str(BINARY), str(path)], env=environment(root, True))
    try:
        session.wait(lambda: completed(session, lambda frame: b'-- RW L1 C1 Text' in frame))
        frame = step(session, b'\x1bgg2\r', b'-- RW L2 C1 Text')
        hidden(frame)
        expected = ['│  1  ', '│  0  ', '│  1  ']
        frame = command(session, 'set-line-numbers-relative', lambda f: labels(f, expected, True))
        assert cursor(frame) == (2, 1)
        for color in [b'\x1b[48;2;1;2;3m', b'\x1b[38;2;4;5;6m', b'\x1b[38;2;7;8;9m', b'\x1b[38;2;10;11;12m']:
            assert color in frame, color
        config.write_bytes(b"gutter.mode = 'absolute'\n")
        command(session, 'reset-numbering', hidden)
        command(session, 'set-line-numbers-relative', lambda f: labels(f, expected, True))
        assert config.read_bytes() == b"gutter.mode = 'absolute'\n"
        assert base.read_bytes() == original and path.read_bytes() == SOURCE
        finish(session, True)
    finally:
        close(session)
    print('PASS configured gutter controls: included leaves, root false, custom format and four RGB roles, immutable reset and unchanged sources')


def run(root):
    for name, case in [('factory', factory), ('configured', configured)]:
        fixture = root / name
        fixture.mkdir()
        case(fixture)
    print('Gutter controls PTY suite: 2 owned sessions passed, 0 skipped.')
