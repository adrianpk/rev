"""Admission of complete bounded interaction measurements on one candidate."""

import math
import re
import hashlib

from acceptance import require
from configuration_ui.support import ROOT


def idle(value, rate):
    require(type(rate) is int and rate > 0, 'missing interaction clock rate')
    elapsed = value['elapsed_ms']
    budget = max(2, math.ceil(elapsed / 1000 * rate * 0.1)) * 1000 / rate
    require(math.isfinite(elapsed) and elapsed >= 350 and value['output_bytes'] == 0
            and 0 <= value['cpu_ms'] <= budget, 'interaction did not remain idle')


def lifetime(value):
    require(value['restored_reaped_closed'] is True and 0 < value['peak_rss_kib'] <= 131072,
            'missing interaction process memory/cleanup')


def native(rows):
    values = {row['workload']: row for row in rows}
    for name in ['gutter_absolute', 'gutter_relative', 'gutter_hidden']:
        row = values[name]
        count = 0 if name == 'gutter_hidden' else 22
        require(row['logical_lines'] == 100001 and row['viewport_rows'] == 22
                and row['number_rows_per_frame'] == count and row['frames'] == 64,
                'unbounded or missing gutter projection')
    active = values['drag_active']
    require(all(active[key] == value for key, value in {'iterations': 64, 'ticks': 64,
                'tick_ms': 100, 'step_rows': 3, 'scrolled_rows': 192}.items()), 'unbounded drag work')
    released = values['drag_idle']
    require(released['iterations'] == 64 and released['ticks'] == released['deadlines'] == 0,
            'released drag retained work')
    disabled = values['mouse_disabled']
    require(disabled['iterations'] == 64 and disabled['reports'] == 256
            and disabled['redraws'] == disabled['deadlines'] == 0, 'disabled pointer retained work')


def terminal(result, rate, limits):
    lifetime(result)
    require([row['direction'] for row in result['repeat']] == ['down', 'up'], 'missing ordered repeat workload')
    for row in result['repeat']:
        require(row['reports'] == 30 and row['rate_per_second'] == 120, 'missing repeat iterations')
        require(0 <= row['first_ms'] <= limits['repeat_update_ms']
                and 0 <= row['release_ms'] <= limits['repeat_update_ms'], 'repeat latency exceeded')
        require(math.isfinite(row['elapsed_ms']) and 0 < row['elapsed_ms'] <= 4250
                and 0 <= row['cpu_ms'] <= limits['repeat_cpu_ms'], 'unbounded repeat work')
    require(100 <= result['drag_first_tick_ms'] <= limits['drag_first_tick_ms']
            and 0 <= result['drag_cpu_ms'] <= limits['drag_cpu_ms'], 'unbounded live drag work')
    feed = result['disabled_feed']
    require(feed['reports'] == 256 and feed['rate_per_second'] == 120
            and 2100 <= feed['elapsed_ms'] <= limits['disabled_feed_ms']
            and 0 <= feed['cpu_ms'] <= limits['disabled_feed_cpu_ms'], 'missing bounded disabled pointer reports')
    for stage in ['release_idle', 'drag_idle', 'disabled_idle']:
        idle(result[stage], rate)


def bootstrap(result, rate):
    starts = result['starts']
    require(len(starts) == 2 and result['existing_preserved'] is True, 'missing repeated startup control')
    require(0 < result['template_bytes'] <= 262144
            and re.fullmatch(r'[a-f0-9]{64}', result['template_sha256']), 'missing complete bootstrap identity')
    template = (ROOT / 'docs/reference/config.toml').read_bytes()
    require(result['template_bytes'] == len(template)
            and result['template_sha256'] == hashlib.sha256(template).hexdigest(), 'stale bootstrap bytes')
    require(starts[0]['publication_events'] == 1 and starts[0]['mutation_events'] > 0
            and starts[1]['publication_events'] == starts[1]['mutation_events'] == 0, 'bootstrap work repeated or absent')
    for start in starts:
        lifetime(start)
        require(0 < start['startup_ms'] <= 2000 and 0 <= start['cpu_ms'] <= 500, 'unbounded startup work')
        idle(start['idle'], rate)


def validate(result, candidate, limits):
    require(result['candidate'] == candidate and all(re.fullmatch(r'[a-f0-9]{40}', candidate[key])
            for key in ['head', 'tree']), 'stale interaction candidate')
    require(result['interaction_limits'] == limits, 'interaction limits changed')
    native(result['native'])
    rate = result['clock_ticks_per_second']
    terminal(result['interaction'], rate, limits)
    bootstrap(result['bootstrap'], rate)
