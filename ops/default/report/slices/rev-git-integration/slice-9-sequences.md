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

## Contracts Added or Changed

- git-cherry-pick and git-revert have functional native handlers and catalogs.
- keys.git_history has 28 active entries, including P and V, in the complete
  configuration sample and both profiles.
- Exact source/mainline order and original context are typed public requests.
- Ordered applications retain their reviewed source History root; other owned
  ref changes keep the existing destination-history refresh behavior.

## Files of Interest

- `src/git/sequences.rs`, `src/git/work/sequences.rs` and its preparation
  module: exact plans, bounded source metadata, sequential execution and proof.
- `src/git/integration.rs`, `src/git/work/integration/sequence.rs`: actual
  current revision and retained ordered lineage in the shared observation.
- `src/session/git/refs/sequences.rs`: mainline prompts, source admission and
  complete native review through the existing confirmation owner.
- `tests/git_integration/sequences.rs`, `sequences_presentation.rs`: real Git
  trees/messages/parents, conflicts, admission and both-profile native paths.

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

Exact successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation::refreshed_history_and_source -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

These are development results. Final exact-HEAD controller evidence follows
T9.2-T9.3 and the canonical PR-reference follow-up commit.

## Risks and Follow-ups

T9.2 compatible cherry-pick/revert continuation, skip and abort and T9.3 manual
bisect remain pending. Combined PTY/resource acceptance remains Slice 13; the
full User Guide remains Slice 14.
