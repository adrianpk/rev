#!/usr/bin/env python3
"""Check A01-A20 prerequisites or validate the current complete gate's evidence."""

import argparse
import json
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

from acceptance import COMMANDS, catalog, digest, execution, identity, journeys, manifest, native, performance, references, source, terminal
from acceptance_delivery import delivery
from configuration_ui.evidence import prerequisites as configuration_prerequisites, validate as configuration_evidence


def gate(root):
    captured = identity(root)
    (root / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="acceptance-", dir=root / "target") as temporary:
        directory = Path(temporary)
        run = {"identity": captured, "checks": []}
        environment = dict(os.environ, REV_ACCEPTANCE_RUN=str(directory))
        for index, command in enumerate(COMMANDS):
            print(f"Gate {index + 1}/{len(COMMANDS)}: {command}", flush=True)
            path = directory / f"{index}.log"
            with path.open("w") as output:
                result = subprocess.run(shlex.split(command), cwd=root, env=environment,
                                        stdout=output, stderr=subprocess.STDOUT)
            text = path.read_text()
            print(text, end="", flush=True)
            result.check_returncode()
            run["checks"].append({"command": command, "passed": True,
                                  "log": path.name, "sha256": digest(text)})
            (directory / "run.json").write_text(json.dumps(run))
        if identity(root) != captured:
            raise ValueError("candidate changed during gate execution")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--gate", action="store_true")
    modes.add_argument("--prerequisites", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    if args.gate:
        gate(root)
        return
    document = manifest(root)
    journeys(root)
    delivery(root, document, readiness=args.prerequisites)
    if args.prerequisites:
        result = subprocess.run(["cargo", "test", "--all", "--", "--list"], cwd=root,
                                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
        cases = catalog(result.stdout, listing=True)
        references(document["cases"], cases)
        configuration_prerequisites(root, cases)
        print("Acceptance prerequisites: A01-A20 map, native cases, four journeys, 48 fixed identities and report state resolved; no gate result claimed.")
        return
    logs = execution(os.environ.get("REV_ACCEPTANCE_RUN"), identity(root))
    cases = native(logs["cargo test --all"])
    references(document["cases"], cases)
    source(logs["python3 scripts/check-source.py"])
    terminal(logs["python3 scripts/check-pty.py --suite acceptance"])
    performance(logs["python3 scripts/check-performance.py"])
    configuration_evidence(root, logs, cases)
    print(f"Acceptance evidence: A01-A20 passed on the same clean HEAD; {len(cases)} native tests, four PTY journeys, two performance fixtures; zero skips.")


if __name__ == "__main__":
    main()
