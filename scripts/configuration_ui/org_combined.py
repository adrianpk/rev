"""One owned native session connects canonical Org workflows and refusals."""

from . import org_agenda, org_capture, org_tasks
from .org_clocks import clocks
from .org_tasks import document, exact, goto, named
from .org_time import quiet
from .support import (BINARY, PTY, canvas, close, completed, environment, finish,
                      passive, send, snapshot, step, write)

STAGES = ['editing-history', 'clocks', 'capture-refile', 'habits', 'agenda',
          'table-publication', 'optional-tool-failure', 'restoration']
HABIT = ['* TODO Practice', 'SCHEDULED: <2026-10-29 Thu .+1d>',
         ':PROPERTIES:', ':STYLE: habit', ':END:', 'habit EOF']
TABLE = ['* Numbers', '| 2 | 3 | 0 |', '#+TBLFM: $3=$1+$2']
CALCULATED = ['* Numbers', '| 2 | 3 | 5 |', '#+TBLFM: $3=$1+$2']
BLOCK = ['#+BEGIN_SRC sh', 'printf forbidden > must-not-exist', '#+END_SRC',
         '#+RESULTS:', ': previous']
POLICY = org_capture.POLICY + b'''org.agenda.sources=['../../work/projects.org','../../work/habits.org']
org.source.programs=[{name='missing',language='sh',executable='../../work/missing-runner'}]
''' + org_tasks.POLICY.replace(b'org.time.zone="UTC"\n', b'')


def encoded(lines):
    return ('\ufeff' + '\r\n'.join(lines)).encode()


def habits(session, path):
    org_capture.visit(session, path, HABIT)
    quiet(session)
    frame = org_agenda.named(session, 'org-habit-show-graph', b'Habit agenda:')
    assert canvas(frame)[1].startswith('Habit agenda ')
    assert any('Practice' in line for line in canvas(frame).values())
    # The retained graph must consume text without changing its canonical source.
    org_agenda.step(session, PTY['paste_wire'](b'X'), b'Paste is unavailable in this input context')
    frame = org_agenda.step(session, b'\x07', b'Returned to Org source')
    assert exact(frame, HABIT)
    assert path.read_bytes() == encoded(HABIT)


def agenda(session):
    quiet(session)
    org_agenda.named(session, 'org-todo-list', b'TODO keywords [empty: unfinished; | separates]: ')
    frame = org_agenda.send(session, b'\r', lambda f: canvas(f)[1].startswith('Org TODO agenda [complete]')
                            and any('Note 界e\u0301' in line for line in canvas(f).values())
                            and any('Practice' in line for line in canvas(f).values()))
    assert not frame.endswith(PTY['SHOW'])
    org_agenda.step(session, PTY['paste_wire'](b'X'), b'Paste is unavailable in this input context')
    frame = org_agenda.step(session, b'\x07', b'Returned to source')
    assert exact(frame, HABIT)


def table(session, path):
    org_capture.visit(session, path, TABLE)
    goto(session, 2)
    send(session, b'\x03*', lambda f: exact(f, CALCULATED))
    assert path.read_bytes() == encoded(TABLE), 'recalculation saved implicitly'
    send(session, b'\x18u', lambda f: exact(f, TABLE))
    send(session, b'\x1bxundo-redo\r', lambda f: exact(f, CALCULATED))
    step(session, b'\x18\x13', b'Saved ')
    assert path.read_bytes() == encoded(CALCULATED)


def missing_tool(session, root, path):
    org_capture.visit(session, path, BLOCK)
    goto(session, 2)
    quiet(session)
    assert not (root / 'work/missing-runner').exists()
    frame = step(session, b'\x03\x16e', b'with host authority? yes/no:')
    assert b'missing-runner' in frame
    step(session, b'no\r', b'cancelled before launch')
    assert path.read_bytes() == encoded(BLOCK)
    frame = step(session, b'\x03\x16e', b'with host authority? yes/no:')
    assert b'missing-runner' in frame
    frame = step(session, b'yes\r', b'Source Failed')
    assert exact(frame, BLOCK), 'failed execution published a result'
    assert path.read_bytes() == encoded(BLOCK)
    assert not (root / 'work/must-not-exist').exists()


def journey(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    tasks = write(root, 'work/tasks.org', org_tasks.SOURCE)
    origin = write(root, 'work/origin.org', org_capture.encoded(org_capture.ORIGIN))
    inbox = write(root, 'work/inbox.org', org_capture.encoded(['* Inbox']))
    project = write(root, 'work/projects.org', org_capture.encoded(['* Projects']))
    habit = write(root, 'work/habits.org', encoded(HABIT))
    numbers = write(root, 'work/numbers.org', encoded(TABLE))
    block = write(root, 'work/block.org', encoded(BLOCK))
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(tasks)], env=environment(root, colored), cwd=tasks.parent)
    label = 'org-combined-' + ('rgb' if colored else 'no-color')
    try:
        session.ready(b'> tasks.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, org_tasks.LINES)))
        org_tasks.transition(session, b'\x03\x14', tasks)
        print(f'PASS {label} editing-history', flush=True)
        clocks(session, tasks, org_tasks.LINES, goto, exact, document)
        print(f'PASS {label} clocks', flush=True)
        org_capture.visit(session, origin, org_capture.ORIGIN)
        org_capture.capture(session, inbox)
        org_capture.move(session, inbox, project)
        before['work/inbox.org'] = org_capture.encoded(['* Inbox'])
        before['work/projects.org'] = org_capture.encoded(org_capture.PROJECT)
        print(f'PASS {label} capture-refile', flush=True)
        habits(session, habit)
        print(f'PASS {label} habits', flush=True)
        agenda(session)
        print(f'PASS {label} agenda', flush=True)
        table(session, numbers)
        before['work/numbers.org'] = encoded(CALCULATED)
        print(f'PASS {label} table-publication', flush=True)
        missing_tool(session, root, block)
        print(f'PASS {label} optional-tool-failure', flush=True)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
    finally:
        close(session)
    assert snapshot(root) == before, 'combined Org workflow changed unexpected files'
    passive(root)
    print(f'PASS {label} restoration', flush=True)
