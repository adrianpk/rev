# Slice 12: Scroll the pointed pane without changing focus

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-12-pointer-scroll`
PR: pending

## Purpose

Route vertical wheel/trackpad reports to the current document pane or loaded
file tree beneath the pointer while retaining keyboard focus.

## Delivered Behavior

- An admitted vertical report scrolls the pointed pane by three logical rows.
  Current pane ownership is shared with primary-press hit resolution.
- Scrolling changes only the target viewport and any point/selection adjustment
  needed to keep that target visible. Inactive targets retain their focus state.
- Keyboard prefixes, echo, unrelated view state and canonical text/history are
  retained. Prompt/decision owners retain their existing pointer exclusion.
- Empty/single-line and final-empty rows clamp safely at native logical bounds.
  Zero/tiny suspended layouts expose no wheel target. Either gutter/status area
  can address its owning pane; separators and prompts cannot.

## Implementation Notes

The existing View and Tree owners scroll their native state. Document points
use checked display-column/source conversion only when outside the new visible
interval. Tree scrolling uses loaded entries without directory I/O or expansion.
The target is resolved from current composition; no hit map or event is retained.
Native viewport origins clamp at the last logical row or loaded tree entry.
Point/selection is clamped to the nearest visible row only when necessary;
otherwise navigation goals, horizontal offsets and recenter state are retained.

Typing continuity remains governed by the existing transaction context, including
all shared points. When a peer point stays visible, the active contiguous typing
group continues. When scrolling must adjust that point, the next edit starts a
new native group so undo retains coherent shared-view positions. Wheel handling
itself creates no history item and does not explicitly break the typing owner.

The three-row native factory step is preparatory. Its startup setting and mode
activation remain assigned to the recorded controls slice. Reporting stays off
at ordinary startup until that integration.

## Contracts Added or Changed

PTR01 uses the existing native event path. PTR02 scrolls the current pointed
surface without selecting it or changing another pane's point.

## Files of Interest

- `src/pointer/target.rs`: composed surface ownership shared by press and wheel.
- `src/session/pointer/scroll.rs`: current target dispatch without focus changes.
- `src/view/scroll.rs`: checked viewport-first movement.
- `src/tree/navigation.rs`: loaded-preorder scrolling.
- `tests/pointer/scroll.rs`, `tests/pointer/scroll_bounds.rs`: inactive targets,
  clamping, source fidelity, retained points/regions and native typing history.

## Validation

Development checks passed:

- `cargo test --test pointer scroll::`: inactive document/tree routing, retained
  original point, keyboard prefix/C-g and actual frame/cursor checks passed.
- `cargo test --test pointer`: 28 native groups passed, including both directions
  through 10,001 logical rows, empty/final-empty/CRLF/unterminated bounds,
  unchanged visible points/goals, native typing context, shared active regions,
  current resized geometry and every existing pointer regression.
- `cargo test --test pointer scroll_bounds::checked_columns`: widths 1/4/8/32,
  read-only tabbed Unicode/CRLF and retained horizontal offsets passed.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

Final focused and required clean-head checks will be recorded through the
controller after task and PR-reference commits.

## Risks and Follow-ups

Factory activation, configured step, drag selection and autoscroll remain in
recorded later slices. Wheel handling adds no queue, timer or idle work.
