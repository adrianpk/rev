"""Timestamp prompts and source-preserving undo in the owned task PTYs."""
from .support import PTY, send, step


def timestamps(session, source, lines, goto, exact):
    goto(session, 3)
    step(session, b'\x03\x04', b'Org date: ')
    step(session, PTY['paste_wire']('来週'.encode()) + b'\r', b'Unknown Org month')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()
    send(session, b'\x07', lambda frame: b'Quit' in frame and exact(frame, lines))
    step(session, b'\x03\x13', b'Org date: ')
    expected = lines.copy()
    expected[3] = 'SCHEDULED: <2026-10-30 Fri 09:30-10:00>'
    send(session, PTY['paste_wire'](b'2026-10-30 09:30-10:00') + b'\r',
         lambda frame: b'Org timestamp changed' in frame and exact(frame, expected))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(expected)).encode()
    goto(session, 4)
    step(session, b'\x05', b'C40')
    shifted = expected.copy()
    shifted[3] = shifted[3].replace('2026-10-30 Fri', '2026-10-31 Sat')
    send(session, b'\x1b[1;2C', lambda frame: b'Org timestamp changed' in frame and exact(frame, shifted))
    send(session, b'\x18u', lambda frame: exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    send(session, b'\x1bxundo-redo\r', lambda frame: exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()


def readonly_calendar(session, lines, exact):
    step(session, b'\x03>', b'Org date: ')
    send(session, PTY['paste_wire'](b'2026-10-30') + b'\r',
         lambda frame: b'Org calendar date selected' in frame and exact(frame, lines))
    send(session, b'\x03<', lambda frame: b'buffer is read-only' in frame and exact(frame, lines))
