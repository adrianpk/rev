"""Validate the editor's acceptance map and same-run command evidence."""

import ast
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess


BASELINE_COMMANDS = [
    "cargo fmt --all -- --check",
    "cargo clippy --all-targets --all-features -- -D warnings",
    "cargo test --all",
    "python3 scripts/check-source.py",
    "python3 scripts/check-pty.py --suite acceptance",
    "python3 scripts/check-performance.py",
    "python3 scripts/check-acceptance.py",
    "git diff --check",
]
CONFIGURATION_COMMANDS = [
    "python3 scripts/check-config-reference.py",
    "python3 scripts/check-configuration-ui.py --suite terminal",
    "python3 scripts/check-configuration-ui.py --suite performance",
]
COMMANDS = BASELINE_COMMANDS[:-2] + CONFIGURATION_COMMANDS + BASELINE_COMMANDS[-2:]
EVIDENCE_COMMANDS = COMMANDS[:-2]
JOURNEYS = {"acceptance_document", "acceptance_blocks", "acceptance_workspace", "acceptance_failure"}
WORKLOADS = {"load", "search", "insert", "delete", "scroll", "split_redraw", "highlighting", "syntax_work"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(root, *arguments):
    return subprocess.check_output(["git", *arguments], cwd=root, text=True).strip()


def identity(root):
    require(not git(root, "status", "--porcelain"), "acceptance requires a clean worktree")
    return {"head": git(root, "rev-parse", "HEAD"), "tree": git(root, "rev-parse", "HEAD^{tree}")}


def digest(text):
    return hashlib.sha256(text.encode()).hexdigest()


def manifest(root):
    document = json.loads((root / "scripts/acceptance.json").read_text())
    require(document["version"] == 1, "unknown acceptance map version")
    rows = document["cases"]
    require([row["id"] for row in rows] == [f"A{i:02}" for i in range(1, 21)], "A01-A20 must occur exactly once in order")
    spec = (root / "ops/default/spec/rev-editor.md").read_text()
    require(re.findall(r"^\| (A\d+) \|", spec, re.M) == [row["id"] for row in rows], "specification acceptance identities changed")
    for row in rows:
        require(row["condition"] and row["native"] and row["pty"], f"{row['id']}: missing required evidence mapping")
        pairs = [(case["target"], case["test"]) for case in row["native"]]
        require(len(set(pairs)) == len(pairs), f"{row['id']}: duplicate native reference")
        for target, name in pairs:
            require(target.startswith(("tests/", "examples/")) and ".." not in Path(target).parts and (root / target).is_file(), f"{row['id']}: missing native target {target}")
            require(re.fullmatch(r"[a-z_][a-z_0-9:]*", name), f"{row['id']}: invalid test name")
        require(set(row["pty"]) <= JOURNEYS and len(set(row["pty"])) == len(row["pty"]), f"{row['id']}: unknown or duplicate PTY journey")
        require(set(row["performance"]) <= WORKLOADS and len(set(row["performance"])) == len(row["performance"]), f"{row['id']}: unknown or duplicate workload")
    require({j for row in rows for j in row["pty"]} == JOURNEYS, "incomplete PTY map")
    require({w for row in rows for w in row["performance"]} == WORKLOADS, "incomplete performance map")
    return document


def catalog(text, listing=False):
    target = None
    cases = set()
    for line in text.splitlines():
        match = re.search(r"Running (?:unittests )?(\S+) \(", line)
        if match:
            target = match[1]
        pattern = r"(\S+): test$" if listing else r"test (\S+) \.\.\. ok$"
        match = re.fullmatch(pattern, line)
        if match:
            require(target is not None, "native case has no Cargo target")
            pair = (target, match[1])
            require(pair not in cases, f"duplicate native result: {pair}")
            cases.add(pair)
    require(cases, "native evidence contains no tests")
    return cases


def references(rows, cases):
    for row in rows:
        for case in row["native"]:
            pair = (case["target"], case["test"])
            require(pair in cases, f"{row['id']}: missing native case {pair}")


def native(text):
    results = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;", text)
    require(results and all(all(int(x) == 0 for x in row[1:]) for row in results), "native tests failed, skipped or filtered")
    require("... ignored" not in text and "... FAILED" not in text, "missing required native execution")
    cases = catalog(text)
    require(sum(int(row[0]) for row in results) == len(cases), "native summary and case results disagree")
    return cases


def source(text):
    metrics = re.findall(r"^[a-z_][a-z_0-9]*: cyclomatic=(\d+), nesting=(\d+)$", text, re.M)
    require(metrics and all(int(paths) <= 10 and int(nesting) <= 3 for paths, nesting in metrics), "missing or invalid source metrics")
    require("Source quality passed: safe Rust, parameters <= 5, nesting <= 3, cyclomatic <= 10." in text, "missing source-quality completion")


def terminal(text):
    names = re.findall(r"^PASS acceptance (document|blocks|workspace|failure):", text, re.M)
    require(len(names) == 4 and {"acceptance_" + name for name in names} == JOURNEYS, "missing or duplicate PTY journey")
    require("Acceptance PTY suite: 4 journeys passed, 0 skipped." in text, "missing complete PTY summary")
    require(not re.search(r"\b(?:SKIP|FAIL)\b", text), "PTY evidence contains a skipped or failed case")


def performance(text):
    start = text.find('{\n')
    require(start >= 0, "missing performance records")
    records, _ = json.JSONDecoder().raw_decode(text[start:])
    fixtures = records["fixtures"]
    require([f["lines"] for f in fixtures] == [20_000, 40_000], "missing representative fixture")
    require(records["hardware"]["profile"] == "release", "measurements must use release")
    for fixture in fixtures:
        require(fixture["bytes"] > fixture["lines"] and re.fullmatch(r"[a-f0-9]{64}", fixture["sha256"]), "missing fixture identity")
        work = fixture["native"]
        require(len(work) == len(WORKLOADS) and {w["workload"] for w in work} == WORKLOADS, "missing or duplicate measured workload")
        for record in work:
            require(record["samples"] > 0 and math.isfinite(record["median_us"]) and record["median_us"] > 0, "missing timing sample")
            require(math.isfinite(record["max_us"]) and record["max_us"] >= record["median_us"], "invalid timing range")
        syntax = next(record for record in work if record["workload"] == "syntax_work")
        require(syntax["cold_parse_us"] > 0 and syntax["cold_callbacks"] > 0, "missing cold parser reference")
        require(0 < syntax["max_incremental_callbacks"] < syntax["cold_callbacks"] / 4 and syntax["unchanged_callbacks"] == 0, "missing incremental or unchanged parser evidence")
        terminal = fixture["terminal"]
        require(terminal["startup_ms"] > 0 and terminal["idle"]["elapsed_ms"] >= 350 and terminal["idle"]["output_bytes"] == 0, "missing real PTY startup/idle evidence")
    require("Performance workloads: 2 fixtures, 8 native workloads each, 2 real PTY startup/idle/restoration checks; passed." in text, "missing performance completion")


def execution(directory, current):
    require(directory, "same-run gate evidence is required; use --prerequisites for readiness only")
    directory = Path(directory)
    run = json.loads((directory / "run.json").read_text())
    require(run["identity"] == current, "stale gate evidence: clean HEAD/tree changed")
    checks = run["checks"]
    require([c["command"] for c in checks] == EVIDENCE_COMMANDS, "missing or reordered gate checks")
    logs = []
    for index, check in enumerate(checks):
        require(check["passed"] is True and check["log"] == f"{index}.log", "failed or invalid gate record")
        log = (directory / check["log"]).read_text()
        require(digest(log) == check["sha256"], "changed gate log")
        logs.append(log)
    return dict(zip(EVIDENCE_COMMANDS, logs, strict=True))


def journeys(root):
    tree = ast.parse((root / "scripts/check-pty.py").read_text())
    function = next(f for f in tree.body if isinstance(f, ast.FunctionDef) and f.name == "acceptance")
    assignment = function.body[0]
    require(isinstance(assignment, ast.Assign) and isinstance(assignment.value, ast.List), "acceptance must enumerate its real journeys")
    names = [node.id for node in assignment.value.elts]
    require(len(names) == 4 and set(names) == JOURNEYS, "actual acceptance journeys changed")
    functions = {f.name for f in tree.body if isinstance(f, ast.FunctionDef)}
    require(JOURNEYS <= functions, "missing journey implementation")
