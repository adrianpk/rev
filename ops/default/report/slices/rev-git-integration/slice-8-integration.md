# Slice 8: Merge Rebase and Sequence Control

Status: reviewing
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-08-integration`
PR: `#265`

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

T8.3 verifies stale and incompatible sequence state, external apply-backend
rebase, configured-hook failure, cancellation, fresh native continuation and
both automatic/ask source reload policies. Ordinary commit cannot implicitly
finish an active sequence. A changing rebase must bind Git's original object
to the reviewed source; ancestry by itself is insufficient. Real resulting
content remains visible if that source binding changes.

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
external Git clients. The sealed autosquash descriptor carries its exact byte reservation with a
retained cleanup owner, independently of diagnostic-record retention. No
dependency or persistent repository store was added.

## Contracts Added or Changed

- All 14 approved Integration native commands have handlers, catalog entries
  and matching command/binding references.
- Refs has 48 active bindings, including all direct/prefix integration actions.
  Independent `keys.git_sequence` has 21 navigation/control bindings.
- Process outcomes distinguish in-progress, completed, skipped, aborted,
  canceled and uncertain results; cancellation never implies sequence abort.
- Exact-action reviews use y, C-Enter or C-c C-c; Return does not submit.
- Normal/amend/fixup commit admission rejects active Git-owned sequence state.

## Files of Interest

- `src/git/integration.rs`, `src/git/work/integration.rs` and its sequence
  module: exact requests, bounded Git-owned metadata and postconditions.
- `src/session/git/refs/integration.rs`: source guards, exact review, sequence
  presentation and ordinary file-owner reconciliation.
- `src/input/keymap.rs`, `docs/reference/config.toml`: independent native
  Sequence input and complete active configuration sample.
- `tests/git_integration/integration.rs`, `integration_presentation.rs`: real
  object/ref/file assertions, conflict stops and both-profile native review.
- `tests/git_integration/integration_guards.rs`: stale fingerprints, missing
  lineage, external sequences, actual launch races, hooks and cancellation.

## Validation

T8.3 development strict Clippy passed. The integration command below passed
all 19 tests (81.15 seconds); the shared process command passed all 9 tests
(2.94 seconds). Prior reference checks passed 21 tests, and documentation checked
526 commands, 268 configuration entries, 76 pages and 485 links.

SHA-1/SHA-256 autosquash fixtures compare actual trees, ordered messages and
authors with Git's own sequencer, including amend-message fixups. Native tests
exercise both profiles, stale continuation review, automatic/ask reload and
retained source/window/history ownership. Independent stale metadata, index,
worktree, HEAD and missing-state cases reject before effects. Missing original,
branch or onto evidence, empty state directories, autostash, executable todos,
ambiguous targets and the 256-source bound retain explicit rejection.

Two regression tests failed before correction: normal commit consumed a merge
stop, and rebase incorrectly verified a result begun from a changed HEAD.
The sequence admission guard and actual original-object binding correct those
specific boundaries. Earlier development checks also exposed a completed-merge
index lookup, one strict-Clippy iterator requirement and missing primary binding
catalog rows; all were corrected without weakening checks, assertions, fixture
5-second deadlines or resource limits.

The first final controller batch passed all eight checks at
`87eb209b9385c865b64e7cf73582594ae62fccc8` (receipt
`RBFWG5G2NZ5TTCTFGOGQCT2DDM`). Before handoff, the new sealed-driver lifetime
was corrected to carry its memory reservation through retained cleanup. The
resulting HEAD requires a new complete receipt; the earlier receipt is not
candidate evidence.

Final ordered check contract, executed through the controller after the
PR-reference follow-up commit; the exact clean-HEAD receipt is retained outside
the repository:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- --skip integration:: --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

Combined PTY/resource acceptance
remains Slice 13; the full User Guide remains Slice 14.
