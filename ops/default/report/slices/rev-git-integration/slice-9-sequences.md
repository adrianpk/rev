# Slice 9: Ordered Revisions and Manual Bisect

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-09-sequences`
PR: pending

## Purpose

Deliver GIT11 and its GIT17-GIT19 contracts through T9.1-T9.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

T9.1 delivers reviewed ordered applications:

- History P / V prepares cherry-pick / revert from the independent ordered
  revision marks, or one selected revision when no ordered set exists. a/U
  retain explicit append/remove order; * remains the separate range base.
- Each merge requires an explicit one-based mainline choice. Final native
  review shows every full source object, chosen full parent, exact order and
  original branch/HEAD before y, C-Enter or C-c C-c can submit. Return does not
  accept; n or C-g cancels. Both interaction profiles use the same requests.
- From Refs, the existing native git-log command opens the exact selected
  commit-bearing ref as a source History root while retaining the current
  working branch. Ordered applications preserve that reviewed root and its
  independent marks across the resulting history refresh.
- One finite request owns sequential installed-Git steps in the confirmed
  order, including different mainlines for different merge revisions. Each
  step checks the current refs, clean index/disk and absence of another
  sequence. Native source stamps protect unmodified visiting buffers.
- Success checks actual branch, single parent, authored cherry-pick message,
  resulting index/tree, clean disk and unrelated ref preservation. Revert
  verifies the actual new commit's source identity. Existing Git hooks and
  signing remain authoritative.
- A real conflict stop exposes Git's current family, target and exact paths,
  with retained original context, current step and verified completed prefix.
  Refresh preserves that exact lineage; it does not execute remaining work.

T9.2 adds compatible checked sequence controls:

- The Operation view binds continue, skip and abort to fresh Git metadata,
  exact retained target/order/mainlines, original/current branch and HEAD,
  index and disk evidence. Generic controls dispatch the same compatible family.
- Cherry-pick uses { / l / Y; revert uses } / j / Q. Continue requires saved
  staged resolutions. Skip explicitly omits the current target; subsequent
  targets retain their reviewed order, including through another conflict stop.
- Explicit abort restores the original branch, HEAD and tree after a completed
  prefix. Process cancellation remains separate from sequence abort.
- Wrong requested or mixed actual families, changed targets, stale metadata,
  changed index/HEAD and missing sequences reject before changing Git. External
  work remains observable; unavailable required retained lineage is not guessed.
- The view identifies busy controls while owned repository work finishes.
  Background source-History refreshes retain Operation feedback, including
  wrong-family rejection and a pending exact control review.

## Implementation Notes

The existing Git owner admits at most 256 exact distinct targets. Source
metadata is read in one bounded cat-file batch through sealed anonymous stdin;
full IDs do not exceed structured argv admission. Metadata, complete messages,
worker output/scratch and shared session memory retain their existing limits.
The sealed input reservation follows an owned retained cleanup if required.

Git owns each actual revision operation and durable state. Native progress
holds only the finite reviewed request and verified prefix needed for later
compatible controls. Missing or changed target/HEAD/branch cannot inherit that
lineage. Changing-child execution facts survive postcondition reads and the
preflight for a later step. No dependency, persistent store or general process
framework was added.

Ordered control rechecks the same shared sequence fingerprint before its
changing child. Continue verifies the actual new commit's single parent,
source-authored message/author where applicable, resolved index/tree and branch.
Skip verifies the unchanged current HEAD and complete clean result before
advancing. Git abort owns the current step; restoring a completed prefix then
requires a fresh clean/ref/sequence fence and checked reset --merge to the exact
original object. No restoration occurs on process cancellation or failed proof.
New immutable progress retains the same finite confirmed request at each stop.
Post-command observation errors preserve the changing argv and indeterminate
possible-effects outcome; they do not claim completion or automatic restoration.

## Contracts Added or Changed

- git-cherry-pick and git-revert and their six family-specific continue/skip/abort
  commands have functional native handlers and catalogs.
- keys.git_sequence has 27 active entries, including the six ordered controls.
- keys.git_history has 28 active entries, including P and V, in the complete
  configuration sample and both profiles.
- Exact source/mainline order and original context are typed public requests.
- Ordered applications retain their reviewed source History root; other owned
  ref changes keep the existing destination-history refresh behavior.
- Control preparation and changing operations retain actual busy ownership;
  a background ancestry callback does not overwrite Operation feedback.

## Files of Interest

- `src/git/sequences.rs`, `src/git/work/sequences.rs` and its preparation
  and control modules: exact plans, bounded source metadata, sequential execution,
  compatible controls and proof.
- `src/git/integration.rs`, `src/git/work/integration/sequence.rs`: actual
  current revision and retained ordered lineage in the shared observation.
- `src/session/git/refs/sequences.rs`: mainline prompts, source admission and
  complete native review through the existing confirmation owner.
- `tests/git_integration/sequences.rs`, `sequences_controls.rs` and
  `sequences_presentation.rs`: real Git trees/messages/parents, repeated conflict
  stops, checked controls, stale evidence, hook uncertainty and both profiles.

## Validation

T9.1 development checks passed. All 6 sequence tests passed in 22.19 seconds.
Real SHA-1/SHA-256 application trees and ordered authored messages agree with
installed Git; the mixed-mainline fixture also checks actual resulting files
against Git using each separately chosen parent. The existing branch-switch
history/source regression passed in 1.86 seconds. Reference checks passed 21
tests. Documentation checked 528 commands, 268 configuration entries, 76 pages
and 485 links.

Initial native testing exposed an early fixture wait that matched preparation
rather than confirmation. The corrected wait requires the actual Confirm
prompt and keeps every full-ID/order/content/cancellation assertion and the
5-second deadline. The subsequent mark-role assertion exposed destination-HEAD
reanchoring after ordered application; preserving its reviewed source root
corrected that behavior. The focused native test and complete sequence filter
then passed, along with the existing destination-history regression. Strict
Clippy required a standard slice-size calculation and one collapsed branch.

T9.2 development formatting and strict Clippy pass. All 11 sequence tests
passed in 88.10 seconds; five added tests cover six family/control combinations,
repeated conflict stops, final-step controls, stale/mixed-family evidence,
both-profile native review and real post-commit-hook uncertainty. The existing
19 shared integration tests passed in 76.89 seconds and the branch/history/source
regression in 1.56 seconds. Reference tests pass 21 checks; documentation checks
534 commands, 268 configuration entries, 76 pages and 485 links.

Native testing first exposed a rendered SHA-256 ID assertion that ignored line
wrapping. The corrected assertion retains the complete displayed ID. A later
wait exposed background History feedback replacing a wrong-family rejection;
the callback now preserves Operation feedback. Native availability now reports
owned Git work as busy, and the fixture waits for that observable state to clear.
Every wait retains its 5-second deadline and all original state/content checks.

The real hook regression reproduced a committed change whose later sequence
observation failed. The corrected outcome is indeterminate with possible effects,
retaining the actual changing argv and verified fixture parent/content rather
than claiming an abort. Reference validation also detected missing primary
bindings catalog entries; the complete six-command catalog now passes.

Exact successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation::refreshed_history_and_source -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

These are development results. Final exact-HEAD controller evidence follows
T9.3 and the canonical PR-reference follow-up commit.

## Risks and Follow-ups

T9.3 manual bisect and canonical PR preparation remain pending. Combined
PTY/resource acceptance remains Slice 13; the full User Guide remains Slice 14.
