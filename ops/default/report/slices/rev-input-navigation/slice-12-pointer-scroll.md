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

## Implementation Notes

The existing View and Tree owners scroll their native state. Document points
use checked display-column/source conversion only when outside the new visible
interval. Tree scrolling uses loaded entries without directory I/O or expansion.
The target is resolved from current composition; no hit map or event is retained.

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
- `tests/pointer/scroll.rs`: inactive pane/tree and keyboard continuity checks.

## Validation

Development checks passed:

- `cargo test --test pointer scroll::`: inactive document/tree routing, retained
  original point, keyboard prefix/C-g and actual frame/cursor checks passed.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

Final focused and required clean-head checks will be recorded through the
controller after task and PR-reference commits.

## Risks and Follow-ups

Factory activation, configured step, drag selection and autoscroll remain in
recorded later slices. Wheel handling adds no queue, timer or idle work.
