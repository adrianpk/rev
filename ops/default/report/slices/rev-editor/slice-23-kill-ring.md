# Slice 23: Copy kill and yank linear text

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-23-kill-ring`
PR: `#23`

## Purpose

Copy or kill active linear regions into a bounded session ring and insert its
newest entry through one coherent, reversible edit.

## Delivered Behavior

- `C-w` / `kill-region` removes the active ordered region, retaining its exact
  Unicode and newline bytes. Reversed endpoints select the same source range;
  adjacent text remains unchanged. Publication deactivates the region.
- `M-w` / `kill-ring-save` copies the active region without changing document
  text, point, mark, activation, revision, dirty state or undo/redo history.
  Copy also works on read-only source text.
- The session retains at most 32 owned linear entries, newest first. A new copy
  or independent kill evicts the oldest entry when full. Consecutive directional
  kills replace the newest entry with their composed text without spending a slot.
- `C-y` / `yank` inserts the newest entry at point as one checked transaction
  and one undo step, separate from adjacent typing. Shared view points and stored
  mark use the existing edit map; undo/redo removes or restores the entire entry.
- Retained text survives source edits, file visits and closing the source buffer.
  Entries belong to one session. Copying after undo preserves its redo future.
- Missing/inactive regions and empty-ring yank produce visible refusals. Empty
  regions do not add entries or edits. Read-only kill/yank and transient-focus
  refusals preserve document state, selection and existing ring contents.
- Actual terminal journeys exercise copy, kill and yank, separate undo, insertion
  into another visited file, exact CRLF/BOM saves and restored terminal exit.

## Implementation Notes

- T23.1: `e167d8c7cf8d8ee907a09996c146f47655cab0f5` adds the bounded ring,
  linear region handlers, canonical bindings and atomic yank behavior.
- T23.2: `4ee8ffb17378608dd1cf6b391d9c33b995fc0601` adds native boundary,
  retention, shared-view and real terminal evidence.
- The ring owns Rope fragments with a newest-first queue preallocated for 32
  entries. New entries evict from the back; coalescing replaces only the front.
  This owner contains linear text and remains independent of rectangle storage.
- A kill candidate captures its composed text and whether it replaces the newest
  entry before document mutation. Ring publication follows successful checked
  deletion without a further queue allocation; rejection drops the candidate.
- Region calculation validates the selected view's qualified point and shared
  active mark. Both region and directional kills use the same deletion owner.
- Yank builds one replacement payload from the newest retained fragment and
  calls the canonical edit path once. Buffer preparation and publication retain
  their existing admission, history, position mapping and selection behavior.
- Non-kill commands invalidate kill continuity while preserving ring entries.
  Copy preserves document selection and history; each copy starts a new entry.

## Contracts Added or Changed

- `Command::Linear` registers implemented `kill-region`, `kill-ring-save` and
  `yank` commands with `C-w`, `M-w` and `C-y` bindings.
- `Session::kill_ring` borrows at most 32 entries in newest-first order through
  an exact-size, double-ended iterator. `killed_text` returns the newest entry.
- The retained-text owner now stores a bounded queue rather than one entry;
  its revision/view/point-qualified directional continuity remains enforced.
- Native linear handlers respect prompt, inventory and decision ownership.
  Copy requires an active region but no document write authority.
- `CHANGELOG.md` documents region copy/kill, yank and session retention.

## Files of Interest

- `src/kill.rs`: bounded owned entries and composed kill publication.
- `src/session/linear.rs`: checked region copy/kill and single-edit yank.
- `src/session/kill.rs`: shared kill path and borrowed ring access.
- `src/command/linear.rs`, `src/command.rs`: native command names and registry.
- `src/input/bindings.rs`, `src/session.rs`: keys, focus and continuity routing.
- `tests/editing/linear.rs`, `tests/commands.rs`: native semantic evidence.
- `scripts/check-pty.py`: actual keys, cross-file yank, saves and cleanup.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 230 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test editing`: passed; seventy-seven tests, zero skips. Fourteen
  linear cases verify unchanged copy state, full/partial/reversed regions,
  single-publication yank and undo/redo, empty ring, 32-entry eviction/coalescing,
  absent/empty regions, read-only and transient focus, redo preservation, shared
  view mapping, source closure, session isolation and fragments spanning chunks.
- `python3 scripts/check-pty.py --suite terminal`: passed; forty-six cases,
  zero skips. New journeys use C-w/M-w/C-y, empty-ring refusal, atomic yank undo,
  cross-file insertion and exact Unicode/CRLF/BOM saves. Owned fixtures stop
  after their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
