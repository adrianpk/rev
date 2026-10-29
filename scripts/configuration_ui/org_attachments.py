"""Owned attachment decisions, file outcomes and canonical source history."""
from .org_tasks import exact, goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['* Task', ':PROPERTIES:', ':DIR: assets', ':END:', 'body EOF']
ATTACHED = ['* Task  :ATTACH:'] + LINES[1:]
NOTE = ['* Attached note 界', 'resource EOF']


def encoded(lines):
    return ('\ufeff' + '\r\n'.join(lines)).encode()


def copy(session, answer):
    step(session, b'\x03\x01', b'Attachment [a/c/m/l/o/O/f/s]: ')
    step(session, b'c\r', b'Attach file: ')
    step(session, b'input/report.txt\r', b'Confirm attachment yes/no: ')
    return step(session, answer, b'Attachment declined;' if answer == b'no\r' else b'Attachment result:')


def journey(root, colored):
    write(root, 'user/rev/config.toml', b'')
    source = write(root, 'work/source.org', encoded(LINES))
    original = write(root, 'work/input/report.txt', b'new report\x00\xff')
    target = write(root, 'work/assets/report.txt', b'old report')
    write(root, 'work/assets/note one.org', encoded(NOTE))
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> source.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        goto(session, 5)
        quiet(session)
        assert snapshot(root) == before
        frame = copy(session, b'no\r')
        assert exact(frame, LINES) and snapshot(root) == before
        target.chmod(0o444)
        frame = copy(session, b'yes\r')
        assert b'file Rejected' in frame and exact(frame, LINES)
        assert target.read_bytes() == b'old report' and source.read_bytes() == encoded(LINES)
        target.chmod(0o644)
        frame = copy(session, b'yes\r')
        assert b'heading Changed' in frame and exact(frame, ATTACHED)
        assert target.read_bytes() == original.read_bytes() == b'new report\x00\xff'
        assert source.read_bytes() == encoded(LINES)
        quiet(session)
        step(session, b'\x18\x13', b'Saved ')
        assert source.read_bytes() == encoded(ATTACHED)
        send(session, b'\x18u', lambda f: exact(f, LINES))
        assert target.read_bytes() == b'new report\x00\xff'
        named(session, 'undo-redo', b'-- RW --')
        assert exact(PTY['last_frame'](session), ATTACHED)
        step(session, b'\x03\x01', b'Attachment [a/c/m/l/o/O/f/s]: ')
        frame = step(session, b'f\r', b'2 attachments')
        assert exact(frame, ATTACHED)
        step(session, b'\x03\x01', b'Attachment [a/c/m/l/o/O/f/s]: ')
        step(session, b'O\r', b'Open attachment [number/name]: ')
        send(session, b'1\r', lambda f: b'Org link destination' in f and exact(f, NOTE))
        send(session, b'\x03&', lambda f: b'Org link return point' in f and exact(f, ATTACHED))
        goto(session, 5)
        step(session, b'\x03\x0c', b'Org link target: ')
        step(session, b'\r', b'Link description: ')
        inserted = ATTACHED[:4] + ['[[attachment:report.txt][report.txt]]body EOF']
        send(session, b'\r', lambda f: b'Org link inserted' in f and exact(f, inserted))
        send(session, b'\x18u', lambda f: exact(f, ATTACHED))
        step(session, b'\x03\x01', b'Attachment [a/c/m/l/o/O/f/s]: ')
        frame = step(session, b'\x07', b'Attachment cancelled;')
        assert exact(frame, ATTACHED)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        step(session, b'\x18\x13', b'Saved ')
        assert source.read_bytes() == encoded(ATTACHED)
        finish(session, colored)
        before['work/source.org'] = encoded(ATTACHED)
        before['work/assets/report.txt'] = b'new report\x00\xff'
        assert snapshot(root) == before, 'attachments changed unexpected fixture files'
    finally:
        close(session)
