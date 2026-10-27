"""Verify factory absolute numbers and source fidelity through owned PTYs."""

from configuration_ui.support import (BINARY, PTY, canvas, close, completed,
                                      cursor, environment, finish, send, step,
                                      write)


def absolute(root):
    for name, source, lines, colored in [
        ('unicode-crlf', '\ufeff\t界e\u0301\r\nlast\n'.encode(), 3, True),
        ('empty-monochrome', b'', 1, False),
    ]:
        fixture = root / name
        fixture.mkdir()
        path = write(fixture, 'source.txt', source)
        session = PTY['Session']([str(BINARY), str(path)], env=environment(fixture, colored))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1 Text')))
            frame = PTY['last_frame'](session)
            content = canvas(frame)
            for row in range(1, 23):
                label = f'{row:>4} │' if row <= lines else '     │'
                assert content[row].startswith(label), (row, content[row])
            assert cursor(frame) == (1, 7)
            assert b'\x1b[1m' in frame and b'\x1b[22m' in frame, 'current label lost non-color emphasis'
            status = f'-- RW -- L{lines} C1 Text'.encode()
            frame = send(session, b'\x1bgg' + str(lines).encode() + b'\r',
                         lambda frame: PTY['status_contains'](frame, status) and b'Goto line: ' not in frame)
            assert cursor(frame) == (lines, 7), 'logical line and text origin disagree'
            step(session, PTY['paste_wire'](b'Z'), f'** RW -- L{lines} C2 Text'.encode())
            step(session, b'\x18\x13', b'Saved ')
            assert path.read_bytes() == source + b'Z', 'labels or presentation changed saved bytes'
            step(session, b'\x18u', f'** RW -- L{lines} C1 Text'.encode())
            frame = step(session, b'\x18\x13', b'Saved ')
            assert path.read_bytes() == source, 'undo/save lost original BOM or separators'
            restored = canvas(frame)
            assert {r: restored[r] for r in range(1, 23)} == {r: content[r] for r in range(1, 23)}
            finish(session, colored)
            print(f'PASS absolute gutter {name}: real/padding labels, current emphasis, cursor, edit/save/undo and exact bytes')
        finally:
            close(session)
    print('Absolute gutter PTY suite: 2 owned sessions passed, 0 skipped.')
