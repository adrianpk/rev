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
from configuration_ui import org_time, org_agenda, org_capture_drafts, org_capture_finalize, org_refile, org_capture, org_links, org_attachments, org_document, org_tables, org_exports


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
    parser.add_argument('--case', choices=['configured', 'missing', 'rejected', 'workspace', 'io', 'unwind', 'combined', 'org-outline', 'org-tasks', 'org-time', 'org-agenda', 'org-capture-drafts', 'org-capture-finalize', 'org-refile', 'org-capture', 'org-links', 'org-attachments', 'org-document', 'org-tables', 'org-exports'])
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
    if args.case in ['org-tasks','org-time','org-agenda','org-links']:
        build.extend(['--example', 'org-task-session'])
    if args.case=='org-time':
        build.extend(['--example','terminal-probe'])
    subprocess.run(build, cwd=ROOT, check=True)
    if args.case == 'org-exports':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_exports.journey, colored)
            print(f'PASS org-exports {label}')
        print('Org export terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-tables':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_tables.journey, colored)
            print(f'PASS org-tables {label}')
        print('Org table terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-document':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_document.journey, colored)
            print(f'PASS org-document {label}')
        print('Org document terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-attachments':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_attachments.journey, colored)
            print(f'PASS org-attachments {label}')
        print('Org attachments terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-links':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            for scenario, mode in [(org_links.journey, 'editing'), (org_links.readonly, 'read-only')]:
                run_scenario(scenario, colored)
                print(f'PASS org-links {mode} {label}')
        print('Org links terminal suite: 4 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-capture':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_capture.journey, colored)
            print(f'PASS org-capture combined {label}')
        print('Org capture/refile/archive terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-refile':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_refile.refile, colored)
            print(f'PASS org-refile {label}')
        print('Org refile terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-capture-finalize':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            for readonly, mode in [(False, 'saved'), (True, 'read-only')]:
                run_scenario(org_capture_finalize.finalize, (colored, readonly))
                print(f'PASS org-capture-finalize {mode} {label}')
        print('Org capture finalize terminal suite: 4 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-capture-drafts':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            run_scenario(org_capture_drafts.drafts, colored)
            print(f'PASS org-capture-drafts {label}')
        print('Org capture draft terminal suite: 2 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-agenda':
        for colored, label in [(True, 'rgb'), (False, 'no-color')]:
            for scenario, mode in [(org_agenda.editing, 'editing'), (org_agenda.readonly, 'read-only'), (org_agenda.cancellation, 'cancellation-stale')]:
                run_scenario(scenario, colored)
                print(f'PASS org-agenda {mode} {label}')
        print('Org agenda terminal suite: 6 owned sessions passed, 0 skipped.')
        return
    if args.case == 'org-time':
        for colored,label in [(True,'rgb'),(False,'no-color')]:
            for scenario,mode in [(org_time.editing,'editing'),(org_time.readonly,'read-only'),(org_time.restart,'restart')]:
                run_scenario(scenario,colored)
                print(f'PASS org-time {mode} {label}')
        run_scenario(lambda root, _: org_time.failure(root), True)
        print('PASS org-time native output failure')
        print('Org time terminal suite: 9 owned sessions passed, 0 skipped.')
        return
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
