"""Reject incomplete or stale acceptance evidence without rerunning editor journeys."""

import copy
import json
import subprocess
import re
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
    # All prior cases plus each combined stage and owned cleanup are required;
    # missing work, reordered stages or a copied footer cannot establish delivery.
    def test_config_terminal(self):
        from configuration_ui.evidence import COMBINED_CASES, COMBINED_STAGES, PTY_CASES, ORG_GROUPS, ORG_COMBINED_STAGES, terminal
        good = ''.join(f'PASS configuration-ui {name}\n' for name in PTY_CASES if name not in COMBINED_CASES)
        for name in COMBINED_CASES:
            good += ''.join(f'PASS {name} {stage}\n' for stage in COMBINED_STAGES)
            good += f'PASS configuration-ui {name}\n'
        good += 'Configuration/UI terminal suite: 10 sessions passed, 0 skipped.\n'
        candidate = {'head': 'a' * 40, 'tree': 'b' * 40}
        for name in ORG_GROUPS:
            if name == 'org-combined':
                good += ''.join(f'PASS org-combined-{mode} {stage}\n' for mode in ['rgb', 'no-color']
                                for stage in ORG_COMBINED_STAGES)
            good += f'PASS org-workflows {name}\n'
        receipt = dict(candidate=candidate, cases=ORG_GROUPS, sessions=sum(ORG_GROUPS.values()),
                       elapsed_seconds=1.0, isolated_roots=True, restored_reaped_closed=True)
        good += 'Org terminal evidence: ' + json.dumps(receipt) + '\n'
        good += f'Org terminal suite: {sum(ORG_GROUPS.values())} owned sessions passed, 0 skipped.\n'
        terminal(good, candidate)
        for name, bad in {
            'missing': good.replace('PASS configuration-ui unwind\n', ''),
            'duplicate': good.replace('PASS configuration-ui unwind', 'PASS configuration-ui io'),
            'skipped': good + 'SKIP unavailable PTY\n',
            'focused': good.replace('10 sessions', '1 sessions'),
            'prior only': good[:good.index('PASS combined-rgb')] + 'Configuration/UI terminal suite: 10 sessions passed, 0 skipped.\n',
            'old summary': good.replace('10 sessions', '8 sessions'),
            'combined missing': good.replace('PASS configuration-ui combined-no-color\n', ''),
            'combined duplicated': good.replace('configuration-ui combined-no-color', 'configuration-ui combined-rgb'),
            'stage missing': good.replace('PASS combined-rgb selection\n', ''),
            'cleanup missing': good.replace('PASS combined-no-color restoration\n', ''),
            'stage duplicated': good.replace('combined-rgb selection', 'combined-rgb navigation'),
            'stage order': good.replace('PASS combined-rgb workspace\nPASS combined-rgb selection', 'PASS combined-rgb selection\nPASS combined-rgb workspace'),
            'failed': good + 'FAIL unavailable stage\n',
            'Org missing': good.replace('PASS org-workflows org-outline\n', ''),
            'Org duplicate': good.replace('PASS org-workflows org-outline', 'PASS org-workflows org-tasks'),
            'Org order': good.replace('PASS org-workflows org-feeds\nPASS org-workflows org-execution', 'PASS org-workflows org-execution\nPASS org-workflows org-feeds'),
            'Org stage missing': good.replace('PASS org-combined-rgb capture-refile\n', ''),
            'Org cleanup missing': good.replace('PASS org-combined-no-color restoration\n', ''),
            'Org stage duplicate': good.replace('org-combined-rgb clocks', 'org-combined-rgb habits'),
            'Org count': good.replace('"org-agenda": 6', '"org-agenda": 5'),
            'Org stale head': good.replace('"head": "' + 'a' * 40, '"head": "' + 'c' * 40),
            'Org stale tree': good.replace('"tree": "' + 'b' * 40, '"tree": "' + 'c' * 40),
            'Org roots': good.replace('"isolated_roots": true', '"isolated_roots": false'),
            'Org elapsed': good.replace('"elapsed_seconds": 1.0', '"elapsed_seconds": 0'),
            'Org lifetime': good.replace('"restored_reaped_closed": true', '"restored_reaped_closed": false'),
            'Org receipt duplicate': good + 'Org terminal evidence: ' + json.dumps(receipt) + '\n',

        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                terminal(bad, candidate)

    # Prior catalog totals and unbounded/missing compiled-template output cannot
    # establish the delivered bootstrap and gutter/mouse reference contract.
    def test_config_reference(self):
        from configuration_ui.evidence import reference
        good = ('Packaged template: 10013 exact reference bytes; copied native binary works without runtime sample files or HOME/XDG creation.\n'
                'Configuration reference: 178 schema/catalog entries, 54 active RGB roles, 263 default bindings and nine optional examples agree; links valid.\n')
        reference(good)
        for name, bad in {
            'prior catalog': good.replace('178 schema', '72 schema'),
            'prior palette': good.replace('54 active', '29 active'),
            'missing bindings': good.replace('263 default bindings', '111 default bindings'),
            'missing compiled': good.split('\n', 1)[1],
            'empty compiled': good.replace('10013 exact', '0 exact'),
            'oversize compiled': good.replace('10013 exact', '262145 exact'),
        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                reference(bad)

    # Buffered old-size autoscroll frames remain bounded, but cannot satisfy
    # shrink readiness; mixed geometry or regression after shrink must fail.
    def test_live_resize_frames(self):
        from configuration_ui.combined import resized_frames
        def frame(width, height):
            return (b'\x1b[?25l' + b''.join(f'\x1b[{row};1H'.encode() + b' ' * width
                    for row in range(1, height + 1)) + b'\x1b[?25h')
        old, new = frame(120, 24), frame(80, 12)
        self.assertFalse(resized_frames(old))
        self.assertFalse(resized_frames(old + new[:-1]))
        self.assertTrue(resized_frames(old + new))
        for name, bad in {
            'wrong width': old + frame(120, 12),
            'wrong height': old + frame(80, 13),
            'old after new': old + new + old,
            'invalid coordinate': new.replace(b'[12;1H', b'[13;1H'),
        }.items():
            with self.subTest(name=name), self.assertRaises(AssertionError):
                resized_frames(bad)

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
            mutations = ['missing', 'duplicate', 'native', 'terminal', 'performance', 'combined']
            for name in mutations:
                bad = copy.deepcopy(document)
                if name == 'combined':
                    next(row for row in bad['cases'] if row['id'] == 'CFG24')['pty'].pop()
                else:
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

    # The historical future ticket now requires the actual conversion outcome.
    # Readiness may inspect reviewing work; a full gate still requires delivery.
    def test_conversion_provenance(self):
        from configuration_ui.delivery import conversion
        with tempfile.TemporaryDirectory(prefix='rev-conversion-proof-') as directory:
            root = Path(directory)
            ticket, report = conversion_fixture(root)
            ticket_text, report_text = ticket.read_text(), report.read_text()
            conversion(root)
            cases = {
                'identity': (ticket_text.replace('TKT-20261027092143', 'TKT-20261027092144'), report_text),
                'status': (ticket_text.replace('status: solved', 'status: reviewing'), report_text),
                'resolution': (ticket_text.replace('resolution: fixed', 'resolution: moved'), report_text),
                'task evidence': (ticket_text.replace('commits:', 'missing_commits:'), report_text),
                'unknown task': (ticket_text.replace('commits: [', 'commits: [' + 'f' * 40 + ', '), report_text),
                'branch': (ticket_text, report_text.replace('feat/rev-input-03-whitespace-conversion', 'changed-branch')),
                'PR': (ticket_text, report_text.replace('pulls/1234', 'pulls/1235')),
                'report status': (ticket_text, report_text.replace('Status: delivered', 'Status: reviewing')),
                'integration': (ticket_text, report_text.replace('Integrated:', 'Missing integration:')),
                'unknown integration': (ticket_text, re.sub(r'Integrated: `[^`]+`', 'Integrated: `' + 'f' * 40 + '`', report_text)),
            }
            for name, (candidate_ticket, candidate_report) in cases.items():
                with self.subTest(name=name), self.assertRaises(ValueError):
                    ticket.write_text(candidate_ticket)
                    report.write_text(candidate_report)
                    conversion(root)
            ticket.write_text(ticket_text)
            report.write_text(report_text)
            report.unlink()
            with self.assertRaisesRegex(ValueError, 'missing conversion slice report'):
                conversion(root)
            report.write_text(report_text)
            duplicate = root / 'ops/default/ticket/reviewing' / ticket.name
            duplicate.parent.mkdir(parents=True)
            duplicate.write_text(ticket_text.replace('status: solved', 'status: reviewing'))
            with self.assertRaisesRegex(ValueError, 'duplicate conversion ticket'):
                conversion(root, readiness=True)
            ticket.unlink()
            report.write_text(report_text.replace('Status: delivered', 'Status: reviewing'))
            conversion(root, readiness=True)
            with self.assertRaisesRegex(ValueError, 'conversion outcome not integrated'):
                conversion(root)

    # Successful process exit does not replace bounded samples, fixture identity,
    # positive read/parser controls, exact limits or event-driven idle evidence.
    def test_config_performance(self):
        from configuration_ui.evidence import performance
        good = performance_document()
        footer = '\nOrg performance suite: 6 native workloads, 1 PTY session passed, 0 skipped.\nConfiguration/UI performance suite: 13 native workloads, 4 PTY sessions passed, 0 skipped.\n'
        performance(json.dumps(good, indent=2) + footer, good['candidate'])
        for name in ['missing', 'samples', 'time', 'rss', 'fixture', 'reads', 'recursive', 'parser', 'idle']:
            bad = copy.deepcopy(good)
            change_performance(bad, name)
            with self.subTest(name=name), self.assertRaises((ValueError, AssertionError)):
                performance(json.dumps(bad, indent=2) + footer, good['candidate'])

    # A complete old workload set or copied footer cannot replace the new
    # bounded controls, exact candidate, release lifetime or one-time publication.
    def test_interaction_performance(self):
        from configuration_ui.evidence import performance
        good = performance_document()
        footer = '\nOrg performance suite: 6 native workloads, 1 PTY session passed, 0 skipped.\nConfiguration/UI performance suite: 13 native workloads, 4 PTY sessions passed, 0 skipped.\n'
        indices = {row['workload']: index for index, row in enumerate(good['native'])}
        changes = [
            ('stale HEAD', ['candidate', 'head'], 'c' * 40),
            ('stale tree', ['candidate', 'tree'], 'c' * 40),
            ('changed ceiling', ['interaction_limits', 'native_cpu_ms'], 501),
            ('CPU growth', ['native', indices['gutter_absolute'], 'cpu_ticks'], 9999),
            ('wall growth', ['native', indices['drag_active'], 'elapsed_ms'], 1001),
            ('hidden rows', ['native', indices['gutter_hidden'], 'number_rows_per_frame'], 22),
            ('small buffer', ['native', indices['gutter_relative'], 'logical_lines'], 10),
            ('tick cadence', ['native', indices['drag_active'], 'tick_ms'], 99),
            ('tick step', ['native', indices['drag_active'], 'step_rows'], 4),
            ('released tick', ['native', indices['drag_idle'], 'ticks'], 1),
            ('idle deadline', ['native', indices['drag_idle'], 'deadlines'], 1),
            ('disabled redraw', ['native', indices['mouse_disabled'], 'redraws'], 1),
            ('disabled reports', ['native', indices['mouse_disabled'], 'reports'], 0),
            ('release tail', ['interaction', 'repeat', 0, 'release_ms'], 151),
            ('repeat CPU', ['interaction', 'repeat', 0, 'cpu_ms'], 201),
            ('repeat iterations', ['interaction', 'repeat', 1, 'reports'], 0),
            ('duplicate direction', ['interaction', 'repeat', 1, 'direction'], 'down'),
            ('early drag tick', ['interaction', 'drag_first_tick_ms'], 99),
            ('late drag tick', ['interaction', 'drag_first_tick_ms'], 251),
            ('released output', ['interaction', 'drag_idle', 'output_bytes'], 1),
            ('idle CPU', ['interaction', 'release_idle', 'cpu_ms'], 350),
            ('disabled output', ['interaction', 'disabled_idle', 'output_bytes'], 1),
            ('disabled feed CPU', ['interaction', 'disabled_feed', 'cpu_ms'], 201),
            ('disabled feed count', ['interaction', 'disabled_feed', 'reports'], 0),
            ('missing cleanup', ['interaction', 'restored_reaped_closed'], False),
            ('absent publication', ['bootstrap', 'starts', 0, 'publication_events'], 0),
            ('repeated publication', ['bootstrap', 'starts', 1, 'publication_events'], 1),
            ('repeated write', ['bootstrap', 'starts', 1, 'mutation_events'], 1),
            ('missing start', ['bootstrap', 'starts'], good['bootstrap']['starts'][:1]),
            ('stale template', ['bootstrap', 'template_sha256'], 'a' * 64),
        ]
        for name, path, value in changes:
            bad = copy.deepcopy(good)
            target = bad
            for part in path[:-1]:
                target = target[part]
            target[path[-1]] = value
            with self.subTest(name=name), self.assertRaises((ValueError, AssertionError)):
                performance(json.dumps(bad, indent=2) + footer, good['candidate'])
        missing = copy.deepcopy(good)
        missing['native'] = missing['native'][:7]
        for name, bad in {
            'old workloads': json.dumps(missing, indent=2) + footer,
            'skipped': json.dumps(good, indent=2) + footer + 'SKIP pointer\n',
            'failed': json.dumps(good, indent=2) + footer + 'FAIL startup\n',
            'old summary': json.dumps(good, indent=2) + footer.replace('4 PTY sessions', '1 PTY session'),
        }.items():
            with self.subTest(name=name), self.assertRaises(ValueError):
                performance(bad, good['candidate'])


    # Each resource dimension must fail closed independently. Matching HEAD/tree
    # and successful process exit cannot replace counts, bounds or owner cleanup.
    def test_org_performance(self):
        from configuration_ui.evidence import performance
        good = performance_document()
        footer = '\nOrg performance suite: 6 native workloads, 1 PTY session passed, 0 skipped.\nConfiguration/UI performance suite: 13 native workloads, 4 PTY sessions passed, 0 skipped.\n'
        performance(json.dumps(good, indent=2) + footer, good['candidate'])
        changes = [
            ('candidate', ['candidate', 'head'], 'c' * 40),
            ('tree', ['candidate', 'tree'], 'c' * 40),
            ('profile', ['profile'], 'debug'),
            ('ceiling', ['limits', 'peak_rss_kib'], 131072),
            ('fixture', ['fixture_identity', 'files'], 259),
            ('hash', ['fixture_identity', 'sha256'], 'unknown'),
            ('read control', ['observation', 'work_open_events'], 0),
            ('source', ['native', 0, 'headings'], 9999),
            ('chunk', ['native', 0, 'max_chunk_us'], 10001),
            ('agenda', ['native', 1, 'sources'], 255),
            ('habit', ['native', 2, 'days'], 31),
            ('formula', ['native', 3, 'last_value'], 9999),
            ('export', ['native', 4, 'backends'], 10),
            ('cancel', ['native', 5, 'max_us'], 150001),
            ('busy', ['native', 5, 'busy_rejected'], 15),
            ('thread', ['native', 5, 'threads_after'], 2),
            ('fd', ['native', 5, 'fds_after'], 5),
            ('child', ['native', 5, 'children_after'], 1),
            ('RSS', ['native', 0, 'peak_rss_kib'], 98305),
            ('growth', ['native', 0, 'retained_growth_kib'], 32769),
            ('missing idle', ['terminal', 'idle'], {}),
            ('polling', ['terminal', 'idle', 'relative', 'cpu_ms'], 100),
            ('output', ['terminal', 'idle', 'armed', 'output_bytes'], 1),
            ('IO', ['terminal', 'idle', 'inactive', 'open_access_events'], 1),
            ('timer', ['terminal', 'expiry_count'], 2),
            ('slow response', ['terminal', 'responses_ms', 0], 151),
            ('cleanup', ['terminal', 'restored_reaped_closed'], False),
        ]
        for name, path, replacement in changes:
            bad = copy.deepcopy(good)
            field = bad['org']
            for key in path[:-1]:
                field = field[key]
            field[path[-1]] = replacement
            with self.subTest(name=name), self.assertRaises(ValueError):
                performance(json.dumps(bad, indent=2) + footer, good['candidate'])
        for name in ['missing', 'duplicate', 'reordered']:
            bad = copy.deepcopy(good)
            rows = bad['org']['native']
            if name == 'missing':
                rows.pop()
            elif name == 'duplicate':
                rows.append(rows[-1])
            else:
                rows.reverse()
            with self.subTest(name=name), self.assertRaises(ValueError):
                performance(json.dumps(bad, indent=2) + footer, good['candidate'])
        with self.assertRaises(ValueError):
            performance(json.dumps(good, indent=2) + footer.split('Configuration/UI')[0], good['candidate'])

    # Current capability text must retain every approved scenario, precise startup
    # exception and typed model default without relabeling earlier gate evidence.
    def test_current_contracts(self):
        from configuration_ui.delivery import navigation_contracts, requirement
        root = Path(__file__).resolve().parent.parent
        paths = ['ops/default/spec/' + name + '.md' for name in [
            'rev-editor', 'rev-editor-model', 'editor-configuration', 'editor-configuration-model',
            'editor-gutter', 'editor-pointer-interaction', 'editor-line-navigation',
            'editor-whitespace-conversion', 'editor-status-layout', 'editor-configuration-bootstrap']]
        paths += ['docs/reference/config.toml']
        with tempfile.TemporaryDirectory(prefix='rev-current-contracts-') as directory:
            fixture = Path(directory); navigation_fixture(root, fixture, paths)
            navigation_contracts(fixture)
            changes = [
                ('missing gutter', 'rev-editor', '### Requirement: GUT13 ', '### Requirement: GUT99 '),
                ('partial status', 'rev-editor', '- **AND** the document and its dirty indication remain visible.', ''),
                ('line scenario', 'rev-editor', '#### Scenario: Visit an admitted logical line', '#### Scenario: Missing line coverage'),
                ('partial startup', 'editor-configuration', 'No existing destination entry SHALL be rewritten or replaced by provisioning.', ''),
                ('historical candidate', 'rev-editor-model', 'b56d705c7a84b823bf069fc874bc762938d6d897', 'a' * 40),
                ('historical gate', 'editor-configuration-model', 'FWDUCVRJCYBKFXEWEFLKPKA35E', 'different-gate'),
                ('model default', 'editor-configuration-model', '| `mouse.scroll_lines` | Integer, `3` |', '| `mouse.scroll_lines` | Integer, `4` |'),
                ('model palette', 'editor-configuration-model', '| `palette.gutter.background` | `#1a1b26` |', '| `palette.gutter.background` | `#000000` |'),
            ]
            for name, file, old, new in changes:
                path = fixture / ('ops/default/spec/' + file + '.md'); original = path.read_text()
                self.assertIn(old, original)
                path.write_text(original.replace(old, new))
                with self.subTest(name=name), self.assertRaises(ValueError):
                    navigation_contracts(fixture)
                path.write_text(original)
            path = fixture / 'ops/default/spec/rev-editor.md'; original = path.read_text()
            path.write_text(original + '\n' + requirement(original, 'PTR01') + '\n')
            with self.assertRaisesRegex(ValueError, 'missing/duplicate requirement'):
                navigation_contracts(fixture)

    # Agreement between edited tables or a reachable unrelated commit cannot
    # replace the approved current map and the nineteen verified PR identities.
    def test_current_delivery(self):
        from configuration_ui.delivery import NAVIGATION_PLAN, NAVIGATION_TRACKER, navigation_maps, navigation_report, navigation_tasks
        from configuration_ui.evidence import manifest
        root = Path(__file__).resolve().parent.parent; document = manifest(root)
        slices = navigation_maps(root, document)
        with tempfile.TemporaryDirectory(prefix='rev-current-delivery-') as directory:
            fixture = Path(directory)
            paths = [NAVIGATION_PLAN, NAVIGATION_TRACKER] + [row[4] for row in slices]
            navigation_fixture(root, fixture, paths)
            navigation_maps(fixture, document)
            for path in [NAVIGATION_PLAN, NAVIGATION_TRACKER]:
                target = fixture / path
                target.write_text(target.read_text().replace('test/rev-input-20-delivery-acceptance', 'test/rev-input-20-altered'))
            with self.assertRaisesRegex(ValueError, 'approved current slice identities'):
                navigation_maps(fixture, document)
            bad = copy.deepcopy(document); bad['input_navigation']['integrations'][0]['head'] = 'a' * 40
            with self.assertRaisesRegex(ValueError, 'prior integration identities'):
                navigation_maps(root, bad)
            fields = slices[3]; path = fixture / fields[4]; original = path.read_text()
            expected = document['input_navigation']['integrations'][3]
            navigation_report(fixture, fields, 4, expected, True)
            for name, text in {
                'branch': original.replace(fields[2], 'changed-branch'),
                'PR': original.replace('pulls/84', 'pulls/999'),
                'integration': original.replace(expected['head'], 'a' * 40),
                'state': original.replace('Status: delivered', 'Status: reviewing'),
                'sections': original.replace('## Validation', '## Missing validation'),
            }.items():
                path.write_text(text)
                with self.subTest(name=name), self.assertRaises(ValueError):
                    navigation_report(fixture, fields, 4, expected, True)
            path.write_text(original)
            current = slices[-1]
            current_path = fixture / current[4]
            current_path.write_text(current_path.read_text().replace('Status: delivered', 'Status: reviewing'))
            navigation_report(fixture, current, 20, None, True)
            with self.assertRaisesRegex(ValueError, 'current report not ready'):
                navigation_report(fixture, current, 20, None, False)
            tracker = (root / NAVIGATION_TRACKER).read_text()
            navigation_tasks(tracker, 1, False)
            for bad in [tracker.replace('[x] T1.1:', '[ ] T1.1:'), tracker.replace('T1.1:', 'T1.4:')]:
                with self.assertRaises(ValueError):
                    navigation_tasks(bad, 1, False)

    # Both inline and block commit arrays must prove the assigned task outcomes;
    # duplicate locations, stale commits or changing a solved ticket cannot pass.
    def test_current_tickets(self):
        from configuration_ui.delivery import navigation_maps, navigation_ticket
        from configuration_ui.evidence import manifest
        root = Path(__file__).resolve().parent.parent; document = manifest(root)
        slices = navigation_maps(root, document)
        rows = document['input_navigation']['tickets']
        with tempfile.TemporaryDirectory(prefix='rev-current-tickets-') as directory:
            fixture = Path(directory)
            paths = [row[4] for row in slices] + ['ops/default/ticket/solved/' + row['file'] for row in rows]
            navigation_fixture(root, fixture, paths)
            for row in rows:
                navigation_ticket(fixture, row, slices)
            row = next(row for row in rows if row['slices'] == [9])
            path = fixture / 'ops/default/ticket/solved' / row['file']; original = path.read_text()
            head = re.search(r'\b[a-f0-9]{40}\b', original)[0]
            for name, text in {
                'resolution': original.replace('resolution: fixed', 'resolution: moved'),
                'capture': original.replace('2026-10-27T19:44:21Z', '2026-10-27T19:44:22Z'),
                'branch': original.replace('feat/rev-input-09-status-layout', 'changed-branch'),
                'PR': original.replace('pulls/89', 'pulls/999'),
                'unknown commit': original.replace(head, 'f' * 40),
                'wrong task commit': original.replace(head, subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()),
                'missing commits': re.sub(r'^  - [a-f0-9]{40}\n', '', original, flags=re.M),
            }.items():
                path.write_text(text)
                with self.subTest(name=name), self.assertRaises(ValueError):
                    navigation_ticket(fixture, row, slices)
            path.write_text(original)
            duplicate = fixture / 'ops/default/ticket/reviewing' / path.name
            duplicate.parent.mkdir(parents=True); duplicate.write_text(original)
            with self.assertRaisesRegex(ValueError, 'missing/duplicate/unsolved'):
                navigation_ticket(fixture, row, slices)


def navigation_fixture(root, fixture, paths):
    for name in paths:
        path = fixture / name; path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes((root / name).read_bytes())
    # Only read-only ancestry/show commands use this live identity. No test
    # writes Git state or treats these document mutations as a gate candidate.
    (fixture / '.git').symlink_to(root / '.git', target_is_directory=(root / '.git').is_dir())


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
               'max_us': 11.0, 'peak_rss_kib': 8192, 'elapsed_ms': 10, 'cpu_ticks': 0} for name in names]
    next(row for row in values if row['workload'] == 'syntax').update(admitted_callbacks=1, oversize_callbacks=0, unchanged_callbacks=0)
    proof = {kind: {'setup_open_access_events': 1, 'work_open_access_events': 0,
                    'recursive_events': 0} for kind in EXPECTED}
    proof['graph']['work_open_access_events'] = 64
    idle = {'elapsed_ms': 350.0, 'output_bytes': 0, 'cpu_ms': 0.0}
    return {'hardware': {'profile': 'release'}, 'native': values, 'observation': proof,
            'org': org_performance_document(),
            **interaction_document(values),
            'budgets_us': BUDGET_US, 'peak_rss_limit_kib': 131072, 'filter_growth_limit_kib': 4096,
            'clock_ticks_per_second': 100,
            'fixtures': {'graph': {'sources': 64, 'edges': 16, 'total_bytes': 2097152, 'max_source_bytes': 262144}},
            'fixture_identity': {'entries': 6280, 'sha256': 'a' * 64},
            'terminal': {'startup_ms': 20, 'restored_reaped_closed': True, 'editing_idle': idle.copy(),
                         'prompt_idle': idle.copy(), 'idle_open_access_events': 0, 'peak_rss_kib': 8192}}


