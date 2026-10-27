#!/usr/bin/env python3
"""Run the approved configuration/UI journeys through owned terminals."""

import argparse
from pathlib import Path
import subprocess
import tempfile

from configuration_ui.journeys import configured, fallback
from configuration_ui.support import ROOT


def terminal(selected):
    for name, scenario in [('configured', configured), ('missing', fallback), ('rejected', fallback)]:
        if selected is not None and selected != name:
            continue
        variants = [True, False] if name == 'configured' else [name == 'rejected']
        for variant in variants:
            run_scenario(scenario, variant)
    print('Configuration/UI terminal journeys passed, 0 skipped.')


def run_scenario(scenario, variant):
    with tempfile.TemporaryDirectory(prefix='rev-configuration-ui-') as directory:
        scenario(Path(directory), variant)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=['terminal'], required=True)
    parser.add_argument('--case', choices=['configured', 'missing', 'rejected'])
    args = parser.parse_args()
    subprocess.run(['cargo', 'build', '--bin', 'rev'], cwd=ROOT, check=True)
    terminal(args.case)


if __name__ == '__main__':
    main()
