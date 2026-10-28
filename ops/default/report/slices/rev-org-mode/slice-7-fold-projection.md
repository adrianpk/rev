# Slice 7: Map folded Org views to canonical source

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-07-fold-projection`
PR: pending

## Purpose

Provide view-local hidden-source projection through existing canonical coordinates.

## Delivered Behavior

Each native Org view admits revision-qualified complete logical-line source
ranges. Overlapping ranges normalize into one hidden interval. Visible screen
rows map bijectively to original logical source rows; hidden source has no screen
position. EOF, foreign/stale authority, partial lines and current-point hiding
have explicit admission outcomes. Source, history and shared peer views survive.

## Implementation Notes

An immutable shared interval table keeps view cloning cheap. Admission allows
at most 4096 ranges, sorts/merges their row intervals, and stores cumulative
hidden counts. Forward/reverse queries use binary search without constructing a
whole-document row array. `View.top` remains a canonical logical row.

Entering a hidden row through mutable reveal removes that containing interval
only in the requesting view. Stale or inactive-mode maps project plain source;
reveal releases stale offsets. New buffer bindings reset visibility.

## Contracts Added or Changed

- ORG03: independent revision-qualified view visibility and non-aliasing row maps.
- ORG18: finite admission, explicit rejection and unchanged source authority.

## Files of Interest

- `src/view/fold.rs`, `src/view/fold/build.rs`: visibility admission and row maps.
- `src/view.rs`: native per-view ownership and checked reveal.
- `tests/org_editing/folds.rs`: independent maps, admission and EOF contracts.

## Validation

Implementation checks:

- `cargo check --lib`: passed.
- `cargo test --test org_editing folds::`: three passed.

The exact ordered controller batch will follow the remaining projection tasks
and pushed canonical PR-reference commit.

## Risks and Follow-ups

Display/input integration and combined stale/edit/tiny-pane coverage remain in
T7.2 and T7.3. Native visibility commands belong to the recorded Slice 8.
