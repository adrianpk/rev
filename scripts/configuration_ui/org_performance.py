"""Additive Org release observations; historical workload ceilings stay intact."""
import hashlib
import json
import math
import tempfile
import time
from pathlib import Path
from .observation import Observation

NAMES = ['org_source', 'org_agenda', 'org_habits', 'org_formulas', 'org_exports', 'org_cancel']
LIMITS = {'peak_rss_kib': 98304, 'retained_growth_kib': 32768,
          'response_ms': 150, 'chunk_ms': 10, 'job_ms': 30000}
DETAILS = [dict(bytes=1048576, headings=10000, chunk_limit_ms=10),
           dict(sources=256, rows=256), dict(days=733, completions=1),
           dict(cells=10000, last_value=10000), dict(backends=11), dict(cycles=16)]


def fixture(root):
    (root / 'work').mkdir()
    (root / 'agenda').mkdir()
    (root / 'user/rev').mkdir(parents=True)
    (root / 'user/rev/config.toml').write_text('')
    lines = [f'* Heading {n:05d} ' + 'x' * 87 + '\n' for n in range(10000)]
    text = ''.join(lines).encode()
    assert len(text) <= 1048576
    text = text[:-1] + b'x' * (1048576 - len(text)) + b'\n'
    (root / 'work/large.org').write_bytes(text)
    for n in range(256):
        (root / f'agenda/{n:03d}.org').write_text(f'* TODO Task {n:03d}\n')
    (root / 'work/habit.org').write_text('* TODO Habit\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n- State "DONE" from "TODO" [2024-02-29 Thu]\n')
    (root / 'work/export.org').write_text('#+TITLE: Native\n* TODO Event\nSCHEDULED: <2024-03-01 Fri 10:00>\nText *bold* [[https://example.invalid][link]].\n| A | B |\n| 1 | 2 |\n')
    return fingerprint(root)


def fingerprint(root):
    digest = hashlib.sha256()
    entries = sorted(root.rglob('*'))
    assert len(entries) == 264, 'Org fixture gained entries'
    paths = [p for p in entries if p.is_file()]
    for path in entries:
        assert not path.is_symlink(), 'unexpected Org symlink'
        digest.update(str(path.relative_to(root)).encode())
    assert len(paths) == 260
    for path in paths:
        digest.update(str(path.relative_to(root)).encode())
        digest.update(path.read_bytes())
    return {'files': len(paths), 'entries': len(entries), 'sha256': digest.hexdigest()}


def records(lines, kind):
    assert kind == 'org'
    rows = [json.loads(s) for s in lines.splitlines() if s != 'ready']
    validate_native(rows)
    return rows


def validate_native(rows):
    assert [r['workload'] for r in rows] == NAMES, 'missing/duplicate Org workload'
    for row, details in zip(rows, DETAILS):
        assert all(row.get(k) == v for k, v in details.items()), row
        assert row['samples'] == (16 if row['workload'] == 'org_cancel' else 3), row
        assert 0 < row['median_us'] <= row['max_us'] <= LIMITS['job_ms'] * 1000, row
        assert math.isfinite(row['max_us']) and 0 < row['elapsed_ms'], row
        assert 0 < row['peak_rss_kib'] <= LIMITS['peak_rss_kib'], row
        assert 0 < row['baseline_rss_kib'] <= LIMITS['peak_rss_kib'], row
        assert 0 <= row['retained_growth_kib'] <= LIMITS['retained_growth_kib'], row
        assert type(row['cpu_ticks']) is int and row['cpu_ticks'] >= 0, row
    assert 0 < rows[0]['max_chunk_us'] <= LIMITS['chunk_ms'] * 1000, rows[0]
    cancel = rows[-1]
    assert cancel['threads_before'] == cancel['threads_after'] == 1, cancel
    assert cancel['fds_before'] == cancel['fds_after'] and cancel['children_after'] == 0, cancel
    assert cancel['cancelled'] == cancel['busy_rejected'] == 16, cancel
    assert rows[-1]['max_us'] <= LIMITS['response_ms'] * 1000, rows[-1]


def run(native, baseline, candidate):
    started = time.perf_counter()
    with tempfile.TemporaryDirectory(prefix='rev-org-performance-') as directory:
        root = Path(directory)
        before = fixture(root)
        with Observation(root, ['work', 'agenda', 'user/rev']) as watch:
            result = native(root, 'org', watch, records)
        from .org_resource_terminal import run as terminal
        terminal_result = terminal(root, baseline)
        assert fingerprint(root) == before, 'Org measurement changed source bytes'
        return {'candidate': candidate, 'profile': 'release', 'elapsed_ms': round((time.perf_counter() - started) * 1000, 3), 'limits': LIMITS, 'fixture_identity': before, 'native': result['records'], 'terminal': terminal_result}


def validate(result, candidate, rate):
    from .org_resource_terminal import STAGES
    assert result['candidate'] == candidate, 'stale Org performance candidate'
    assert result['profile'] == 'release' and result['limits'] == LIMITS, 'Org resource ceilings changed'
    assert math.isfinite(result['elapsed_ms']) and result['elapsed_ms'] > 0
    identity = result['fixture_identity']
    assert identity['files'] == 260 and identity['entries'] == 264
    import re
    assert re.fullmatch('[a-f0-9]{64}', identity['sha256']), 'missing Org fixture hash'
    validate_native(result['native'])
    terminal = result['terminal']
    assert 0 < terminal['startup_ms'] <= 2000
    assert 0 < terminal['peak_rss_kib'] <= LIMITS['peak_rss_kib']
    assert terminal['restored_reaped_closed'] is True and terminal['expiry_count'] == 1
    before = terminal['baseline_owners']
    assert before['threads'] == 1 and not before['children'] and before['fds'] >= 3
    assert list(terminal['idle']) == STAGES, 'missing/duplicate idle state'
    assert len(terminal['responses_ms']) == 11
    assert all(math.isfinite(t) and 0 < t <= LIMITS['response_ms'] for t in terminal['responses_ms'])
    assert rate > 0
    for idle in terminal['idle'].values():
        assert idle['threads'] == before['threads'] and idle['fds'] == before['fds']
        assert idle['children'] == '' and idle['open_access_events'] == idle['output_bytes'] == 0
        budget = max(2, math.ceil(idle['elapsed_ms'] / 1000 * rate * 0.1)) * 1000 / rate
        assert math.isfinite(idle['elapsed_ms']) and idle['elapsed_ms'] >= 350
        assert 0 <= idle['cpu_ms'] <= budget, 'continuing Org idle CPU work'
