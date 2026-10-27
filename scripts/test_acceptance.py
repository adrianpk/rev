"""Reject incomplete or stale acceptance evidence without rerunning editor journeys."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

from acceptance import COMMANDS, catalog, digest, execution, manifest, native, references, source, terminal
from acceptance_delivery import PLAN, TRACKER, integrated, maps


NATIVE = """Running tests/editing.rs (target/debug/deps/editing-fixture)
test direct_input ... ok
test bound_deletion ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
"""


class AcceptanceTests(unittest.TestCase):
    # A passing summary must not conceal an ignored, filtered, missing or
    # duplicate real test result. Cargo listing alone is readiness, not a pass.
    def test_native(self):
        self.assertEqual(len(native(NATIVE)), 2)
        cases = {
            "ignored": NATIVE.replace("bound_deletion ... ok", "bound_deletion ... ignored").replace("2 passed; 0 failed; 0 ignored", "1 passed; 0 failed; 1 ignored"),
            "filtered": NATIVE.replace("0 filtered out", "1 filtered out"),
            "missing": NATIVE.replace("test bound_deletion ... ok\n", ""),
            "duplicate": NATIVE.replace("bound_deletion ... ok", "direct_input ... ok"),
            "listing": NATIVE.replace("test bound_deletion ... ok", "bound_deletion: test").replace("test direct_input ... ok", "direct_input: test"),
        }
        for name, text in cases.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                native(text)

    # Renamed or removed tests cannot leave an apparently complete A-identity.
    def test_references(self):
        rows = [{"id": "A02", "native": [{"target": "tests/editing.rs", "test": "direct_input"}]}]
        references(rows, native(NATIVE))
        rows[0]["native"][0]["test"] = "removed_input"
        with self.assertRaisesRegex(ValueError, "A02: missing native"):
            references(rows, native(NATIVE))
        listing = NATIVE.replace("test direct_input ... ok", "direct_input: test").replace("test bound_deletion ... ok", "bound_deletion: test")
        self.assertEqual(catalog(listing, listing=True), native(NATIVE))

    # Successful process exit without structural measurements cannot stand in
    # for the mandatory source-quality check or its accepted limits.
    def test_source(self):
        text = "edit: cyclomatic=5, nesting=2\nSource quality passed: safe Rust, parameters <= 5, nesting <= 3, cyclomatic <= 10.\n"
        source(text)
        for name, bad in {
            "empty": "",
            "no metrics": text.split("\n", 1)[1],
            "complexity": text.replace("cyclomatic=5", "cyclomatic=11"),
            "nesting": text.replace("nesting=2", "nesting=4"),
            "completion": text.split("\n")[0],
        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                source(bad)

    # One missing/duplicated journey or a skip cannot be replaced by a printed
    # completion sentence from another run or a smaller selection.
    def test_terminal(self):
        text = "".join(f"PASS acceptance {name}: real journey\n" for name in ["document", "blocks", "workspace", "failure"])
        text += "Acceptance PTY suite: 4 journeys passed, 0 skipped.\n"
        terminal(text)
        for name, bad in {
            "missing": text.replace("PASS acceptance failure: real journey\n", ""),
            "duplicate": text.replace("PASS acceptance failure:", "PASS acceptance blocks:"),
            "skip": text + "SKIP unavailable terminal\n",
        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                terminal(bad)

    # Both delivered report formats carry the same full immutable hash;
    # abbreviated or missing integration records cannot satisfy readiness.
    def test_integration(self):
        head = "a" * 40
        for line in [f"Integrated: `{head}`", f"Integrated: Rebase + Fast-forward into dev at `{head}`."]:
            with self.subTest(line=line):
                self.assertEqual(integrated(line), head)
        for line in ["", "Integrated: `abcdef0`", f"Other: `{head}`", f"Integrated: `{head}a`"]:
            with self.subTest(line=line), self.assertRaises(ValueError):
                integrated(line)

    # Editing both plan tables in the same way must still fail the approved
    # identity pin; agreement between two changed documents is insufficient.
    def test_identity(self):
        root = Path(__file__).resolve().parent.parent
        document = manifest(root)
        plan = (root / PLAN).read_text()
        tracker = (root / TRACKER).read_text()
        with tempfile.TemporaryDirectory(prefix="rev-acceptance-map-") as temporary:
            fixture = Path(temporary)
            for path, text in [(PLAN, plan), (TRACKER, tracker)]:
                (fixture / path).parent.mkdir(parents=True, exist_ok=True)
                (fixture / path).write_text(text)
            maps(fixture, document)
            for path, text in [(PLAN, plan), (TRACKER, tracker)]:
                (fixture / path).write_text(text.replace("test/rev-48-acceptance", "test/rev-48-changed"))
            with self.assertRaisesRegex(ValueError, "identities changed"):
                maps(fixture, document)
            (fixture / PLAN).write_text(plan.replace(COMMANDS[0] + "\n" + COMMANDS[1], COMMANDS[1] + "\n" + COMMANDS[0]))
            (fixture / TRACKER).write_text(tracker)
            with self.assertRaisesRegex(ValueError, "command prefix changed"):
                maps(fixture, document)

    # The gate must reject evidence from another commit, changed logs, failed
    # commands and incomplete/reordered execution even when the logs look valid.
    def test_execution(self):
        current = {"head": "a" * 40, "tree": "b" * 40}
        run = {"identity": current, "checks": [
            {"command": command, "passed": True, "log": f"{index}.log", "sha256": digest(NATIVE)}
            for index, command in enumerate(COMMANDS[:6])
        ]}
        with tempfile.TemporaryDirectory(prefix="rev-acceptance-test-") as temporary:
            directory = Path(temporary)
            for check in run["checks"]:
                (directory / check["log"]).write_text(NATIVE)
            (directory / "run.json").write_text(json.dumps(run))
            self.assertEqual(execution(directory, current), [NATIVE] * 6)
            changes = {}
            bad = copy.deepcopy(run); bad["identity"]["head"] = "c" * 40; changes["stale"] = bad
            bad = copy.deepcopy(run); bad["checks"].pop(); changes["missing"] = bad
            bad = copy.deepcopy(run); bad["checks"].reverse(); changes["order"] = bad
            bad = copy.deepcopy(run); bad["checks"][0]["passed"] = False; changes["failed"] = bad
            bad = copy.deepcopy(run); bad["checks"][0]["sha256"] = "d" * 64; changes["changed"] = bad
            for name, bad in changes.items():
                (directory / "run.json").write_text(json.dumps(bad))
                with self.subTest(name=name), self.assertRaises(ValueError):
                    execution(directory, current)


if __name__ == "__main__":
    unittest.main()
