"""Reconciled configuration contracts and ticket prerequisites for gate ownership."""

from datetime import datetime
from pathlib import Path
import re
import subprocess

from acceptance import require

TICKET_IDS = {f'TKT-202610271037{i:02}' for i in range(8, 16)} | {
    'TKT-20261027084156', 'TKT-20261027085102', 'TKT-20261027085234'}


def ancestor(root, head):
    result = subprocess.run(['git', 'merge-base', '--is-ancestor', head, 'HEAD'], cwd=root,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    require(result.returncode == 0, f'unintegrated delivery head {head}')


def reconciled(root):
    base = (root / 'ops/default/spec/rev-editor.md').read_text()
    delta = (root / 'ops/default/spec/editor-configuration.md').read_text()
    for identity in ['R01', 'R17', 'R20', 'R22']:
        pattern = rf'^### Requirement: {identity} .*?(?=^### Requirement: |^## |\Z)'
        old = re.search(pattern, base, re.M | re.S)
        new = re.search(pattern, delta, re.M | re.S)
        require(old and new and old[0].strip() == new[0].strip(), f'incomplete {identity} baseline reconciliation')
    future = root / 'ops/default/ticket/open/20261027092143-convert-tabs-and-spaces.md'
    require(future.is_file() and 'status: open\n' in future.read_text(), 'future conversion ticket must remain open')


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
