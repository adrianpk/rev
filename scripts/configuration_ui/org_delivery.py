"""Admit the approved Org delivery without substituting provenance for execution."""
import json
import re
import subprocess

from acceptance import COMMANDS, digest, require
from acceptance_delivery import SECTIONS, integrated

PLAN = 'ops/default/plan/rev-org-mode.md'
TRACKER = 'ops/default/tracker/rev-org-mode.md'
BASE = '887feeb6c8dfec2a616d7943ac6f2a298d133bbf'
MAP = '43a5aeba71c2673acd4984c6bd62d64f20e76f22eb7f3ee4d8b20e4535cbb0a2'
INTEGRATIONS = 'd68208554c48097f6565291c7d62d00941d53afeca6cc630c894ec86d3b57cb7'
FORGE = 'https://forge.adrianpk.com/adrianpk/rev/pulls/'
FINAL_PR = 165
GATE = [
    'cargo fmt --all -- --check',
    'cargo clippy --all-targets --all-features -- -D warnings',
    'cargo test --all',
    'python3 scripts/check-source.py',
    'python3 scripts/check-pty.py --suite acceptance',
    'python3 scripts/check-performance.py',
    'python3 scripts/check-config-reference.py',
    'python3 scripts/check-configuration-ui.py --suite terminal',
    'python3 scripts/check-configuration-ui.py --suite performance',
    'python3 scripts/check-acceptance.py',
    'git diff --check',
]


def maps(root, document):
    metadata = document['org_delivery']
    require(metadata['plan'] == PLAN and metadata['tracker'] == TRACKER, 'Org delivery paths changed')
    tables = [re.findall(r'^\| Slice \d+ \|.*$', (root / path).read_text(), re.M)
              for path in [PLAN, TRACKER]]
    require(len(tables[0]) == 64 and tables[0] == tables[1], 'Org plan/tracker map mismatch')
    require(digest('\n'.join(tables[0])) == metadata['canonical_map_sha256'] == MAP,
            'approved Org slice identities changed')
    require(metadata['final_pr'] == FINAL_PR, 'canonical final Org PR changed')
    prior = metadata['integrations']
    require(digest(json.dumps(prior, sort_keys=True, separators=(',', ':'))) == INTEGRATIONS,
            'verified Org integration/task identities changed')
    require([row['slice'] for row in prior] == list(range(1, 64)), 'missing/duplicate Org integration')
    return [[part.strip().strip('`') for part in line.strip('|').split('|')] for line in tables[0]]


def section(text, number):
    pieces = re.split(r'^#{2,3} Slice ' + str(number) + r':[^\n]*\n', text, flags=re.M)
    require(len(pieces) == 2, f'missing/duplicate Org Slice {number} task section')
    return re.split(r'^#{2,3} (?:Slice |Delivery closure)', pieces[1], maxsplit=1, flags=re.M)[0]


def tasks(text, number, current):
    body = section(text, number)
    found = re.findall(r'^- \[([ x])\] (T\d+\.\d+): (.+)$', body, re.M)
    require([name for _, name, _ in found] == [f'T{number}.{i}' for i in range(1, 4)],
            'Org task identities changed')
    require(all(state == 'x' for state, _, _ in found), 'Org task outcome incomplete')
    if not current:
        require('- [x] Record exact-head validation, canonical PR/report and verified dev integration.' in body,
                'Org integration outcome incomplete')
    return body


def report(root, fields, expected, current):
    number = int(fields[0].split()[1])
    path = root / fields[4]
    require(path.is_file(), f'missing Org Slice {number} report')
    text = path.read_text()
    headings = re.findall(r'^## (.+)$', text, re.M)
    require([name for name in headings if name in SECTIONS] == SECTIONS, 'incomplete Org report')
    require(f'Branch: `{fields[2]}`\n' in text and 'Delivery set: rev-org-mode\n' in text,
            'Org report identity mismatch')
    states = ['reviewing', 'delivered'] if current else ['delivered']
    require(any(f'Status: {state}\n' in text for state in states), 'Org report not ready')
    if current and expected is None:
        require('PR: pending\n' in text, 'unregistered final Org PR')
        return text
    require(expected is not None, 'final Org PR not registered')
    pr = expected['pr']
    formats = [f'PR: `#{pr}`', f'PR: [#{pr}]({FORGE}{pr})', f'PR: {FORGE}{pr}']
    require(any(line in text.splitlines() for line in formats), 'Org canonical PR mismatch')
    if 'head' in expected:
        require(f"`{expected['head']}`" in text, 'Org report integration mismatch')
    return text


