# Slice 13: Select canonical text by dragging

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-13-pointer-selection`
PR: pending

## Purpose

Use primary press/drag/release to select canonical document text through the
existing checked point movement and linear mark/region owners.

## Delivered Behavior

- A text press focuses its pane, places point and arms one revision-qualified
  source anchor. A click or same-cell motion creates no new region.
- A real text drag activates the ordinary linear mark and extends point through
  the source view's checked text geometry. Release retains the canonical region
  and ends capture; a later orphan motion cannot change it.
- Source bytes, saved state and undo history remain unchanged by selection.

## Implementation Notes

Session retains one minimal capture and a bit distinguishing an armed click from
an active selection. The source ViewId and buffer stamp are rechecked against
current composition before each extension. No rendered text, UI label, event
queue, clipboard write or timer is retained.

## Contracts Added or Changed

PTR03 reuses represented text boundaries. PTR04 selects the native linear region
and retains it after release for ordinary editor operations.

## Files of Interest

- `src/session/pointer/selection.rs`: captured source, checked extension and release.
- `src/session/pointer.rs`: native primary gesture dispatch.
- `tests/pointer/selection.rs`: forward/reverse regions and click boundaries.

## Validation

Development checks passed:

- `cargo test --test pointer selection::`: forward/reverse canonical regions,
  final release endpoint, orphan motion, click/same-cell behavior, empty region
  after a real drag and unchanged file/history state passed.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

Final clean-head evidence will be recorded through the controller after task
and PR-reference commits.

## Risks and Follow-ups

Bounded drag-edge autoscroll and ordinary reporting/settings activation remain
assigned to the recorded later slices. This layer adds no idle work.
