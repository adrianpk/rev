# Slice 12: Scroll the pointed pane without changing focus

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-12-pointer-scroll`
PR: [#92](https://forge.adrianpk.com/adrianpk/rev/pulls/92)

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
- `tests/pointer/scroll.rs`, `tests/pointer/scroll_bounds.rs`,
  `tests/pointer/scroll_workspace.rs`: inactive targets,
  clamping, source fidelity, retained points/regions and native typing history.
- `scripts/configuration_ui/pointer_scroll.py`: real SGR wheel ownership, native
  frame/cursor and source/idle/cleanup checks through the existing PTY owner.
- `docs/reference/terminal-input.md`: viewport, point and focus behavior.

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
- `cargo test --test pointer scroll_workspace::`: three groups passed for
  nested right/relative gutters, different source buffers and loaded-tree
  endpoint reversal after a child is removed externally. No wheel rereads
  entries or changes the other leaf.
- `python3 scripts/check-pty.py --suite pointer-scroll`: two owned sessions
  passed for different tabbed Unicode/CRLF buffers, preserved keyboard prefix,
  nested panes, inactive/focused tree, both directions, changed resize ownership
  and a later primary focus transition.
- `cargo clippy --test pointer -- -D warnings`: focused native lint passed.
- `cargo test --test tree`: all 62 cases passed after reconciling the five
  historical tree/display coordinate fixtures with the delivered gutter.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

The process fixtures retain the existing eight-second deadline and 1-MiB output
limit. They own HOME and every XDG root, compare the complete bounded filesystem
snapshot, assert raw input and terminal restoration, and reap/close every owned
child and descriptor. Each journey observes zero output and process CPU ticks
at idle over 120 ms. The warm two-session suite takes under one second locally,
excluding build cost. No runtime scheduling limit or idle work changes.

Development oracles were corrected to include decimal row growth in the large
fixture's exact EOF scalar, the tree's represented root row before its first
child, and the existing all-shared-points typing context. These retain exact
source/point/history assertions rather than changing the native contracts.

The first tree regression run failed five historical coordinates on both the
slice and unchanged dev `b0b3f6a014bef0b5df52091aed728f218e216e0b`: presentation,
resize lifecycle, overlays, nested layout and configured initial visibility.
Their editing coordinates omitted the already delivered six-cell factory field
(four digits, one padding cell and one separator). Exact cursor origins and the
same selection/search source intervals now include that field. The tree focus,
visible-width, source text, cancellation and resize assertions remain intact;
all 62 cases pass. No runtime behavior or acceptance limit was changed for these
fixture corrections.

Final focused and required clean-head evidence is recorded through the controller
after task and PR-reference commits. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test tree --test windows --test display --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `cargo test --lib --test coordinates`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `python3 scripts/check-pty.py --suite pointer-scroll`
- `git diff --check`


## Risks and Follow-ups

Factory activation, configured step, drag selection and autoscroll remain in
recorded later slices. Wheel handling adds no queue, timer or idle work.
