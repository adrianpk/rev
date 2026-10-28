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

from acceptance import identity
from configuration_ui.observation import Observation, evidence
from configuration_ui.interaction_terminal import run as interaction_terminal
from configuration_ui.performance_bootstrap import run as bootstrap
from configuration_ui.performance_terminal import run as terminal
from configuration_ui.performance_fixtures import fixture, fingerprint
from configuration_ui.interaction_evidence import validate as validate_interaction
from configuration_ui.support import ROOT, environment


KINDS = ['graph', 'entries', 'names', 'redraw', 'syntax', 'gutter', 'drag', 'idle']
BUDGET_US = {'graph': 1_000_000, 'entries_observe': 250_000, 'names_observe': 250_000,
             'entries_filter': 10_000, 'names_filter': 10_000, 'redraw': 100_000, 'syntax': 10_000}
BUDGET_US.update(gutter_absolute=100_000, gutter_relative=100_000, drag_active=100_000, drag_idle=10_000)
BUDGET_US.update(gutter_hidden=100_000, mouse_disabled=10_000)
SAMPLES = {'graph': 3, 'entries_observe': 3, 'names_observe': 3, 'entries_filter': 256,
           'names_filter': 256, 'redraw': 64, 'syntax': 32}
SAMPLES.update(gutter_absolute=64, gutter_relative=64, drag_active=64, drag_idle=64)
SAMPLES.update(gutter_hidden=64, mouse_disabled=64)
EXPECTED = {'graph': ['graph'], 'entries': ['entries_observe', 'entries_filter'],
            'names': ['names_observe', 'names_filter'], 'redraw': ['redraw'], 'syntax': ['syntax']}
EXPECTED.update(gutter=['gutter_absolute', 'gutter_relative', 'gutter_hidden'], drag=['drag_active', 'drag_idle', 'mouse_disabled'])
INTERACTION_LIMITS = {'native_elapsed_ms': 1000, 'native_cpu_ms': 500,
                      'repeat_update_ms': 150, 'repeat_cpu_ms': 200,
                      'drag_first_tick_ms': 250, 'drag_cpu_ms': 100,
                      'disabled_feed_ms': 2500, 'disabled_feed_cpu_ms': 200}


def records(lines, kind):
    values = [json.loads(line) for line in lines.splitlines() if line != 'ready']
    assert [value['workload'] for value in values] == EXPECTED[kind], values
    for value in values:
        assert value['samples'] == SAMPLES[value['workload']]
        assert math.isfinite(value['median_us']) and value['median_us'] > 0
        assert value['median_us'] <= value['max_us'] <= BUDGET_US[value['workload']], value
        assert 0 < value['peak_rss_kib'] <= 131072, value
        assert math.isfinite(value['elapsed_ms']) and value['elapsed_ms'] > 0, value
        assert type(value['cpu_ticks']) is int and value['cpu_ticks'] >= 0, value
        if kind in ['gutter', 'drag']:
            assert value['elapsed_ms'] <= INTERACTION_LIMITS['native_elapsed_ms'], value
            cpu_ms = value['cpu_ticks'] * 1000 / os.sysconf('SC_CLK_TCK')
            assert cpu_ms <= INTERACTION_LIMITS['native_cpu_ms'], value
    if kind in ['entries', 'names']:
        assert values[1]['peak_rss_kib'] - values[0]['peak_rss_kib'] <= 4096, values
    return values


def native(root, kind, watch):
    command = [str(ROOT / 'target/release/examples/configuration-performance'), kind, str(root)]
    process = subprocess.Popen(command, cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env=dict(os.environ, **environment(root)))
    prefix = bytearray()
    deadline = time.monotonic() + 15
    try:
        while b'ready\n' not in prefix:
            remaining = max(0, deadline - time.monotonic())
            assert select.select([process.stdout], [], [], remaining)[0], 'native setup stalled'
            data = os.read(process.stdout.fileno(), 65536)
            if not data:
                _, error = process.communicate(timeout=max(0.01, remaining))
                raise AssertionError(f'{kind} setup exited before barrier: {error[-4000:].decode(errors="replace")}')
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
    candidate = identity(ROOT)
    baseline = runpy.run_path(str(ROOT / 'scripts/check-performance.py'))
    result = {'candidate': candidate, 'interaction_limits': INTERACTION_LIMITS,
              'hardware': baseline['hardware'](), 'budgets_us': BUDGET_US,
              'clock_ticks_per_second': os.sysconf('SC_CLK_TCK'), 'peak_rss_limit_kib': 131072, 'filter_growth_limit_kib': 4096, 'native': [], 'observation': {}}
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
            result['interaction'] = interaction_terminal(root, baseline)
            result['bootstrap'] = bootstrap(baseline)
        assert fingerprint(root) == before, 'performance work changed fixture entries/bytes'
        result['fixture_identity'] = before
        expected = sum(map(len, EXPECTED.values())) if selected is None else len(EXPECTED.get(selected, []))
        assert len(result['native']) == expected, 'missing required native workload'
        assert ('terminal' in result) == (selected is None or selected == 'idle'), 'missing PTY evidence'
    result['measurement_elapsed_ms'] = round((time.perf_counter() - started) * 1000, 3)
    assert identity(ROOT) == candidate, 'performance candidate changed during measurements'
    if selected is None:
        validate_interaction(result, candidate, INTERACTION_LIMITS)
    print(json.dumps(result, indent=2))
    sessions = 4 if 'terminal' in result else 0
    print(f'Configuration/UI performance suite: {len(result["native"])} native workloads, {sessions} PTY sessions passed, 0 skipped.')
