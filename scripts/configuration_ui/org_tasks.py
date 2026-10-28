"""Native task transitions and atomic metadata on owned RGB/NO_COLOR PTYs."""
import re
from datetime import datetime, timezone
from .org_estimates import estimates
from .org_timestamps import timestamps, readonly_calendar
from .org_repeats import repeats
from .org_clocks import clocks, lifecycle

from .support import (BINARY, PTY, ROOT, canvas, close, completed, environment,
                      finish, send, snapshot, step, write)

LINES = ['#+SEQ_TODO: TODO(t) WAIT(w@/!) | DONE(d@)', '* Parent [0/1]',
         '** TODO [#A] Child :work:', 'SCHEDULED: <2026-10-29 Thu>',
         ':PROPERTIES:', ':OWNER: keep', ':END:', 'body 界', '* Peer', 'peer EOF']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()
POLICY = b'org.time.zone="UTC"\norg.todo.log_done=true\norg.todo.log_drawer="LOGBOOK"\norg.todo.block_children=true\n[keys.org]\n"M-j"="org-todo"\n'
STAMP = re.compile(r'\[\d{4}-\d{2}-\d{2} [A-Z][a-z]{2} \d{2}:\d{2}\]')


def document(frame):
    return [text[6:].rstrip() for row, text in canvas(frame).items() if row <= 22]


def exact(frame, lines):
    text = document(frame)
    return text[:len(lines)] == lines and not any(text[len(lines):])


def goto(session, line):
    step(session, b'\x1bxgoto-line\r', b'Goto line: ')
    return step(session, str(line).encode() + b'\r', f'L{line} C1'.encode())


def named(session, name, marker):
    return step(session, b'\x1bx' + name.encode() + b'\r', marker)


def changed(stamp):
    lines = LINES.copy()
    lines[1] = '* Parent [1/1]'
    lines[2] = '** DONE [#A] Child :work:'
    lines[3] = f'CLOSED: {stamp} ' + lines[3]
    return lines[:7] + [':LOGBOOK:', f'- State "DONE" from "TODO" {stamp} \\\\',
                       '  completed 界', ':END:'] + lines[7:]


def transition(session, wire, source):
    before = datetime.now(timezone.utc)
    goto(session, 3)
    step(session, wire, b'Org task state: ')
    step(session, b'd\r', b'Org state note: ')
    frame = send(session, PTY['paste_wire']('completed 界'.encode()) + b'\r',
                 lambda frame: b'Org task state and metadata changed' in frame
                 and any('CLOSED:' in line for line in document(frame)))
    stamps = STAMP.findall(document(frame)[3])
    assert len(stamps) == 1, document(frame)
    assert_utc_stamp(stamps[0], before, datetime.now(timezone.utc))
    expected = changed(stamps[0])
    assert exact(frame, expected), document(frame)
    assert b'** RW --' in frame
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == ('\ufeff' + '\r\n'.join(expected)).encode()
    send(session, b'\x18u', lambda frame: exact(frame, LINES) and b'** RW --' in frame)
    named(session, 'undo-redo', b'-- RW --')
    assert exact(PTY['last_frame'](session), expected)
    send(session, b'\x18u', lambda frame: exact(frame, LINES))
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == SOURCE


def assert_utc_stamp(stamp, before, after):
    value = datetime.fromisoformat(f'{stamp[1:11]}T{stamp[16:21]}:00').replace(tzinfo=timezone.utc)
    assert before.replace(second=0, microsecond=0) <= value <= after.replace(second=0, microsecond=0), stamp


def task_environment(root, colored):
    env = environment(root, colored)
    # A wrong local environment must not override the explicit UTC preference.
    env['TZ'] = 'Invalid/OwnedFixture'
    return env


def tasks(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/tasks.org', SOURCE)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)], env=task_environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> tasks.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: exact(frame, LINES)))
        goto(session, 2)
        named(session, 'org-todo-set', b'Org task state: ')
        send(session, b'DONE\r', lambda frame: b'completion blocked' in frame and exact(frame, LINES))
        for wire in [b'\x03\x14', b'\x1bxorg-todo\r', b'\x1bj']:
            transition(session, wire, source)
        goto(session, 3)
        step(session, b'\x03\x14', b'Org task state: ')
        step(session, b'd\r', b'Org state note: ')
        send(session, b'cancelled\x07', lambda frame: b'Quit' in frame and exact(frame, LINES))
        estimates(session, source, LINES, goto, exact)
        timestamps(session, source, LINES, goto, exact)
        repeats(session, source, LINES, goto, exact, document)
        clocks(session, source, LINES, goto, exact, document)
        lifecycle(session, source, LINES, goto, exact, document)
        assert source.read_bytes() == SOURCE
        if not colored:
            assert b"\x1b[38;2;" not in session.output and b"\x1b[48;2;" not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'task session changed fixture files after undo/save'
    finally:
        close(session)


def readonly(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/tasks.org', SOURCE)
    before = snapshot(root)
    binary = ROOT / 'target/debug/examples/org-task-session'
    session = PTY['Session']([str(binary), str(source)], env=task_environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> tasks.org -- RO -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: exact(frame, LINES)))
        goto(session, 3)
        named(session, 'org-inspect-todo', b'Org task: TODO')
        for wire in [b'\x03\x14', b'\x1bxorg-todo\r', b'\x1bxorg-todo-set\r', b'\x03,', b'\x03\x18e', b'\x03.', b'\x03!',
                     b'\x03\x13', b'\x03\x04', b'\x1bxorg-timestamp-up-day\r', b'\x03\x18\x09', b'\x1bxorg-clock-in\r']:
            send(session, wire, lambda frame: b'buffer is read-only' in frame
                 and b'-- RO --' in frame and exact(frame, LINES))
        readonly_calendar(session, LINES, exact)
        assert source.read_bytes() == SOURCE
        if not colored:
            assert b"\x1b[38;2;" not in session.output and b"\x1b[48;2;" not in session.output
        finish(session, colored)
        assert snapshot(root) == before, 'read-only task session changed fixture files'
    finally:
        close(session)
