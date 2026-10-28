"""Owned native restart recovery with opt-in XDG clock state."""
import os
import tomllib
from .org_tasks import document, exact, task_environment
from .org_clocks import STAMP, date
from .support import BINARY, PTY, close, completed, restoration, send, step, write

LINES = ['* Task 界', 'body EOF']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()
POLICY = b'org.time.zone="UTC"\norg.clock.persist=true\n'


def persistence(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/clock.org', SOURCE)
    env = task_environment(root, colored)
    state = root / 'state/rev/org-clock.toml'
    first = PTY['Session']([str(BINARY), str(source)], env=env, cwd=source.parent)
    try:
        first.ready(b'> clock.org -- RW -- L1 C1')
        first.wait(lambda: completed(first, lambda frame: exact(frame, LINES)))
        frame = send(first, b'\x03\x18\x09', lambda frame: b'Org clock started' in frame
                     and any(line.startswith('CLOCK: ') for line in document(frame)))
        clock = next(line for line in document(frame) if line.startswith('CLOCK: '))
        opened = LINES[:1] + [':LOGBOOK:', clock, ':END:'] + LINES[1:]
        assert exact(frame, opened), document(frame)
        step(first, b'\x18\x13', b'Saved')
        assert source.read_bytes() == ('\ufeff' + '\r\n'.join(opened)).encode()
        saved = tomllib.loads(state.read_text())['clock']
        assert saved['source'] == clock and saved['path'] == str(source), saved
        step(first, b'\x18\x03', b'Org active clock: ')
        os.write(first.master, b'keep\r')
        first.finish(0)
        restoration(first, colored)
    finally:
        close(first)
    before = source.read_bytes()
    second = PTY['Session']([str(BINARY)], env=env, cwd=source.parent)
    try:
        second.ready(b'Org saved clock: ')
        second.wait(lambda: completed(second, lambda frame: exact(frame, [])))
        assert source.read_bytes() == before
        send(second, b'recover\r', lambda frame: b'Org incomplete clock: ' in frame and exact(frame, opened))
        step(second, b'1\r', b'Org clock action: ')
        step(second, b'close\r', b'Org clock end: ')
        frame = send(second, b'\r', lambda frame: b'closed at the accepted end' in frame
                     and any(' => ' in line for line in document(frame)))
        record = next(line for line in document(frame) if line.startswith('CLOCK: '))
        endpoints = STAMP.findall(record)
        assert len(endpoints) == 2 and endpoints[0] == STAMP.findall(clock)[0], record
        minutes = int((date(endpoints[1]) - date(endpoints[0])).total_seconds() // 60)
        duration = f'{minutes // 60}:{minutes % 60:02}'
        assert record == f'{clock}--{endpoints[1]} => {duration:>5}', record
        closed = LINES[:1] + [':LOGBOOK:', record, ':END:'] + LINES[1:]
        assert exact(frame, closed), document(frame)
        assert tomllib.loads(state.read_text()) == {'version': 1}
        step(second, b'\x18\x13', b'Saved')
        assert source.read_bytes() == ('\ufeff' + '\r\n'.join(closed)).encode()
        os.write(second.master, b'\x18\x03')
        second.finish(0)
        restoration(second, colored)
    finally:
        close(second)
    assert sorted(path.name for path in state.parent.iterdir()) == ['org-clock.toml']
