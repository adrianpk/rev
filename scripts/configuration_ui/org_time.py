"""Owned native timer notes, deadline, restart and terminal-error journeys."""
import os
import re
import time
from .org_tasks import goto, exact, document, named
from .support import (BINARY, ROOT, PTY, close, completed, environment, finish,
                      restoration, send, snapshot, step, write)

LINES = ['* Timed notes', 'note 界', 'tail EOF']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()
POLICY = b'org.timer.default="0:00:03"\n'
START = b'\x03\x180'
SET = b'\x03\x18;'
PAUSE = b'\x03\x18,'
STOP = b'\x03\x18_'


def quiesce(session):
    # Reap ordinary completed source coloring before asserting timer inactivity.
    deadline = time.monotonic() + 0.5
    stable = time.monotonic() + 0.05
    length = len(session.output)
    while time.monotonic() < stable:
        assert time.monotonic() < deadline, 'native source observation did not quiesce'
        session.drain()
        if len(session.output) != length:
            length = len(session.output)
            stable = time.monotonic() + 0.05


def quiet(session, seconds=0.15):
    quiesce(session)
    length = len(session.output)
    until = time.monotonic() + seconds
    session.wait(lambda: time.monotonic() >= until)
    assert len(session.output) == length, 'inactive timer produced output'
    assert session.process.poll() is None


def opened(root, colored, readonly=False):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/time.org', SOURCE)
    binary = ROOT / 'target/debug/examples/org-task-session' if readonly else BINARY
    session = PTY['Session']([str(binary), str(source)], env=environment(root, colored), cwd=source.parent)
    access = b'RO' if readonly else b'RW'
    session.ready(b'> time.org -- ' + access + b' -- L1 C1')
    session.wait(lambda: completed(session, lambda frame: exact(frame, LINES)))
    return session, source


def set_timer(session, value):
    step(session, SET, b'Org countdown (minutes or H:MM:SS): ')
    return step(session, value + b'\r', b'Org countdown started')


def editing(root, colored):
    session, source = opened(root, colored)
    before = snapshot(root)
    try:
        named(session, 'org-timer-show-remaining-time', b'No Org timer')
        step(session, START, b'Org relative timer started')
        frame = step(session, PAUSE, b'timer paused: ')
        stamp = re.search(rb'timer paused: (\d+:\d{2}:\d{2})', frame).group(1).decode()
        quiet(session)
        goto(session, 2)
        expected = LINES.copy()
        expected[1] = stamp + ' ' + expected[1]
        send(session, b'\x03\x18.', lambda frame: exact(frame, expected) and b'Org timer inserted' in frame)
        send(session, b'\x18u', lambda frame: exact(frame, LINES))
        expected[1] = '- ' + stamp + ' :: ' + LINES[1]
        send(session, b'\x03\x18-', lambda frame: exact(frame, expected) and b'Org timer inserted' in frame)
        step(session, b'\x05', b'L2 C')
        continued = expected[:2] + ['- ' + stamp + ' :: '] + expected[2:]
        send(session, b'\x1b\r', lambda frame: exact(frame, [line.rstrip() for line in continued]) and b'org-meta-return' in frame)
        step(session, b'\x18\x13', b'Saved')
        assert source.read_bytes() == ('\ufeff' + '\r\n'.join(continued)).encode()
        send(session, b'\x18u', lambda frame: exact(frame, expected))
        send(session, b'\x18u', lambda frame: exact(frame, LINES))
        step(session, b'\x18\x13', b'Saved')
        step(session, STOP, b'Org timer stopped')
        set_timer(session, b'0:00:01')
        step(session, PAUSE, b'timer paused: ')
        quiet(session)
        step(session, PAUSE, b'timer running: ')
        session.wait(lambda: completed(session, lambda frame: b'Org countdown finished' in frame and b'Timer done' in frame))
        assert bytes(session.output).count(b'Org countdown finished') == 1
        quiet(session)
        set_timer(session, b'0:00:01')
        step(session, STOP, b'Org timer stopped')
        quiet(session, 1.1)
        assert source.read_bytes() == SOURCE
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'timer journey changed files after undo/save'
    finally:
        close(session)


def readonly(root, colored):
    session, source = opened(root, colored, readonly=True)
    before = snapshot(root)
    try:
        for wire in [b'\x03\x18.', b'\x03\x18-']:
            send(session, wire, lambda frame: b'buffer is read-only' in frame and exact(frame, LINES))
        named(session, 'org-timer-show-remaining-time', b'No Org timer')
        step(session, START, b'Org relative timer started')
        step(session, PAUSE, b'timer paused: ')
        quiet(session)
        step(session, STOP, b'Org timer stopped')
        set_timer(session, b'0:00:01')
        session.wait(lambda: completed(session, lambda frame: b'Org countdown finished' in frame and exact(frame, LINES)))
        quiet(session)
        assert source.read_bytes() == SOURCE
        finish(session, colored)
        assert snapshot(root) == before, 'read-only timer journey changed files'
    finally:
        close(session)


def restart(root, colored):
    session, source = opened(root, colored)
    before = snapshot(root)
    try:
        set_timer(session, b'0:00:30')
        finish(session, colored)
    finally:
        close(session)
    session = PTY['Session']([str(BINARY), str(source)], env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> time.org -- RW -- L1 C1')
        named(session, 'org-timer-show-remaining-time', b'No Org timer')
        quiet(session)
        finish(session, colored)
        assert snapshot(root) == before, 'restart persisted or restored timer state'
    finally:
        close(session)


def failure(root):
    before = snapshot(root)
    binary = ROOT / 'target/debug/examples/terminal-probe'
    session = PTY['Session']([str(binary), 'org-timer-io'], env=environment(root, True), cwd=root)
    try:
        session.ready(b'probe-ready')
        os.write(session.master, b'!')
        session.finish(1)
        assert b'timer-owner-clean' in session.output
        restoration(session)
        assert snapshot(root) == before, 'timer output failure created files'
    finally:
        close(session)
