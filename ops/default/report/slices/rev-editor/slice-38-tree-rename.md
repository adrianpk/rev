# Slice 38: Reconcile visited paths after tree rename

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-38-tree-rename`
PR: `#38`

## Purpose

Rename selected entries with conflict checks and keep canonical visiting paths,
labels, later saves and loaded tree observations aligned with disk success.

## Delivered Behavior

- C-x t R and M-x tree-rename prompt for a new name at the selected entry's
  current parent. The echo identifies the captured source. The existing single
  filename-component policy applies; C-g cancels without changing disk or
  visiting associations. Rename requires tree focus.
- Existing destination files, directories and symlinks are refused, including
  dangling links. Live intended destinations are also reserved; a directory
  target checks every live reservation under that path prefix. Source identity
  and destination conflict are checked again immediately before filesystem rename.
- Only successful disk rename updates canonical visiting paths and the path
  index. A renamed directory maps all visited descendants by path components,
  including intended missing files. Similarly prefixed sibling paths stay intact.
- Buffers preserve text, dirty/saved states, revision, mark and history. Editing
  windows remain live. File display labels follow the new basename and remain
  unique; directory descendants retain their distinct existing labels. Later
  visits reuse these owners and later saves write their actual new destinations.
- Renaming a symlink moves the link entry while canonical referent buffers keep
  their paths. File links, directory links and dangling links retain their target
  text; no recursive traversal through links is introduced.
- Loaded tree paths, expanded branches and selection follow the moved subtree
  before ordinary refresh, including a renamed root. A refresh failure reports
  the separate error while preserving successful disk publication and associations.
- Observation updates permit rename's expected ctime change only when prior
  and current identity/content/permissions agree. Pre-existing external changes
  remain subject to normal later-save confirmation. An obsolete save capture
  cannot recreate its old destination after the association changes.
- Ordinary conflict, permission or stale-source failures retain existing text
  and associations. Name prompts also retain their captured owner through narrow
  geometry and cancellation without leaking the draft into document text.

## Implementation Notes

- T38.1: `4960ae4b778a5da24a6b4f9426d765a74cd507ec` adds captured
  rename requests, preflight path/owner checks, post-success reconciliation,
  observation handling, bound/named entry, four native cases and reference
  updates. All seven ordered checks passed.
- T38.2: `063ffc65aaa8a553e8eeb7d5260a4661ccd55f9d` adds nine native
  cases and two real PTY journeys. All seven ordered checks passed: 401 native
  tests, 46 focused tree tests and 78 terminal cases.
- The filesystem request canonicalizes the source parent while retaining a
  symlink entry's identity. It captures device/inode and validates one sibling
  destination. It checks the source and target again before std::fs::rename.
- The buffer owner prepares path, label and prior-observation changes before
  disk mutation. After success, it applies the prepared associations and
  rebuilds the canonical index without changing rope state or view identity.
- Work is bounded by live visiting buffers and already loaded tree entries.
  Directory reconciliation computes path suffixes rather than recursively
  scanning unvisited source files. Tree refresh retains the existing lazy bounds.
- A post-rename observation error retains the new path and reports a warning.
  Existing unresolved warnings retain their history and identify the new path.
  Rename does not grant consent to overwrite independently changed disk text.
- Native cases exercise equal labels, link kinds, denied rename, stale source
  and destination captures, external-change consent, obsolete saves, reserved
  directory targets, root rebasing and refresh failure after publication.
- PTY cases use existing finite complete-frame waits and owned fixtures. They
  verify actual bound/named prompts, directory descendants, unchanged sibling
  owners, later new-path saves, cancellation/resize, invalid/occupied names,
  denied mutation, link semantics and source replacement. Both retain the
  existing eight-second fixture deadlines.

## Contracts Added or Changed

- The registry adds tree-rename and the uppercase C-x t R binding. Creation and
  rename share the literal single-component name policy.
- Captured source identity, occupied disk targets and live destination owners
  are checked before publication; failures do not relocate associations.
- Successful directory rename maps canonical descendant paths, while symlink
  rename preserves referent associations and all text/view/history owners.
- Labels and the canonical index follow successful publication. Old save
  captures become inadmissible and fresh saves retain external-change guards.
- Tree selection/root/expanded paths follow the destination; observation or
  refresh failure remains distinct from a disk rename that already succeeded.
- The file-tree reference, command catalog and changelog describe the delivered
  rename, conflict, association and later-save behavior.

## Files of Interest

- `src/file/rename.rs`, `src/file/name.rs`: captured entry identity, name and
  destination validation, filesystem publication and explicit success outcome.
- `src/buffer/rename.rs`: preflight ownership, descendant mapping, labels/index
  publication and preserved observation guards.
- `src/file/location.rs`, `src/file/observation.rs`: relocation metadata and
  comparison of expected rename observations.
- `src/tree/rename.rs`, `src/session/tree/rename.rs`: tree rebasing, name prompts
  and feedback separating publication from observation failures.
- `tests/tree/rename.rs`, `tests/tree/rename_failures.rs`: thirteen native cases.
- `scripts/check-pty.py`: renamed-path save and refusal/link/resize journeys.
- `docs/reference/file-tree.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing rename and association contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 401 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test tree`: passed; 46 tests, zero skips. New cases cover
  files/directories, marks/history, labels/indexes, prompts, symlinks, permissions,
  stale captures/saves, external changes, reserved targets and root/refresh behavior.
- `python3 scripts/check-pty.py --suite terminal`: passed; 78 cases, zero skips.
  New journeys verify real rename prompts, descendant and equal-basename owners,
  later saves to new paths, cancellation, denied writes and stale-source refusal.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
