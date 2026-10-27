"""Configuration/interaction acceptance map and exact-run evidence checks."""

import json
import math
from pathlib import Path
import re
import subprocess

from acceptance import CONFIGURATION_COMMANDS, digest, references, require
from acceptance_delivery import integrated
from configuration_ui.delivery import ancestor, reconciled, tickets
from configuration_ui.performance import BUDGET_US, EXPECTED, records

GROUPS = {'configured': ['configured-rgb', 'configured-no-color'], 'missing': ['missing'],
          'rejected': ['rejected'], 'workspace': ['workspace-rgb', 'workspace-no-color'],
          'io': ['io'], 'unwind': ['unwind']}
PTY_CASES = [name for group in GROUPS.values() for name in group]
PERFORMANCE = [name for group in EXPECTED.values() for name in group]
IDS = [f'CFG{i:02}' for i in range(1, 23)] + ['R01', 'R17', 'R20', 'R22'] + [f'UI{i:02}' for i in range(1, 5)]
MAP = 'scripts/configuration_ui/acceptance.json'
PLAN = 'ops/default/plan/rev-configuration-ui.md'
TRACKER = 'ops/default/tracker/rev-configuration-ui.md'
SECTIONS = ['Purpose', 'Delivered Behavior', 'Implementation Notes', 'Contracts Added or Changed',
            'Files of Interest', 'Validation', 'Risks and Follow-ups']


def manifest(root):
    document = json.loads((root / MAP).read_text())
    require(document['version'] == 1, 'unknown configuration acceptance map')
    require([row['id'] for row in document['cases']] == IDS, 'configuration requirement identities changed')
    for row in document['cases']:
        require(row['condition'] and row['native'], f"{row['id']}: empty requirement evidence")
        pairs = [(case['target'], case['test']) for case in row['native']]
        require(len(set(pairs)) == len(pairs), 'duplicate configuration native reference')
        for target, name in pairs:
            require(target.startswith('tests/') and '..' not in Path(target).parts and (root / target).is_file(), f'missing native target {target}')
            require(re.fullmatch(r'[a-z_][a-z_0-9:]*', name), 'invalid native test identity')
        require(set(row['pty']) <= set(PTY_CASES) and len(set(row['pty'])) == len(row['pty']), 'unknown/duplicate configuration PTY reference')
        require(set(row['performance']) <= set(PERFORMANCE) and len(set(row['performance'])) == len(row['performance']), 'unknown/duplicate configuration performance reference')
    require({case for row in document['cases'] for case in row['pty']} == set(PTY_CASES), 'incomplete configuration terminal map')
    require({case for row in document['cases'] for case in row['performance']} == set(PERFORMANCE), 'incomplete configuration performance map')
    return document


def delivery(root, document, readiness):
    tables = [re.findall(r'^\| Slice \d+ \|.*$', (root / path).read_text(), re.M) for path in [PLAN, TRACKER]]
    require(tables[0] == tables[1] and len(tables[0]) == 30, 'configuration slice map mismatch')
    require(digest('\n'.join(tables[0])) == document['canonical_map_sha256'], 'approved configuration slice identities changed')
    for number, line in enumerate(tables[0], 1):
        fields = [part.strip().strip('`') for part in line.strip('|').split('|')]
        report(root, fields, number, readiness)
    tickets(root, document, readiness)
    reconciled(root, readiness)


def report(root, fields, number, readiness):
    path = root / fields[4]
    if number == 30 and readiness and not path.exists():
        return
    require(path.is_file(), f'missing configuration Slice {number} report')
    text = path.read_text()
    require(re.findall(r'^## (.+)$', text, re.M) == SECTIONS, f'Slice {number}: incomplete report')
    require(f'Branch: `{fields[2]}`' in text and 'Delivery set: rev-configuration-ui' in text, 'report identity mismatch')
    states = ['reviewing', 'delivered'] if number == 30 and readiness else ['delivered']
    require(any(f'Status: {state}\n' in text for state in states), f'Slice {number}: report not ready')
    if number != 30 or not readiness:
        ancestor(root, integrated(text))
    if number != 30 or not readiness or 'PR: pending' not in text:
        require(re.search(r'^PR: \[#\d+\]\(https://forge\.adrianpk\.com/adrianpk/rev/pulls/\d+\)$', text, re.M), f'Slice {number}: missing canonical PR')


