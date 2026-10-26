# Slice 45: Exercise cross-feature Unicode and file fidelity

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `test/rev-45-adversarial-text`
PR: `#45`

## Purpose

Verify that combined Unicode, history, syntax, shared-view and filesystem
operations preserve exact document content and coherent state at their boundaries.

## Delivered Behavior

- Six file cases cover empty, one-line, LF, BOM/CRLF, mixed separators and long
  lines without a final newline. Each combines shared Unicode paste, literal
  search/replacement, whole-grapheme deletion, saving and a fresh reopen.
- Every resulting edit, undo and redo compares current syntax captures with an
  independent full parse and validates every shared point against its buffer
  stamp and grapheme/cell coordinates. Saved logical state becomes clean again
  after redo; BOM, original separators and final-newline presence remain exact.
- The long-line case selects a unique literal from both sides of an observed
  rope chunk boundary and verifies search and cancellation at scalar positions.
- Twenty-four split/delete/keep cycles alternate nested orientations, suspend
  and restore presentation, retain undo/redo payloads and reject retired view
  identities. Removing a yank owner cannot transfer its replacement authority
  to another view after resize and history replay.
- One shared CRLF rectangle joins a ZWJ emoji to its right neighbor and a
  detached combining mark to its left neighbor. Repeated undo/redo from the
  peer restores whole rows and keeps every point on a valid grapheme boundary.
- Real two-link symlink cycles, unreadable directories and canceled tree
  candidates retain document metadata, BOM/CRLF bytes, current syntax and both
  history arms. A refused occupied rename preserves both files; a later real
  save and history replay remain usable.

## Implementation Notes

- T45.1: `84c23e4f3c1b546574cafbc834bce7adaf2aeb31` adds the file-fidelity matrix and existing syntax-suite helpers.
- T45.2: `24ca6751d3806673884cb0b20d594e3a8c7fa7dc` adds window, rectangle, yank and filesystem boundary regressions
  in their existing subsystem suites and strengthens the unique chunk-seam oracle.
- Fixture correction: `2926c64622fb85451d772a961cf1393b7ea9bd4e` separates per-orientation window execution and
  bounded tree selection. Permission checks restore owned directory access
  before asserting results. All source-quality caps and behavioral assertions
  remain enforced.
- Fixtures use real temporary files, canonical buffer owners, public commands
  and the repository's existing snapshot helpers. No terminal timing or color
  sequence is used as an oracle for native state. The long-line fixture spans
  actual chunks rather than depending on a fixed chunk capacity.
- Six native regression functions are added. The file function runs six named
  document variants; window lifecycle runs both orientations. Existing subsystem
  tests continue to cover normal prompt cancellation and individual edge cases.

## Contracts Added or Changed

- Regression evidence now connects qualified Unicode positions, incremental
  syntax, shared-view mapping and retained saved/history state to exact file
  publication and fresh ingress.
- Filesystem refusal and canceled candidates are checked against full existing
  document snapshots, including redo and file association, rather than text alone.
- The existing safe Rust, five-parameter, three-level nesting and ten-path
  cyclomatic limits apply to the new tests as well as production sources.

## Files of Interest

- `tests/syntax/fidelity.rs`, `tests/syntax.rs`: combined file/syntax/history matrix.
- `tests/windows/resize.rs`: repeated successful view creation and retirement.
- `tests/editing/yank.rs`: refusal after yank-owner retirement and history replay.
- `tests/rectangles/yank.rs`: shared multi-row ZWJ/combining seams with CRLF.
- `tests/recovery/paths.rs`, `tests/recovery.rs`: real path failures and tree
  candidate cancellation with retained buffer state.

## Validation

All five ordered checks passed through Looper on task HEAD `2926c64622fb85451d772a961cf1393b7ea9bd4e` under
validation `K7TBLZE6LGI2RANJVQX3VZXYAV`. Looper records the final clean PR-head run
after the report and PR-reference commits. The dossier's focused command is
already the required `cargo test --all`; it runs once in the ordered list.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 454 native tests, zero ignored.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
