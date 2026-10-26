# Slice 37: Open and create tree entries

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-37-tree-create`
PR: pending

## Purpose

Open tree files through canonical buffer ingress and create named files or
directories without replacing existing data or discarding editor text.

## Delivered Behavior

- Enter, C-x t o and M-x tree-open open selected files through the same pipeline
  as find-file. Existing canonical or symlink visits reuse their live buffers,
  including modified text and history. A successful open returns to the retained
  editing window; an error keeps the tree usable and preserves existing text.
- On a directory, Enter and tree-open retain expansion/collapse behavior.
  Symlinks remain tree leaves; file links use normal canonical ingress, while
  a directory-link open reports the regular-file requirement.
- C-x t n or M-x tree-create-file prompts for an empty file's name. C-x t d or
  M-x tree-create-directory prompts for a directory's name. Each request captures
  the selected directory; file and symlink selections refuse creation rather
  than silently choosing another location.
- Names must be one nonempty normal filename component. Controls, whitespace-only
  names, dot/parent components and path separators are refused. Unicode and
  spaces remain valid. Existing files, directories and symlinks, including
  dangling links, are never replaced.
- Publication revalidates the parent and uses exclusive file creation or
  directory creation. Cancellation writes nothing. A failed operation identifies
  its path/operation and preserves document text, point and history.
- Successful creation expands the parent and selects the new entry while
  retaining workspace focus. It creates no editable buffer until explicitly
  opened. If creation succeeds but observation fails, the echo reports Created
  and the separate refresh failure rather than concealing the disk result.
- Opens and published saves refresh already loaded parent directories. Collapsed
  branches stay lazy. Refresh errors preserve the successful file outcome and
  report the separate tree error. Unsupported extensions retain plain Text mode.
- Name prompts preserve their captured location across narrow/restore geometry.
  C-g cancels back to the retained workspace owner; Tab retains the draft and
  shows a name-entry hint. Auto-hide continues to release tree focus safely.

## Implementation Notes

- T37.1: `215ef188ed4494036f1d193a61ef85532bb816f2`, with command-name
  separation `6f72e3bd1b93883aec5b68529f009dcb0cd770ec`, adds canonical
  opening, captured creation requests, exclusive publication, reconciliation,
  bound/named routes, four native cases and reference updates. All seven
  ordered checks passed on the corrected head.
- T37.2: `0abd8a723fdc6fe47e440f6c07b549353580e3a8` adds seven native
  cases and two real PTY journeys. All seven ordered checks passed: 388 native
  tests, 33 focused tree tests and 76 terminal cases.
- The filesystem creation request owns its directory and kind until consumed.
  Validation rejects invalid components and occupied entries before publication;
  create_new and create_dir also reject a later destination conflict.
- The session retains that request in the ordinary argument prompt owner. Tree
  opening delegates directly to Session::load, keeping canonical identity,
  decoding, buffer reuse, history and selected-window publication in one owner.
- New-entry reconciliation preserves the distinction between published disk
  creation and a failed observation. Open/save synchronization uses the existing
  bounded refresh of previously expanded branches, with no eager hidden scan.
- Native cases cover both creation kinds, name/conflict classes, captured parent
  changes, denied writes, leaf admission, published creation with denied refresh,
  canonical dirty-buffer reuse and filesystem name-length rejection.
- PTY cases use existing complete-frame waits and owned temporary fixtures.
  They verify actual command prompts, creation/open/save, alias reuse and undo,
  nested creation, cancellation, invalid names, parent-link replacement, denied
  writes and prompt resize. Both retain the existing eight-second deadlines.

## Contracts Added or Changed

- The registry adds tree-open, tree-create-file and tree-create-directory with
  Enter/C-x t o, C-x t n and C-x t d routes into the shared handlers.
- Tree file opening delegates to canonical ingress; it does not install a
  separate tree-owned text copy or discard an existing modified buffer.
- Creation requires a directory selection and a validated single name. Captured
  requests revalidate their parent and never overwrite an occupied destination.
- Creation and refresh have separate outcomes. New entries are selected without
  inventing a buffer, and successful opens/saves survive tree refresh errors.
- The tree reference, command catalog and changelog document opening, creation,
  cancellation, focused input and loaded-parent synchronization.

## Files of Interest

- `src/tree/create.rs`: captured requests, validation, exclusive publication
  and new-entry/loaded-parent observation.
- `src/session/tree/create.rs`: opening, name prompts and publication feedback.
- `src/session/file.rs`, `src/session/save.rs`: shared ingress and published-save
  synchronization with visible tree observations.
- `src/command/tree.rs`, `src/input/tree.rs`: named and bound entry routes.
- `tests/tree/create.rs`, `tests/tree/create_failures.rs`: eleven native cases.
- `scripts/check-pty.py`: creation/open/save/reuse and refusal/resize journeys.
- `docs/reference/file-tree.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing opening and creation contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 388 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test tree`: passed; 33 tests, zero skips. New cases cover
  creation kinds, names/conflicts, prompts, captured parents, permissions,
  published outcomes, canonical reuse and filesystem name-length rejection.
- `python3 scripts/check-pty.py --suite terminal`: passed; 76 cases, zero skips.
  New journeys verify bound/named creation, canonical opening, Text fallback,
  dirty-buffer/history reuse, saves, cancellation, refused writes and resize.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
