"""Release measurements at the real Org event-loop and timer boundaries."""
import os
from pathlib import Path
import time
from .support import ROOT, PTY, close, completed, environment, finish, passive, send, step
from .terminal_evidence import owned
from .org_time import START, SET, PAUSE, STOP, quiesce
from .observation import Observation

STAGES = ['org_idle', 'relative', 'paused_relative', 'armed', 'paused_countdown',
          'expired', 'stopped', 'closed_view', 'inactive']


def owners(process):
    root = Path(f'/proc/{process.pid}')
    return {'threads': len(list((root / 'task').iterdir())),
            'fds': len(list((root / 'fd').iterdir())),
            'children': (root / f'task/{process.pid}/children').read_text().strip()}


def idle(session, watch, baseline):
    quiesce(session)
    watch.drain()  # Earlier explicit commands are positive controls, not idle work.
    state = owners(session.process)
    assert state['threads'] == 1 and not state['children'], state
    measured = baseline['idle'](session)
    assert not watch.drain(), 'inactive Org state read fixture files'
    assert owners(session.process) == state, 'idle owner counts changed'
    measured.update(state)
    measured['open_access_events'] = 0
    return measured


def named(session, command, marker):
    return step(session, b'\x1bx' + command.encode() + b'\r', marker)


def stage(session, wire, marker):
    started = time.perf_counter()
    step(session, wire, marker)
    elapsed = (time.perf_counter() - started) * 1000
    assert elapsed <= 150, f'Org response exceeded 150 ms: {elapsed}'
    session.org_response_ms.append(round(elapsed, 3))
    return round(elapsed, 3)


def timers(session, watch, baseline, result):
    stage(session, START, b'Org relative timer started')
    result['responses_ms'] = session.org_response_ms
    result['idle']['relative'] = idle(session, watch, baseline)
    stage(session, PAUSE, b'timer paused: ')
    result['idle']['paused_relative'] = idle(session, watch, baseline)
    stage(session, STOP, b'Org timer stopped')
    step(session, SET, b'Org countdown (minutes or H:MM:SS): ')
    stage(session, b'0:00:01\r', b'Org countdown started')
    result['idle']['armed'] = idle(session, watch, baseline)
    stage(session, PAUSE, b'timer paused: ')
    result['idle']['paused_countdown'] = idle(session, watch, baseline)
    stage(session, PAUSE, b'timer running: ')
    session.wait(lambda: completed(session, lambda f: b'Org countdown finished' in f and b'Timer done' in f))
    assert bytes(session.output).count(b'Org countdown finished') == 1, 'expiry repeated'
    result['expiry_count'] = 1
    result['idle']['expired'] = idle(session, watch, baseline)
    stage(session, START, b'Org relative timer started')
    stage(session, STOP, b'Org timer stopped')
    result['idle']['stopped'] = idle(session, watch, baseline)


def run(root, baseline):
    result = {'idle': {}}
    with owned(root), Observation(root, ['work', 'agenda', 'user/rev']) as watch:
        started = time.perf_counter()
        session = PTY['Session']([str(ROOT / 'target/release/rev'), str(root / 'work/habit.org')],
                                  colors=True, env=environment(root), cwd=root / 'work')
        session.org_response_ms = []
        try:
            session.wait(lambda: completed(session, lambda f: b'> habit.org -- RW -- L1 C1' in f))
            result['startup_ms'] = round((time.perf_counter() - started) * 1000, 3)
            assert result['startup_ms'] <= 2000
            result['idle']['org_idle'] = idle(session, watch, baseline)
            result['baseline_owners'] = owners(session.process)
            timers(session, watch, baseline, result)
            stage(session, b'\x1bxorg-habit-show-graph\r', b'Habit agenda: ')
            stage(session, b'\x07', b'> habit.org -- RW --')
            result['idle']['closed_view'] = idle(session, watch, baseline)
            stage(session, b'\x1bxtext-mode\r', b'Text mode')
            result['idle']['inactive'] = idle(session, watch, baseline)
            assert owners(session.process) == result['baseline_owners'], 'Org owners leaked'
            status = Path(f'/proc/{session.process.pid}/status').read_text()
            result['peak_rss_kib'] = int(next(s.split()[1] for s in status.splitlines() if s.startswith('VmHWM:')))
            assert 0 < result['peak_rss_kib'] <= 98304
            finish(session)
        finally:
            close(session)
    passive(root)
    result['restored_reaped_closed'] = True
    assert list(result['idle']) == STAGES
    return result
