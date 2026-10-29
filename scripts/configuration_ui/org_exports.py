"""Owned physical export prompts, exact output bytes and source preservation."""
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['* Heading 界', 'Body', 'EOF']
MARKDOWN = '# Heading 界\nBody unsaved\nEOF'.encode()


def choose(session, command, path, marker):
    named(session, command, b'Export scope')
    step(session, b'buffer\r', b'Export OPTIONS')
    step(session, b'toc:nil num:nil\r', b'Export to local file: ')
    return step(session, path.encode() + b'\r', marker)


def journey(root, colored):
    write(root, 'user/rev/config.toml', b'org.time.zone="UTC"\n')
    source = write(root, 'work/notes.org', encoded(LINES))
    write(root, 'work/keep.txt', b'keep')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        goto(session, 2)
        step(session, b'\x05', b'L2 C5')
        current = LINES.copy()
        current[1] += ' unsaved'
        send(session, PTY['paste_wire'](b' unsaved'), lambda f: exact(f, current))
        choose(session, 'org-export-to-markdown', 'output.md', b'Exported Org document')
        assert (source.parent / 'output.md').read_bytes() == MARKDOWN
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-text', 'ascii.txt', b'U+754C')
        assert not (source.parent / 'ascii.txt').exists()
        choose(session, 'org-export-to-unicode', 'keep.txt', b'yes/no:')
        frame = step(session, b'no\r', b'Org export declined')
        assert exact(frame, current)
        assert (source.parent / 'keep.txt').read_bytes() == b'keep'
        named(session, 'org-export-to-org', b'Export scope')
        frame = step(session, b'\x07', b'Org export cancelled')
        assert exact(frame, current)
        send(session, b'\x18u', lambda f: exact(f, LINES))
        assert source.read_bytes() == encoded(LINES)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/output.md'] = MARKDOWN
        assert snapshot(root) == before
    finally:
        close(session)
