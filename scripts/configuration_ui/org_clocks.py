"""Native clock source transactions extend the owned task PTY journey."""
from datetime import datetime, timezone
import re
from .support import send, step

STAMP = re.compile(r'\[\d{4}-\d{2}-\d{2} [A-Z][a-z]{2} \d{2}:\d{2}\]')


def date(stamp):
    return datetime.fromisoformat(f'{stamp[1:11]}T{stamp[16:21]}:00').replace(tzinfo=timezone.utc)


def clocks(session, source, lines, goto, exact, document):
    goto(session, 3)
    before = datetime.now(timezone.utc).replace(second=0, microsecond=0)
    frame = send(session, b'\x03\x18\x09', lambda frame: b'Org clock started' in frame
                 and any(line.startswith('CLOCK: ') for line in document(frame)))
    start_line = next(line for line in document(frame) if line.startswith('CLOCK: '))
    start = STAMP.findall(start_line)
    assert len(start) == 1, start_line
    assert before <= date(start[0]) <= datetime.now(timezone.utc), start
    opened = lines[:7] + [':LOGBOOK:', start_line, ':END:'] + lines[7:]
    assert exact(frame, opened), document(frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(opened)).encode()
    goto(session, 12)
    send(session, b'\x03\x18\x0a', lambda frame: b'org-clock-goto' in frame and b'L3 C1' in frame)
    frame = send(session, b'\x1bxorg-clock-out\r', lambda frame: b'org-clock-out' in frame
                 and any(' => ' in line for line in document(frame)))
    closed_line = next(line for line in document(frame) if line.startswith('CLOCK: '))
    endpoints = STAMP.findall(closed_line)
    assert len(endpoints) == 2 and endpoints[0] == start[0], closed_line
    assert date(endpoints[0]) <= date(endpoints[1]) <= datetime.now(timezone.utc), endpoints
    minutes = int((date(endpoints[1]) - date(endpoints[0])).total_seconds() // 60)
    duration = f'{minutes // 60}:{minutes % 60:02}'
    assert closed_line == f'CLOCK: {endpoints[0]}--{endpoints[1]} => {duration:>5}', closed_line
    closed = lines[:7] + [':LOGBOOK:', closed_line, ':END:'] + lines[7:]
    assert exact(frame, closed), document(frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(closed)).encode()
    send(session, b'\x18u', lambda frame: exact(frame, opened))
    send(session, b'\x1bxundo-redo\r', lambda frame: exact(frame, closed))
    send(session, b'\x18u', lambda frame: exact(frame, opened))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    goto(session, 3)
    send(session, b'\x1bxorg-clock-in\r', lambda frame: b'Org clock started' in frame
         and any(line.startswith('CLOCK: ') for line in document(frame)))
    send(session, b'\x03\x18\x11', lambda frame: b'org-clock-cancel' in frame and exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()


def lifecycle(session, source, lines, goto, exact, document):
    goto(session, 3)
    frame = send(session, b'\x03\x18\x09', lambda frame: b'Org clock started' in frame
                 and any(line.startswith('CLOCK: ') for line in document(frame)))
    child = next(line for line in document(frame) if line.startswith('CLOCK: '))
    opened = lines[:7] + [':LOGBOOK:', child, ':END:'] + lines[7:]
    assert exact(frame, opened), document(frame)
    goto(session, 12)
    frame = send(session, b'\x03\x18\x09', lambda frame: b'Org clock started' in frame
                 and sum(line.startswith('CLOCK: ') for line in document(frame)) == 2)
    records = [line for line in document(frame) if line.startswith('CLOCK: ')]
    assert STAMP.findall(records[0])[0] == STAMP.findall(child)[0] and ' => ' in records[0]
    assert len(STAMP.findall(records[1])) == 1 and ' => ' not in records[1], records
    expected = opened.copy()
    expected[8] = records[0]
    expected = expected[:12] + [':LOGBOOK:', records[1], ':END:'] + expected[12:]
    assert exact(frame, expected), document(frame)
    step(session, b'\x18\x03', b'Org active clock: ')
    send(session, b'\x07', lambda frame: b'Quit' in frame and exact(frame, expected))
    send(session, b'\x03\x18\x0a', lambda frame: b'org-clock-goto' in frame and b'L12 C1' in frame)
    step(session, b'\x18\x03', b'Org active clock: ')
    step(session, b'keep\r', b'y=save')
    send(session, b'\x07', lambda frame: b'Quit' in frame and exact(frame, expected))
    send(session, b'\x1bxorg-clock-out\r', lambda frame: b'No active' in frame and exact(frame, expected))
    step(session, b'\x03\x18\x1a', b'Org incomplete clock: ')
    step(session, b'1\r', b'Org clock action: ')
    step(session, b'close\r', b'Org clock end: ')
    frame = send(session, b'\r', lambda frame: b'closed at the accepted end' in frame)
    closed = [line for line in document(frame) if line.startswith('CLOCK: ')]
    endpoints = STAMP.findall(closed[1])
    assert len(endpoints) == 2 and endpoints[0] == STAMP.findall(records[1])[0], closed
    minutes = int((date(endpoints[1]) - date(endpoints[0])).total_seconds() // 60)
    duration = f'{minutes // 60}:{minutes % 60:02}'
    assert closed[1] == f'CLOCK: {endpoints[0]}--{endpoints[1]} => {duration:>5}'
    final = expected.copy()
    final[13] = closed[1]
    assert exact(frame, final), document(frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(final)).encode()
    send(session, b'\x18u', lambda frame: exact(frame, expected))
    send(session, b'\x18u', lambda frame: exact(frame, opened))
    send(session, b'\x18u', lambda frame: exact(frame, lines))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(lines)).encode()
