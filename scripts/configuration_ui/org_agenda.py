"""Checked multi-source agenda actions on owned RGB/NO_COLOR terminals."""
import os
from .org_tasks import exact, goto
from .org_time import quiesce
from .support import (BINARY, ROOT, PTY, canvas, close, completed, environment,
                      finish, snapshot, visit, write)

ONE = ['#+TODO: TODO(t) | DONE(d@)', '* TODO One 界 :keep:', 'body']
TWO = ['* TODO Two e\u0301', 'tail EOF']
FIRST = ('\ufeff' + '\r\n'.join(ONE)).encode()
SECOND = ('\ufeff' + '\r\n'.join(TWO)).encode()
POLICY = b'org.time.zone="UTC"\norg.agenda.sources=["../../work/one.org","../../work/two.org"]\n'


def last_frame(session):
    # Projections deliberately hide the cursor. A complete hidden frame ends
    # after row 24's reset; all 24 rows must still have exactly 80 cells. This
    # case does not alter the visible-cursor contract of historical journeys.
    start = session.output.rfind(PTY['HIDE'])
    if start < 0:
        return None
    frame = bytes(session.output[start:])
    if not frame.endswith((PTY['SHOW'], b'\x1b[0m')):
        return None
    try:
        canvas(frame)
    except AssertionError:
        return None
    return frame


def send(session, keys, predicate):
    session.output.clear()
    os.write(session.master, keys)
    session.wait(lambda: (frame := last_frame(session)) is not None and predicate(frame))
    assert session.process.poll() is None and PTY['LEAVE'] not in session.output
    return last_frame(session)


def step(session, keys, marker):
    return send(session, keys, lambda f: PTY['status_contains'](f, marker))


def named(session, name, marker):
    return step(session, b'\x1bx' + name.encode() + b'\r', marker)


def opened(root, colored, locked=False):
    write(root, 'user/rev/config.toml', POLICY)
    one = write(root, 'work/one.org', FIRST)
    two = write(root, 'work/two.org', SECOND)
    binary = ROOT / 'target/debug/examples/org-task-session' if locked else BINARY
    session = PTY['Session']([str(binary), str(one)], env=environment(root, colored), cwd=one.parent)
    access = b'RO' if locked else b'RW'
    try:
        session.ready(b'> one.org -- ' + access + b' -- L1 C1')
        session.wait(lambda: completed(session, lambda frame: exact(frame, ONE)))
    except BaseException:
        close(session)
        raise
    return session, one, two


def query(session):
    quiesce(session)
    named(session, 'org-todo-list', b'TODO keywords [empty: unfinished; | separates]: ')
    frame = send(session, b'\r', lambda f: 'Org TODO agenda [complete]' in canvas(f)[1]
                and any('One 界' in row for row in canvas(f).values())
                and any('Two e\u0301' in row for row in canvas(f).values()))
    assert not frame.endswith(PTY['SHOW']), 'agenda must hide the source cursor'
    return frame


def bulk(session, operation, value):
    step(session, b'*', b'2 visible source targets marked')
    step(session, b'B', b'Bulk action [')
    step(session, operation + b'\r', b'Bulk value [ordinary source grammar]: ')
    return send(session, value + b'\r', lambda f: canvas(f)[1].startswith('Agenda actions: 2 targets'))


def outcomes(frame, first, second):
    rows = canvas(frame)
    assert rows[2].startswith(first + ' one.org: One 界:'), rows[2]
    assert rows[3].startswith(second + ' two.org: Two e\u0301:'), rows[3]


def undo(session):
    # Ordinary history reports success with an empty echo. The source bytes are
    # asserted after returning/visiting; no invented success message is required.
    return send(session, b'\x1bxorg-agenda-undo\r', lambda f:
                canvas(f)[1].startswith('Org TODO agenda') and not canvas(f)[24].strip()
                and not f.endswith(PTY['SHOW']))


def no_color(session, colored):
    if not colored:
        assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output


