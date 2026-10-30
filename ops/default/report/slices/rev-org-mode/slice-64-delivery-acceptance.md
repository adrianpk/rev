# Slice 64: prepare complete native Org delivery acceptance

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `test/rev-org-64-delivery-acceptance`
PR: [#165](https://forge.adrianpk.com/adrianpk/rev/pulls/165)
Integrated: `48923e0c8a83743721c46cd465abd34f8f1605ce`
Repair 1 status: delivered
Repair 1 branch: `fix/rev-org-mode-validation-1`
Repair 1 title: `fix(gate): correct native Org acceptance`
Repair 1 PR: [#166](https://forge.adrianpk.com/adrianpk/rev/pulls/166)
Repair 1 integrated: `501cd6d49ce412fb49342b6cfc015c5d67a6f822`
Repair 1 failed candidate: `85992b8bd289bd309fba64da169fb04a32640847`

## Purpose

Prepare ORG01, ORG18 and ORG31 acceptance under the approved
[plan](../../../plan/rev-org-mode.md) and [tracker](../../../tracker/rev-org-mode.md).

## Delivered Behavior

The existing acceptance collector admits all 64 approved slice identities and
reports, 192 completed tracker tasks and actual ORG01-ORG31 scenario execution.
Readiness remains prerequisite resolution, without a full-gate success claim.

## Implementation Notes

The additive Org delivery manifest pins all 63 earlier canonical PR integration
commits and 189 task commits. Forgejo closed-PR state was verified before capture;
Git ancestry places each task in its owning slice integration range. Established
report heading and PR formats remain readable without rewriting earlier records.
The final slice requires its canonical PR, delivered report, integration ancestry
and three task commits before runtime admission. Prerequisites permit its reviewing
report and pending integration only. Provenance does not replace semantic checks:
all 34 native assertions must occur in actual Cargo results, alongside the existing
same-candidate terminal stage assertions and release resource observations.

The eleven frozen gate commands and their order are explicitly admitted, with
no change to `scripts/check-all.sh`. Final integration must point to the reviewed
PR-reference report stored at that commit, not a later closure commit. Native,
terminal and resource records must still share that exact clean HEAD and tree.

The controller pins 91 exact documentation closure paths; 88 currently exist.
The absent `docs/reference/panels.md`, `docs/reference/org-time.md` and
`docs/how-to/org-mode.md` are permissions, not required new documents. Closure
must retain the actual tested SHA, validation identifier, ordered commands and
runtime observations separately from its later documentation SHA. Existing
historical closure metadata stays unchanged. No final runtime result is claimed
by fixture preparation or this slice's prerequisite checks.

The approved 192 task descriptions must match between plan and tracker, with
unchanged identities and spec/model ownership links. All 18 historical closure
records retain their original candidate and gate identifiers. Negative fixtures
reject changed canonical records, unrelated task commits, incomplete reports,
altered gate order, omitted native assertions, stale HEAD/tree evidence and
premature use of a later closure commit. Fixture Git history tests the reviewed
snapshot boundary independently of runtime execution.

The retained gate failed during `cargo test --all` in
`gutter::completion::file_candidates`. The fixture typed its full temporary root
into a 61-column prompt, then expected the entire label and draft to remain
visible. The longer owned temporary path exceeded that width; the existing prompt
correctly scrolled horizontally to retain its cursor. This was a fixture input
error, not a source or layout regression.

Repair 1 uses the relative draft `al`, resolved by the native path navigator from
the visited fixture file's parent. It retains the exact `alpha.txt`/`alpine.txt`
candidates, every original resize including zero/tiny geometry, cursor ownership,
full prompt assertion, cancellation, independent views, source identity and empty
history assertions. No width, deadline, resource ceiling, runtime behavior or
acceptance assertion changes. GUT06's preserved draft/focus/view contract remains
intact. Absolute-path behavior remains covered by the existing path-completion
suite. A long owned TMPDIR reproduction confirms the fixture no longer depends
on temporary root length. All test files remain privately owned and cleaned up.

The original delivery metadata above continues to identify merged Slice 64.
The separate repair metadata identifies the active approved repair; it does not
replace the original canonical integration or reuse its validation as repair proof.

## Contracts Added or Changed

- ORG01: complete source-preserving native scenario admission.
- ORG18: all 34 agreed scenarios require actual execution with zero skips.
- ORG31: canonical delivery provenance and same-candidate evidence stay distinct.

## Files of Interest

- `scripts/configuration_ui/org_delivery.py`: Org integration and task admission.
- `scripts/configuration_ui/acceptance.json`: additive immutable delivery records.
- `scripts/configuration_ui/evidence.py`: current runtime and prerequisite admission.
- `scripts/test_acceptance.py`: negative acceptance fixtures.

## Validation

Required ordered checks passed on clean implementation candidate
`0a6e1e3eec7f1159ae7546b356c8850bdd6c737b`, controller receipt
`XYGFTOC3SF3NHFK5SUWGDAGXD7`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test configuration --test commands --test display`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-source.py`
- `git diff --check`

All 1081 selected native tests passed, including 39 display tests and the 1042
Org/command/configuration tests selected in Slice 63; zero failures, ignored or
filtered tests. All 34 scenario native assertions resolve against these actual
execution results. All 26 checker tests passed, including adversarial integration,
task, report, frozen-gate, historical closure and reviewed-snapshot fixtures.

The reference checker confirms 178 schema/catalog entries, 54 RGB roles, 263
bindings, nine optional examples and 24545 exact packaged template bytes. The
prerequisite collector resolves 64 fixed slice identities/reports, 63 integrated
canonical PRs, 189 owning task commits, all 192 task descriptions and every Org
scenario's compiled native/terminal/resource reference. Catalog resolution is not
terminal/resource runtime proof. All prior manifest entries are unchanged; the
new delivery object is additive. Source quality retains safe Rust, five
parameters, nesting three and cyclomatic ten.

Task commits: T64.1 `269eaa204e335e6d69d4a6482323f9ff5bc8edef`,
T64.2 `77457cd6fa91`, T64.3 `0a6e1e3eec7f1159ae7546b356c8850bdd6c737b`.
Canonical PR [#165](https://forge.adrianpk.com/adrianpk/rev/pulls/165) is verified merged
into dev at `48923e0c8a83743721c46cd465abd34f8f1605ce`. Final PR-reference receipt
`TYB2VHDJJPD7RTVIYICVMZLU2Z` passed the same eight ordered commands on that
exact clean candidate: 1081 native tests, 26 checker tests and all 34 mapped
native assertions in actual execution, with zero skips. All 64 canonical PRs
and their integrated ancestry are verified. This integration closure changes
documentation only; the controller must run the unchanged full gate on its
new clean, pushed candidate before delivery-set closure.

Retained controller gate receipt `2SQHFEABZZYGZ66KYYIHGDKSZV` failed on
`85992b8bd289bd309fba64da169fb04a32640847`; execution stopped at the gutter
fixture before later gate stages. A focused reproduction of
`cargo test --test gutter completion::file_candidates -- --exact --nocapture`
failed before the correction and passed afterward (one selected test).
The same focused test passed with an owned TMPDIR prefix exceeding 100 bytes;
the original geometry and all assertions remain unchanged. Required repair prefix passed on clean implementation candidate
`9ec70bea291925e6195b47d5116f0ab6c6f35f24`, controller receipt `2IKQB6IPP3XXY5REOBHL2MIRCZ`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `python3 scripts/check-source.py`
- `git diff --check`

All 1716 native tests passed with zero failures, ignored or filtered tests,
including all 33 gutter tests and every existing path-completion test. All 34
mapped Org native assertions occur in these actual results. Source-quality
limits remain safe Rust, five parameters, nesting three and cyclomatic ten.

Focused controller receipt `5E7P7ACV7SFIDNS7KDB5ODX5RT` passed on the same candidate:

- `python3 -c 'import os, subprocess, tempfile; from_context = tempfile.TemporaryDirectory(prefix="rev-gutter-long-root-" + "x" * 96); root = from_context.__enter__(); result = subprocess.run(["cargo", "test", "--test", "gutter", "completion::file_candidates", "--", "--exact", "--nocapture"], env=dict(os.environ, TMPDIR=root)); from_context.cleanup(); result.check_returncode(); print("Long owned TMPDIR: original geometry/assertions passed; fixture removed.")'`
- `python3 -c 'import json, sys; from pathlib import Path; sys.path.insert(0, "scripts"); from configuration_ui.org_delivery import delivery; root = Path.cwd(); delivery(root, json.loads((root / "scripts/configuration_ui/acceptance.json").read_text()), readiness=False); print("Original 64 canonical delivery records and reviewed snapshot preserved; repair metadata remains separate.")'`
- `git diff --check`

The owned long-root fixture was removed after its selected test. Original 64
canonical reports/integrations, 192 task outcomes and the reviewed Slice 64
snapshot still pass delivery admission with separate repair metadata. This is
focused repair evidence, not a successful complete gate. Canonical repair PR
[#166](https://forge.adrianpk.com/adrianpk/rev/pulls/166) is verified merged into
dev at `501cd6d49ce412fb49342b6cfc015c5d67a6f822`. Final required receipt
`RDUOKIX2PS6CK5LM4JD52FK5D3` passed the same five pinned commands and all 1716
native tests with zero skips on that exact clean PR-reference candidate. Final
focused receipt `ADEZXU3ATMZHJH5ZENTKJCEDLW` passed the same three focused
commands there, including the long-root fixture and original integration admission.
This later documentation closure preserves those tested identities. The controller
must run the unchanged full gate on the new clean, pushed candidate.


## Risks and Follow-ups

The unchanged full gate and later exact-path delivery documentation closure
remain controller-owned. A later documentation head cannot inherit the tested
candidate identity.
