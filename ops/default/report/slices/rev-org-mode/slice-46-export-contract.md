# Slice 46: prepare checked native Org export snapshots

Status: drafting
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

## Implementation Notes

This is the checked preparation layer for native renderers. It has no command,
file publication, code evaluation or automatic export effect.

## Contracts Added or Changed

- ORG25: immutable export snapshots and typed native selection.
- ORG02: unsaved source fidelity and canonical revision provenance.

## Files of Interest

- `src/org/export.rs`: checked owned export work and immutable snapshot.
- `src/config/export.rs`: typed backend selection.
- `tests/org_io/export.rs`: source fidelity, scope and immutability.

## Validation

- `cargo test --test org_io --test configuration -- export_`: six selected tests passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed.
- Changed-source quality: passed with unchanged native limits.

## Risks and Follow-ups

None recorded.
