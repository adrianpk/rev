"""Release/stop and live drag measurements using the existing PTY lifetime."""

import os
import select
import time

from .navigation import repeat
from .observation import Observation
from .performance_terminal import memory, quiet
from .pointer_focus import report as press
from .pointer_selection import report as drag
from .pointer_scroll import report as wheel
from .support import ROOT, PTY, close, completed, environment, finish, send, step


def feed_disabled(session, wire, ticks):
    started = time.monotonic()
    before = ticks(session.process)
    for index in range(64):
        due = started + index / 30
        remaining = max(0, due - time.monotonic())
        assert not select.select([session.master], [], [], remaining)[0], 'disabled pointer emitted output'
        assert os.write(session.master, wire) == len(wire), 'short disabled report write'
    elapsed = (time.monotonic() - started) * 1000
    cpu = (ticks(session.process) - before) * 1000 / os.sysconf('SC_CLK_TCK')
    assert elapsed <= 2500 and cpu <= 200, 'disabled input exceeded bounded work'
    return {'reports': 256, 'rate_per_second': 120, 'elapsed_ms': round(elapsed, 3), 'cpu_ms': cpu}


def run(root, baseline):
    idle = baseline['idle']
    ticks = baseline['cpu_ticks']
    frequency = os.sysconf('SC_CLK_TCK')
    with Observation(root) as watch:
        session = PTY['Session']([str(ROOT / 'target/release/rev'), str(root / 'work/large.txt')],
                                 env=environment(root))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1')))
            step(session, b'\x1bxtoggle-file-tree\r', b'File tree hidden')
            watch.drain()
            reports = []
            for direction, key, target in [('down', b'\x1b[B', 31), ('up', b'\x1b[A', 1)]:
                before = ticks(session.process)
                started = time.monotonic()
                first, tail = repeat(session, key, 30, 120, target)
                cpu_ms = (ticks(session.process) - before) * 1000 / frequency
                assert cpu_ms <= 200, 'repeat CPU exceeded 200 ms'
                reports.append({'direction': direction, 'reports': 30, 'rate_per_second': 120, 'first_ms': first, 'release_ms': tail,
                                'elapsed_ms': round((time.monotonic() - started) * 1000, 3), 'cpu_ms': cpu_ms})
            assert not watch.drain(), 'repeat reopened fixture entries'
            released = quiet(session, watch, idle)
            started = time.monotonic()
            before = ticks(session.process)
            send(session, press(8, 0) + drag(8, 22), lambda frame: PTY['status_contains'](frame, b'-- RW -- L25 C1'))
            active_ms = (time.monotonic() - started) * 1000
            active_cpu = (ticks(session.process) - before) * 1000 / frequency
            assert active_cpu <= 100, 'drag CPU exceeded 100 ms'
            assert 100 <= active_ms <= 250, 'first drag tick missed its bounded deadline'
            # Releasing outside the text retires capture without a redundant
            # frame. The following quiet interval proves that no tick survives.
            os.write(session.master, drag(8, 22, True))
            drag_idle = quiet(session, watch, idle)
            step(session, b'\x1bxmouse-mode\r', b'Mouse mode disabled')
            ignored = press(8, 0) + drag(8, 22) + wheel(False, 8, 0) + drag(8, 22, True)
            disabled_feed = feed_disabled(session, ignored, ticks)
            disabled_idle = quiet(session, watch, idle)
            step(session, b'\x18', b'C-x')
            step(session, b'\x07', b'Quit')
            assert PTY['status_contains'](PTY['last_frame'](session), b'-- RW -- L25 C1'), 'disabled reports moved point'
            peak = memory(session.process)
            finish(session)
        finally:
            close(session)
    return {'repeat': reports, 'release_idle': released, 'drag_first_tick_ms': round(active_ms, 3),
            'drag_cpu_ms': active_cpu, 'drag_idle': drag_idle, 'disabled_idle': disabled_idle,
            'disabled_feed': disabled_feed, 'peak_rss_kib': peak, 'restored_reaped_closed': True}