def ancestry(root, previous, head, outcomes):
    commits = set(subprocess.check_output(['git', 'rev-list', previous + '..' + head],
                                         cwd=root, text=True).splitlines())
    require(head in commits and len(outcomes) == 3, 'Org integration range missing')
    require(all(task['head'] in commits for task in outcomes), 'Org task outside its integration')
    require(len({task['head'] for task in outcomes}) == 3, 'duplicate Org task commit')


def final(root, text, previous, fields, expected):
    head = integrated(text)
    log = subprocess.check_output(['git', 'log', '--format=%H\t%s', previous + '..' + head], cwd=root, text=True)
    outcomes = []
    for i in range(1, 4):
        matches = re.findall(r'^([a-f0-9]{40})\t.*\(T64\.' + str(i) + r'\)$', log, re.M)
        require(len(matches) == 1, 'missing/duplicate final Org task commit')
        outcomes.append({'head': matches[0]})
    ancestry(root, previous, head, outcomes)
    snapshot = subprocess.check_output(['git', 'show', head + ':' + fields[4]], cwd=root, text=True)
    require('Status: reviewing\n' in snapshot and f'Branch: `{fields[2]}`\n' in snapshot,
            'final Org integration is not the reviewed candidate')
    require(f"PR: [#{expected['pr']}]({FORGE}{expected['pr']})\n" in snapshot,
            'final Org integration lacks canonical PR reference')
    return head


def delivery(root, document, readiness):
    gate_contract(root)
    coherence(root, document)
    rows = maps(root, document)
    metadata = document['org_delivery']
    tracker = (root / TRACKER).read_text()
    ancestors = set(subprocess.check_output(['git', 'rev-list', 'HEAD'], cwd=root, text=True).splitlines())
    previous = BASE
    for number, fields in enumerate(rows, 1):
        current = number == 64 and readiness
        expected = metadata['integrations'][number - 1] if number < 64 else (
            {'pr': metadata['final_pr']} if metadata['final_pr'] is not None else None)
        text = report(root, fields, expected, current)
        tasks(tracker, number, current)
        if number < 64:
            require([task['id'] for task in expected['tasks']] == [f'T{number}.{i}' for i in range(1, 4)],
                    'Org task assignment changed')
            head = expected['head']
            ancestry(root, previous, head, expected['tasks'])
        elif current:
            continue
        else:
            require(expected is not None, 'final Org PR not registered')
            head = final(root, text, previous, fields, expected)
        require(head in ancestors, 'unintegrated Org delivery head')
        previous = head


def gate_contract(root):
    require(COMMANDS == GATE, 'frozen Org gate commands/order changed')
    wrapper = (root / 'scripts/check-all.sh').read_text()
    require(wrapper.splitlines() == ['#!/bin/sh', 'set -eu', 'cd "$(dirname "$0")/.."',
                                    'exec python3 scripts/check-acceptance.py --gate'],
            'frozen Org gate entrypoint changed')
    for path in [PLAN, TRACKER]:
        require('Full gate: `scripts/check-all.sh`\n' in (root / path).read_text(),
                'Org gate ownership changed')


def coherence(root, document):
    plan = (root / PLAN).read_text()
    tracker = (root / TRACKER).read_text()
    approved = re.findall(r'^- (T\d+\.\d+): (.+)$', plan, re.M)
    tracked = re.findall(r'^- \[[ x]\] (T\d+\.\d+): (.+)$', tracker, re.M)
    require(approved == tracked and len(approved) == 192, 'Org plan/tracker task contract mismatch')
    require(digest(json.dumps(approved, separators=(',', ':'))) ==
            'ae66e5da2a1751f5e36cba52650f99316df67bf1d9955eb346d84ef02b89e87b',
            'approved Org task descriptions changed')
    states = [re.search(r'^Status: (.+)$', text, re.M)[1] for text in [plan, tracker]]
    require(states[0] == states[1] and states[0] in ['Approved', 'Completed'], 'Org lifecycle mismatch')
    spec = (root / 'ops/default/spec/editor-org-mode.md').read_text()
    model = (root / 'ops/default/spec/editor-org-mode-model.md').read_text()
    require('Model: [Org Mode Model](editor-org-mode-model.md)\n' in spec and
            'Specification: [Org Mode](editor-org-mode.md)\n' in model, 'Org spec/model ownership mismatch')
    historical = document['org_delivery']['historical_closures']
    require(digest(json.dumps(historical, sort_keys=True, separators=(',', ':'))) ==
            '355f70003fa6f853f5360ab07b96ccd48034ecf602f5aebdda958522380b50a2',
            'historical closure identities changed')
    for row in historical:
        text = (root / row['path']).read_text()
        for key, value in row['fields'].items():
            require(f'{key}: `{value}`\n' in text, 'historical tested candidate/gate changed')