def editing(root, colored):
    session, one, two = opened(root, colored)
    before = snapshot(root)
    try:
        # Canonical unsaved text is the agenda authority, even though disk still
        # contains the older title. Neither remote mutation may save implicitly.
        goto(session, 2)
        step(session, b'\x05', b'L2 C')
        send(session, PTY['paste_wire'](b' unsaved'), lambda f: exact(f, [ONE[0], ONE[1] + ' unsaved', ONE[2]]))
        frame = query(session)
        assert 'One 界 :keep: unsaved' in '\n'.join(canvas(frame).values())
        frame = bulk(session, b'priority', b'A')
        # The result title includes the retained unsaved source spelling.
        rows = canvas(frame)
        assert rows[2].startswith('Changed one.org: One 界 :keep: unsaved:'), rows[2]
        assert rows[3].startswith('Changed two.org: Two e\u0301:'), rows[3]
        assert one.read_bytes() == FIRST and two.read_bytes() == SECOND
        step(session, b'q', b'Returned to agenda')
        undo(session)
        step(session, b'q', b'Returned to source')
        expected = [ONE[0], '* TODO [#A] One 界 :keep: unsaved', ONE[2]]
        assert exact(last_frame(session), expected)
        # One undo removes one source action; a second removes the earlier user
        # edit. Redo exposes the same transaction and ordinary save preserves bytes.
        send(session, b'\x18u', lambda f: exact(f, [ONE[0], ONE[1] + ' unsaved', ONE[2]]))
        send(session, b'\x18u', lambda f: exact(f, ONE))
        named(session, 'undo-redo', b'** RW --')
        assert exact(last_frame(session), [ONE[0], ONE[1] + ' unsaved', ONE[2]])
        send(session, b'\x18u', lambda f: exact(f, ONE))
        step(session, b'\x18\x13', b'Saved')
        visit(session, str(two), b'> two.org')
        assert exact(last_frame(session), TWO)
        step(session, b'\x18\x13', b'Saved')
        assert one.read_bytes() == FIRST and two.read_bytes() == SECOND
        no_color(session, colored)
        finish(session, colored)
        assert snapshot(root) == before, 'agenda actions changed files after undo/save'
    finally:
        close(session)


def readonly(root, colored):
    session, one, two = opened(root, colored, locked=True)
    before = snapshot(root)
    try:
        query(session)
        frame = bulk(session, b'priority', b'A')
        outcomes(frame, 'Rejected', 'Changed')
        assert 'read-only' in canvas(frame)[2]
        assert one.read_bytes() == FIRST and two.read_bytes() == SECOND
        step(session, b'q', b'Returned to agenda')
        undo(session)
        step(session, b'q', b'Returned to source')
        assert exact(last_frame(session), ONE)
        assert b'-- RO --' in last_frame(session)
        visit(session, str(two), b'> two.org')
        assert exact(last_frame(session), TWO)
        no_color(session, colored)
        finish(session, colored)
        assert snapshot(root) == before, 'read-only agenda failure changed files'
    finally:
        close(session)


def cancellation(root, colored):
    session, one, two = opened(root, colored)
    before = snapshot(root)
    try:
        query(session)
        step(session, b't', b'Org task state: ')
        step(session, b'DONE\r', b'Org state note: ')
        send(session, b'cancelled\x07', lambda f: b'Agenda action Cancelled:' in f
             and canvas(f)[1].startswith('Org TODO agenda'))
        assert one.read_bytes() == FIRST and two.read_bytes() == SECOND
        # A disk replacement after observation must visibly reject its target;
        # the independent first target still succeeds with no implicit write.
        two.write_bytes(SECOND.replace(b'Two', b'External replacement'))
        frame = bulk(session, b'priority', b'A')
        outcomes(frame, 'Changed', 'Rejected')
        assert 'changed' in canvas(frame)[3]
        assert one.read_bytes() == FIRST
        assert two.read_bytes() == SECOND.replace(b'Two', b'External replacement')
        step(session, b'q', b'Returned to agenda')
        undo(session)
        step(session, b'q', b'Returned to source')
        assert exact(last_frame(session), ONE)
        two.write_bytes(SECOND)
        no_color(session, colored)
        finish(session, colored)
        assert snapshot(root) == before, 'cancelled/stale agenda changed fixture files'
    finally:
        close(session)
