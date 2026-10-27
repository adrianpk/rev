"""Reject incomplete or stale acceptance evidence without rerunning editor journeys."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

from acceptance import COMMANDS, EVIDENCE_COMMANDS, catalog, digest, execution, manifest, native, references, source, terminal
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
            for index, command in enumerate(EVIDENCE_COMMANDS)
        ]}
        with tempfile.TemporaryDirectory(prefix="rev-acceptance-test-") as temporary:
            directory = Path(temporary)
            for check in run["checks"]:
                (directory / check["log"]).write_text(NATIVE)
            (directory / "run.json").write_text(json.dumps(run))
            self.assertEqual(execution(directory, current), dict.fromkeys(EVIDENCE_COMMANDS, NATIVE))
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


class ConfigurationAcceptanceTests(unittest.TestCase):
    # Eight actual named terminal completions are required; a smaller selection
    # or a copied footer cannot establish the complete configuration journey set.
    def test_config_terminal(self):
        from configuration_ui.evidence import PTY_CASES, terminal
        good = ''.join(f'PASS configuration-ui {name}\n' for name in PTY_CASES)
        good += 'Configuration/UI terminal suite: 8 sessions passed, 0 skipped.\n'
        terminal(good)
        for name, bad in {
            'missing': good.replace('PASS configuration-ui unwind\n', ''),
            'duplicate': good.replace('PASS configuration-ui unwind', 'PASS configuration-ui io'),
            'skipped': good + 'SKIP unavailable PTY\n',
            'focused': good.replace('8 sessions', '1 sessions'),
        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                terminal(bad)

    # Colored prefix frames must not satisfy a command's retained status marker
    # before the final key dispatches and publishes the requested new geometry.
    def test_colored_prefix(self):
        import runpy
        from types import SimpleNamespace
        root = Path(__file__).resolve().parent.parent
        harness = runpy.run_path(str(root / 'scripts/check-pty.py'))
        prefix = harness['HIDE'] + b'\x1b[1;1Hmain.rs -- RW\x1b[24;1H\x1b[38;2;192;202;245mC-x\x1b[0m' + harness['SHOW']
        session = SimpleNamespace(output=bytearray(prefix), height=24)
        self.assertFalse(harness['journey_ready'](session, b'main.rs -- RW', None))
        session.output = bytearray(prefix.replace(b'C-x', b'Quit'))
        self.assertTrue(harness['journey_ready'](session, b'main.rs -- RW', None))

    # The exact requirement set and real native/terminal/workload references
    # must survive independent map edits, not merely a count of passing commands.
    def test_config_manifest(self):
        from configuration_ui.evidence import MAP, manifest
        root = Path(__file__).resolve().parent.parent
        document = manifest(root)
        with tempfile.TemporaryDirectory(prefix='rev-config-map-') as directory:
            fixture = Path(directory)
            (fixture / 'tests').symlink_to(root / 'tests', target_is_directory=True)
            path = fixture / MAP
            path.parent.mkdir(parents=True)
            mutations = ['missing', 'duplicate', 'native', 'terminal', 'performance']
            for name in mutations:
                bad = copy.deepcopy(document)
                change_manifest(bad, name)
                path.write_text(json.dumps(bad))
                with self.subTest(name=name), self.assertRaises(ValueError):
                    manifest(fixture)

    # Frozen slice identity and required report metadata reject missing or
    # altered artifacts before a prerequisite pass can be mistaken for delivery.
    def test_config_reports(self):
        from configuration_ui.evidence import PLAN, TRACKER, delivery, manifest
        root = Path(__file__).resolve().parent.parent
        document = manifest(root)
        delivery(root, document, readiness=True)
        row = next(line for line in (root / PLAN).read_text().splitlines() if line.startswith('| Slice 1 |'))
        fields = [part.strip().strip('`') for part in row.strip('|').split('|')]
        with tempfile.TemporaryDirectory(prefix='rev-config-report-') as directory:
            fixture = Path(directory)
            for name in [PLAN, TRACKER]:
                path = fixture / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text((root / name).read_text())
            with self.assertRaisesRegex(ValueError, 'missing configuration Slice 1 report'):
                delivery(fixture, document, readiness=True)
            report = fixture / fields[4]
            report.parent.mkdir(parents=True)
            report.write_text((root / fields[4]).read_text().replace(fields[2], 'changed-branch'))
            with self.assertRaisesRegex(ValueError, 'report identity mismatch'):
                delivery(fixture, document, readiness=True)
            for name in [PLAN, TRACKER]:
                path = fixture / name
                path.write_text(path.read_text().replace(fields[2], 'changed-branch'))
            with self.assertRaisesRegex(ValueError, 'approved configuration slice identities changed'):
                delivery(fixture, document, readiness=True)

    # Successful process exit does not replace bounded samples, fixture identity,
    # positive read/parser controls, exact limits or event-driven idle evidence.
    def test_config_performance(self):
        from configuration_ui.evidence import performance
        good = performance_document()
        footer = '\nConfiguration/UI performance suite: 7 native workloads, 1 PTY session passed, 0 skipped.\n'
        performance(json.dumps(good, indent=2) + footer)
        for name in ['missing', 'samples', 'time', 'rss', 'fixture', 'reads', 'recursive', 'parser', 'idle']:
            bad = copy.deepcopy(good)
            change_performance(bad, name)
            with self.subTest(name=name), self.assertRaises((ValueError, AssertionError)):
                performance(json.dumps(bad, indent=2) + footer)


def change_manifest(document, name):
    if name == 'missing':
        document['cases'].pop()
    elif name == 'duplicate':
        document['cases'][0]['id'] = 'CFG02'
    elif name == 'native':
        document['cases'][0]['native'][0]['target'] = 'tests/absent.rs'
    elif name == 'terminal':
        document['cases'][0]['pty'] = ['unknown']
    else:
        document['cases'][0]['performance'] = ['unknown']


def performance_document():
    from configuration_ui.evidence import EXPECTED
    from configuration_ui.performance import BUDGET_US, SAMPLES
    names = [name for group in EXPECTED.values() for name in group]
    values = [{'workload': name, 'samples': SAMPLES[name], 'median_us': 10.0,
               'max_us': 11.0, 'peak_rss_kib': 8192} for name in names]
    values[-1].update(admitted_callbacks=1, oversize_callbacks=0, unchanged_callbacks=0)
    proof = {kind: {'setup_open_access_events': 1, 'work_open_access_events': 0,
                    'recursive_events': 0} for kind in EXPECTED}
    proof['graph']['work_open_access_events'] = 64
    idle = {'elapsed_ms': 350.0, 'output_bytes': 0, 'cpu_ms': 0.0}
    return {'hardware': {'profile': 'release'}, 'native': values, 'observation': proof,
            'budgets_us': BUDGET_US, 'peak_rss_limit_kib': 131072, 'filter_growth_limit_kib': 4096,
            'clock_ticks_per_second': 100,
            'fixtures': {'graph': {'sources': 64, 'edges': 16, 'total_bytes': 2097152, 'max_source_bytes': 262144}},
            'fixture_identity': {'entries': 6280, 'sha256': 'a' * 64},
            'terminal': {'startup_ms': 20, 'restored_reaped_closed': True, 'editing_idle': idle.copy(),
                         'prompt_idle': idle.copy(), 'idle_open_access_events': 0, 'peak_rss_kib': 8192}}


def change_performance(document, name):
    changes = {
        'samples': ('samples', 0), 'time': ('max_us', 2000000), 'rss': ('peak_rss_kib', 999999),
    }
    if name in changes:
        key, value = changes[name]
        document['native'][0][key] = value
    elif name == 'missing':
        document['native'].pop()
    elif name == 'fixture':
        document['fixture_identity']['sha256'] = ''
    elif name == 'reads':
        document['observation']['redraw']['work_open_access_events'] = 1
    elif name == 'recursive':
        document['observation']['redraw']['recursive_events'] = 1
    elif name == 'parser':
        document['native'][-1]['admitted_callbacks'] = 0
    else:
        document['terminal']['prompt_idle']['cpu_ms'] = 350


if __name__ == "__main__":
    unittest.main()
