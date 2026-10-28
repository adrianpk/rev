"""Native Org visibility through the existing owned controlling-terminal driver."""

from .support import (BINARY, PTY, canvas, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['* Root 界e\u0301', ':LOGBOOK:', 'clock note', ':END:', 'root body',
         '** Child', 'child body', '*** Grand', 'grand body', '** Sibling',
         'sib body', '* Peer', 'peer body']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()


def document(frame):
    return [text[6:].rstrip() for row, text in canvas(frame).items() if row <= 22]


def visible(frame, indexes):
    text = document(frame)
    expected = [LINES[index] for index in indexes]
    return text[:len(expected)] == expected and not any(text[len(expected):])


def change(session, wire, echo, indexes):
    return send(session, wire, lambda frame: canvas(frame)[24].rstrip() == echo
                and visible(frame, indexes))


def named(session, name, indexes):
    step(session, b'\x1bx', b'M-x ')
    step(session, PTY['paste_wire'](name.encode()), b'M-x ' + name.encode())
    return change(session, b'\r', 'Org show-all' if name == 'org-show-all' else name, indexes)


def goto(session, line):
    step(session, b'\x1bxgoto-line\r', b'Goto line: ')
    return step(session, str(line).encode() + b'\r', f'-- RW -- L{line} C1'.encode())


def cycles(session):
    for indexes in [[0, 11, 12], [0, 5, 9, 11, 12], list(range(13))]:
        frame = change(session, b'\t', 'org-cycle', indexes)
        assert b'-- RW -- L1 C1' in frame
    for indexes in [[0, 11], [0, 5, 7, 9, 11], list(range(13))]:
        change(session, b'\x1b[Z', 'org-shifttab', indexes)
    goto(session, 2)
    named(session, 'org-hide-drawer', [i for i in range(13) if i != 2])
    named(session, 'org-show-drawer', list(range(13)))
    goto(session, 1)


def shared(session):
    named(session, 'org-overview', [0, 11])
    step(session, b'\x18' + b'3', b'-- RW -- L1 C1')
    step(session, b'\x1bxorg-show-all\r', b'Org show-all')
    frame = PTY['last_frame'](session)
    text = canvas(frame)
    assert ':LOGBOOK:' in text[2][:39] and '* Peer' in text[2][40:], text[2]
    step(session, b'\x18o', b'-- RW -- L1 C1')
    # The peer retained its overview phase, so its next native cycle is contents.
    step(session, b'\x1b[Z', b'org-shifttab')
    text = canvas(PTY['last_frame'](session))
    assert 'clock note' in text[3][:39] and '*** Grand' in text[3][40:], text[3]
    step(session, b'\x18' + b'0', b'-- RW -- L1 C1')
    assert visible(PTY['last_frame'](session), list(range(13)))


def search_cancel(session):
    named(session, 'org-overview', [0, 11])
    frame = send(session, b'\x13grand body', lambda frame: 'grand body' in document(frame)
                 and 'sib body' not in document(frame) and 'child body' not in document(frame))
    assert b'\x1b[4m' in frame, 'search underline missing'
    change(session, b'\x07', 'Quit', [0, 11])
    step(session, b'\x1bxorg-', b'M-x org-')
    change(session, b'\x07', 'Quit', [0, 11])
    named(session, 'org-show-all', list(range(13)))


def outline(root, colored):
    write(root, 'user/rev/config.toml', b'')
    source = write(root, 'work/notes.org', SOURCE)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)], env=environment(root, colored))
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: visible(frame, list(range(13)))))
        cycles(session)
        shared(session)
        search_cancel(session)
        step(session, b'\x18\x13', b'Saved')
        assert source.read_bytes() == SOURCE, 'folding changed BOM/CRLF/EOF bytes'
        frame = PTY['last_frame'](session)
        assert b'-- RW --' in frame and b'** RW --' not in frame
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'outline session changed fixture files'
    finally:
        close(session)
