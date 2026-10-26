# Slice 24: Rotate only the current yank range

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-24-yank-pop`
PR: `#24`

## Purpose

Rotate older linear ring entries through precisely the current yank insertion,
with qualified replacement authority and coherent per-command undo.

## Delivered Behavior

- `M-y` / `yank-pop` replaces the immediately preceding yank or yank-pop range
  with the next older ring entry. Repeated rotation wraps through retained entries
  without reordering the ring or changing text outside that insertion.
- A successful yank captures its buffer identity, initiating view, text revision,
  literal inserted scalar range and ring position. Each successful pop refreshes
  that capture for the replacement's new length and revision.
- The captured range follows inserted bytes rather than the adjusted cursor.
  Combining accents, ZWJ emoji or CRLF formed with adjacent text preserve those
  neighboring bytes across replacement and undo.
- Any other command, input notice/prefix, typing, native edit request, view
  selection or file visit invalidates replacement authority. Returning point or
  text to its prior value does not revive it. Failed native edit/visit requests
  also end continuity.
- An absent or invalidated capture produces a short visible refusal while
  preserving text, points, mark, history and ring entries. Native dispatch also
  respects transient input ownership.
- Each successful rotation publishes one checked replacement and one undo step.
  Shared view points map independently across unequal entry lengths. Undo/redo
  restores complete payloads and logical states without restoring yank authority.
- A fresh yank starts again at the newest entry and captures only its own new
  insertion. Geometry changes and ignored key releases preserve an otherwise
  immediate sequence; actual repeated meta commands rotate again.

## Implementation Notes

- T24.1: `a538935b48083ba5806434472300f0d7b8d4ab07` adds qualified capture,
  canonical M-y dispatch, exact-range rotation and history/join evidence.
- T24.2: `4c2675d43c564170a1c2a433b1f01e110ee5c3ba` adds invalidation,
  shared-view, capacity, event and actual terminal evidence.
- The session owns an optional capture whose span contains buffer identity and
  revision. Pop consumes the old capture before validating its selected view and
  stamp, preventing ordinary failure from leaving stale replacement authority.
- Ring selection uses checked circular indexing. Entry order remains immutable
  during rotation; only the sequence's captured position advances.
- Yank and pop share one publication helper. It computes the literal inserted
  range before mutation, calls the canonical edit path once and installs the new
  capture only after successful publication.
- Existing buffer preparation still owns source admission, history and shared
  position mapping. No text mutation bypasses that owner.
- Command boundaries retain capture only for pop. Native editing, visiting and
  selecting entry points clear it independently of decoded keyboard commands.

## Contracts Added or Changed

- `LinearCommand::Pop` registers the implemented `yank-pop` command and M-y key.
- Session yank continuity is qualified by buffer, view, revision, scalar range
  and ring position. A fresh insertion resets the ring position to newest.
- Exact inserted ranges remain independent of grapheme-adjusted point positions,
  including edits that form a cluster or CRLF across an insertion boundary.
- Successful rotation refreshes capture; unrelated activity and refusal consume
  it. History navigation never reconstructs that temporary authority.
- `CHANGELOG.md` documents immediate rotation and its command boundary.

## Files of Interest

- `src/session/yank.rs`: qualified capture, validation and shared publication.
- `src/session/linear.rs`: pop dispatch, messages and focus ownership.
- `src/session.rs`: command and input continuity boundaries.
- `src/session/edit.rs`, `src/session/file.rs`, `src/session/windows.rs`:
  native edit, visit and view-selection invalidation.
- `src/kill.rs`: checked older-entry selection without ring reordering.
- `src/command/linear.rs`, `src/input/bindings.rs`: name and canonical binding.
- `tests/editing/yank.rs`, `tests/commands.rs`: native semantic evidence.
- `scripts/check-pty.py`: actual rotation, undo and shared-pane journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 244 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test editing`: passed; ninety-one tests, zero skips. Fourteen
  yank cases verify unequal ranges, circular rotation, history and fresh yanks,
  exact Unicode/CRLF joins, absent captures, command/input/edit/view/target
  invalidation, shared mapping, key phases, resize continuity and 1/32-entry rings.
- `python3 scripts/check-pty.py --suite terminal`: passed; forty-eight cases,
  zero skips. New journeys exercise actual M-y, unequal Unicode entries, undo,
  fresh insertion scope, shared-view mapping and refusal after another-pane edit
  or history navigation. Exact saved bytes and terminal restoration are checked.
  Owned fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
