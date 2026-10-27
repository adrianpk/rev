"""Release measurements for the admitted configuration and UI workload bounds."""

import json
import math
import os
from pathlib import Path
import runpy
import select
import subprocess
import tempfile
import time

from configuration_ui.performance_fixtures import fixture, fingerprint
from configuration_ui.support import ROOT


KINDS = ['graph', 'entries', 'names', 'redraw', 'syntax']
BUDGET_US = {'graph': 1_000_000, 'entries_observe': 250_000, 'names_observe': 250_000,
             'entries_filter': 10_000, 'names_filter': 10_000, 'redraw': 100_000, 'syntax': 10_000}
EXPECTED = {'graph': ['graph'], 'entries': ['entries_observe', 'entries_filter'],
            'names': ['names_observe', 'names_filter'], 'redraw': ['redraw'], 'syntax': ['syntax']}


def records(lines, kind):
    values = [json.loads(line) for line in lines.splitlines() if line != 'ready']
    assert [value['workload'] for value in values] == EXPECTED[kind], values
    for value in values:
        assert value['samples'] > 0
        assert math.isfinite(value['median_us']) and value['median_us'] > 0
        assert value['median_us'] <= value['max_us'] <= BUDGET_US[value['workload']], value
        assert 0 < value['peak_rss_kib'] <= 131072, value
    if kind in ['entries', 'names']:
        assert values[1]['peak_rss_kib'] - values[0]['peak_rss_kib'] <= 4096, values
    return values


def native(root, kind):
    command = [str(ROOT / 'target/release/examples/configuration-performance'), kind, str(root)]
    process = subprocess.Popen(command, cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE)
    prefix = bytearray()
    deadline = time.monotonic() + 15
    try:
        while b'ready\n' not in prefix:
            remaining = max(0, deadline - time.monotonic())
            assert select.select([process.stdout], [], [], remaining)[0], 'native setup stalled'
            data = os.read(process.stdout.fileno(), 65536)
            assert data, 'native exited before workload barrier'
            prefix.extend(data)
            assert len(prefix) <= 65536, 'native setup output exceeded 64 KiB'
        output, error = process.communicate(b'go\n', timeout=15)
        assert process.returncode == 0, (error[-4000:] + output[-4000:]).decode(errors='replace')
        assert len(output) + len(prefix) <= 65536, 'native output exceeded 64 KiB'
        return records((prefix + output).decode(), kind)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=1)
        for stream in [process.stdin, process.stdout, process.stderr]:
            stream.close()


def run(selected):
    baseline = runpy.run_path(str(ROOT / 'scripts/check-performance.py'))
    result = {'hardware': baseline['hardware'](), 'budgets_us': BUDGET_US,
              'peak_rss_limit_kib': 131072, 'filter_growth_limit_kib': 4096, 'native': []}
    with tempfile.TemporaryDirectory(prefix='rev-configuration-performance-') as directory:
        root = Path(directory)
        result['fixtures'] = fixture(root)
        before = fingerprint(root)
        for kind in KINDS if selected is None else [selected]:
            result['native'].extend(native(root, kind))
        assert fingerprint(root) == before, 'performance work changed fixture entries/bytes'
        result['fixture_identity'] = before
    print(json.dumps(result, indent=2))
    print(f'Configuration/UI performance suite: {len(result["native"])} native workloads passed, 0 skipped.')
