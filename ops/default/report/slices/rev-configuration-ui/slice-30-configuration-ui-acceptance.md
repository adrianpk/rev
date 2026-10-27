# Slice 30: Verify delivery acceptance and references

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `test/rev-config-ui-30-configuration-ui-acceptance`
PR: [#79](https://forge.adrianpk.com/adrianpk/rev/pulls/79)
Integrated: `d4450c013f3350d2a5268c35ca799cde560a70d1`

## Purpose

Connect the accepted configuration and interaction contracts to actual native,
terminal and performance evidence and prepare controller-owned final validation.

## Delivered Behavior

- The new acceptance map covers CFG01-CFG22, UI01-UI04 and the complete modified
  R01/R17/R20/R22 requirements with named native tests, all eight configuration
  PTY cases and all seven bounded-work workloads.
- `scripts/check-all.sh` retains its existing entry point. Its owner now runs
  eleven stages: the original first six checks, configuration reference, new
  terminal suite, new performance suite, acceptance evidence and whitespace.
  All original eight commands retain their relative order and protections.
- Evidence lookup uses exact command names. The verifier requires all nine
  preceding stages in order, success flags, immutable log checksums and the same
  clean HEAD/tree. Existing A01-A20 mappings, four historical journeys, two
  performance fixtures and native/source evidence remain mandatory.
- Configuration evidence rejects missing/duplicate/reordered PTY cases, focused
  subsets, skipped suites, missing native cases, invalid sample counts, timing/
  memory regressions, absent fixture identities, parser controls, recursive or
  cached reads and missing configured PTY startup/idle/restoration evidence.
- Historical document/workspace/failure PTY journeys now assert Unicode panel
  separators and ignore SGR styling when detecting intermediate prefix messages.
  Their file bytes, geometry, shared-state and restoration assertions are retained;
  all three focused journeys pass. A colored-prefix regression guard is included.
- Readiness lists native cases without executing the full native suite. It
  verifies the frozen 30-row plan/tracker map, canonical report identities,
  complete report sections and integrated ancestry for all 29 prior slices.
- The original dependency boundary admits exactly the two approved additions,
  pinned TOML and serde, while retaining all other product/runtime restrictions.
- Four complete approved requirement replacements are synchronized with the
  current baseline. Historical baseline candidate metadata and all original
  implementation reports remain distinct from this delivery's evidence.
- Defaults, rectangle copy/kill/yank semantics and model references now agree
  with effective buffer policies. Reference checks include rectangle links.
- All eleven tickets are solved with full integrated heads, canonical PRs and
  report references after verified integration of their assigned slices. The
  future conversion ticket remains open.
- All 30 reports are delivered with immutable integration markers. Slice 30
  integrated through canonical PR #79. The authorized repair PR #80 and all
  final acceptance checks are integrated and validated by the controller.

## Implementation Notes

- T30.1: `63f8b9a3fbd4dba906dfe168a3a3c0a741872e02` adds the requirement map, extends gate evidence by exact
  command identity, retains baseline protections and adds negative guard cases.
- T30.2: `486a94c719a1eb029c97895f287b03e4681d9e32` reconciles complete baseline amendments, model/reference
  behavior and verified ticket outcomes without rewriting historical reports.
- T30.3: `530437313969939016a873d9b873df8be2d41bd9` prepares report/ticket ancestry and acceptance prerequisites, preserving
  controller ownership of integration, final validation and documentation closure.
- Existing Python/Cargo discovery and standard-library checks are reused. There
  are no new runtime dependencies, product features or generic schema layers.
- Guard tests exercise missing/duplicate/skipped terminal evidence, changed
  maps, native identities, stale HEAD/tree, changed/failed/reordered logs,
  sample/time/RSS bounds, lost fixture identity and read/parser/idle controls.

## Contracts Added or Changed

- CFG01-CFG22 and UI01-UI04 have a complete requirement-to-executable evidence map.
- R01/R17/R20/R22 are reconciled as complete requirements, including existing
  scenarios and the selected tab/rectangle/default-key semantics.
- A01-A20 retain their original evidence map and command-relative protections.
  The gate uses nine named preceding logs instead of six positional logs.
- Current configuration reports and tickets require actual integrated ancestry;
  a readiness pass is explicitly separate from final acceptance evidence.

## Files of Interest

- `scripts/configuration_ui/acceptance.json`: 30 requirement mappings and eleven ticket assignments.
- `scripts/configuration_ui/evidence.py`: map/report and actual suite evidence checks.
- `scripts/configuration_ui/delivery.py`: complete replacements and ticket/commit prerequisites.
- `scripts/acceptance.py`, `scripts/acceptance_delivery.py`, `scripts/check-acceptance.py`:
  retained baseline gate/evidence with explicit approved extensions.
- `scripts/test_acceptance.py`: negative acceptance guard cases.
- `ops/default/spec/rev-editor.md`, its model and the accepted configuration
  delta: selected full replacements and separate original provenance.
- `docs/reference/rectangles.md`: coherent copy/kill/yank and history policy.

## Validation

Successful focused development checks:

- `cargo test --all -- --list`: real native target/case discovery completed;
  this lists tests and does not establish runtime success.
- `python3 -m unittest discover -s scripts -p test_acceptance.py`: twelve guard tests passed.
- Native-reference discovery resolved every mapped case and the frozen original
  eight commands remain in order within eleven configured stages.
- Focused historical `acceptance_document`, `acceptance_workspace` and
  `acceptance_failure` journeys passed after the accepted separator/color update.
- Full requirement replacement equality, rectangle local links, all eleven
  ticket metadata/status/identity records and 29 prior integrated heads passed.
  Acta returned complete bounded evidence for the 29 canonical merges.

Controller validation `QSZOEPTDW22LQ4HKPGSW6UCFRK` passed all eight checks on
clean PR head `d4450c013f3350d2a5268c35ca799cde560a70d1`, after the PR-reference
follow-up commit, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-source.py`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `git diff --check`

Final-gate palette regression.

The first final gate stopped at `palette::message_colors`: the assertion still
addressed the bottom terminal row after find-file completion moved its query
above the candidates. That row correctly used the normal candidate foreground.

The correction locates the query by its `Band::Echo` identity and checks its
message palette role. It also checks both normal and selected completion rows
against their independent palette roles. Warning/error and document-immutability
assertions remain intact. Runtime code and the full-gate command are unchanged.

Focused reproduction after correction:

- `cargo test --test display palette::message_colors -- --exact --nocapture`:
  passed, one test executed with no failures or skips.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

Canonical repair [PR #80](https://forge.adrianpk.com/adrianpk/rev/pulls/80),
branch `fix/rev-configuration-ui-validation-1`, integrated by Rebase +
Fast-forward into dev at `db5885640fb3592e844e3ec198346a99415c32ca`. Controller validation
`7SBZYXAIGU5Z3P6ANNC3UFFTLN` passed all five ordered checks on that clean
head, including 218 display, path-completion, command and panel tests with zero
failures or ignored tests:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test display --test path_completion --test commands --test panels`
- `python3 scripts/check-source.py`
- `git diff --check`

Final controller gate `scripts/check-all.sh` passed on immutable candidate
`95713efb7f79fbc72417615a921841f8d49730ed` under validation
`FWDUCVRJCYBKFXEWEFLKPKA35E` on 2026-10-27. Its eleven stages passed
formatting, Clippy, 716 native tests, source checks, four baseline PTY journeys,
two original performance fixtures, 59 schema/catalog entries, 25 active RGB
roles, 109 default bindings, eight configuration/UI terminal sessions, seven
new bounded-work workloads, A01-A20 and CFG/UI acceptance, and whitespace.
No required cases were skipped. Controller gate elapsed time was 71.572 seconds.

## Risks and Follow-ups

All 30 reports and eleven solved tickets are closed against the controller
evidence recorded above. The plan, tracker, configuration spec/model and
interaction spec are completed. Only configured documentation closure paths
differ from the tested candidate; executable source, validation scripts,
dependencies and acceptance definitions retain their tested versions.

The historical baseline candidate, this gate candidate and the later
documentation closure commit remain distinct. The future conversion ticket
remains open. No owned test fixtures remain running. Main promotion and
publication require a separate maintainer instruction.
