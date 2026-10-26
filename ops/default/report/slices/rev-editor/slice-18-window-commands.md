# Slice 18: Delete and cycle editing windows

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-18-window-commands`
PR: `#18`

## Purpose

Cycle and remove editing views while preserving a valid selected leaf, nested
layout, canonical buffers and unsaved text after their last visible pane closes.

## Delivered Behavior

- `C-x o` / `other-window` select editing leaves in first-child-before-second-child
  screen order and wrap from the final leaf to the first. A sole view cycles to
  itself. Point and scroll remain independent in each surviving view.
- `C-x 0` / `delete-window` remove the selected view and reconnect its sibling
  subtree. The first editing leaf of that sibling becomes selected. Deleting the
  sole editing view is refused visibly without changing layout or text.
- `C-x 1` / `delete-other-windows` retain the selected identity and its complete
  view state, remove other editing views and expand the remaining pane. Calling
  it on the sole view reports the existing single-window state safely.
- Window removal never kills a buffer, changes its saved state or discards
  history. A modified buffer with no visible pane still blocks unsaved exit;
  switching back reuses its identity, edits and retained undo history.
- Traversal uses logical editing leaves independently of geometry. Cycle and
  removal remain valid while presentation is suspended. Prompt, inventory and
  decision focus remain separate and cannot be stolen by native window dispatch.
- Repeating the final reported key of a completed `C-x` command cannot become
  accidental document text. Changing editing focus ends ordinary typing continuity.

## Implementation Notes

- T18.1: `56a933e7857aed9896435025fae3d74d0ea9074d` adds editing-leaf traversal,
  checked deletion, sibling promotion, keep-selected behavior and command bindings.
- T18.2: `fa7726410cfd25766adbd81a8ed23c3ce2c4ef69` adds native and real-terminal
  lifecycle evidence, including hidden modified file buffers and recovered undo.
- `Windows` remains the sole structural owner. Deletion prepares a candidate tree,
  validates its selected replacement, and publishes registry/tree/selection changes
  together. Existing buffer storage and file indexes are not involved in view removal.
- Leaf order follows the recursive layout rather than identity allocation order.
  Promoting a sibling preserves its nested directions, leaves and independent state.
  Editing traversal does not acquire members from other pane or transient owners.
- Keeping one view preserves that exact record and identity rather than creating a
  replacement. Removed IDs stay unavailable and subsequent splits allocate fresh IDs.
- Geometry and frame composition derive the expanded panes on the next redraw.
  Deletion and traversal do not require currently fitting presentation rectangles.
- Window commands and checked native selection respect input focus. The common
  command boundary ends typing groups; prefix completion suppresses reported key
  repeats until another press, preserving normal editing after command completion.

## Contracts Added or Changed

- `Windows::order` exposes a bounded snapshot of editing-leaf order and `next_view`
  resolves deterministic wraparound selection without relying on surface geometry.
- `Windows::delete_selected` reconnects the sibling subtree and returns its live
  replacement identity; `delete_others` retains the selected leaf unchanged.
- `window::Error::OnlyWindow` reports sole-view deletion refusal. No view operation
  removes a canonical buffer or bypasses dirty-buffer quit admission.
- `Command::Window` delegates all three canonical commands through common naming
  and execution; `C-x 0`, `C-x 1` and `C-x o` route to those same handlers.
- `CHANGELOG.md` describes pane cycling, deletion and retained unsaved buffers.

## Files of Interest

- `src/window/lifecycle.rs`: logical order, wraparound and view-only removal.
- `src/window/layout.rs`: editing traversal and checked sibling promotion.
- `src/session/windows.rs`: focused command orchestration and native selection.
- `src/command/window.rs`, `src/input/bindings.rs`: canonical names and key sequences.
- `src/input.rs`, `tests/commands.rs`: consumed-prefix repeat protection.
- `tests/windows.rs`: lifecycle, focus, identity and hidden-buffer invariants.
- `scripts/check-pty.py`: actual window keys, cursor positions and retained file edits.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 180 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test windows`: passed; twenty tests, zero skips. Added cases cover
  layout-order cycle/wrap, sibling subtree promotion, selected-view retention,
  sole deletion refusal, repeated split/delete with fresh identities, hidden
  modified buffer/history preservation, suspended commands, separate transient
  focus and typing boundaries across view changes.
- `python3 scripts/check-pty.py --suite terminal`: passed; thirty-three cases,
  zero skips. New journeys verify real cycle/delete/keep keys, selected hardware
  cursor, independent point, reclaimed geometry, sole-view refusal and both paths
  hiding a modified visited file while retaining unsaved-exit protection and undo.
  Owned fixtures stop after their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
