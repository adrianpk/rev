"""Owned native refile, cancellation, independent save and undo/redo journey."""
from .org_tasks import exact, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

SOURCE = ['* Note 界', '** Child e\u0301', '* Stay EOF']
DESTINATION = ['* Projects', 'body EOF']
MOVED = DESTINATION + ['** Note 界', '*** Child e\u0301']
SOURCE_BYTES = ('\ufeff' + '\r\n'.join(SOURCE)).encode()
TARGET_BYTES = ('\ufeff' + '\r\n'.join(DESTINATION)).encode()
MOVED_BYTES = ('\ufeff' + '\r\n'.join(MOVED) + '\r\n').encode()
POLICY = b"org.refile.targets=['../../work/destination.org']\n"


def refile(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/source.org', SOURCE_BYTES)
    target = write(root, 'work/destination.org', TARGET_BYTES)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> source.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, SOURCE)))
        step(session, b'\x03\x17', b'Refile target [number/label]: ')
        frame = step(session, b'\x07', b'Refile cancelled;')
        assert exact(frame, SOURCE)
        assert snapshot(root) == before
        step(session, b'\x03\x17', b'Refile target [number/label]: ')
        frame = send(session, b'1\r', lambda f: b'Refile result:' in f
                     and exact(f, ['* Stay EOF']))
        assert snapshot(root) == before, 'in-memory refile wrote files'
        quiet(session)
        step(session, b'\x18\x13', b'Saved ')
        assert source.read_bytes() == b'\xef\xbb\xbf* Stay EOF'
        step(session, b'\x18\x06', b'Find file: ')
        frame = send(session, PTY['paste_wire'](str(target).encode()) + b'\r',
                     lambda f: exact(f, MOVED) and b'destination.org' in f)
        if not colored:
            assert b'\x1b[38;2;' not in frame and b'\x1b[48;2;' not in frame
        assert target.read_bytes() == TARGET_BYTES
        step(session, b'\x18\x13', b'Saved ')
        assert target.read_bytes() == MOVED_BYTES
        frame = named(session, 'org-refile-results', b'source: saved')
        assert b'destination: saved' in frame
        send(session, b'\x1bxundo\r', lambda f: exact(f, DESTINATION))
        assert target.read_bytes() == MOVED_BYTES
        send(session, b'\x1bxundo-redo\r', lambda f: exact(f, MOVED))
        quiet(session)
        finish(session, colored)
        before['work/source.org'] = b'\xef\xbb\xbf* Stay EOF'
        before['work/destination.org'] = MOVED_BYTES
        assert snapshot(root) == before, 'refile changed unexpected fixture files'
    finally:
        close(session)
