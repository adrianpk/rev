"""Native capture draft editing/abort on owned bounded RGB/NO_COLOR PTYs."""
from .org_tasks import exact, named
from .org_time import quiet
from .support import (BINARY, PTY, canvas, close, completed, environment,
                      finish, send, snapshot, step, write)

LINES = ['* Origin 界', 'e\u0301 body', 'tail EOF']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()
POLICY = b'''org.capture.templates=[{key='t',description='Note',kind='entry',target={type='file',file='../../work/destination.org'},template='* %^{Topic|default|one|two} %?'}]
'''


def drafts(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    origin = write(root, 'work/origin.org', SOURCE)
    target = write(root, 'work/destination.org', b'\xef\xbb\xbf* Existing\r\nunchanged EOF')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(origin)],
                             env=environment(root, colored), cwd=origin.parent)
    try:
        session.ready(b'> origin.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: exact(frame, LINES)))
        step(session, b'\x03c', b'Capture template key: ')
        step(session, b't\r', b'Topic: default')
        quiet(session)
        frame = send(session, b'\r', lambda f: b'Capture draft:' in f
                     and b'Capture' in f and exact(f, ['* default']))
        assert b'*Org Capture*' in frame
        if not colored:
            assert b'\x1b[38;2;' not in frame and b'\x1b[48;2;' not in frame
        frame = send(session, PTY['paste_wire']('界e\u0301'.encode()),
                     lambda f: exact(f, ['* default 界e\u0301']))
        step(session, b'\x18\x13', b'draft retained')
        step(session, b'\x18\x17', b'draft retained')
        send(session, b'\x18u', lambda f: exact(f, ['* default']))
        frame = send(session, b'\x03\x0b', lambda f: exact(f, LINES)
                     and b'Capture aborted; destination unchanged' in f)
        assert b'> origin.org -- RW -- L1 C1' in frame
        assert origin.read_bytes() == SOURCE
        assert target.read_bytes() == b'\xef\xbb\xbf* Existing\r\nunchanged EOF'
        # Preparation and string-prompt cancellation also preserve the source.
        step(session, b'\x03c', b'Capture template key: ')
        step(session, b't\r', b'Topic: default')
        frame = step(session, b'\x07', b'Capture aborted; destination unchanged')
        assert exact(frame, LINES)
        quiet(session)
        canvas(frame)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'capture draft/abort changed fixture files'
    finally:
        close(session)
