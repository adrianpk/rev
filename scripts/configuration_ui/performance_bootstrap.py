"""Observe one complete first publication and a read-only second startup."""

import hashlib
import os
from pathlib import Path
import tempfile
import time

from .observation import ACCESS, OPEN, Observation
from .performance_terminal import memory, quiet
from .support import (ROOT, PTY, close, completed, environment, finish, passive,
                      snapshot, startup_snapshot)

CREATE = 0x100
CLOSE_WRITE = 0x8
MOVED_TO = 0x80
DELETE = 0x200
MUTATIONS = CREATE | CLOSE_WRITE | MOVED_TO | DELETE


def startup(root, watch, baseline):
    started = time.monotonic()
    session = PTY['Session']([str(ROOT / 'target/release/rev'), str(root / 'work/source.txt')],
                             env=environment(root))
    try:
        session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1')))
        elapsed = (time.monotonic() - started) * 1000
        cpu = baseline['cpu_ticks'](session.process) * 1000 / os.sysconf('SC_CLK_TCK')
        assert elapsed <= 2000 and cpu <= 500, 'bootstrap startup exceeded bounded work'
        events = watch.drain()
        publication = sum(name == b'config.toml' and bool(mask & (CREATE | MOVED_TO)) for _, mask, name in events)
        mutations = sum(bool(mask & MUTATIONS) for _, mask, _ in events)
        idle = quiet(session, watch, baseline['idle'])
        peak = memory(session.process)
        finish(session)
    finally:
        close(session)
    return {'startup_ms': round(elapsed, 3), 'cpu_ms': cpu, 'publication_events': publication,
            'mutation_events': mutations, 'idle': idle, 'peak_rss_kib': peak,
            'restored_reaped_closed': True}


def run(baseline):
    with tempfile.TemporaryDirectory(prefix='rev-bootstrap-performance-') as directory:
        root = Path(directory)
        (root / 'user/rev').mkdir(parents=True)
        (root / 'work').mkdir()
        (root / 'work/source.txt').write_bytes(b'x\n')
        expected = startup_snapshot(root)
        path = root / 'user/rev/config.toml'
        assert not path.exists()
        with Observation(root, ['user/rev', 'work'], OPEN | ACCESS | MUTATIONS) as watch:
            first = startup(root, watch, baseline)
            assert first['publication_events'] == 1 and first['mutation_events'] > 0, 'first publication control missing'
            assert snapshot(root) == expected, 'first startup changed unexpected bytes/entries'
            before = retained(path)
            watch.drain()  # Discard only the collector's snapshot reads.
            second = startup(root, watch, baseline)
            assert second['publication_events'] == second['mutation_events'] == 0, 'bootstrap work repeated'
            assert retained(path) == before and snapshot(root) == expected, 'existing configuration was replaced'
        passive(root)
        content = path.read_bytes()
        return {'starts': [first, second], 'template_bytes': len(content),
                'template_sha256': hashlib.sha256(content).hexdigest(), 'existing_preserved': True}


def retained(path):
    stat = path.stat()
    return stat.st_dev, stat.st_ino, stat.st_mode, stat.st_size, stat.st_mtime_ns, stat.st_ctime_ns
