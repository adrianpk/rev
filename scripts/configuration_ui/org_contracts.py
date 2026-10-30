"""Resolve the approved Org scenarios without claiming runtime execution."""
import re
from pathlib import Path
from acceptance import references, require
from .org_performance import NAMES
from .org_resource_terminal import STAGES

SPEC = 'ops/default/spec/editor-org-mode.md'
EVIDENCE = 'ops/default/spec/editor-org-mode-evidence.md'
TARGETS = ['org_source', 'org_editing', 'org_time', 'org_agenda', 'org_tables',
           'org_io', 'configuration', 'commands']


def scenarios(root):
    parts = re.split(r'^### Requirement: (ORG\d{2}) (.+)$', (root / SPEC).read_text(), flags=re.M)
    result = {parts[i]: re.findall(r'^#### Scenario: (.+)$', parts[i + 2], re.M)
              for i in range(1, len(parts), 3)}
    require(list(result) == [f'ORG{n:02}' for n in range(1, 32)], 'Org requirement identities changed')
    require(sum(map(len, result.values())) == 34, 'Org scenario obligations changed')
    return result


def native_path(case):
    target = Path(case['target'])
    names = case['test'].split('::')
    return target if len(names) == 1 else target.with_suffix('') / Path(*names[:-1]).with_suffix('.rs')


def rows(root, document, groups):
    expected = scenarios(root)
    declared = document['org_contracts']
    require([r['id'] for r in declared] == list(expected), 'missing/duplicate Org requirement map')
    flattened = []
    for row in declared:
        require([s['name'] for s in row['scenarios']] == expected[row['id']], 'missing/reordered Org scenario map')
        for scenario in row['scenarios']:
            check(root, scenario, groups)
            flattened.append(dict(id=row['id'] + ': ' + scenario['name'], **scenario))
    require(table(declared) in (root / EVIDENCE).read_text(), 'Org evidence table differs from declared map')
    return flattened


def check(root, scenario, groups):
    for key, allowed in [('pty', set(groups)), ('performance', set(NAMES) | {'idle:' + s for s in STAGES})]:
        values = scenario[key]
        require(values and len(values) == len(set(values)) and set(values) <= allowed,
                f'unknown/missing/duplicate Org {key} reference')
    native = scenario['native']
    require(native and len({(c['target'], c['test']) for c in native}) == len(native), 'missing/duplicate Org native reference')
    for case in native:
        require(case['target'] in {f'tests/{name}.rs' for name in TARGETS}, 'unknown Org native target')
        require(re.fullmatch('[a-z_][a-z_0-9:]*', case['test']), 'invalid Org native identity')
        require((root / case['target']).is_file() and (root / native_path(case)).is_file(), 'missing Org native source')


def table(declared):
    lines = ['| Requirement / scenario | Native assertion | Owned terminal | Resource observation |',
             '| --- | --- | --- | --- |']
    for row in declared:
        for scenario in row['scenarios']:
            tests = ', '.join(f"[`{c['test']}`](../../../{native_path(c).as_posix()})" for c in scenario['native'])
            terminal = ', '.join(f'[`{name}`](../report/slices/rev-org-mode/slice-61-combined-terminal.md#validation)' for name in scenario['pty'])
            resource = ', '.join(f'[`{name}`](../report/slices/rev-org-mode/slice-62-bounded-work.md#validation)' for name in scenario['performance'])
            lines.append(f"| {row['id']}: {scenario['name']} | {tests} | {terminal} | {resource} |")
    return '\n'.join(lines)


def prerequisites(root, document, groups, cases):
    mapped = rows(root, document, groups)
    references(mapped, cases)
    print('Org prerequisites: ORG01-ORG31, 34 scenarios, named native/terminal/resource references resolved; no runtime result claimed.')
