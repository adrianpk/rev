# Slice 33: finalize or abort Org capture safely

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-33-capture-finalize`
PR: pending

## Purpose

Deliver checked native capture publication under the
[approved plan](../../../plan/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely)
and [tracker](../../../tracker/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely).

## Delivered Behavior

T33.1 revalidates the retained destination's actual path, version, BOM and text,
then prepares bounded entry, plain, list, table or missing-heading insertion.
Entries retain relative heading levels, file preambles and target metadata.
Ordered lists reuse native counter/reset rules and separate source marker edits
from inserted text, preserving ordinary shared-point mapping. Missing date-tree
headings retain the captured date and chronological insertion anchor.

## Implementation Notes

The ordinary buffer save owner can publish a prepared edit after actual file
replacement. It reserves history/point admission before IO. Failed replacement
leaves canonical source, history and points unchanged; successful replacement
publishes one ordinary edit. Directory-sync/observation failures retain actual
publication and the ordinary unresolved-save warning. Future candidate text is
an immutable prepared rope, never a fabricated canonical buffer identity.

T33.2 exposes working `C-c C-c` / `org-capture-finalize`. Validation uses the
existing job slot; C-g restores the draft. The live source is checked again
before the ordinary saved-edit boundary. Complete save returns home; rejected
publication keeps the draft and source unchanged. A post-replacement failure
retains an explicit inserted-once receipt, canonical source and draft; retry
saves that pinned source without another insertion. Changed draft/source pins
reject automatic retry, and abort reports actual prior publication.

## Contracts Added or Changed

- ORG10: exact destination revalidation and coherent capture insertion planning.
- ORG18: ordinary save checks, read-only state, history and publication truth.

## Files of Interest

- `src/org/capture/finalize.rs`: retained evidence, bounded work and checked edits.
- `src/org/capture/finalize/`: native placement and heading normalization.
- `src/org/list/numbering.rs`: shared counter/reset behavior for capture.
- `src/buffer/save.rs`: pre-admitted edit and ordinary file publication.
- `tests/files/capture_saved.rs`: real replacement, rejection and unresolved completion.

## Validation

T33.1: `0cdd4a33a0f358f4de544889e761878099d618f8` (report introduction).

- `cargo test --lib finalize_ -- --nocapture`: two tests cover six placement
  scenarios and chronological date-tree planning, with unchanged disk bytes.
- `cargo test --test files capture_saved -- --nocapture`: one file-backed test
  covers complete, permission-rejected and completion-unresolved publication.
- `cargo clippy --lib --test files -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter, nesting and complexity limits.

- `cargo test --test org_agenda capture_publication -- --nocapture`: four
  file-backed session workflows pass, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.
- Changed-source quality remains green without changing its limits.

- `cargo test --test commands --test configuration --test org_editing`: 478
  tests pass, zero failures or ignored tests, including actual finalize command
  invocation and the retained source/role regressions.
- `python3 scripts/check-config-reference.py`: 149 leaves, 54 RGB roles, 228
  default bindings and 18370 exact packaged reference bytes agree.

## Risks and Follow-ups

T33.3 remains in progress. The delivery-set full gate has not run.
