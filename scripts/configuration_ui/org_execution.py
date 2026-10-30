"""Owned selected-code consent and execution, with exact canonical source."""
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .org_exports import choose
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['#+NAME: say', '#+BEGIN_SRC sh', 'printf executed > marker', "printf 'hello'",
         '#+END_SRC', '#+RESULTS: say', ': previous', '',
         '#+CALL: say()', '#+RESULTS: say', ': old call']
POLICY = b"org.time.zone='UTC'\n[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'\n"


def journey(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/notes.org', encoded(LINES))
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        assert not (source.parent / 'marker').exists(), 'visit executed source'
        choose(session, 'org-export-to-markdown', 'output.md', b'Exported Org document')
        assert not (source.parent / 'marker').exists(), 'export executed source'
        exported = (source.parent / 'output.md').read_bytes()
        assert b'printf executed > marker' in exported
        goto(session, 3)
        step(session, b'\x03\x16e', b'Execute shell')
        assert not (source.parent / 'marker').exists(), 'ran before confirmation'
        step(session, b'no\r', b'cancelled before launch')
        assert not (source.parent / 'marker').exists(), 'decline executed source'
        step(session, b'\x03\x03', b'Execute shell')
        frame = step(session, b'yes\r', b'Source Succeeded')
        updated = LINES.copy()
        updated[6] = ': hello'
        assert exact(frame, updated)
        assert (source.parent / 'marker').read_bytes() == b'executed'
        named(session, 'org-babel-execution-results', b'Execution detail')
        frame = step(session, b'stdout\r', b'hello')
        assert exact(frame, updated)
        send(session, b'\x18u', lambda frame: exact(frame, LINES))
        goto(session, 9)
        step(session, b'\x03\x03', b'Execute shell')
        frame = step(session, b'yes\r', b'Source Succeeded')
        called = LINES.copy()
        called[10] = ': hello'
        assert exact(frame, called)
        send(session, b'\x18u', lambda frame: exact(frame, LINES))
        assert source.read_bytes() == encoded(LINES)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/marker'] = b'executed'
        before['work/output.md'] = exported
        assert snapshot(root) == before
    finally:
        close(session)