def org_performance_document():
    from configuration_ui.org_performance import NAMES, DETAILS, LIMITS
    from configuration_ui.org_resource_terminal import STAGES
    rows = [dict(workload=name, samples=3, median_us=10, max_us=11,
                 peak_rss_kib=8192, baseline_rss_kib=4096, retained_growth_kib=2048,
                 elapsed_ms=1, cpu_ticks=0, **detail) for name, detail in zip(NAMES, DETAILS)]
    rows[0]['max_chunk_us'] = 100
    rows[-1].update(samples=16, threads_before=1, threads_after=1,
                    fds_before=4, fds_after=4, children_after=0, busy_rejected=16, cancelled=16)
    owners = dict(threads=1, fds=6, children='')
    idle = dict(elapsed_ms=350, output_bytes=0, cpu_ms=0, open_access_events=0, **owners)
    return dict(candidate={'head': 'a' * 40, 'tree': 'b' * 40}, profile='release',
                elapsed_ms=4000, limits=LIMITS.copy(), native=rows,
                observation=dict(work_open_events=260, work_access_events=260),
                fixture_identity=dict(files=260, entries=264, sha256='a' * 64),
                terminal=dict(startup_ms=20, peak_rss_kib=8192, restored_reaped_closed=True,
                              expiry_count=1, baseline_owners=owners,
                              responses_ms=[1] * 11, idle={stage: idle.copy() for stage in STAGES}))


