"""Priority and effort steps extend the existing owned native task journey."""
from .support import PTY, send, step


def estimates(session, source, lines, goto, exact):
    goto(session, 3)
    step(session, b'\x03,', b'Org priority: ')
    expected = lines.copy()
    expected[2] = expected[2].replace('[#A]', '[#C]')
    send(session, b'C\r', lambda frame: b'Org priority changed' in frame and exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, lines) and b'-- RW --' in frame)
    step(session, b'\x1bxorg-priority\r', b'Org priority: ')
    send(session, b'9\r', lambda frame: b'uppercase A-Z' in frame and exact(frame, lines))
    step(session, b'\x03\x18e', b'Org effort: ')
    send(session, b'1.5:30\r', lambda frame: b'unsigned integers' in frame and exact(frame, lines))
    step(session, b'\x03\x18e', b'Org effort: ')
    expected = lines[:6] + [':EFFORT: 1d3h5min'] + lines[6:]
    send(session, b'1d3h5min\r', lambda frame: b'Org effort changed' in frame and exact(frame, expected))
    step(session, b'\x1bxorg-inspect-effort\r', b'1d3h5min = 1625 minutes')
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(expected)).encode()
    send(session, b'\x18u', lambda frame: exact(frame, lines) and b'** RW --' in frame)
    step(session, b'\x1bxundo-redo\r', b'-- RW --')
    assert exact(PTY['last_frame'](session), expected)
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()
    step(session, b'\x03\x18e', b'Org effort: ')
    send(session, b'cancelled\x07', lambda frame: b'Quit' in frame and exact(frame, lines))
