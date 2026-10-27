"""Reconciled configuration contracts and ticket prerequisites for gate ownership."""

from datetime import datetime
from pathlib import Path
import re
import subprocess

from acceptance import require
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
