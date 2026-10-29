"""Owned RGB/NO_COLOR document edits, exact source format and history."""
from .org_tasks import exact as source_canvas, goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['Text.', '', '<<<wide target>>>', 'wide target', '',
         '#+CAPTION: Keep', 'A paragraph', '', '#+begin_example',
         '[fn:9] literal', '#+end_example', 'EOF']


def encoded(lines):
    return ('\ufeff' + '\r\n'.join(lines)).encode()


def exact(frame, lines):
    # Terminal cells do not retain trailing source spaces; persisted bytes below
    # independently verify BOM, CRLF, definition spacing and final newline state.
    return source_canvas(frame, [line.rstrip() for line in lines])


def journey(root, colored):
    write(root, 'user/rev/config.toml', b'')
    source = write(root, 'work/notes.org', encoded(LINES))
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        step(session, b'\x05', b'L1 C6')
        notes = LINES.copy()
        notes[0] += '[fn:1]'
        notes += ['', '* Footnotes', '', '[fn:1] ']
        send(session, b'\x03\x18f', lambda f: exact(f, notes))
        assert source.read_bytes() == encoded(LINES)
        filled = notes.copy()
        filled[-1] += 'Note 界'
        send(session, 'Note 界'.encode(), lambda f: exact(f, filled))
        send(session, b'\x03\x03', lambda f: b'L1 C6' in f and exact(f, filled))
        send(session, b'\x03\x0f', lambda f: b'L16 C1' in f and exact(f, filled))
        send(session, b'\x18u', lambda f: exact(f, notes))
        send(session, b'\x18u', lambda f: exact(f, LINES))
        named(session, 'undo-redo', b'Org')
        session.wait(lambda: completed(session, lambda f: exact(f, notes)))
        named(session, 'undo-redo', b'Org')
        session.wait(lambda: completed(session, lambda f: exact(f, filled)))
        goto(session, 7)
        step(session, b'\x00', b'Mark set')
        step(session, b'\x05', b'L7 C12')
        step(session, b'\x03\x18\x06', b'Emphasis marker')
        emphasis = filled.copy()
        emphasis[6] = '*A paragraph*'
        send(session, b'*\r', lambda f: exact(f, emphasis))
        send(session, b'\x18u', lambda f: exact(f, filled))
        goto(session, 7)
        named(session, 'org-insert-entity', b'Org entity name: ')
        entities = filled.copy()
        entities[6] = '\\alpha{}A paragraph'
        send(session, b'alpha\r', lambda f: exact(f, entities))
        named(session, 'org-edit-affiliated-keyword', b'Affiliated keyword')
        step(session, b'CAPTION\r', b'Affiliated value: ')
        final = entities.copy()
        final[5] = '#+CAPTION: Wide caption'
        send(session, b'Wide caption\r', lambda f: exact(f, final))
        goto(session, 4)
        send(session, b'\x03\x0f', lambda f: b'L3 C1' in f and exact(f, final))
        send(session, b'\x03&', lambda f: b'L4 C1' in f and exact(f, final))
        goto(session, 10)
        frame = step(session, b'\x03\x18f', b'Literal block')
        assert exact(frame, final)
        goto(session, 7)
        named(session, 'org-insert-entity', b'Org entity name: ')
        frame = step(session, b'\x07', b'Org document action cancelled;')
        assert exact(frame, final)
        quiet(session)
        step(session, b'\x18\x13', b'Saved ')
        assert source.read_bytes() == encoded(final)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/notes.org'] = encoded(final)
        after = snapshot(root)
        assert after == before, f'document fixture entries differ: {[k for k in set(after) | set(before) if after.get(k) != before.get(k) or (k in after) != (k in before)]}'
    finally:
        close(session)
