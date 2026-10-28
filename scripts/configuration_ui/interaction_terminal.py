"""Release/stop and live drag measurements using the existing PTY lifetime."""

import os
import time

from .navigation import repeat
from .observation import Observation
from .performance_terminal import memory, quiet
from .pointer_focus import report as press
from .pointer_selection import report as drag
from .support import ROOT, PTY, close, completed, environment, finish, send, step


def run(root, idle):
    with Observation(root) as watch:
        session = PTY['Session']([str(ROOT / 'target/release/rev'), str(root / 'work/large.txt')],
                                 env=environment(root))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1')))
            step(session, b'\x1bxtoggle-file-tree\r', b'File tree hidden')
            watch.drain()
            reports = []
            for key, target in [(b'\x1b[B', 31), (b'\x1b[A', 1)]:
                first, tail = repeat(session, key, 30, 120, target)
                reports.append({'reports': 30, 'rate_per_second': 120, 'first_ms': first, 'release_ms': tail})
            assert not watch.drain(), 'repeat reopened fixture entries'
            released = quiet(session, watch, idle)
            started = time.monotonic()
            send(session, press(8, 0) + drag(8, 22), lambda frame: PTY['status_contains'](frame, b'-- RW -- L25 C1'))
            active_ms = (time.monotonic() - started) * 1000
            assert 100 <= active_ms <= 250, 'first drag tick missed its bounded deadline'
            # Releasing outside the text retires capture without a redundant
            # frame. The following quiet interval proves that no tick survives.
            os.write(session.master, drag(8, 22, True))
            drag_idle = quiet(session, watch, idle)
            peak = memory(session.process)
            finish(session)
        finally:
            close(session)
    return {'repeat': reports, 'release_idle': released, 'drag_first_tick_ms': round(active_ms, 3),
            'drag_idle': drag_idle, 'peak_rss_kib': peak, 'restored_reaped_closed': True}