def interaction_document(values):
    from configuration_ui.performance import INTERACTION_LIMITS
    from configuration_ui.support import ROOT
    import hashlib
    rows = {row['workload']: row for row in values}
    for name in ['gutter_absolute', 'gutter_relative', 'gutter_hidden']:
        rows[name].update(logical_lines=100001, frames=64, viewport_rows=22,
                          number_rows_per_frame=0 if name == 'gutter_hidden' else 22)
    rows['drag_active'].update(iterations=64, ticks=64, tick_ms=100, step_rows=3, scrolled_rows=192)
    rows['drag_idle'].update(iterations=64, ticks=0, deadlines=0)
    rows['mouse_disabled'].update(iterations=64, reports=256, redraws=0, deadlines=0)
    quiet = {'elapsed_ms': 350, 'output_bytes': 0, 'cpu_ms': 0}
    lifetime = {'peak_rss_kib': 8192, 'restored_reaped_closed': True}
    start = {**lifetime, 'startup_ms': 5, 'cpu_ms': 0, 'idle': quiet.copy()}
    template = (ROOT / 'docs/reference/config.toml').read_bytes()
    return {'candidate': {'head': 'a' * 40, 'tree': 'b' * 40}, 'interaction_limits': INTERACTION_LIMITS,
            'interaction': {**lifetime, 'repeat': [
                {'direction': direction, 'reports': 30, 'rate_per_second': 120,
                 'first_ms': 1, 'release_ms': 1, 'elapsed_ms': 400, 'cpu_ms': 20}
                for direction in ['down', 'up']], 'release_idle': quiet.copy(),
                'drag_first_tick_ms': 105, 'drag_cpu_ms': 0, 'drag_idle': quiet.copy(),
                'disabled_idle': quiet.copy(), 'disabled_feed': {'reports': 256, 'rate_per_second': 120,
                                                               'elapsed_ms': 2101, 'cpu_ms': 0}},
            'bootstrap': {'starts': [{**start, 'publication_events': 1, 'mutation_events': 4},
                                     {**start, 'publication_events': 0, 'mutation_events': 0}],
                          'template_bytes': len(template), 'template_sha256': hashlib.sha256(template).hexdigest(),
                          'existing_preserved': True}}


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
        next(row for row in document['native'] if row['workload'] == 'syntax')['admitted_callbacks'] = 0
    else:
        document['terminal']['prompt_idle']['cpu_ms'] = 350


