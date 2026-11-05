# Slice 8: Merge Rebase and Sequence Control

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-08-integration`
PR: pending

## Purpose

Deliver GIT10 and its GIT16-GIT19 integration through approved T8.1-T8.3 in the
[plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

T8.1 delivers exact merge/rebase starts and sequence inspection:

- Refs = / m f selects fast-forward-only merge, i / m m permits fast-forward
  or merge, I / m n requires a merge commit when Git has integration work,
  and e / r r selects ordinary rebase onto one exact commit-bearing ref.
- Native review shows the full selected ref/commit, current branch/HEAD and
  explicit policy. y, C-Enter or C-c C-c accepts; Return does not submit.
  n or C-g cancels before launch. Both profiles retain their editing owner.
- Complete refs and clean disk/index are required. Unsaved repository-backed
  buffers reject; retained source stamps and Git facts are checked again.
  Autostash, expanded rebase updateRefs and unreviewed branch mergeOptions
  cannot broaden the request. Existing Git hooks and signing remain authoritative.
- Success checks actual branch, target ancestry, merge parent policy, clean
  resulting disk/index and unrelated ref preservation. A real conflict stop
  publishes InProgress with actual child failure, exact sequence family,
  matching original/target lineage and exact conflict paths.
- `git-operation-status` reads Git-owned state, including external sequences.
  The native read-only Git Operation view shows full current/original/onto
  objects and available step information. q returns to Status without aborting.
  Process cancellation also does not claim to abort Git state.
- Source updates reconcile through the existing file reload/ask owner.
  `keys.git_sequence` has an independent both-profile navigation map and a
  complete active reference sample.

T8.2 adds finite autosquash and compatible controls:

- E / r a prepares a linear source plan of at most 256 full commit IDs without
  starting rebase. Review orders pick, fixup, squash and amend-message fixup
  actions. Missing or ambiguous fixup targets reject instead of being guessed.
- Confirmation reconstructs the exact source range and ordered plan. A sealed
  anonymous descriptor supplies only those actions to a fixed noninteractive
  sequence driver; subjects never become executable todo text. Git retains
  hooks, author identity and signing policy.
- Family-specific and generic continue/skip/abort requests first refresh actual
  Git state, then require one explicit review. Metadata, refs, index, worktree,
  original lineage and visiting source stamps are checked again before effects.
- The Operation view exposes eligible controls. Merge > / X continues/aborts;
  rebase ] / ; / Z continues/skips/aborts. Refs provides m c / m a and
  r c / r s / r A. Empty sequence directories remain observable; autostash,
  executable todo content, expanded ref updates and missing lineage reject.
- Actual completed skip and abort outcomes remain distinct from completion,
  in-progress stops, process cancellation and uncertain effects.

## Implementation Notes

The existing installed-Git worker owns structured argv, bounded process life,
source admission and immutable publication. Status, Refs and Sequence publish
one compatible observation. Sequence metadata retains exact private bytes;
labels and display rows cannot construct authority. Actual changing-child
exit/output remains available across post-effect observation and verification.

Sequence metadata admits 64 ordinary files and 256 merge targets within the
existing detail limit, 1 MiB worker scratch and shared 32 MiB Git budget.
Symlinked/oversized state rejects. Metadata and HEAD are read again before
publication. Observation and changing-operation timeouts keep their existing
independent bounds. Revalidation does not claim an exclusive transaction against
external Git clients. No dependency or persistent repository store was added.

## Contracts Added or Changed

- `git-merge`, `git-merge-fast-forward`, `git-merge-no-fast-forward`,
  `git-rebase` and `git-operation-status` have native handlers, catalog entries
  and matching reference documentation.
- Refs has all eight direct/prefix integration bindings. The independent
  `keys.git_sequence` map provides 16 active navigation bindings.
- Process outcomes distinguish an observed in-progress stop from completion.
- Existing exact-action review accepts the approved explicit submit aliases.

## Files of Interest

- `src/git/integration.rs`, `src/git/work/integration.rs` and its sequence
  module: exact requests, bounded Git-owned metadata and postconditions.
- `src/session/git/refs/integration.rs`: source guards, exact review, sequence
  presentation and ordinary file-owner reconciliation.
- `src/input/keymap.rs`, `docs/reference/config.toml`: independent native
  Sequence input and complete active configuration sample.
- `tests/git_integration/integration.rs`, `integration_presentation.rs`: real
  object/ref/file assertions, conflict stops and both-profile native review.

## Validation

T8.2 development formatter and strict focused Clippy passed. The exact
integration command below passed 10 tests (33.35 seconds), covering real
SHA-1/SHA-256 autosquash trees, ordered messages and authors against Git's own
sequencer, all merge/rebase continuation/skip/abort outcomes, external merge
observation and both-profile native plan/control review. The reference command
passed 21 tests; documentation checked 526 commands, 268 configuration entries,
76 pages and 485 links. No fixture deadline or assertion changed.

An initial merge-continuation postcondition incorrectly read an index field
that is absent after sequence completion; direct resulting-index evidence
corrected it. Strict Clippy required a fixed-size chunk iterator. The first
reference run exposed missing primary binding catalog rows; the complete nine
rows now pass the unchanged catalog checks.


T8.1 development checks passed:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration:: -- --test-threads=1` (6 tests, 13.82 seconds)
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1` (21 tests)
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` (517 commands, 268 configuration entries, 76 pages and 485 links)
- `git diff --check`

The initial compile required adding Sequence to the process-row page match.
Strict Clippy required a typed sequence-file record and one collapsed branch.
All subsequent focused checks passed. No assertion, fixture deadline or resource
ceiling was weakened. T8.3 and final exact-HEAD controller evidence remain pending.

## Risks and Follow-ups

T8.3 extended sequence/reload acceptance remains pending. Combined PTY/resource acceptance
remains Slice 13; the full User Guide remains Slice 14.
