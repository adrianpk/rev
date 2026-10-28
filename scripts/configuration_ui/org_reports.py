"""One real terminal report journey shares the owned task fixture and cleanup."""
from .support import PTY, send, step


def reports(session, source, lines, goto, exact, document):
    fixture = ['* Report é', ':PROPERTIES:', ':EFFORT: 2:00', ':END:',
               'CLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 1:00',
               'CLOCK: [2024-01-01 Mon 12:00]',
               '#+BEGIN: clocktable :scope file :block untilnow :maxlevel 2',
               'manual report', '#+END:']
    step(session, b'\x1b<', b'L1 C1')
    step(session, b'\x00', b'Mark set')
    step(session, b'\x1b>', b'L10 C9')
    send(session, b'\x17', lambda frame: exact(frame, ['']) and b'** RW --' in frame)
    send(session, PTY['paste_wire']('\r\n'.join(fixture).encode()),
         lambda frame: exact(frame, fixture) and b'** RW --' in frame)
    step(session, b'\x03\x18\x04', b'Clock scope [start-date end-date]: ')
    send(session, b'file\r', lambda frame: b'actual 1:00; effort 2:00; open 1; unresolved 0; overlaps 0' in frame
         and exact(frame, fixture))
    goto(session, 7)
    table = ['| Headline | Time | EFFORT | Open | Unresolved |', '|-|-|-|-|-|',
             '| *Total time* | *1:00* | 2:00 | 1 | 0 |',
             '| Report é | 1:00 | 2:00 | 1 | 0 |', '',
             'Open: 1; unresolved: 0; overlapping intervals: 0.']
    expected = fixture[:7] + table + fixture[8:]
    frame = send(session, b'\x03\x18\x15', lambda frame: b'Org clocktable updated' in frame
                 and exact(frame, expected))
    assert exact(frame, expected), document(frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(expected)).encode()
    goto(session, 7)
    send(session, b'\x03\x03', lambda frame: b'Org clocktable unchanged' in frame and exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, fixture))
    send(session, b'\x18u', lambda frame: exact(frame, ['']))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()