def prerequisites(root, cases):
    document = manifest(root)
    delivery(root, document, readiness=True)
    references(document['cases'], cases)
    print('Configuration/UI prerequisites: 30 requirements, named native cases, eight PTY cases, seven performance workloads and 30 slice identities resolved; no gate result claimed.')


def terminal(text):
    names = re.findall(r'^PASS configuration-ui ([a-z-]+)$', text, re.M)
    require(names == PTY_CASES, 'missing, reordered or duplicate configuration PTY case')
    require('Configuration/UI terminal suite: 8 sessions passed, 0 skipped.' in text, 'missing full configuration PTY summary')
    require(not re.search(r'\b(?:SKIP|FAIL)\b', text), 'configuration PTY skip/failure')


def performance(text):
    start = text.find('{\n')
    require(start >= 0, 'missing configuration performance records')
    result, _ = json.JSONDecoder().raw_decode(text[start:])
    require([row['workload'] for row in result['native']] == PERFORMANCE, 'missing configuration workload')
    require(result['hardware']['profile'] == 'release', 'configuration performance profile changed')
    require(result['budgets_us'] == BUDGET_US and result['peak_rss_limit_kib'] == 131072 and result['filter_growth_limit_kib'] == 4096, 'configuration performance budgets changed')
    fixture = result['fixture_identity']
    require(0 < fixture['entries'] <= 6400 and re.fullmatch(r'[a-f0-9]{64}', fixture['sha256']), 'missing configuration fixture identity')
    graph = result['fixtures']['graph']
    require(graph == {'sources': 64, 'edges': 16, 'total_bytes': 2097152, 'max_source_bytes': 262144}, 'missing admitted graph bounds')
    syntax = next(row for row in result['native'] if row['workload'] == 'syntax')
    require(syntax['admitted_callbacks'] > 0 and syntax['oversize_callbacks'] == syntax['unchanged_callbacks'] == 0, 'missing syntax work control')
    for kind, names in EXPECTED.items():
        selected = [row for row in result['native'] if row['workload'] in names]
        records('\n'.join(json.dumps(row) for row in selected), kind)
        proof = result['observation'][kind]
        require(proof['recursive_events'] == 0, 'recursive performance observation')
        if kind == 'graph':
            require(proof['work_open_access_events'] >= 64, 'missing graph read control')
        else:
            require(proof['setup_open_access_events'] > 0 and proof['work_open_access_events'] == 0, 'missing cached read control')
    terminal = result['terminal']
    require(0 < terminal['startup_ms'] <= 2000 and terminal['restored_reaped_closed'] is True, 'missing configured PTY lifetime')
    for stage in ['editing_idle', 'prompt_idle']:
        idle = terminal[stage]
        rate = result['clock_ticks_per_second']
        require(rate > 0, 'missing idle clock rate')
        budget = max(2, math.ceil(idle['elapsed_ms'] / 1000 * rate * 0.1)) * 1000 / rate
        require(idle['elapsed_ms'] >= 350 and idle['output_bytes'] == 0 and 0 <= idle['cpu_ms'] <= budget, 'missing configured idle evidence')
    require(terminal['idle_open_access_events'] == 0 and 0 < terminal['peak_rss_kib'] <= 131072, 'configured idle/memory guard failed')
    require('Configuration/UI performance suite: 7 native workloads, 1 PTY session passed, 0 skipped.' in text, 'missing configuration performance completion')


def validate(root, logs, cases):
    document = manifest(root)
    delivery(root, document, readiness=False)
    references(document['cases'], cases)
    reference, pty, measured = (logs[command] for command in CONFIGURATION_COMMANDS)
    require('Configuration reference: 59 schema/catalog entries, 25 active RGB roles, 111 default bindings and three optional examples agree; links valid.' in reference, 'missing configuration catalog evidence')
    terminal(pty)
    performance(measured)
    print('Configuration/UI acceptance: CFG01-CFG22, R01/R17/R20/R22 and UI01-UI04 passed on the same clean HEAD; zero skips.')


def discover(root):
    from acceptance import catalog
    result = subprocess.run(['cargo', 'test', '--all', '--', '--list'], cwd=root, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
    prerequisites(root, catalog(result.stdout, listing=True))
