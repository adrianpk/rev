# Slice 33: Kill rectangles as one transaction

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-33-rectangle-kill`
PR: `#33`
Integrated: Rebase + Fast-forward into dev at `80ae94eb8b3b71999e79a8d00cfa23873be4f5c9`.

## Purpose

Remove and retain a display-column rectangle through one checked transaction,
with coherent shared-view mapping and one undo/redo operation across all rows.

## Delivered Behavior

- C-x r k and M-x kill-rectangle share the implemented rectangle handler.
  Both require active rectangle mode and a nonzero display-column span, and
  retain the removed rows in the existing separate rectangle store.
- Row removal follows the same geometry as copy: touched wide Unicode clusters
  remain whole and partially selected tabs contribute selected cells as spaces.
  The unselected cells of a touched tab remain as spaces in source text;
  untouched tabs retain their original bytes.
- Short and empty source rows receive no invented padding. LF/CRLF separators
  remain intact. Rows with no selected existing text contribute retained empty
  fragments but no text replacement.
- Every changed row publishes together. One undo restores all original text,
  the stored mark, initiating cursor and retained clean state. Redo reapplies
  the entire block. The separate retained rectangle survives undo/redo.
- The initiating cursor moves to the top-left source row/column, following any
  retained tab-prefix spaces. Other live shared points, marks and viewport line
  anchors map through the complete transaction. Undo from another pane restores
  the original initiating view when it remains live.
- Read-only, inactive, zero-width and prompt-owned refusals preserve source text,
  history and the previously retained rectangle. Invalid prepared row authority
  cannot publish a partial operation. The linear kill ring remains independent.

## Implementation Notes

- T33.1: `cc6c158a59fd71ab025eb01b96264ff21de04fc1` adds base-qualified
  row removals, retained tab cells, explicit cursor/origin ownership, canonical
  publication, shared bound/named routing, three native cases and product docs.
  All seven ordered checks passed on that task head.
- T33.2: `e99ace6bb3d6841e9e5cdd2196ba1a091091cedc` adds eight native
  cases and two real PTY journeys. All seven ordered checks passed, including
  344 native tests, 23 focused rectangle tests and 68 terminal cases.
- The rectangle owner creates at most one ordered, disjoint Replacement for
  each changed source row. Every scalar range belongs to the same BASE revision.
  Each replacement contains only the unselected spaces of partially touched
  edge tabs; line separators never enter those ranges.
- The session completes block extraction, row preparation, all live-point
  capture, candidate construction and viewport mapping before publication.
  It installs the retained rectangle only after successful canonical publication.
  Dropping any rejected candidate preserves the old store and source state.
- The existing buffer edit owner supplies read-only/source admission, private
  rope candidates, grapheme repair, mark affinity, one history group and replay.
  The operation records the initiating view and final cursor explicitly.
- Native invalid-row tests supply a foreign, stale, out-of-range or overlapping
  final replacement after valid earlier rows. Additional publication tests change
  read-only or selection authority after preparation and verify no source change.
- A deterministic 4098-row fixture includes 4096 mixed-width middle rows and
  verifies one revision, one history group, retained rows and whole undo/redo
  while the viewport starts scrolled. Work remains proportional to selected
  existing text; no new dependency or arbitrary source-size cap is introduced.

## Contracts Added or Changed

- The fixed catalog exposes kill-rectangle and the nested C-x r k binding.
  Held/released suffix events cannot repeat the operation or insert a literal k.
- Copy and kill use the same selected parts. Partial tabs retain their actual
  unselected cells, while whole selected clusters and complete tabs are removed
  without splitting Unicode or changing untouched tab bytes.
- All row edits use one declared BASE coordinate space and one canonical
  publication. Source-qualified and late admission failures leave every row
  unchanged and do not replace the prior rectangle.
- Rectangle kills end typing/linear-yank continuity and remain one independent
  undo group. Retained rectangle text stays separate from the linear ring.
- The rectangle reference and command catalog document removal, cursor placement,
  refusal and history behavior. CHANGELOG.md extends the existing rectangle
  workflow with the delivered kill and whole-block undo/redo behavior.

## Files of Interest

- `src/rectangle/kill.rs`: base row replacements and final top-left cursor.
- `src/rectangle.rs`: selected tab cells and retained edge-space geometry.
- `src/session/rectangle.rs`: preparation, shared publication and store handoff.
- `src/command/rectangle.rs`, `src/input/rectangle.rs`: named and bound entry.
- `tests/rectangles/kill.rs`: eleven native kill/atomicity cases.
- `scripts/check-pty.py`: bound/named kill, shared undo and exact save journeys.
- `docs/reference/rectangles.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing removal and history contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 344 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test rectangles`: passed; 23 tests, zero skips. Eleven kill
  cases cover atomic rows/history, tab edges/cursor, bound/named routing,
  Unicode/CRLF, short rows, shared views, refused/invalid admission, large
  row counts and repeat/release isolation.
- `python3 scripts/check-pty.py --suite terminal`: passed; 68 cases, zero skips.
  New journeys verify real C-x r k and M-x kill-rectangle, all-row undo/redo,
  shared initiating-view restoration, independent ring, exact BOM/CRLF saves,
  saved-state replay and unchanged bytes of untouched tabs.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
