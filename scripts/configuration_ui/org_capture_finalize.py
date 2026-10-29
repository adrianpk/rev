"""Checked capture save/rejection on owned RGB/NO_COLOR terminals."""
import os
from .org_tasks import exact
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

ORIGIN = ['* Origin 界', 'tail EOF']
ORIGIN_BYTES = ('\ufeff' + '\r\n'.join(ORIGIN)).encode()
TARGET = b'\xef\xbb\xbf* Tasks\r\nbody EOF'
POLICY = b'''org.time.zone='UTC'
org.capture.templates=[{key='t',description='Note',kind='entry',target={type='file+headline',file='../../work/destination.org',headline='Tasks'},template='* %^{Topic|Note} %?'}]
'''


def finalize(root, variant):
    colored, readonly = variant
    write(root, 'user/rev/config.toml', POLICY)
    origin = write(root, 'work/origin.org', ORIGIN_BYTES)
    target = write(root, 'work/destination.org', TARGET)
    if readonly:
        target.chmod(0o444)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(origin)],
                             env=environment(root, colored), cwd=origin.parent)
    try:
        session.ready(b'> origin.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: exact(frame, ORIGIN)))
        step(session, b'\x03c', b'Capture template key: ')
        step(session, b't\r', b'Topic: Note')
        send(session, b'\r', lambda f: b'Capture draft:' in f and exact(f, ['* Note']))
        frame = send(session, PTY['paste_wire']('界e\u0301'.encode()),
                     lambda f: exact(f, ['* Note 界e\u0301']))
        if not colored:
            assert b'\x1b[38;2;' not in frame and b'\x1b[48;2;' not in frame
        quiet(session)
        if readonly:
            frame = step(session, b'\x03\x03', b'Capture not inserted; draft retained:')
            assert exact(frame, ['* Note 界e\u0301'])
            assert target.read_bytes() == TARGET
            frame = step(session, b'\x03\x0b', b'Capture aborted; destination unchanged')
            assert exact(frame, ORIGIN)
            assert target.stat().st_mode & 0o777 == 0o444
        else:
            frame = step(session, b'\x03\x03', b'Capture finalized and saved')
            assert exact(frame, ORIGIN)
            expected = TARGET + '\r\n** Note 界e\u0301\r\n'.encode()
            assert target.read_bytes() == expected
            before['work/destination.org'] = expected
            assert b'> origin.org -- RW -- L1 C1' in frame
        assert origin.read_bytes() == ORIGIN_BYTES
        quiet(session)
        finish(session, colored)
        assert snapshot(root) == before, 'capture changed unexpected fixture files'
    finally:
        close(session)
