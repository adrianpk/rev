# Slice 36: Traverse directories lazily and safely

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-36-tree-navigation`
PR: `#36`
Integrated: Rebase + Fast-forward into dev at `50ec7694c578e327b272b481b9da7c707a8321cf`.

## Purpose

Navigate visible tree entries, read directory children on explicit expansion,
and refresh observations without losing editor text or usable tree focus.

## Delivered Behavior

- Up/Down or C-p/C-n move through visible entries without wrapping. Right or
  C-f expands a directory. Left or C-b collapses it or selects its visible
  parent. Enter toggles a directory and reports leaf activation without editing.
- Direct children load only when expanding that directory. Directories sort
  first, followed by native filename order, case-sensitive and locale-independent.
  Dot entries remain visible. Collapsing removes loaded descendants; reopening
  observes fresh children. Drawing and visible movement perform no filesystem I/O.
- Symlinks remain leaves, including directory links, cycles, chains and dangling
  links. Expansion revalidates a stale directory before reading its children.
- C-x t r and M-x tree-refresh reload the root and previously expanded branches.
  New directories stay collapsed. Selection follows its path or falls back to
  the nearest remaining visible ancestor when that path disappears.
- An expansion failure reports its operation/path and retains existing entries.
  Refresh collapses a failed branch while reloading accessible siblings. A
  missing root retains a selected collapsed entry and can recover on refresh.
- Selection scrolls above the tree footer and survives geometry changes. Tree
  text uses safe Unicode/control drawing. Navigation, errors and refresh retain
  editing buffers, point, history and the established workspace focus rules.
- Six named navigation commands use the same handlers as focused bindings:
  tree-previous-line, tree-next-line, tree-expand, tree-collapse,
  tree-toggle-directory and tree-refresh. Navigation requires tree focus;
  repeated Enter cannot toggle a directory twice.

## Implementation Notes

- T36.1: `62490595d7d8803365e083d8168f26cf08e261b7` adds observed
  entries, iterative visible navigation, lazy directory reads, refresh,
  selection projection, bound/named routing, four native cases and reference
  updates. All seven ordered checks passed.
- T36.2: `391dfd769d054c173b1794e26ef18ed07e63b5f5`, with permission
  fixture correction `b2d2144e6eaa66c1add562361b6c9f4ff32fb777`, adds eight
  native cases and two real PTY journeys. All seven ordered checks passed on
  the corrected head: 377 native tests, 22 focused tree tests and 74 PTY cases.
- The tree owns a flat visible preorder with path, depth, observed kind and
  expansion state. A direct read completes before inserting children. Refresh
  retains the expanded path set and iterates over only those surviving branches;
  it does not recursively discover unopened directories or follow links.
- State and work grow with explicitly opened branches and their direct entries.
  Refresh reads at most the previously expanded directories still reachable in
  that visible structure. Collapse releases its descendants. Indentation is
  bounded by pane width, and projection draws only available terminal rows.
- Tests exercise 200 explicitly expanded levels, stable raw filename identity,
  symlink variants, denied/deleted directories, missing-root recovery, stale
  directory-to-link replacement and external rename/add/remove reconciliation.
- PTY cases use existing finite frame waits and owned fixtures. They verify
  real arrows, Enter, named/bound refresh, parent transfer, scroll/resize,
  denied reads, removed selections, safe child labels and exact terminal cleanup.
  The two added journeys retain the existing eight-second fixture deadlines.

## Contracts Added or Changed

- The command catalog and tree reference document the six navigation names,
  focused arrow/control aliases and C-x t r refresh binding.
- Entry ordering uses directory-first native filename comparison; display
  replacement text never determines filesystem identity or selection.
- Symlink entries remain non-expandable leaves. Direct expansion revalidates
  the selected path and publishes no partial child list on failure.
- Refresh preserves surviving expanded paths and selection identity, collapses
  failed branches and reports the first failure with its operation/path.
- Visible selection has an independent tree viewport and remains above the
  footer; editor geometry, buffers and history retain their existing owners.

## Files of Interest

- `src/tree.rs`, `src/tree/entry.rs`: observed entries, labels and viewport.
- `src/tree/navigation.rs`, `src/tree/refresh.rs`: visible movement, lazy
  publication, parent/collapse behavior and iterative reconciliation.
- `src/session/tree.rs`, `src/command/tree.rs`, `src/input/tree.rs`: named and
  focused bound routes, admission and repeat suppression.
- `src/display/tree.rs`: selected-row cursor and reverse-video projection.
- `tests/tree/navigation.rs`, `tests/tree/failures.rs`: twelve new native cases.
- `scripts/check-pty.py`: navigation/refresh and selection/error/resize journeys.
- `docs/reference/file-tree.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing traversal, refresh and failure contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 377 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test tree`: passed; 22 tests, zero skips. New cases cover
  ordering, visible traversal, lazy reads, refresh selection, command routing,
  deep branches, links, permissions, deletion, replacement and native names.
- `python3 scripts/check-pty.py --suite terminal`: passed; 74 cases, zero skips.
  New journeys verify real navigation/refresh, collapsed branches, symlink
  leaves, external changes, selected-row scrolling, resize and denied reads.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
