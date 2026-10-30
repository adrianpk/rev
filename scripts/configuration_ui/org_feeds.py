"""Explicit owned HTTP feed transport, canonical bytes, history and failures."""
import shutil
import socket
import subprocess
import time

from .org_document import encoded, exact
from .org_tasks import named
from .org_time import quiet
from .support import (BINARY, PTY, close, completed, environment, finish,
                      read, send, snapshot, step, write)

LINES = ['* Inbox', 'keep']
UPDATED = LINES + ['  :FEEDSTATUS:', '(("stable" t nil))', '  :END:',
                   '** News', '[[https://example.test/item]]', ': Literal & text', '']
XML = b"<rss version='2.0'><channel><item><guid>stable</guid><title>News</title><link>https://example.test/item</link><description>Literal &amp; text</description></item></channel></rss>"


def ready(server, address):
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        assert server.poll() is None, 'owned goserve exited during startup'
        try:
            with socket.create_connection(address, timeout=0.1):
                return
        except OSError:
            time.sleep(0.01)
    raise AssertionError('owned goserve did not listen within three seconds')


def requests(log):
    return read(log).count(b'GET /news.xml ')


def run(root, colored, server, port):
    curl = shutil.which('curl')
    assert curl is not None, 'curl is required for the selected feed transport'
    policy = f"""org.time.zone='UTC'
org.tools.fetch=['{curl}','--fail','--silent','--show-error','--max-time','2','--noproxy','*','{{target}}']
[[org.feeds.sources]]
name='News'
source='http://127.0.0.1:{port}/news.xml'
file='../../work/notes.org'
headline='Inbox'
"""
    write(root, 'user/rev/config.toml', policy.encode())
    source = write(root, 'work/notes.org', encoded(LINES))
    log = root / 'server.log'
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        assert requests(log) == 0, 'visit fetched a feed'
        step(session, b'\x03\x18g', b'Fetch using ')
        assert requests(log) == 0 and source.read_bytes() == encoded(LINES)
        step(session, b'no\r', b'cancelled before launch')
        assert requests(log) == 0, 'decline fetched a feed'
        step(session, b'\x03\x18g', b'Fetch using ')
        frame = step(session, b'yes\r', b'1 added, 0 failed')
        assert exact(frame, UPDATED)
        assert requests(log) == 1 and source.read_bytes() == encoded(UPDATED)
        send(session, b'\x18u', lambda f: exact(f, LINES) and b'** RW --' in f)
        named(session, 'undo-redo', b'-- RW --')
        assert exact(PTY['last_frame'](session), UPDATED)
        quiet(session)
        step(session, b'\x03\x18g', b'Fetch using ')
        frame = step(session, b'yes\r', b'0 added, 0 failed')
        assert exact(frame, UPDATED)
        assert requests(log) == 2 and source.read_bytes() == encoded(UPDATED)
        named(session, 'org-feed-results', b'Feed result (1..1')
        step(session, b'1\r', b'0 added, 1 skipped')
        step(session, b'\x03\x18G', b'Feed (News): ')
        frame = step(session, b'News\r', b'0 added, 0 failed')
        assert b'L1 C1' in frame and exact(frame, UPDATED)
        assert requests(log) == 2, 'inbox navigation fetched a feed'
        (root / 'http/news.xml').unlink()
        step(session, b'\x03\x18g', b'Fetch using ')
        frame = step(session, b'yes\r', b'0 added, 1 failed')
        assert exact(frame, UPDATED)
        assert requests(log) == 3 and source.read_bytes() == encoded(UPDATED)
        named(session, 'org-feed-results', b'Feed result (1..1')
        step(session, b'1\r', b'NotPublished')
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/notes.org'] = encoded(UPDATED)
        del before['http/news.xml']
        before['server.log'] = read(log)
        assert snapshot(root) == before, 'feed journey changed unowned fixture entries'
        assert server.poll() is None, 'owned HTTP fixture exited early'
    finally:
        close(session)


def journey(root, colored):
    goserve = shutil.which('goserve')
    assert goserve is not None, 'goserve is required for the owned HTTP fixture'
    feed = write(root, 'http/news.xml', XML)
    with socket.socket() as listener:
        listener.bind(('127.0.0.1', 0))
        port = listener.getsockname()[1]
    with (root / 'server.log').open('wb') as log:
        server = subprocess.Popen([goserve, '-root', str(feed.parent), '-addr',
                                   f'127.0.0.1:{port}', '-no-cache'],
                                  stdout=log, stderr=log, env=environment(root, colored))
        try:
            ready(server, ('127.0.0.1', port))
            run(root, colored, server, port)
        finally:
            server.terminate()
            server.wait(timeout=6)
            assert server.returncode is not None, 'owned goserve survived cleanup'
