"""Owned native table keys, exact canvas/source format, history and cancellation."""
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['#+NAME: numbers', '#+CAPTION: Keep', '|Name|Count|', '|--+--|',
         '|界e\u0301|2|', '|Long|11|', '#+TBLFM: $2=$1+1', '', '', 'Text.',
         '#+begin_example', '|A|B|', '#+end_example', 'EOF']


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
        goto(session, 3)
        aligned = LINES.copy()
        aligned[2:6] = ['| Name | Count |', '|------+-------|',
                        '| 界e\u0301  |     2 |', '| Long |    11 |']
        send(session, b'\x03\x03', lambda f: exact(f, aligned))
        step(session, b'\x06\x06\t', b'L3 C10')
        frame = step(session, b'\x1b[Z', b'L3 C3')
        assert exact(frame, aligned)
        named(session, 'org-table-end-of-field', b'L3 C7')
        named(session, 'org-table-beginning-of-field', b'L3 C3')
        frame = step(session, b'\r', b'reference rewriting')
        assert exact(frame, aligned)
        goto(session, 9)
        step(session, b'\x03|', b'Table size (columns x rows, default 5x2): ')
        created = aligned[:8] + ['|   |', '|---|'] + aligned[9:]
        send(session, b'1x1\r', lambda f: exact(f, created))
        send(session, b'\x18u', lambda f: exact(f, aligned))
        named(session, 'undo-redo', b'Org')
        session.wait(lambda: completed(session, lambda f: exact(f, created)))
        current = created.copy()
        current[8] = '| X  |'
        send(session, b'X', lambda f: exact(f, current))
        current[8] = '| X |'
        current.insert(10, '|   |')
        send(session, b'\t', lambda f: b'L11 C3' in f and exact(f, current))
        current[10] = '| Z  |'
        send(session, b'Z', lambda f: exact(f, current))
        inserted = current.copy()
        inserted[10] = '| Z |'
        inserted.insert(10, '|   |')
        named(session, 'org-table-insert-row', b'Org table')
        session.wait(lambda: completed(session, lambda f: exact(f, inserted)))
        send(session, b'\x18u', lambda f: exact(f, current))
        goto(session, 8)
        named(session, 'org-table-create', b'Table size (columns x rows, default 5x2): ')
        frame = step(session, b'\x07', b'Org table cancelled; source unchanged')
        assert exact(frame, current)
        goto(session, 14)
        frame = named(session, 'org-table-align', b'No native Org table at point')
        assert exact(frame, current)
        goto(session, 3)
        frame = named(session, 'org-table-insert-hline', b'reference rewriting')
        assert exact(frame, current)
        goto(session, 12)
        step(session, b'\x05', b'L12 C6')
        current[11] += ' edited'
        send(session, b' edited', lambda f: exact(f, current))
        assert source.read_bytes() == encoded(LINES)
        quiet(session)
        step(session, b'\x18\x13', b'Saved ')
        assert source.read_bytes() == encoded(current)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/notes.org'] = encoded(current)
        after = snapshot(root)
        assert after == before, 'table fixture entries changed outside the canonical source'
    finally:
        close(session)
