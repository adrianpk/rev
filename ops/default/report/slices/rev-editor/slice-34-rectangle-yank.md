# Slice 34: Yank blocks with coherent padding and undo

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-34-rectangle-yank`
PR: pending

## Purpose

Insert retained rectangle rows with display-column placement, necessary prefix
padding and line creation in one checked undoable transaction.

## Delivered Behavior

- C-x r y and M-x yank-rectangle share the implemented handler. It inserts the
  separate retained rectangle across successive target rows, starting at the
  selected view's display column, without using or changing the linear ring.
- Insertion into an interior wide/tab cell on a later row snaps to that glyph's
  source start. Existing target text remains whole. A short line receives only
  the prefix spaces needed to insert a nonempty fragment at the target column.
- Copied row fragments retain their actual unequal lengths, including whole
  wide-cluster expansion and partial-tab spaces. There is no synthetic right
  padding. Complete retained tabs remain literal tabs and use destination stops.
- Empty fragments advance the block without padding existing rows. Missing rows
  are created with the destination buffer's prevailing LF/CRLF separator; ties
  and new text use LF. New empty rows receive separators but no column spaces.
- All row insertions, padding and new separators publish as one transaction.
  One undo restores original text, initiating cursor, mark and clean state;
  redo reapplies the complete block. The rectangle remains available for reuse.
- The initiating cursor ends after the last row's actual insertion. Shared
  points and marks follow the complete edit using their established affinities.
  Undo from another pane restores the original initiating view while it is live.
- An absent block, read-only buffer or focused prompt refuses insertion without
  publishing text, padding or lines. Held/released suffixes cannot insert the
  block twice; each fresh invocation creates its own complete history group.

## Implementation Notes

- T34.1: `ec1f99efc1f9d8645c7816819776e662a806774e` adds qualified
  rectangle insertion planning, snapped placement, prefix padding, combined EOF
  extension, canonical publication/cursor ownership, bound/named routing,
  three native cases and product reference. All seven ordered checks passed.
- T34.2: `e061feba7e4b35e844ca905ad164be904e3e2a09` adds eight native
  cases and two real PTY journeys. All seven ordered checks passed, including
  355 native tests, 34 focused rectangle tests and 70 terminal cases.
- The rectangle owner prepares ordered insertions in one BASE revision. It
  qualifies the initial point and uses the existing Coordinates policy for
  each source row. Later row positions are never computed from already edited
  text. Row/point arithmetic uses checked addition.
- Existing row edits preserve all target characters and separators. New rows
  share one EOF span, merged with any existing last-row insertion at the same
  offset. This avoids duplicate base ranges while retaining exact ordered text.
- The plan tracks inserted scalar counts and the final initiating point.
  The session captures every live point, prepares the canonical candidate,
  records the initiating view, maps viewports and publishes once. The buffer
  retains source/read-only admission, grapheme repair and coherent history.
- No full-rope String copy, new dependency or arbitrary source cap is introduced.
  Explicit insertion prepares only the retained fragments, necessary spaces
  and added separators. A deterministic 2050-row fixture verifies one EOF
  replacement, one revision, one history group and complete undo/redo.
- PTY journeys use existing bounded complete-frame waits and owned fixtures.
  They exercise cross-file retained text, exact BOM/newline saves, target
  snapping, shared undo origin and unchanged bytes of untouched tabs.

## Contracts Added or Changed

- The fixed catalog exposes yank-rectangle and the nested C-x r y binding.
  Named and bound invocation share one handler with focused-input admission.
- Nonempty row insertion pads only a short target prefix. Interior wide/tab
  targets snap to the existing source start; empty fragments add no spaces.
  Complete retained tabs use destination tab stops rather than width padding.
- Missing target rows use the buffer's prevailing separator without normalizing
  existing LF/CRLF. All EOF text shares one qualified insertion range.
- A block insertion has one history group and explicit final cursor/view
  ownership. Stored rectangle rows and the linear ring remain independent.
- The rectangle reference and command catalog document placement, line creation,
  refusal and replay. CHANGELOG.md completes the reader-facing rectangle workflow.

## Files of Interest

- `src/rectangle/yank.rs`: qualified base insertions, padding and EOF extension.
- `src/session/rectangle.rs`: shared publication and explicit initiating cursor.
- `src/command/rectangle.rs`, `src/input/rectangle.rs`: named and bound entry.
- `tests/rectangles/yank.rs`: eleven native insertion and history cases.
- `scripts/check-pty.py`: cross-file and shared-pane yank/save/history journeys.
- `docs/reference/rectangles.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing insertion and replay contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 355 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test rectangles`: passed; 34 tests, zero skips. Eleven yank
  cases cover padding and line creation, target edges, bound/named routing,
  LF/CRLF and EOF merging, unequal/tab fragments, Unicode joins, shared views,
  refused/empty-row insertion, large append and repeat/release isolation.
- `python3 scripts/check-pty.py --suite terminal`: passed; 70 cases, zero skips.
  New journeys verify real C-x r y and M-x yank-rectangle, cross-file retained
  text, all-row undo/redo, exact BOM/newline saves, shared initiating-view
  restoration, independent ring and unchanged bytes of untouched target tabs.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
