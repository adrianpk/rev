# Slice 30: Verify delivery acceptance and references

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `test/rev-config-ui-30-configuration-ui-acceptance`
PR: pending

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
- Ten tickets whose assigned slices are integrated are solved with full head,
  canonical PR and report references. The acceptance ticket is reviewing until
  Slice 30 integrates. All eleven identities and commit ancestry are checked;
  the future conversion ticket remains open.
- All 30 report paths are prepared. The first 29 have uniform immutable
  integration markers; the current report and tracker retain pending integration
  and final-gate state. No full gate has been run by this worker.

## Implementation Notes

- T30.1: `63f8b9a3fbd4` adds the requirement map, extends gate evidence by exact
  command identity, retains baseline protections and adds negative guard cases.
- T30.2: `486a94c719a1` reconciles complete baseline amendments, model/reference
  behavior and verified ticket outcomes without rewriting historical reports.
- T30.3 prepares report/ticket ancestry and acceptance prerequisites, preserving
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

Final readiness requires controller evidence for the clean PR head after the
PR-reference follow-up commit, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-source.py`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `git diff --check`

## Risks and Follow-ups

After canonical Slice 30 integration, gate preparation must close this report/
tracker integration entry, add the verified head, move the acceptance ticket
from reviewing to its approved solved path with that head and update its plan
link. The strict runtime evidence mode requires all 30 reports and eleven
solved tickets; readiness alone allows this current unmerged slice.

The controller then runs the unchanged `scripts/check-all.sh` entry point on
one clean exact dev candidate. Only after its success may the approved closure
paths record the gate identity/candidate, close this delivery's plan/tracker/
spec/model/interaction status and remaining closure checkboxes, and record Acta.
The historical baseline candidate, gate candidate and later documentation-only
closure commit remain distinct. No main promotion or publication is included.
