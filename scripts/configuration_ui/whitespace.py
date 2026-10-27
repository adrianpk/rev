"""Exercise explicit conversion and exact saved bytes through owned PTYs."""

from configuration_ui.support import (BINARY, PTY, close, completed, environment,
                                      finish, step, write)


def run(root):
    for name, width, source, expanded, compressed, line, column in [
        ('unicode-crlf', 4, '\ufeff界e\u0301\tX\r\n\tY', '\ufeff界e\u0301 X\r\n    Y', '\ufeff界e\u0301 X\r\n\tY', 2, 6),
        ('configured-final-line', 8, 'a\tB\n\tC\n', 'a       B\n        C\n', 'a\tB\n\tC\n', 3, 1),
    ]:
        fixture = root / name
        fixture.mkdir()
        write(fixture, 'user/rev/config.toml', f'editor.tab_width = {width}\n'.encode())
        path = write(fixture, 'source.txt', source.encode())
        session = PTY['Session']([str(BINARY), str(path)], env=environment(fixture))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1 Text')))
            step(session, b'\x1bxuntabify\r', b'Region is not active')
            status = f'L{line} C{column} Text'.encode()
            select = b'\x1b<\x00\x1b>'
            step(session, select, status)
            step(session, b'\x1bxuntabify\r', b'** RW ' + status)
            save(session)
            assert path.read_bytes() == expanded.encode(), 'untabify changed source fidelity'
            step(session, b'\x18u', b'** RW ' + status)
            save(session)
            assert path.read_bytes() == source.encode(), 'one undo did not restore every run'
            step(session, b'\x1bxundo-redo\r', b'** RW ' + status)
            save(session)
            assert path.read_bytes() == expanded.encode(), 'redo did not restore conversion'
            step(session, select, status)
            step(session, b'\x1bxtabify\r', b'** RW ' + status)
            save(session)
            assert path.read_bytes() == compressed.encode(), 'tabify did not preserve columns/fidelity'
            step(session, select, status)
            step(session, b'\x1bxtabify\r', b'No whitespace changed')
            step(session, b'\x1bxuntabify', b'M-x untabify')
            step(session, b'\x07', b'Quit')
            assert path.read_bytes() == compressed.encode(), 'canceled command wrote source'
            finish(session)
            print(f'PASS whitespace {name}: explicit region, both M-x commands, one undo/redo, exact saves, no-op and cancellation')
        finally:
            close(session)
    print('Whitespace PTY suite: 2 owned sessions passed, 0 skipped.')


def save(session):
    step(session, b'\x18\x13', b'Saved ')
