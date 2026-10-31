# Slice 1: Named bookmark commands

Status: delivered
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-01-commands`
PR: [#185](https://forge.adrianpk.com/adrianpk/rev/pulls/185)

## Purpose

Deliver BMK01 and the initial naming and set/jump/delete portions of BMK04-BMK05
through native session commands and the existing canonical file visit boundary.

## Delivered Behavior

Set, replace, jump to and delete named file positions within a session.
`bookmark-set-no-overwrite` refuses an occupied name. Replacement retains a stable
record identity; deletion and recreation allocate a different identity. Multiple
names can refer to one file or point. Names are case-sensitive, retain spaces,
and accept at most 256 UTF-8 bytes without controls. The collection admits 1024
records and paths of at most 4096 encoded bytes without silent eviction.

`C-x r m`, `C-x r M` and `C-x r b` invoke set, no-overwrite and jump.
`bookmark-delete` is available through M-x or configured keys. The visited
basename is the empty set-draft default. Vertical completion uses configured
matching, candidate limits and minibuffer keys. Tab fills the selected name;
Return accepts a typed name or an explicitly navigated candidate. Errors retain
the draft; cancellation and resize preserve the source location.

## Implementation Notes

The session owns one native collection independently of source text, editing
marks and views. A record holds its canonical native path and revision-qualified
scalar point. Prepared canonical visits, point admission and view reveal complete
before the destination view is published. Aliases reuse the live buffer,
including unsaved text, and explicit tree roots remain selected. Read-only,
modified and intended new file sources are eligible; unnamed and synthetic
sources cannot create records. No bookmark command writes source bytes or adds
a source undo transaction.

A changed source revision or a closed source buffer is rejected as stale in
this increment. No position is silently clamped or guessed. A removed disk file
still uses its canonical live buffer while that owner remains available.

## Contracts Added or Changed

Only the four implemented bookmark commands enter the compiled catalog.
Existing rectangle chords remain intact. Name completion shares the current
prompt focus, effective keymap, geometry and rendering facilities; painting
performs no bookmark filesystem operations. Available bindings, commands,
prompt behavior, the complete startup sample and Unreleased notes are current.

## Files of Interest

- `src/bookmark.rs`: bounded collection, stable record identity and admission.
- `src/session/bookmarks.rs`: native prompts, mutations and checked visits.
- `src/input/names.rs`, `src/display/completion.rs`: vertical name completion.
- `src/command/bookmark.rs`, `src/input/keymap/defaults.rs`: catalog and chords.
- `tests/bookmarks.rs`: native workflow, limits, identity and preservation coverage.
- `docs/reference/commands.md`, `bindings.md`, `prompts.md`, `config.toml`:
  current public controls and complete factory configuration.
- `docs/reference/defaults.md`: complete 74-entry editor keymap reference.

## Validation

Development check passed all 11 native bookmark tests:

- `cargo test --locked --test bookmarks`

Coverage includes replacement/refusal, deletion/recreation, multiple names,
Unicode names and paths, literal spaces and case, byte and collection caps,
canonical aliases, modified/read-only/intended sources, synthetic result refusal,
source bytes/dirty state/marks/undo preservation, stale-origin retention,
basename defaults, native chords/M-x, vertical selection/Tab/cancellation,
zero-size resize, configured matching modes and custom minibuffer bindings.
A separate focused `cargo test --locked --test bookmarks synthetic_source` passed
after the fixture explicitly completed both the query and source-selection stages.

T1.1 is commit `156b405c312cede11ab4df164fdccd3f5f6307ce`.
T1.2 is commit `78f45a805a751b5fa4a4dac59cb679c2cd073454`.
The canonical PR's final clean HEAD is validated by the controller, which stores
its exact-candidate receipt outside the repository. Its ordered check list is:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test commands --test files --test configuration`
3. `git diff --check`

The documentation checker initially rejected the old 71-entry editor map count;
the reference now records all 74 entries including the three bookmark chords.
An additional controller-owned `cargo run --locked --quiet --example documentation-check`
batch checks the current public references before the required batch.
The worker hands off only after both batches succeed. The combined
delivery-set gate remains assigned to the controller after all six integrations.

Canonical PR #185 merged through Rebase + Fast-forward into `dev` at
`8a77b4857940bc34310434657ed0fa259f5f7a4f`. Controller receipt
`22OPS6UBYMWKZ5L2JUQOVXTVZI` passed the exact three-command list above at that
clean head: 379 tests passed (11 bookmarks, 161 commands, 133 configuration,
74 files). Documentation receipt `N4NDXVG2MJDW75XJ3ML62IHE3E` passed on the same
candidate. The controller verified and recorded the merge in Acta.
Slice 1 of 6 completed.

## Risks and Follow-ups

This increment is session-local and intentionally rejects changed or closed
source revisions. The approved later slices own mapped/contextual locations,
persistence, native list/ordered navigation and gutter markers.
