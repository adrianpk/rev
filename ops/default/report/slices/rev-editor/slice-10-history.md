# Slice 10: Restore per-buffer undo and redo

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-10-history`
PR: `#10`

## Purpose

Restore retained per-buffer edits coherently while keeping publication revisions,
logical history states, live positions and clean-state acknowledgment distinct.

## Delivered Behavior

- `undo` reverses one retained transaction and `undo-redo` reapplies it. Both
  publish fresh revisions, preserve exact Unicode/line-ending bytes and update
  canonical text, mark, live points and history together.
- `C-x u` and reported `C-/`/`C-_` reach the common undo handler. Empty history
  navigation preserves confirmed state and displays `Nothing to undo` or
  `Nothing to redo`. Read-only buffers reject either mutation direction.
- Undo restores the recorded initiating cursor to the caller-selected live
  initiating or invoking point. Other live points map from their current
  positions rather than reverting to old snapshots.
- Returning to a retained saved logical state restores the clean indication
  despite newer revisions. An exact current-revision save acknowledgment moves
  that marker without discarding undo or redo history.
- Successful new edits discard redo. Canceled preparation and rejected edits
  preserve it. A new branch receives fresh logical identities, even when its
  bytes equal a discarded state, preventing false clean acknowledgments.
- Undoing all initial edits makes the buffer clean and allows ordinary guarded
  exit. History remains independent across buffers and coherent under retention.

## Implementation Notes

- T10.1: `ca57d28a6027aca29186077a8932bcc24aaf5e69` introduces logical
  state identity, replay publication and command integration.
  `20923bfd678a19798f86212c9cd61592819ce0c6` separates history admission from
  replay preparation to retain the required complexity bound.
- T10.2: `fdf16328e74c9c90571ee494d5cc8413013ce155` completes repeated
  navigation, branching, saved-state, position, retention and terminal coverage.
- Replay builds a private rope candidate from retained inserted/removed
  fragments and publishes through the existing checked edit owner. Fragment
  insertion traverses borrowed chunks rather than flattening document text.
- Inverse changes preserve ordering even when adjacent deletions become
  insertions at the same source offset. CRLF accounting reuses merged seam
  windows, and point adjustment retains valid grapheme boundaries.
- Queue capacity, source revision, logical state, read-only state and live
  positions are admitted before confirmed text changes. Queue transfer and
  publication contain no recoverable work after the retained record is moved.
- Original transaction stamps remain historical provenance. Each replay uses
  a new buffer stamp, so stale ranges and prepared edits remain stale even
  after returning to an equal logical state.
- Both history queues share the existing retained payload budget. Moving an
  operation between queues neither duplicates its accounting nor prunes part
  of it. Discarding redo subtracts its retained payload before new-edit pruning.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `State` is a buffer-qualified logical identity, independent of `Stamp`.
  `Buffer::is_modified` compares current and saved states; `mark_saved` accepts
  only the exact current stamp and preserves both history queues.
- `Buffer::navigate` accepts undo/redo direction, every current live point and
  the current index of the point receiving initiating-cursor restoration.
- Transactions retain original before/after states and initiating positions.
  Replay maps non-initiating points independently and restores recorded mark.
- `Command` registers `undo` and `undo-redo`; native named dispatch and undo
  bindings use the same checked session handler and reset navigation continuity.
- `CHANGELOG.md` records undo, redo and recovery of the clean indication.

## Files of Interest

- `src/buffer.rs`: qualified logical states and saved-state acknowledgment.
- `src/buffer/edit.rs`, `src/buffer/edit/history.rs`: preparation, publication,
  history admission, replay and queue transfer.
- `src/buffer/edit/mapping.rs`: initiating cursor metadata and mapping.
- `src/session/history.rs`, `src/command.rs`, `src/input/bindings.rs`,
  `src/input.rs`: native command and binding integration.
- `tests/editing.rs`: thirty-two transaction and editing integration tests.
- `scripts/check-pty.py`: actual undo encodings and clean ordinary exit.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; eighty-seven tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test editing`: passed; thirty-two tests, zero skips. Coverage
  includes repeated undo/redo, saved logical states, exact-stamp acknowledgment,
  redo branching/cancellation, current non-initiating positions, invalid and
  read-only admission, independent buffers, compound inverses, chunk seams,
  stale preparations and traversal of all 512 retained operations.
- `python3 scripts/check-pty.py --suite terminal`: passed; twelve cases, zero
  skips. The added journey checks `C-x u`, actual CSI-u reports of `C-/` and
  `C-_`, unavailable history, visible Unicode/cursor/dirty updates, return to
  clean state and ordinary exit with terminal restoration. Owned fixtures stop
  after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
