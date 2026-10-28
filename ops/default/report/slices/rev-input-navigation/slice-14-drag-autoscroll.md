# Slice 14: Extend pointer selection with bounded autoscroll

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-14-drag-autoscroll`
PR: pending

## Purpose

Extend the canonical linear region beyond the source pane's visible text through
bounded active-drag work in the existing native input owner.

## Delivered Behavior

A captured outside drag scrolls only its revision-qualified source view and maps
its point to a checked source boundary at the near edge. Reentry pauses the
work; release retains the ordinary region and removes capture. Source bytes,
revision and history remain unchanged.

## Implementation Notes

The capture holds one current pointer position and one deadline. Checked native
coordinates clamp the edge row to the document and the column to the source text
partition; other panes and decorations never supply selectable text. Every step
revalidates the source view and revision. A blocked document edge retires the
deadline. The existing host deadline entrypoint performs at most one step.
The native input owner waits for the earlier Escape or drag deadline, blocks
indefinitely when neither exists, and rechecks ready input after a timeout.
Cancellation/release is dispatched before due pointer work. Motion updates one
position without moving its deadline; a late wake performs one step and starts
the next 100 ms interval from that wake. Each step is bounded by the lesser of
the native factory scroll step (three) and the current content height. Existing
32-event/four-millisecond input batch limits remain unchanged.

## Contracts Added or Changed

PTR04 extends the native mark/point region across the source viewport edge.
PTR05 retains finite source ownership and removes work when capture ends.

## Files of Interest

- `src/session/pointer/autoscroll.rs`: captured edge, checked source projection and deadline service.
- `src/session/pointer/selection.rs`: edge tracking and capture retirement.
- `src/session/events.rs`: controlled-clock native host entrypoints.
- `tests/pointer/autoscroll.rs`: forward/reverse ownership, pause and release.

## Validation

Development `cargo test --test pointer` passed 42 groups, including controlled
100 ms source-edge extension, pause/release and upward scrolling from a lower
pane without changing its peer. `python3 scripts/check-source.py` passed the
unchanged safe Rust, parameter, nesting and complexity limits.

Development `cargo test --lib --test pointer` passed 13 library and 46 pointer
groups. It covers one-row height bounds, 1,000 reports without deadline debt,
late wakes, document limits/empty text, due-time input cancellation and injected
native poll-boundary release priority. Focused Clippy and the unchanged source
quality guard passed.

Final ordered and focused evidence will be recorded through the controller on
the clean candidate after task and PR-reference commits.

## Risks and Follow-ups

Ordinary reporting and typed mouse controls remain assigned to Slice 15.
