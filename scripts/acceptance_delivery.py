"""Check the approved delivery map, source scope and report prerequisites."""

import re
import subprocess
import tomllib

from acceptance import BASELINE_COMMANDS as COMMANDS, digest, require


PLAN = "ops/default/plan/rev-editor.md"
TRACKER = "ops/default/tracker/rev-editor.md"
SECTIONS = ["Purpose", "Delivered Behavior", "Implementation Notes", "Contracts Added or Changed",
            "Files of Interest", "Validation", "Risks and Follow-ups"]


def maps(root, document):
    plan = (root / PLAN).read_text()
    tracker = (root / TRACKER).read_text()
    tables = []
    for text in [plan, tracker]:
        rows = re.findall(r"^\| Slice \d+ \|.*$", text, re.M)
        require(digest("\n".join(rows)) == document["canonical_map_sha256"], "approved slice identities changed")
        tables.append(rows)
    require(tables[0] == tables[1] and len(tables[0]) == 48, "plan/tracker map mismatch")
    prefix = plan.split("## Validation contract\n", 1)[1].split("```sh\n", 1)[1].split("```", 1)[0].strip().splitlines()
    require(prefix == COMMANDS[:4], "required command prefix changed")
    require(COMMANDS[4:] == ["python3 scripts/check-pty.py --suite acceptance",
                           "python3 scripts/check-performance.py", "python3 scripts/check-acceptance.py",
                           "git diff --check"], "frozen gate suffix changed")
    return tables[0], tracker


def scope(root):
    package = tomllib.loads((root / "Cargo.toml").read_text())
    expected = {"crossterm": "=0.29.0", "ropey": "=1.6.1", "unicode-segmentation": "=1.13.3",
                "unicode-width": "=0.2.2", "tree-sitter": "=0.27.1", "tree-sitter-rust": "=0.24.2",
                "toml": "=1.1.8", "serde": "=1.0.229"}
    dependencies = {name: value if isinstance(value, str) else value["version"]
                    for name, value in package["dependencies"].items()}
    require(dependencies == expected and not package.get("dev-dependencies") and not package.get("build-dependencies"), "approved direct dependency scope changed")
    require(not package.get("features"), "unapproved optional runtime scope")
    require(package["package"]["name"] == "rev" and package["package"]["edition"] == "2024" and package["package"]["publish"] is False, "native product boundary changed")
    require((root / "Cargo.lock").is_file(), "missing dependency lock")
    wrapper = (root / "scripts/check-all.sh").read_text()
    require(wrapper.splitlines() == ["#!/bin/sh", "set -eu", 'cd "$(dirname "$0")/.."',
                                    "exec python3 scripts/check-acceptance.py --gate"], "full gate entrypoint changed")
    for name in ["check-source.py", "check-pty.py", "check-performance.py", "check-acceptance.py"]:
        require((root / "scripts" / name).is_file(), f"missing required gate script {name}")


def integrated(text):
    match = re.search(r"^Integrated: (?:Rebase \+ Fast-forward into dev at )?`([a-f0-9]{40})`\.?$", text, re.M)
    require(match, "missing integrated head")
    return match[1]


def report(root, path, branch, number, current):
    require(path.is_file(), f"missing Slice {number} report")
    text = path.read_text()
    require(re.findall(r"^## (.+)$", text, re.M) == SECTIONS, f"Slice {number}: report sections changed")
    require(f"Branch: `{branch}`" in text, f"Slice {number}: report branch changed")
    require("Delivery set: rev-editor" in text, f"Slice {number}: report delivery changed")
    states = ["reviewing", "delivered"] if current else ["delivered"]
    require(any(f"Status: {state}\n" in text for state in states), f"Slice {number}: report is not ready")
    require(re.search(r"^PR: `#\d+`$", text, re.M), f"Slice {number}: missing canonical PR reference")
    if not current:
        subprocess.run(["git", "merge-base", "--is-ancestor", integrated(text), "HEAD"], cwd=root, check=True)


def delivery(root, document, readiness=False):
    rows, tracker = maps(root, document)
    scope(root)
    for number, line in enumerate(rows, 1):
        fields = [part.strip().strip("`") for part in line.strip("|").split("|")]
        section = tracker.split(f"### Slice {number}:", 1)[1].split("\n### ", 1)[0].split("\n## Integrated closure", 1)[0]
        completed = section.count("- [x]")
        pending = section.count("- [ ]")
        require((completed, pending) == (3, 0) or (number == 48 and (completed, pending) == (0, 3)), f"Slice {number}: inconsistent task state")
        path = root / fields[4]
        if number == 48 and readiness and (not path.exists() or "PR: pending" in path.read_text()):
            continue
        report(root, path, fields[2], number, number == 48)
    require("Active slice: Slice 48 - Verify the complete integrated editor contract" in tracker, "active delivery identity changed")
