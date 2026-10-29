# Slice 46: prepare checked native Org export snapshots

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-46-export-contract`
PR: pending

## Purpose

Prepare checked native export data under the [approved plan](../../../plan/rev-org-mode.md#slice-46-prepare-checked-native-org-export-snapshots)
and [tracker](../../../tracker/rev-org-mode.md#slice-46-prepare-checked-native-org-export-snapshots).

## Delivered Behavior

T46.1 captures unsaved canonical text, real revision evidence, one operation
instant/zone, file metadata, typed options and buffer/region/subtree scope.
Projected native structural offsets are separate from canonical source evidence.
Configuration selects a fixed native backend set and finite include, expansion
and rendered-output budgets. No export preparation writes or saves a source.

T46.2 resolves local quoted includes through the shared checked canonical
reader, including nested relative paths and explicit line ranges. It rejects
cycles, missing files, unsupported options, incomplete native structure and
bounded source/node/expansion limits. Source blocks remain literal. Text-only
macros support positional arguments and bounded nesting; executable definitions
reject, while unknown names remain source with explicit diagnostics.

T46.3 verifies physical include aliases, strict ingress, selected scope, aggregate
limits, source revisions, read-only source, BOM/CRLF/EOF and destination
preservation. Original source evidence retains BOM metadata. Freshness checks
separate pure canonical revision checks from owned disk observation checks;
new canonical visits supersede disk evidence. Repeated OPTIONS lines compose.

A focused drawer regression first failed because native drawer children were
eligible as actions. Export preparation now derives ancestor ownership once;
includes, definitions and invocations inside drawers remain literal. The
regression passes without changing native editing structure or limits.

## Implementation Notes

This is the checked preparation layer for native renderers. It has no command,
file publication, code evaluation or automatic export effect.

## Contracts Added or Changed

- ORG25: immutable export snapshots and typed native selection.
- ORG02: unsaved source fidelity and canonical revision provenance.

## Files of Interest

- `src/org/export.rs`: checked owned export work and immutable snapshot.
- `src/config/export.rs`: typed backend selection.
- `tests/org_io/export.rs`: source fidelity, scope, bounds and immutability.
- `src/org/export/current.rs`: canonical and disk freshness boundaries.
- `docs/reference/org-export.md`: admitted preparation profile and limits.
- `scripts/check-config-reference.py`: exact coverage of the new typed leaves and enum.

## Validation

- `cargo test --test org_io --test configuration -- export_`: six selected tests passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed.
- Changed-source quality: passed with unchanged native limits.

- `cargo test --test org_io export::`: six export contract workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed after include/macro implementation.
- Changed-source quality: passed with unchanged native limits after factoring bounded parsing and composition responsibilities.

- `cargo test --test org_io export::`: all 14 export workflows passed, zero failures or ignored tests, including the corrected drawer regression.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed after lifecycle and scope coverage.
- Changed-source quality: passed with unchanged parameter, nesting and cyclomatic limits.
- `git diff --check`: passed.

Final ordered validation is recorded through the controller for the clean PR
candidate before handoff. Integration remains open for controller verification.

## Risks and Follow-ups

None recorded.
