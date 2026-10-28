"""List, checkbox and literal-context steps inside the owned Org PTY journey."""

from .support import canvas, send, step, visit

LINES = ['* Tasks [0/2]', '- [ ] Parent [0/2]', '  - [ ] One',
         '  - [ ] Two', '- [ ] Peer', '* Choices [1/2]', '- [X] Red',
         '- [ ] Blue', '* Sequence', '1. First', '   continued', '2. Second',
         '* Timer text', '- 0:01:02 :: note', '* Literal', '#+begin_example',
         '- [ ] untouched', '#+end_example']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()


def list_journey(session, source, goto, document):
    def exact(frame, lines):
        text = document(frame)
        return text[:len(lines)] == [line.rstrip() for line in lines] and not any(text[len(lines):])

    def action(wire, command, lines):
        return send(session, wire, lambda frame: canvas(frame)[24].rstrip() == command and exact(frame, lines))

    def undo():
        return send(session, b'\x18u', lambda frame: exact(frame, LINES) and b'-- RW --' in frame)

    frame = visit(session, str(source), b'> lists.org -- RW -- L1 C1')
    assert exact(frame, LINES)
    goto(session, 3)
    expected = LINES.copy()
    expected[1], expected[2] = '- [-] Parent [1/2]', '  - [X] One'
    action(b'\x03\x03', 'org-ctrl-c-ctrl-c', expected)
    undo()
    goto(session, 4)
    expected = LINES.copy()
    expected[1], expected[3] = '- [-] Parent [1/2]', '  - [X] Two'
    action(b'\x03\x18\x02', 'org-toggle-checkbox', expected)
    undo()
    goto(session, 8)
    expected = LINES.copy()
    expected[6], expected[7] = '- [ ] Red', '- [X] Blue'
    action(b'\x03\x18\x12', 'org-toggle-radio-button', expected)
    undo()
    goto(session, 10)
    expected = LINES.copy()
    expected[9:12] = ['1. Second', '2. First', '   continued']
    frame = action(b'\x1b[1;3B', 'org-metadown', expected)
    assert b'** RW -- L11 C1' in frame, 'ordered item point did not follow source'
    undo()
    goto(session, 14)
    step(session, b'\x05', b'-- RW -- L14 C18')
    expected = LINES[:14] + ['-  :: '] + LINES[14:]
    action(b'\x1b\r', 'org-meta-return', expected)
    undo()
    goto(session, 5)
    step(session, b'\x05', b'-- RW -- L5 C11')
    expected = LINES[:5] + ['- [ ] '] + LINES[5:]
    expected[0] = '* Tasks [0/3]'
    frame = action(b'\x1b[13;4u', 'org-insert-item-checkbox', expected)
    assert b'** RW -- L6 C7' in frame, 'checkbox insertion point or dirty state missing'
    undo()
    goto(session, 1)
    action(b'\x03#', 'org-update-statistics-cookies', LINES)
    goto(session, 17)
    send(session, b'\x03\x03', lambda frame: exact(frame, LINES)
         and 'No Org checkbox' in canvas(frame)[24] and b'-- RW --' in frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == SOURCE, 'list operations changed BOM/CRLF/EOF bytes after undo'
