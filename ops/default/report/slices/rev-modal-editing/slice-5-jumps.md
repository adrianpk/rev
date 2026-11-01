# Slice 5: Jump history and Insert continuation

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-05-jumps`
PR: pending

## Purpose

Return through admitted navigation using checked per-view source locations, and
perform one complete Normal command from Insert without losing prompt ownership.

## Delivered Behavior

- Normal C-o/C-i traverses a view's bounded jump history with checked counts.
  Explicit line, buffer, paragraph and viewport jumps, accepted searches and
  successful file visits record origin/destination pairs. Ordinary character or
  word movement and history traversal do not add entries. A new admitted jump
  after backward traversal discards the forward branch; failure preserves it.
- Live destinations follow native edits and undo/redo. Removed positions become
  unavailable until native history restores them. Closed file destinations need
  matching file identity, content and local source context before reopening.
  Dirty or read-only live owners are reused, native rename carries file identity,
  and missing or replaced files fail without moving the view or history index.
- Splits inherit a bounded history copy and then evolve independently. View
  deletion releases its observations. Jump storage remains session-local and
  independent of bookmark entries, dirty state and persistence.
- Insert C-o enters Normal for one semantic command, including counts, operators,
  character targets, argument prompts and deferred navigation. Compatible
  completion returns to Insert; a different view, source or external revision
  retires the continuation. Prompt/completion Tab ownership is preserved.

## Implementation Notes

T5.1-T5.3 share native location and publication contracts, committed together as
`72577005071349848a7eb8297d8caa568bc2d98a`. T5.4 adds cross-owner safety regressions and
this report. Shared location context was extracted from bookmarks without changing
bookmark search or persistence behavior.

Each view retains at most 256 observations. A location captures at most 64 source
scalars on either side and at most 4096 bytes of file path. Counted traversal
selects one bounded index instead of repeating primitive commands. Weak registry
ownership releases observations with their last view/history owner; maintenance
runs during native actions rather than idle polling.

Native prepared transactions capture location snapshots, include them in history
payload accounting and validate them again before publication. The existing
anchor mapper handles edits, permutations and grapheme-boundary loss without
clamping a stale destination. Undo restores retained observations. Closing a
buffer prepares all fallible evidence before publishing any location changes.
Reopening qualifies every matching closed observation so subsequent dirty edits
continue to rebase all of that source's retained destinations.

Traversal refreshes the current point only when leaving the newest history entry;
older destinations keep their original edit/undo evidence. A fresh valid origin
cannot deduplicate an unavailable observation merely because its old coordinates
match. File reopening checks both admitted metadata and current disk identity
before publication and rejects incomplete or changed evidence.

The one-command continuation records whether a Normal command has actually
started. Native input expiry cannot finish it early. Counts and operators retain
its lifetime, prompts own their component keys, and held layout work finishes
before continuation completion. Already queued later input observes restored
Insert state. Existing input batching, layout budgets and worker ownership remain
unchanged.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md) and
[tracker](../../../tracker/rev-modal-editing.md) retain MOD08-MOD09 and MOD12-MOD13.
The [paired model](../../../spec/editor-modal-editing-model.md) records delivered
location, traversal and continuation ownership. Command and binding inventories
contain 402 canonical commands. The complete sample/defaults include 125 Normal
bindings and 77 Insert bindings, with existing Visual maps preserved. Unreleased
explains checked jump history and temporary Normal commands from Insert.

## Files of Interest

- `src/location.rs`, `src/location/registry.rs` and `src/file/context.rs`: bounded
  native observations, shared checked context, closing, reopening and rename.
- `src/buffer/edit/locations.rs`, `history.rs`, `grouping.rs` and `mapping.rs`:
  prepared edits, ordinary undo/redo and retained payload accounting.
- `src/jump.rs`, `src/view.rs` and `src/session/jumps/`: bounded per-view history,
  checked resolution and semantic Insert continuation.
- `src/session/navigation.rs`, `search/`, `file.rs`, `buffers.rs` and native visit
  owners: recording only admitted nonlocal navigation.
- `tests/modal/jumps.rs` and `jump_safety.rs`: traversal, counts, files, source
  lifetime, split independence, deferred input and component ownership.

## Validation

Exploratory checks passed during implementation:

- `cargo check --locked --quiet` before final safety additions.
- `cargo test --locked --lib --quiet`: 120 top-level library tests passed before
  the final reopening and continuation-deadline changes.
- `cargo test --locked --test modal jump -- --nocapture`: all 17 jump and
  continuation regressions passed after those changes.
- The existing source-check executable passed all 43 changed Rust files and the
  modal test root, including explanatory comments and complexity limits.

The deferred-input fixture uses the existing 4000-line geometry and unchanged
five-second settlement bound so layout is demonstrably pending before queued
text arrives. A smaller source completed synchronously and could not exercise
that assertion. The edit fixture explicitly restores point to EOF after native
replace, whose initiating point follows its insertion. Original source, input
ordering and undo assertions remain intact.

Final exact-HEAD evidence is recorded by Looper and Acta after the PR-reference
commit. The focused batch checks native owners and inventories before the
canonical ordered batch; neither executes the integrated full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --lib
cargo test --locked --test configuration --test commands --test editing --test search --test core --test display --test coordinates
cargo run --locked --quiet --example source-check -- --source src/bookmark/context.rs --source src/buffer.rs --source src/buffer/edit.rs --source src/buffer/edit/bookmarks.rs --source src/buffer/edit/grouping.rs --source src/buffer/edit/history.rs --source src/buffer/edit/locations.rs --source src/buffer/edit/mapping.rs --source src/buffer/lifecycle.rs --source src/buffer/rename.rs --source src/buffer/visits.rs --source src/command.rs --source src/command/modal.rs --source src/command/navigation.rs --source src/file.rs --source src/file/context.rs --source src/input/grammar.rs --source src/input/keymap/defaults.rs --source src/input/keymap/vim.rs --source src/jump.rs --source src/lib.rs --source src/location.rs --source src/location/registry.rs --source src/session.rs --source src/session/bookmarks/navigation.rs --source src/session/buffers.rs --source src/session/events.rs --source src/session/feeds/publication.rs --source src/session/file.rs --source src/session/goto.rs --source src/session/jumps.rs --source src/session/jumps/once.rs --source src/session/jumps/resolve.rs --source src/session/layout.rs --source src/session/links/visit.rs --source src/session/modal.rs --source src/session/navigation.rs --source src/session/search.rs --source src/session/search/continuation.rs --source src/view.rs --source tests/modal.rs --source tests/modal/jump_safety.rs --source tests/modal/jumps.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

Canonical ordered batch:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test bookmarks --test files --test windows --test recovery
git diff --check
```

## Risks and Follow-ups

The controller owns the integrated full gate. Remaining modal states and terminal
acceptance follow their recorded slices. Existing unrelated repository-wide
source-check failures remain outside this slice; every changed Rust file receives
the focused source-quality check above. The in-process checks create no persistent
external fixture or service.
