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

from configuration_ui.observation import Observation, evidence
from configuration_ui.performance_terminal import run as terminal
from configuration_ui.performance_fixtures import fixture, fingerprint
from configuration_ui.support import ROOT


KINDS = ['graph', 'entries', 'names', 'redraw', 'syntax', 'idle']
BUDGET_US = {'graph': 1_000_000, 'entries_observe': 250_000, 'names_observe': 250_000,
             'entries_filter': 10_000, 'names_filter': 10_000, 'redraw': 100_000, 'syntax': 10_000}
SAMPLES = {'graph': 3, 'entries_observe': 3, 'names_observe': 3, 'entries_filter': 256,
           'names_filter': 256, 'redraw': 64, 'syntax': 32}
EXPECTED = {'graph': ['graph'], 'entries': ['entries_observe', 'entries_filter'],
            'names': ['names_observe', 'names_filter'], 'redraw': ['redraw'], 'syntax': ['syntax']}


def records(lines, kind):
    values = [json.loads(line) for line in lines.splitlines() if line != 'ready']
    assert [value['workload'] for value in values] == EXPECTED[kind], values
    for value in values:
        assert value['samples'] == SAMPLES[value['workload']]
        assert math.isfinite(value['median_us']) and value['median_us'] > 0
        assert value['median_us'] <= value['max_us'] <= BUDGET_US[value['workload']], value
        assert 0 < value['peak_rss_kib'] <= 131072, value
    if kind in ['entries', 'names']:
        assert values[1]['peak_rss_kib'] - values[0]['peak_rss_kib'] <= 4096, values
    return values


def native(root, kind, watch):
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
        before = watch.drain()
        output, error = process.communicate(b'go\n', timeout=15)
        assert process.returncode == 0, (error[-4000:] + output[-4000:]).decode(errors='replace')
        proof = evidence(kind, before, watch.drain())
        assert len(output) + len(prefix) <= 65536, 'native output exceeded 64 KiB'
        return {'records': records((prefix + output).decode(), kind), 'observation': proof}
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=1)
        for stream in [process.stdin, process.stdout, process.stderr]:
            stream.close()


def watched(root, kind):
    with Observation(root) as watch:
        return native(root, kind, watch)


def run(selected):
    started = time.perf_counter()
    baseline = runpy.run_path(str(ROOT / 'scripts/check-performance.py'))
    result = {'hardware': baseline['hardware'](), 'budgets_us': BUDGET_US,
              'peak_rss_limit_kib': 131072, 'filter_growth_limit_kib': 4096, 'native': [], 'observation': {}}
    with tempfile.TemporaryDirectory(prefix='rev-configuration-performance-') as directory:
        root = Path(directory)
        result['fixtures'] = fixture(root)
        before = fingerprint(root)
        kinds = KINDS[:-1] if selected is None else [selected]
        for kind in [value for value in kinds if value != 'idle']:
            measured = watched(root, kind)
            result['native'].extend(measured['records'])
            result['observation'][kind] = measured['observation']
        if selected is None or selected == 'idle':
            result['terminal'] = terminal(root, baseline['idle'])
        assert fingerprint(root) == before, 'performance work changed fixture entries/bytes'
        result['fixture_identity'] = before
        expected = 7 if selected is None else len(EXPECTED.get(selected, []))
        assert len(result['native']) == expected, 'missing required native workload'
        assert ('terminal' in result) == (selected is None or selected == 'idle'), 'missing PTY evidence'
    result['measurement_elapsed_ms'] = round((time.perf_counter() - started) * 1000, 3)
    print(json.dumps(result, indent=2))
    print(f'Configuration/UI performance suite: {len(result["native"])} native workloads, {int("terminal" in result)} PTY session passed, 0 skipped.')
