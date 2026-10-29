"""Owned RGB/NO_COLOR native links, readonly visits and explicit viewer flow."""
from .org_tasks import exact, goto
from .org_time import quiet
from .support import (BINARY, ROOT, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['[[file:destination.org::*Target][Go 界]]',
         '[[https://example.invalid/check][External]]', '* Home', 'body EOF']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()
DESTINATION = ['* Target', 'body 界 EOF']
TARGET = ('\ufeff' + '\r\n'.join(DESTINATION)).encode()
POLICY = b"org.tools.viewer=['/bin/sh','viewer.sh','{target}']\n"


def journey(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/source.org', SOURCE)
    target = write(root, 'work/destination.org', TARGET)
    target.chmod(0o444)
    write(root, 'work/viewer.sh', b'printf "%s" "$1" > called; printf opened\n')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> source.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        goto(session, 4)
        step(session, b'\x03\x0c', b'Org link target: ')
        step(session, b'*Home\r', b'Link description: ')
        edited = LINES[:3] + ['[[*Home][Jump 界]]body EOF']
        frame = send(session, PTY['paste_wire']('Jump 界'.encode()) + b'\r',
                     lambda f: b'Org link inserted' in f and exact(f, edited))
        assert source.read_bytes() == SOURCE
        send(session, b'\x18u', lambda f: exact(f, LINES))
        goto(session, 1)
        frame = send(session, b'\x03\x0f', lambda f: b'Org link destination' in f
                     and exact(f, DESTINATION))
        step(session, b'\x03l', b'Org link stored')
        send(session, b'\x03&', lambda f: exact(f, LINES) and b'Org link return point' in f)
        goto(session, 2)
        step(session, b'\x03\x0f', b'Run configured viewer? yes/no: ')
        assert not (source.parent / 'called').exists()
        step(session, b'\x07', b'Org link cancelled;')
        assert not (source.parent / 'called').exists()
        step(session, b'\x03\x0f', b'Run configured viewer? yes/no: ')
        frame = step(session, b'yes\r', b'Viewer completed:')
        assert exact(frame, LINES)
        assert (source.parent / 'called').read_bytes() == b'https://example.invalid/check'
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/called'] = b'https://example.invalid/check'
        assert snapshot(root) == before, 'links changed unexpected fixture files'
        assert source.read_bytes() == SOURCE and target.read_bytes() == TARGET
    finally:
        close(session)


def readonly(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/source.org', SOURCE)
    write(root, 'work/destination.org', TARGET)
    before = snapshot(root)
    binary = ROOT / 'target/debug/examples/org-task-session'
    session = PTY['Session']([str(binary), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> source.org -- RO -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        frame = step(session, b'\x03\x0c', b'read-only')
        assert exact(frame, LINES)
        step(session, b'\x03l', b'Org link stored')
        send(session, b'\x03\x0f', lambda f: b'Org link destination' in f and exact(f, DESTINATION))
        frame = send(session, b'\x03&', lambda f: exact(f, LINES) and b'-- RO --' in f)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'readonly link workflow changed fixture bytes'
    finally:
        close(session)
