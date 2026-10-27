# Slice 11: Focus panes and place point by pointer

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-11-pointer-hit-focus`
PR: pending

## Purpose

Resolve native primary presses against the current composed workspace and place
point through the existing checked movement owner.

## Delivered Behavior

- Primary presses select the owning editing pane and resolve its visible text
  through the same gutter partition used by rendering.
- Number fields and status rows can focus their pane without changing point.
  Shared separators, prompts and outside cells do not identify document text.
- Tree and editing focus use existing focus owners; active prompt owners retain
  their input. Source text and history remain canonical.

## Implementation Notes

Hit destinations are disposable projections of current composition, view and
buffer revision. No hit map is retained. Checked movement validates the target
point, resets navigation and reveals it through the ordinary motion owner.
Workspace reveal is synchronized before hit resolution, matching resize drawing.

## Contracts Added or Changed

PTR02 identifies pane focus. PTR03 maps visible cells to checked source points
without treating decorations or padding as source positions.

## Files of Interest

- `src/pointer/hit.rs`: current composed pane ownership.
- `src/view/hit.rs`: checked gutter/text projection.
- `src/session/pointer.rs`: native press dispatch and existing focus transitions.
- `src/session/motion.rs`: shared checked movement owner.
- `tests/pointer/focus.rs`: pane focus, text and decoration regressions.

## Validation

Development checks passed:

- `cargo test --test pointer`: seven groups passed, including shared-pane point
  placement, frame cursor agreement, decoration focus and unchanged source/history.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

Final clean-head evidence will be recorded after task and PR-reference commits.

## Risks and Follow-ups

Factory reporting remains disabled until the recorded handler/settings slices
integrate. Wheel routing, drag capture and autoscroll belong to their recorded
later slices. This slice covers primary press focus and point placement.
