"""Additive Org release observations; historical workload ceilings stay intact."""
import hashlib
import json
import math
import tempfile
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
    paths = sorted(p for p in root.rglob('*') if p.is_file())
    assert len(paths) == 260
    for path in paths:
        digest.update(str(path.relative_to(root)).encode())
        digest.update(path.read_bytes())
    return {'files': len(paths), 'sha256': digest.hexdigest()}


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


def run(native, baseline):
    with tempfile.TemporaryDirectory(prefix='rev-org-performance-') as directory:
        root = Path(directory)
        before = fixture(root)
        with Observation(root, ['work', 'agenda', 'user/rev']) as watch:
            result = native(root, 'org', watch, records)
        assert fingerprint(root) == before, 'Org measurement changed source bytes'
        return {'limits': LIMITS, 'fixture_identity': before, 'native': result['records']}
