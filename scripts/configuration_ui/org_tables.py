"""Owned native table keys, exact canvas/source format, history and cancellation."""
import csv
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['#+NAME: numbers', '#+CAPTION: Keep', '|Name|Count|', '|--+--|',
         '|界e\u0301|2|', '|Long|11|', '#+TBLFM: $2=$1+1', '', '', 'Text.',
         '#+begin_example', '|A|B|', '#+end_example', 'EOF']
DATA = 'A,B\r\n"é,界","q""uote"\r\n'.encode()
EXPORTED = '"A","B"\n"é,界","q""uote"\n'.encode()


def journey(root, colored):
    write(root, 'user/rev/config.toml', b'')
    source = write(root, 'work/notes.org', encoded(LINES))
    write(root, 'work/input.csv', DATA)
    write(root, 'work/bad.csv', b'A,B\n"broken\ncell",x\n')
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
        inserted = aligned.copy()
        inserted.insert(3, '|      |       |')
        send(session, b'\r', lambda f: exact(f, inserted))
        send(session, b'\x18u', lambda f: exact(f, aligned))
        structure(session, aligned)
        recalculation(session, aligned)
        interchange(session, root, aligned)
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
        ruled = current.copy()
        ruled.insert(3, '|------+-------|')
        named(session, 'org-table-insert-hline', b'Org table')
        session.wait(lambda: completed(session, lambda f: exact(f, ruled)))
        send(session, b'\x18u', lambda f: exact(f, current))
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
        before['work/output.csv'] = EXPORTED
        after = snapshot(root)
        assert after == before, 'table fixture entries changed outside checked source/data outputs'
    finally:
        close(session)


def interchange(session, root, original):
    # Both existing owned sessions exercise real M-x/file operations. Python's
    # independent CSV reader checks semantic values, alongside exact file bytes.
    goto(session, 8)
    named(session, 'org-table-import', b'Import table file (.csv/.tsv): ')
    imported = original.copy()
    imported[7:8] = ['| A    | B      |', '| é,界 | q"uote |']
    send(session, b'input.csv\r', lambda f: exact(f, imported))
    goto(session, 8)
    named(session, 'org-table-export', b'Export table file (.csv/.tsv): ')
    step(session, b'output.csv\r', b'Exported table data')
    output = root / 'work/output.csv'
    assert output.read_bytes() == EXPORTED
    with output.open(encoding='utf-8', newline='') as stream:
        assert list(csv.reader(stream, strict=True)) == [['A', 'B'], ['é,界', 'q"uote']]
    assert (root / 'work/input.csv').read_bytes() == DATA
    send(session, b'\x18u', lambda f: exact(f, original))
    named(session, 'undo-redo', b'Org')
    session.wait(lambda: completed(session, lambda f: exact(f, imported)))
    send(session, b'\x18u', lambda f: exact(f, original))
    goto(session, 8)
    named(session, 'org-table-import', b'Import table file (.csv/.tsv): ')
    frame = step(session, b'bad.csv\r', b'Embedded cell line breaks')
    assert exact(frame, original)


def structure(session, original):
    # Existing RGB/NO_COLOR sessions exercise physical structural keys, checked
    # formula digits, sort prompts and whole-operation history on one source.
    goto(session, 3)
    moved = original.copy()
    moved[2:7] = ['| Count | Name |', '|-------+------|',
                  '|     2 | 界e\u0301  |', '|    11 | Long |',
                  '#+TBLFM: $1=$2+1']
    send(session, b'\x1b[1;3C', lambda f: exact(f, moved))
    send(session, b'\x18u', lambda f: exact(f, original))

    named(session, 'undo-redo', b'Org')
    session.wait(lambda: completed(session, lambda f: exact(f, moved)))
    send(session, b'\x1b[1;3D', lambda f: exact(f, original))
    expanded = original.copy()
    expanded[2:7] = ['|   | Name | Count |', '|---+------+-------|',
                     '|   | 界e\u0301  |     2 |', '|   | Long |    11 |',
                     '#+TBLFM: $3=$2+1']
    send(session, b'\x1b[1;4C', lambda f: exact(f, expanded))
    send(session, b'\x1b[1;4D', lambda f: exact(f, original))
    goto(session, 5)
    named(session, 'org-table-next-field', b'Org table')
    session.wait(lambda: completed(session, lambda f: b'Checking Org table' not in f and exact(f, original)))
    step(session, b'\x03^', b'Sort type (a/A alphabetical, n/N numeric): ')
    descending = original.copy()
    descending[4:6] = list(reversed(descending[4:6]))
    send(session, b'N\r', lambda f: exact(f, descending))
    send(session, b'\x18u', lambda f: exact(f, original))
    step(session, b'\x03^', b'Sort type (a/A alphabetical, n/N numeric): ')
    frame = step(session, b'\x07', b'Org table cancelled; source unchanged')
    assert exact(frame, original)
    goto(session, 6)
    send(session, b'\x1b[1;3A', lambda f: exact(f, descending))
    send(session, b'\x18u', lambda f: exact(f, original))
    killed = original.copy()
    del killed[5]
    send(session, b'\x1b[1;4A', lambda f: exact(f, killed))
    send(session, b'\x18u', lambda f: exact(f, original))
    goto(session, 7)
    step(session, b'\x05', b'L7 C17')
    unsupported = original.copy()
    unsupported[6] += '::weird=unknown()'
    send(session, b'::weird=unknown()', lambda f: exact(f, unsupported))
    goto(session, 3)
    frame = step(session, b'\x1b[1;3C', b'reference rewriting')
    assert exact(frame, unsupported)
    send(session, b'\x18u', lambda f: exact(f, original))


def recalculation(session, original):
    # The same owned sessions check explicit physical keys, named replay,
    # exact source canvases and failure preservation without widening limits.
    goto(session, 7)
    blank = original.copy()
    blank[6] = ''
    send(session, b'\x01\x0b', lambda f: exact(f, blank))
    draft = original.copy()
    draft[6] = '#+TBLFM: $1=$2*2+1'
    send(session, draft[6].encode(), lambda f: exact(f, draft))
    calculated = draft.copy()
    calculated[4] = draft[4].replace('界e\u0301', '5')
    calculated[5] = draft[5].replace('Long', '23')
    goto(session, 5)
    send(session, b'\x03*', lambda f: exact(f, calculated))
    send(session, b'\x18u', lambda f: exact(f, draft))
    named(session, 'undo-redo', b'Org')
    session.wait(lambda: completed(session, lambda f: exact(f, calculated)))
    send(session, b'\x18u', lambda f: exact(f, draft))
    for suffix, diagnostic in [('::@2$1=1/0', b'DivisionByZero'),
                               ('::$1=sin($2)', b'Unsupported')]:
        goto(session, 7)
        step(session, b'\x05', b'L7 C19')
        rejected = draft.copy()
        rejected[6] += suffix
        send(session, suffix.encode(), lambda f: exact(f, rejected))
        goto(session, 5)
        frame = named(session, 'org-table-recalculate', diagnostic)
        assert exact(frame, rejected)
        send(session, b'\x18u', lambda f: exact(f, draft))
    send(session, b'\x18u', lambda f: exact(f, blank))
    send(session, b'\x18u', lambda f: exact(f, original))
