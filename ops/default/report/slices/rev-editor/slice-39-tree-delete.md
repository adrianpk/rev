# Slice 39: Confirm deletion while preserving open text

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-39-tree-delete`
PR: pending

## Purpose

Delete selected tree entries with explicit decisions, preserving open text and
reconciling actual removed visiting paths under D07, including partial failure.

## Delivered Behavior

- C-x t D and M-x tree-delete identify the selected path and ask for y/n.
  A nonempty directory requires another explicit decision identifying its
  children and recursive removal. Canceling either stage leaves disk intact.
  Tree focus is required; decisions retain their captured entry through resize.
- Source device/inode is rechecked at each decision and immediately before
  removal. An empty directory receives only nonrecursive removal authority;
  later children cannot silently acquire permission for recursive deletion.
- File, directory and dangling symlinks remove only the link entry. Recursive
  removal does not traverse child symlink referents or relocate their buffers.
- Open buffers retain their text, revision, logical state, mark and history;
  shared editing views retain their points. Actual absent visiting paths become
  missing associations. Their warnings keep close/quit guarded even for text
  that was clean before deletion. Canonical path reservations remain live.
- Revisiting an absent path reuses its buffer and does not create a file.
  Explicit save can recreate retained text; missing parents are not created.
  Old save captures are invalidated, and an external new occupant still requires
  a fresh overwrite decision. Successful save clears the missing warning.
- Recursive removal can fail after some children were deleted. The error
  identifies the operation/path and explicitly warns of possible partial removal.
  Only observed absent visiting paths become missing; other observations retain
  associations. Tree refresh runs after success or failure and preserves usable
  branches/ancestor selection. Refresh errors remain separate from disk results.
- Decision frames use the echo cursor; invalid keys and bracketed paste retain
  the question, and repeated keys cannot approve a decision. Failure feedback
  puts the partial-removal notice first so path length cannot hide that fact.

## Implementation Notes

- T39.1: `e83b50ec10f31e6129f48b9ad6e6cb5bf1252d28` adds captured
  deletion, two-stage decisions, filesystem publication, missing associations,
  command dispatch, four native cases and reader references. All seven ordered
  checks passed: 405 native tests, 50 focused tree tests and 78 terminal cases.
- T39.2: `fa5d92ce59fc2608d028bb01027a2429c731cc11` adds six native
  failure cases and two real PTY journeys, mark assertions and visible partial
  failure feedback. All seven ordered checks passed: 411 native tests,
  56 focused tree tests and 80 terminal cases.
- The file owner uses standard-library entry inspection/removal, including
  remove_dir_all only after the separate recursive decision. Capture resolves
  the parent while preserving a selected symlink's own entry identity.
- Buffer reconciliation is bounded by live visiting buffers under the captured
  path prefix. It inspects actual paths after the filesystem attempt, retaining
  canonical reservations and changing no rope or view identity. Observation
  errors are reported rather than treated as absence. Existing warnings survive.
- Ordinary tree refresh retains its lazy loaded-branch bounds. Recursive disk
  removal necessarily visits the explicitly approved subtree; it never loads
  unvisited document text into editor buffers and makes no rollback claim.
- Native failures use real permission-denied operations. The partial-removal
  fixture denies the last child in observed Linux directory order and requires
  proof that some children were removed and others remain. Assertions inspect
  each actual path, retained text and refreshed entries rather than assuming
  all-or-nothing publication.
- PTY fixtures exercise real bound/named decisions, refused paste, cancellation
  at both stages, resize, denied recursion, child links, shared views, undo/redo,
  absent revisit and explicit recreation. Existing eight-second fixture deadlines
  and owned cleanup remain unchanged.

## Contracts Added or Changed

- The command registry adds tree-delete and uppercase C-x t D. Nonempty
  directories require a distinct recursive confirmation; stale identity or
  newly arrived children cannot widen prior consent.
- Missing visiting associations retain their canonical owners and full ropes.
  A missing warning guards lifecycle closure and invalidates old save captures;
  explicit later save owns recreation and external-change admission.
- Deletion results distinguish complete removal from filesystem errors that
  may follow partial child removal. Association observation and tree refresh
  report their own errors without denying removals that already happened.
- File-tree and command references plus Unreleased notes describe the delivered
  deletion, confirmation, retained-text and explicit-save behavior.

## Files of Interest

- `src/file/delete.rs`: captured entry identity, staged consent and removal.
- `src/buffer/delete.rs`, `src/file/location.rs`: actual-absence reconciliation,
  retained reservations and explicit-save warning metadata.
- `src/session/tree/delete.rs`, `src/tree/delete.rs`: selected entry decisions,
  focused feedback and refresh after filesystem outcomes.
- `src/session.rs`, `src/session/save.rs`, `src/session/tree.rs`: pending decision,
  echo cursor, cancellation and resize ownership.
- `tests/tree/delete.rs`, `tests/tree/delete_failures.rs`: ten native cases.
- `scripts/check-pty.py`: two deletion journeys through the actual executable.
- `docs/reference/file-tree.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing deletion and recreation contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 411 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test tree`: passed; 56 tests, zero skips. New cases cover
  retained clean/dirty text and marks, explicit recreation, recursive consent,
  cancellation, late children, symlinks, denied/stale operations, partial removal,
  external recreation and missing-root recovery.
- `python3 scripts/check-pty.py --suite terminal`: passed; 80 cases, zero skips.
  New journeys cover bound/named decisions, both cancellation stages, resize,
  failed recursion, link referents, shared views/history and explicit later saves.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
