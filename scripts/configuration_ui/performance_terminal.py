"""Configured release startup and event-driven idle through one owned PTY."""

import time

from configuration_ui.observation import Observation, no_recursion
from configuration_ui.support import (ROOT, PTY, close, completed, environment,
                                      finish, passive, step)


def memory(process):
    with open(f'/proc/{process.pid}/status') as source:
        status = source.read()
    peak = int(next(line.split()[1] for line in status.splitlines() if line.startswith('VmHWM:')))
    assert 0 < peak <= 131072, 'configured editor RSS exceeded 128 MiB'
    return peak


def quiet(session, watch, idle):
    result = idle(session)
    assert not watch.drain(), 'idle editor opened or read fixture entries'
    return result


def run(root, idle):
    with Observation(root) as watch:
        started = time.perf_counter()
        session = PTY['Session']([str(ROOT / 'target/release/rev'), str(root / 'work/main.rs')],
                                 colors=True, env=environment(root))
        try:
            session.wait(lambda: completed(session, lambda frame: b'-- RW L1 C1 Rust' in frame))
            elapsed = (time.perf_counter() - started) * 1000
            assert elapsed <= 2000, 'configured startup exceeded 2 seconds'
            step(session, b'\x07', b'Quit')
            startup = watch.drain()
            assert startup, 'startup observation control missing'
            no_recursion(startup)
            editing = quiet(session, watch, idle)
            step(session, b'\x18\x06', b'Find file: ')
            assert watch.drain(), 'prompt observation control missing'
            prompt = quiet(session, watch, idle)
            peak = memory(session.process)
            step(session, b'\x07', b'Quit')
            finish(session)
        finally:
            close(session)
    passive(root)
    return {'startup_ms': round(elapsed, 3), 'startup_budget_ms': 2000,
            'editing_idle': editing, 'prompt_idle': prompt, 'peak_rss_kib': peak,
            'idle_open_access_events': 0, 'restored_reaped_closed': True}
