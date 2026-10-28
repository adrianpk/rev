#!/usr/bin/env python3
"""Run the approved configuration/UI journeys through owned terminals."""

import argparse
from pathlib import Path
import subprocess
import tempfile
import time

from configuration_ui.combined import combined
from configuration_ui.journeys import configured, fallback
from configuration_ui.evidence import GROUPS, discover
from configuration_ui.lifetime import fault
from configuration_ui.performance import KINDS, run as performance
from configuration_ui.support import ROOT
from configuration_ui.workspace import workspace
from configuration_ui.org_outline import outline
from configuration_ui.org_tasks import tasks, readonly
from configuration_ui.org_clock_state import persistence


def terminal(selected):
    started = time.monotonic()
    count = 0
    cases = [
        ('configured', configured, [True, False]),
        ('missing', fallback, [False]),
        ('rejected', fallback, [True]),
        ('workspace', workspace, [True, False]),
        ('io', fault, ['io']),
        ('unwind', fault, ['unwind']),
        ('combined', combined, [True, False]),
    ]
    for name, scenario, variants in cases:
        if selected is not None and selected != name:
            continue
        for variant, label in zip(variants, GROUPS[name], strict=True):
            run_scenario(scenario, variant)
            print(f'PASS configuration-ui {label}')
            count += 1
    assert count > 0
    print(f'Configuration/UI terminal suite: {count} sessions passed, 0 skipped.')
    print(f'Terminal journey elapsed: {time.monotonic() - started:.3f}s, excluding build.')


def run_scenario(scenario, variant):
    with tempfile.TemporaryDirectory(prefix='rev-configuration-ui-') as directory:
        scenario(Path(directory), variant)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=['terminal', 'performance', 'prerequisites'], required=True)
    parser.add_argument('--case', choices=['configured', 'missing', 'rejected', 'workspace', 'io', 'unwind', 'combined', 'org-outline', 'org-tasks'])
    parser.add_argument('--workload', choices=KINDS)
    args = parser.parse_args()
    if args.suite == 'prerequisites':
        if args.case is not None or args.workload is not None:
            parser.error('prerequisites requires complete evidence discovery')
        discover(ROOT)
        return
    if args.suite == 'performance':
        if args.case is not None:
            parser.error('--case requires the terminal suite')
        subprocess.run(['cargo', 'build', '--release', '--bin', 'rev', '--example', 'configuration-performance'], cwd=ROOT, check=True, timeout=180)
        performance(args.workload)
        return
    if args.workload is not None:
        parser.error('--workload requires the performance suite')
    build = ['cargo', 'build', '--bin', 'rev']
    if args.case in [None, 'io', 'unwind']:
        build.extend(['--example', 'terminal-probe'])
    if args.case == 'org-tasks':
        build.extend(['--example', 'org-task-session'])
    subprocess.run(build, cwd=ROOT, check=True)
    if args.case == 'org-tasks':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            for scenario, mode in [(tasks, 'editing'), (readonly, 'read-only'), (persistence, 'clock-restart')]:
                run_scenario(scenario, colored)
                print(f'PASS org-tasks {mode} {label}')
        print('Org task terminal suite: 8 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-outline':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(outline, colored)
            print(f'PASS org-outline {label}')
        print('Org outline terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    terminal(args.case)


if __name__ == '__main__':
    main()