def conversion_fixture(root):
    from acceptance_delivery import SECTIONS
    def git(*arguments):
        return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', '-c', 'commit.gpgsign=false',
            '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.org', *arguments], cwd=root, text=True).strip()
    git('init', '--quiet', '--initial-branch=dev')
    commits = []
    for number in range(3):
        (root / 'source.txt').write_text(f'fixture {number}')
        git('add', 'source.txt')
        git('commit', '--quiet', '-m', f'fixture {number}')
        commits.append(git('rev-parse', 'HEAD'))
    ticket = root / 'ops/default/ticket/solved/20261027092143-convert-tabs-and-spaces.md'
    ticket.parent.mkdir(parents=True)
    ticket.write_text('---\nid: TKT-20261027092143\nreported_at: 2026-10-27T09:21:43Z\nstatus: solved\nkind: task\nscope: domain\nresolution: fixed\nclosed_at: 2026-10-27T12:00:00Z\nbranch: feat/rev-input-03-whitespace-conversion\npr: https://forge.adrianpk.com/adrianpk/rev/pulls/1234\ncommits: [' + ', '.join(commits) + ']\n---\n')
    report = root / 'ops/default/report/slices/rev-input-navigation/slice-3-whitespace-conversion.md'
    report.parent.mkdir(parents=True)
    report.write_text('# Slice 3: Add explicit tabify and untabify\n\nStatus: delivered\nDelivery set: rev-input-navigation\nBranch: `feat/rev-input-03-whitespace-conversion`\nPR: [#1234](https://forge.adrianpk.com/adrianpk/rev/pulls/1234)\nIntegrated: `' + commits[-1] + '`\n' + ''.join('\n## ' + section + '\n\nFixture evidence.\n' for section in SECTIONS))
    return ticket, report


if __name__ == "__main__":
    unittest.main()
