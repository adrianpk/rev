"""Atomic repeat completion extends the existing owned native task journey."""
from datetime import datetime, timezone
from .support import PTY, send, step


def repeats(session, source, lines, goto, exact, document):
    goto(session, 3)
    step(session, b'\x03\x13', b'Org date: ')
    prepared = lines.copy()
    prepared[3] = 'SCHEDULED: <2024-02-28 Wed +1d --2d>'
    send(session, PTY['paste_wire'](b'<2024-02-28 Wed +1d --2d>') + b'\r',
         lambda frame: b'Org timestamp changed' in frame and exact(frame, prepared))
    step(session, b'\x03\x14', b'Org task state: ')
    before = datetime.now(timezone.utc)
    step(session, b'd\r', b'Org state note: ')
    frame = send(session, PTY['paste_wire']('repeated 界'.encode()) + b'\r',
                 lambda frame: b'Org task state and metadata changed' in frame
                 and any(':LAST_REPEAT:' in line for line in document(frame)))
    stamp = next(line.split(': ', 1)[1] for line in document(frame) if line.startswith(':LAST_REPEAT:'))
    value = datetime.fromisoformat(f'{stamp[1:11]}T{stamp[16:21]}:00').replace(tzinfo=timezone.utc)
    assert before.replace(second=0, microsecond=0) <= value <= datetime.now(timezone.utc), stamp
    expected = prepared.copy()
    expected[3] = 'SCHEDULED: <2024-02-29 Thu +1d>'
    expected = expected[:6] + [f':LAST_REPEAT: {stamp}'] + expected[6:7] + [
        ':LOGBOOK:', f'- State "DONE" from "TODO" {stamp} \\\\',
        '  repeated 界', ':END:'] + expected[7:]
    assert exact(frame, expected), document(frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(expected)).encode()
    send(session, b'\x18u', lambda frame: exact(frame, prepared))
    send(session, b'\x1bxundo-redo\r', lambda frame: exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, prepared))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()
