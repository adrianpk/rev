# Slice 11: Stashes and Linked Worktrees

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-11-repository`
PR: pending

## Purpose

Deliver GIT13 and its GIT17-GIT19 contracts through T11.1-T11.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T11.1: Status z z or z l opens exact bounded Stashes. c creates a stash from
  saved tracked and untracked changes; ignored files stay excluded. Return or v
  inspects bounded read-only patch detail for the selected full object.
- a reviews apply while retaining the stash; p reviews pop; b reviews a new
  branch at the stash base; k reviews removal of its exact entry. The native
  confirmation shows the operation, full object, checked HEAD and affected paths.
  y, C-Enter or C-c C-c accepts; Return alone does not. n or C-g cancels.
- Restoration uses the full object. Pop and branch remove the captured stash
  entry only after successful restoration and a separately revalidated list.
  A changed list cannot silently redirect its old ordinal to another stash.
  Ambiguous duplicate-object removal rejects before launch.
- Saved index/working-tree/untracked evidence and actual refs/sequence state
  invalidate stale review. Affected modified native buffers reject without a
  hidden save; unrelated unsaved text remains owned. Apply/pop/branch require
  complete clean saved state. Existing automatic/ask file reload policy applies.
- Conflicts retain the stash and expose actual unmerged index paths through
  the existing conflict owner. No Git sequence, continuation or rollback is
  invented. Failed or uncertain effects keep actual diagnostics and ownership.
- T11.2 linked-worktree management and T11.3 additional acceptance remain pending.

## Implementation Notes

The existing installed-Git worker, process records, common-storage admission,
sequence snapshots and native ref-confirmation owner also own stash requests.
Typed observations retain raw selector bytes and full SHA-1/SHA-256 object IDs;
subjects and escaped labels cannot construct authority. Show uses explicit
read-only patch arguments with external diff/text conversion disabled.

A stash plan captures complete index entries, bounded binary working-tree diff
and hashes of saved untracked bytes. These supplement Status labels, which do
not prove unchanged file contents. Immutable full object targets supply apply
and branch arguments. The ordinal needed by Git removal is checked against the
complete captured list before use, followed by actual remaining-object checks.
A conflict leaves the entry intact. Restoration is rechecked before removal;
actual saved index/working-tree/untracked evidence must remain unchanged by the
removal. Restoration/removal failures are not retried.

Stash counts use the existing max_refs bound. Path counts/bytes, detail bytes,
process output and retained plans share existing Git configuration, the 1 MiB
scratch limit and the 32 MiB admission owner. Reached caps remain explicit and
partial observations cannot authorize changing actions. No dependency, Python
prerequisite or process framework was added; five-second fixture waits and
existing operation/cancellation/cleanup limits remain unchanged.

## Contracts Added or Changed

- Seven native commands: git-stash-list/create/show/apply/pop/branch/drop.
- keys.git_stash has 25 active defaults in both interaction profiles. p is pop;
  Up and C-p navigate to the previous stash. Status has z z and z l entry routes.
- The catalogs now contain 554 commands and 270 configuration entries.

## Files of Interest

- `src/git/repository.rs`, `src/git/work/repository.rs`: typed exact stash
  observations, saved-state plans, bounded details and checked execution.
- `src/session/git/repository.rs`, `src/session/git/refs.rs`: native lists,
  prompts, exact review, affected source owners and existing reload policy.
- `src/input/keymap.rs`, `src/input/keymap/defaults.rs`: dedicated configurable
  stash context with the approved prefix/action meanings in both profiles.
- `tests/git_integration/repository.rs`, `repository_presentation.rs`: real
  Git objects/disk/index and native command/key/confirmation/source behavior.

## Validation

T11.1 development passed eight real Git/native cases in 55.91 seconds: both
object formats with raw paths and binary untracked bytes; create/show/apply/drop
saved-file outcomes; pop and branch base/ref outcomes; changed tracked/untracked
content, index, HEAD and list rejection; actual conflicted pop retaining its
stash; complete-list and partial-detail bounds; both native profiles and
modified affected/unrelated-source ownership, including a renamed original
visiting source. The isolated ordinal case passed in 2.51 seconds before the
final combined run.

An earlier ordinal fixture failed because storing the current top stash again
was a Git no-op. The corrected fixture stores the older object, asserts that
three entries now exist and verifies that the old ordinal names the other
object before invoking the unchanged stale-target assertion. No deadline,
observable requirement or resource limit changed.

Formatter and strict focused Clippy passed. Four existing native ref cases
passed in 9.70 seconds, protecting shared confirmation, source and history
owners. All 21 command/configuration reference cases passed in 13.63 seconds.
An earlier reference check rejected duplicated alias targets in the binding
table; grouping aliases under one target corrected the documentation. The
existing context-reference test now also checks the stash context. Documentation
passed at 554 commands, 270 configuration entries, 76 pages and 485 local links.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration repository:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration repository::stash_ordinal -- --test-threads=1`
- `git diff --check`

T11.2-T11.3 evidence remains pending. Final controller validation belongs to the
exact clean canonical PR head after its reference follow-up commit. No final
receipt is claimed here.

## Risks and Follow-ups

T11.2 and T11.3 remain within this slice. Combined PTY/resource acceptance and
full learning-path guidance remain in their approved later slices. Integration
remains controller-owned.
