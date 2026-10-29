"""Owned physical project exports, partial results and source immutability."""
import json
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .html_document import parse
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['#+OPTIONS: toc:nil num:nil', '* Heading 界', 'Body', 'EOF']


def project(session, marker):
    named(session, 'org-publish', b'Publish project')
    return step(session, b'docs\r', marker)


def journey(root, colored):
    work = root / 'work'
    config = '\n'.join([
        'org.time.zone="UTC"', '[[org.publish.projects]]', 'name="docs"',
        'roots=[' + json.dumps(str(work)) + ']', 'include=["notes.org"]',
        'output=' + json.dumps(str(work / 'site')),
        'formats=["html","markdown"]', 'assets=["assets/**"]', ''])
    write(root, 'user/rev/config.toml', config.encode())
    source = write(root, 'work/notes.org', encoded(LINES))
    write(root, 'work/assets/data.bin', bytes([0, 255, 13, 10]))
    write(root, 'work/site/notes.html', b'previous')
    write(root, 'work/site/unowned', b'keep')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=work)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        goto(session, 3)
        step(session, b'\x05', b'L3 C5')
        current = LINES.copy()
        current[2] += ' unsaved'
        send(session, PTY['paste_wire'](b' unsaved'), lambda f: exact(f, current))
        quiet(session)
        project(session, b'Replace project output')
        frame = step(session, b'no\r', b'2/3 files complete')
        assert exact(frame, current)
        assert (work / 'site/notes.html').read_bytes() == b'previous'
        assert (work / 'site/notes.md').read_bytes() == '# Heading 界\nBody unsaved\nEOF'.encode()
        assert (work / 'site/assets/data.bin').read_bytes() == bytes([0, 255, 13, 10])
        project(session, b'data.bin? yes/no:')
        step(session, b'yes\r', b'notes.html? yes/no:')
        step(session, b'yes\r', b'notes.md? yes/no:')
        frame = step(session, b'yes\r', b'3/3 files complete')
        assert exact(frame, current)
        document = parse((work / 'site/notes.html').read_bytes())
        assert document.texts('h1') == ['Heading 界']
        assert document.texts('p') == ['Body unsaved\nEOF']
        outputs = {p: p.read_bytes() for p in (work / 'site').rglob('*') if p.is_file()}
        project(session, b'data.bin? yes/no:')
        frame = step(session, b'\x07', b'Project cancelled')
        assert exact(frame, current)
        assert all(p.read_bytes() == data for p, data in outputs.items())
        named(session, 'org-publish-results', b'Publishing result')
        step(session, b'1\r', b'NotPublished')
        assert source.read_bytes() == encoded(LINES)
        assert (work / 'site/unowned').read_bytes() == b'keep'
        send(session, b'\x18u', lambda f: exact(f, LINES))
        after = snapshot(root)
        permitted = {'work/site/notes.md', 'work/site/notes.html', 'work/site/assets/data.bin', 'work/site/assets'}
        for name, value in before.items():
            if name not in permitted:
                assert after[name] == value, name
        assert set(after) - set(before) <= permitted
        quiet(session)
        if not colored:
            assert b"\x1b[38;2;" not in session.output and b"\x1b[48;2;" not in session.output
        finish(session, colored)
    finally:
        close(session)
