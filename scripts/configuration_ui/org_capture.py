"""Combined owned capture/refile/archive journey with real file outcomes."""
import re
from .org_tasks import exact, goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

ORIGIN = ['* Origin 界', 'tail EOF']
NOTE = 'TODO Note 界e\u0301'
INBOX = ['* Inbox', '** ' + NOTE, '*** Child', 'body EOF']
PROJECT = ['* Projects', '** ' + NOTE, '*** Child', 'body EOF']
POLICY = b'''org.time.zone='UTC'
org.refile.targets=['../../work/projects.org']
org.capture.templates=[{key='t',description='Note',kind='entry',target={type='file+headline',file='../../work/inbox.org',headline='Inbox'},template='* TODO %^{Topic|Note} %?'}]
'''


def encoded(lines):
    return ('\ufeff' + '\r\n'.join(lines) + '\r\n').encode()


def visit(session, path, lines):
    step(session, b'\x18\x06', b'Find file: ')
    return send(session, PTY['paste_wire'](str(path).encode()) + b'\r',
                lambda f: exact(f, lines) and path.name.encode() in f)


def capture(session, inbox):
    step(session, b'\x03c', b'Capture template key: ')
    step(session, b't\r', b'Topic: Note')
    send(session, b'\r', lambda f: b'Capture draft:' in f and exact(f, ['* TODO Note']))
    draft = ['* ' + NOTE, '** Child', 'body EOF']
    send(session, PTY['paste_wire']('界e\u0301\n** Child\nbody EOF'.encode()),
         lambda f: exact(f, draft))
    quiet(session)
    frame = step(session, b'\x03\x03', b'Capture finalized and saved')
    assert exact(frame, ORIGIN)
    assert inbox.read_bytes() == encoded(INBOX)


def move(session, inbox, project):
    visit(session, inbox, INBOX)
    goto(session, 2)
    step(session, b'\x03\x17', b'Refile target [number/label]: ')
    send(session, b'1\r', lambda f: b'Refile result:' in f and exact(f, ['* Inbox']))
    assert inbox.read_bytes() == encoded(INBOX)
    assert project.read_bytes() == encoded(['* Projects'])
    quiet(session)
    step(session, b'\x18\x13', b'Saved ')
    assert inbox.read_bytes() == encoded(['* Inbox'])
    visit(session, project, PROJECT)
    step(session, b'\x18\x13', b'Saved ')
    assert project.read_bytes() == encoded(PROJECT)


def archive_bytes(archive, project):
    value = archive.read_bytes()
    timestamp = re.search(rb':ARCHIVE_TIME: (\[\d{4}-\d{2}-\d{2} [A-Z][a-z]{2} \d{2}:\d{2}\])\r\n', value)
    assert timestamp, 'archive has no captured operation timestamp'
    lines = ['* Previous', '* ' + NOTE, ':PROPERTIES:',
             ':ARCHIVE_CATEGORY: projects', f':ARCHIVE_FILE: {project}',
             ':ARCHIVE_ITAGS: ', ':ARCHIVE_OLPATH: Projects',
             ':ARCHIVE_TIME: ' + timestamp[1].decode(), ':ARCHIVE_TODO: TODO',
             ':END:', '** Child', 'body EOF']
    assert value == encoded(lines), 'archive source/metadata/format differs'
    return value


def archive_move(session, project, archive, colored):
    goto(session, 2)
    frame = step(session, b'\x03\x18\x01', b'Archive rejected; nothing moved:')
    assert exact(frame, PROJECT)
    assert project.read_bytes() == encoded(PROJECT)
    assert archive.read_bytes() == encoded(['* Previous'])
    # Change only the owned fixture prerequisite; retry uses fresh observations.
    archive.chmod(0o644)
    send(session, b'\x03\x18\x01', lambda f: b'Archive result:' in f
         and exact(f, ['* Projects']))
    assert project.read_bytes() == encoded(PROJECT)
    assert archive.read_bytes() == encoded(['* Previous'])
    step(session, b'\x18\x06', b'Find file: ')
    send(session, PTY['paste_wire'](str(archive).encode()) + b'\r',
         lambda f: b':ARCHIVE_FILE:' in f and b'projects.org_archive' in f)
    send(session, b'\x1bxundo\r', lambda f: exact(f, ['* Previous']))
    assert archive.read_bytes() == encoded(['* Previous'])
    send(session, b'\x1bxundo-redo\r', lambda f: b':ARCHIVE_OLPATH: Projects' in f)
    quiet(session)
    frame = step(session, b'\x18\x13', b'Saved ')
    value = archive_bytes(archive, project)
    if not colored:
        assert b'\x1b[38;2;' not in frame and b'\x1b[48;2;' not in frame
    visit(session, project, ['* Projects'])
    step(session, b'\x18\x13', b'Saved ')
    frame = named(session, 'org-archive-results', b'Archive result: source: saved')
    assert b'destination: saved' in frame
    return value


def journey(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    origin = write(root, 'work/origin.org', encoded(ORIGIN))
    inbox = write(root, 'work/inbox.org', encoded(['* Inbox']))
    project = write(root, 'work/projects.org', encoded(['* Projects']))
    archive = write(root, 'work/projects.org_archive', encoded(['* Previous']))
    archive.chmod(0o444)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(origin)],
                             env=environment(root, colored), cwd=origin.parent)
    try:
        session.ready(b'> origin.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, ORIGIN)))
        capture(session, inbox)
        move(session, inbox, project)
        value = archive_move(session, project, archive, colored)
        assert origin.read_bytes() == encoded(ORIGIN)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/projects.org_archive'] = value
        assert snapshot(root) == before, 'journey changed unexpected fixture files'
    finally:
        close(session)
