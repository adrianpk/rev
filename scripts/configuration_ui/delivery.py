"""Reconciled configuration contracts and ticket prerequisites for gate ownership."""

from datetime import datetime
from pathlib import Path
import re
import subprocess
import tomllib
import json

from acceptance import digest, require
from acceptance_delivery import SECTIONS, integrated

TICKET_IDS = {f'TKT-202610271037{i:02}' for i in range(8, 16)} | {
    'TKT-20261027084156', 'TKT-20261027085102', 'TKT-20261027085234'}


def ancestor(root, head):
    result = subprocess.run(['git', 'merge-base', '--is-ancestor', head, 'HEAD'], cwd=root,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    require(result.returncode == 0, f'unintegrated delivery head {head}')


def reconciled(root, readiness=False):
    base = (root / 'ops/default/spec/rev-editor.md').read_text()
    delta = (root / 'ops/default/spec/editor-configuration.md').read_text()
    for identity in ['R01', 'R17', 'R20', 'R22']:
        pattern = rf'^### Requirement: {identity} .*?(?=^### Requirement: |^## |\Z)'
        old = re.search(pattern, base, re.M | re.S)
        new = re.search(pattern, delta, re.M | re.S)
        require(old and new and old[0].strip() == new[0].strip(), f'incomplete {identity} baseline reconciliation')
    conversion(root, readiness)


def tickets(root, document, readiness):
    rows = document['tickets']
    require({'TKT-' + row['file'][:14] for row in rows} == TICKET_IDS and len(rows) == 11, 'approved ticket identities changed')
    for row in rows:
        paths = [root / 'ops/default/ticket' / state / row['file'] for state in ['solved', 'reviewing']]
        present = [path for path in paths if path.is_file()]
        require(len(present) == 1, f"missing/duplicate ticket {row['file']}")
        path = present[0]
        fields = dict(re.findall(r'^([a-z_]+): (.*)$', path.read_text().split('---', 2)[1], re.M))
        ticket(root, path, fields, row['slices'], readiness)


def ticket(root, path, fields, slices, readiness):
    require(fields['id'] == 'TKT-' + path.name[:14] and fields['status'] == path.parent.name, 'ticket path/identity mismatch')
    required = ['title', 'kind', 'severity', 'priority', 'scope', 'tags', 'source', 'reported_at', 'commits']
    require(all(key in fields for key in required), 'incomplete ticket metadata')
    require(fields['kind'] == 'task' and fields['scope'] in ['ui', 'domain', 'ops', 'docs'], 'ticket classification changed')
    datetime.fromisoformat(fields['reported_at'])
    reviewing = readiness and 30 in slices and path.parent.name == 'reviewing'
    require(reviewing or path.parent.name == 'solved', 'ticket outcome not integrated')
    require(reviewing or fields.get('resolution') == 'fixed', 'missing implemented resolution')
    commits = re.findall(r'[a-f0-9]{40}', fields['commits'])
    require(len(commits) == len(slices) - int(reviewing), 'missing assigned slice integration evidence')
    for head in commits:
        ancestor(root, head)


def conversion(root, readiness=False):
    """Keep the original captured ticket as verified conversion provenance."""
    name = '20261027092143-convert-tabs-and-spaces.md'
    paths = [root / 'ops/default/ticket' / state / name for state in ['open', 'ready', 'in_progress', 'reviewing', 'blocked', 'solved', 'closed']]
    present = [path for path in paths if path.is_file()]
    require(len(present) == 1, 'missing/duplicate conversion ticket outcome')
    path = present[0]
    fields = dict(re.findall(r'^([a-z_]+): (.*)$', path.read_text().split('---', 2)[1], re.M))
    require(fields.get('id') == 'TKT-20261027092143' and fields.get('reported_at') == '2026-10-27T09:21:43Z', 'conversion capture identity changed')
    require(fields.get('status') == path.parent.name, 'conversion ticket status/path mismatch')
    require(fields.get('kind') == 'task' and fields.get('scope') == 'domain', 'conversion ticket classification changed')
    reviewing = readiness and path.parent.name == 'reviewing'
    require(reviewing or path.parent.name == 'solved', 'conversion outcome not integrated')
    if not reviewing:
        require(fields.get('resolution') == 'fixed' and fields.get('closed_at'), 'missing conversion resolution')
        datetime.fromisoformat(fields['closed_at'])
    report = root / 'ops/default/report/slices/rev-input-navigation/slice-3-whitespace-conversion.md'
    require(report.is_file(), 'missing conversion slice report')
    text = report.read_text()
    require(re.findall(r'^## (.+)$', text, re.M) == SECTIONS, 'incomplete conversion report')
    branch = 'feat/rev-input-03-whitespace-conversion'
    require(fields.get('branch') == branch and f'Branch: `{branch}`' in text and 'Delivery set: rev-input-navigation\n' in text, 'conversion delivery identity changed')
    status = 'reviewing' if reviewing else 'delivered'
    require(f'Status: {status}\n' in text, 'conversion report status mismatch')
    pr = re.search(r'^PR: \[#(\d+)\]\((https://forge\.adrianpk\.com/adrianpk/rev/pulls/(\d+))\)$', text, re.M)
    require(pr and pr[1] == pr[3] and fields.get('pr') == pr[2], 'missing/mismatched canonical conversion PR')
    commits = re.findall(r'\b[a-f0-9]{40}\b', fields.get('commits', ''))
    require(len(commits) >= 3 and len(set(commits)) == len(commits), 'missing/duplicate conversion task evidence')
    for head in commits:
        ancestor(root, head)
    if not reviewing:
        ancestor(root, integrated(text))


NAVIGATION_PLAN = 'ops/default/plan/rev-input-navigation.md'
NAVIGATION_TRACKER = 'ops/default/tracker/rev-input-navigation.md'
NAVIGATION_MAP = 'd299928a1bbc47e6a20f620c57a86b1cbc316ab75fb81f00a09311325efdc2bb'
NAVIGATION_TICKETS = {'20261027190521': [1], '20261027191322': [2], '20261027092143': [3],
                      '20261027193500': list(range(4, 9)), '20261027194421': [9],
                      '20261027190629': list(range(10, 16)), '20261027193636': [16, 17]}
REQUIREMENTS = {'gutter': [f'GUT{i:02}' for i in [1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14]],
                'pointer-interaction': [f'PTR{i:02}' for i in range(1, 7)],
                'line-navigation': ['NAV01', 'NAV02'], 'whitespace-conversion': ['WSP01', 'WSP02'],
                'status-layout': ['R05', 'STA01', 'STA02'],
                'configuration-bootstrap': ['CFG01', 'CFG14', 'CFG15', 'CFG23', 'CFG24']}


def requirement(text, identity):
    pattern = rf'^### Requirement: {identity} .*?(?=^### Requirement: |^## |\Z)'
    blocks = re.findall(pattern, text, re.M | re.S)
    require(len(blocks) == 1, f'missing/duplicate requirement {identity}')
    return blocks[0].strip()


def navigation_contracts(root):
    base = (root / 'ops/default/spec/rev-editor.md').read_text()
    config = (root / 'ops/default/spec/editor-configuration.md').read_text()
    for name, identities in REQUIREMENTS.items():
        source = (root / f'ops/default/spec/editor-{name}.md').read_text()
        require(re.findall(r'^### Requirement: ([A-Z]+\d+) ', source, re.M) == identities, 'approved capability identities changed')
        for identity in identities:
            target = config if identity.startswith('CFG') else base
            require(requirement(source, identity) == requirement(target, identity), f'incomplete {identity} current reconciliation')
    navigation_model(root)
    historical_validation(root)


def historical_validation(root):
    for name in ['rev-editor', 'rev-editor-model']:
        text = (root / f'ops/default/spec/{name}.md').read_text()
        require('Validated candidate: `b56d705c7a84b823bf069fc874bc762938d6d897`\n' in text,
                'original editor validation provenance changed')
    for name in ['editor-configuration', 'editor-configuration-model']:
        text = (root / f'ops/default/spec/{name}.md').read_text()
        require('Validated candidate: `95713efb7f79fbc72417615a921841f8d49730ed`\n' in text
                and 'Gate validation: `FWDUCVRJCYBKFXEWEFLKPKA35E`\n' in text,
                'historical configuration validation provenance changed')


def navigation_model(root):
    sample = tomllib.loads((root / 'docs/reference/config.toml').read_text())
    text = (root / 'ops/default/spec/editor-configuration-model.md').read_text()
    for owner in ['gutter', 'mouse', 'palette.gutter']:
        defaults = sample
        for part in owner.split('.'):
            defaults = defaults[part]
        for leaf, value in defaults.items():
            key = owner + '.' + leaf
            rows = re.findall(r'^\| `' + re.escape(key) + r'` \| ([^|]+) \|', text, re.M)
            require(len(rows) == 1, f'missing/duplicate model leaf {key}')
            literal = re.search(r'`([^`]+)`', rows[0])
            require(literal is not None, f'missing model default {key}')
            actual = literal[1] if owner == 'palette.gutter' else tomllib.loads('value = ' + literal[1])['value']
            require(type(actual) is type(value) and actual == value, f'current model default drift {key}')


def navigation_maps(root, document):
    metadata = document['input_navigation']
    require(metadata['plan'] == NAVIGATION_PLAN and metadata['tracker'] == NAVIGATION_TRACKER,
            'current delivery paths changed')
    tables = [re.findall(r'^\| Slice \d+ \|.*$', (root / path).read_text(), re.M)
              for path in [NAVIGATION_PLAN, NAVIGATION_TRACKER]]
    require(len(tables[0]) == 20 and tables[0] == tables[1], 'current delivery slice map mismatch')
    require(digest('\n'.join(tables[0])) == metadata['canonical_map_sha256'] == NAVIGATION_MAP,
            'approved current slice identities changed')
    prior = metadata['integrations']
    require(digest(json.dumps(prior, sort_keys=True, separators=(',', ':'))) == '28f12cc8f167c1cb2919a3df0cc4e4f909052e802590173b20dd767d62ea4c62',
            'verified prior integration identities changed')
    require([row['slice'] for row in prior] == list(range(1, 20)), 'missing/duplicate prior integration')
    require(len({row['pr'] for row in prior}) == len({row['head'] for row in prior}) == 19,
            'duplicate prior integration identity')
    return [[part.strip().strip('`') for part in row.strip('|').split('|')] for row in tables[0]]


def navigation_report(root, fields, number, expected, readiness):
    path = root / fields[4]
    require(path.is_file(), f'missing current Slice {number} report')
    text = path.read_text()
    require(re.findall(r'^## (.+)$', text, re.M) == SECTIONS, 'incomplete current report')
    require(f'Branch: `{fields[2]}`' in text and 'Delivery set: rev-input-navigation\n' in text,
            'current report identity mismatch')
    current = number == 20 and readiness
    require('Status: delivered\n' in text or (current and 'Status: reviewing\n' in text), 'current report not ready')
    if current and 'PR: pending\n' in text:
        return text
    match = re.search(r'^PR: \[#(\d+)\]\(https://forge\.adrianpk\.com/adrianpk/rev/pulls/(\d+)\)$', text, re.M)
    require(match and match[1] == match[2], 'missing current canonical PR')
    if expected:
        require(int(match[1]) == expected['pr'] and integrated(text) == expected['head'], 'current integration mismatch')
    if not current:
        ancestor(root, integrated(text))
    return text


def navigation_tasks(text, number, current):
    section = text.split(f'### Slice {number}:', 1)[1].split('\n### ', 1)[0].split('\n## Delivery closure', 1)[0]
    tasks = re.findall(r'^- \[([ x])\] (T\d+\.\d+):', section, re.M)
    require([identity for _, identity in tasks] == [f'T{number}.{index}' for index in range(1, 4)],
            'current task identities changed')
    if not current:
        require(all(state == 'x' for state, _ in tasks) and '- [x] Record exact clean-head validation' in section,
                'current integrated task not complete')


def navigation_ticket(root, row, slices):
    states = ['open', 'ready', 'in_progress', 'reviewing', 'blocked', 'solved', 'closed']
    paths = [root / 'ops/default/ticket' / state / row['file'] for state in states]
    present = [path for path in paths if path.is_file()]
    require(len(present) == 1 and present[0].parent.name == 'solved', 'missing/duplicate/unsolved current ticket')
    path = present[0]
    text = path.read_text(); header = text.split('---', 2)[1]
    fields = dict(re.findall(r'^([a-z_]+):[ \t]*(.*)$', header, re.M))
    require(fields['id'] == 'TKT-' + path.name[:14] and fields['status'] == 'solved'
            and fields['resolution'] == 'fixed', 'current ticket outcome mismatch')
    require(all(name in fields for name in ['title', 'kind', 'severity', 'priority', 'scope', 'tags', 'source', 'reported_at', 'closed_at', 'commits']), 'incomplete current ticket')
    require(fields['kind'] in ['bug', 'task'] and fields['scope'] in ['ui', 'domain'], 'current ticket classification changed')
    require(fields['reported_at'] == datetime.strptime(path.name[:14], '%Y%m%d%H%M%S').isoformat() + 'Z', 'current ticket capture identity changed')
    datetime.fromisoformat(fields['reported_at']); datetime.fromisoformat(fields['closed_at'])
    final = slices[row['slices'][-1] - 1]
    require(fields['branch'] == final[2], 'current ticket delivery identity changed')
    report = (root / final[4]).read_text()
    require(f']({fields["pr"]})' in report, 'current ticket/report PR mismatch')
    declared = re.search(r'^commits:([^\n]*(?:\n[ \t]+[^\n]*)*)', header, re.M)
    heads = re.findall(r'\b[a-f0-9]{40}\b', declared[0])
    require(len(heads) >= 3 * len(row['slices']) and len(heads) == len(set(heads)), 'missing/duplicate current task commits')
    tasks = set()
    for head in heads:
        ancestor(root, head)
        subject = subprocess.check_output(['git', 'show', '-s', '--format=%s', head], cwd=root, text=True).strip()
        match = re.search(r'\((T\d+\.[123])\)$', subject)
        require(match is not None, 'current ticket commit lacks task identity')
        tasks.add(match[1])
    require({f'T{number}.{index}' for number in row['slices'] for index in range(1, 4)} <= tasks,
            'current ticket missing assigned task outcome')


def navigation(root, document, readiness):
    slices = navigation_maps(root, document)
    metadata = document['input_navigation']
    tracker = (root / NAVIGATION_TRACKER).read_text()
    for number, fields in enumerate(slices, 1):
        expected = metadata['integrations'][number - 1] if number < 20 else None
        navigation_report(root, fields, number, expected, readiness)
        navigation_tasks(tracker, number, number == 20 and readiness)
    rows = metadata['tickets']
    require({row['file'][:14]: row['slices'] for row in rows} == NAVIGATION_TICKETS
            and len(rows) == 7, 'current ticket assignment changed')
    for row in rows:
        navigation_ticket(root, row, slices)
    navigation_contracts(root)
