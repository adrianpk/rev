# Slice 9: Ordered Revisions and Manual Bisect

Status: reviewing
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-09-sequences`
PR: `#266`

## Purpose

Deliver GIT11 and its GIT17-GIT19 contracts through T9.1-T9.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- History P/V prepares ordered cherry-pick/revert from independent a/U marks,
  or one selected revision. The range base remains separate. Each merge needs
  an explicit mainline number; review shows full source and chosen parent IDs.
- Native confirmation preserves the exact order and original branch/HEAD.
  A real conflict exposes Git's actual family, current target and exact paths,
  retaining the verified completed prefix and remaining mainline choices.
- Checked cherry-pick controls use { / l / Y; revert uses } / j / Q. Continue
  requires saved staged resolutions; skip omits only the current target before
  continuing the reviewed order. Abort explicitly restores the original context,
  including a verified completed prefix. Cancellation does not imply abort.
- Operation ? collects explicit bad/good bisect boundaries, resolves full commit
  IDs once and reviews the original branch/object before checkout. G/B/| reviews
  a separate good/bad/skip classification for the exact current candidate.
- Bisect checks actual boundary refs, candidate metadata and graph membership.
  A unique first bad commit is distinguished from ambiguity caused by skipped
  candidates. ~ explicitly resets to the checked original branch, object, index
  and disk; a moved original branch cannot inherit the retained permission.
- All changing actions protect unmodified visiting source owners and reuse the
  existing reload policy. Return does not confirm; y, C-Enter or C-c C-c submits,
  and n or C-g cancels. Both interaction profiles use the same native requests.
  Manual bisect never starts a build/test or infers a verdict.

## Implementation Notes

Git owns durable sequence state. Native progress retains only one finite
confirmed request, its original context and verified prefix. Fresh metadata,
refs, HEAD, index/disk and source stamps fence each changing request. Duplicate
or stale targets, wrong requested or mixed actual families and missing sequences
reject before effects. External work remains observable; missing required
original lineage is not reconstructed from a label.

Ordered application checks actual parents, authored cherry-pick message/author,
resolved index/tree, branch and unrelated refs. Git abort owns the current step;
restoring its completed prefix additionally requires a fresh clean/ref/sequence
fence and checked reset --merge to the original object. Hooks/signing remain
Git-owned. Post-command observation failure retains changing argv and an
indeterminate possible-effects result instead of implying rollback.

Bisect verifies each exact good/bad/skipped-ref transition independently of
stdout. Bounded actual graph membership establishes a unique remaining bad
commit or proves that only skipped alternatives remain. Git's ambiguity exit 2
is admitted only with that state proof. Reset verifies the original full ref and
object, clean index/disk and removal of actual bisect state. No automatic retry,
reset, test runner, interactive external editor or arbitrary command is added.

Existing limits remain: four repository contexts, 32 MiB shared admission, 1 MiB
worker scratch, structured argv bounds, bounded metadata/detail/diagnostics and
configured observation/mutation deadlines. Ordered targets and bisect refs each
have finite admission of at most 256; configured ref limits also apply. Bisect
reads at most 257 candidate IDs and bounded count output. Larger ranges use
bounded ancestry checks under the existing deadline. Sealed input avoids argv
expansion and retains its reservation with owned cleanup. No dependency,
persistent store or general process framework was added.

## Contracts Added or Changed

- Eight ordered-application commands and five manual-bisect commands have native
  handlers and complete command/binding catalogs. Existing generic controls
  dispatch compatible ordered application families through the shared owner.
- keys.git_sequence has 32 active sample entries; keys.git_history retains 28.
  The catalogs contain 539 commands and 268 configuration entries.
- Ordered changes preserve the reviewed source History root and its independent
  marks. Other ref changes keep their existing destination-history behavior.
- Busy controls remain visible while owned Git work finishes. Background
  ancestry refreshes preserve Operation feedback and pending control reviews.

## Files of Interest

- `src/git/sequences.rs`, `src/git/work/sequences.rs` and its preparation/control
  modules: exact ordered plans, bounded source metadata, execution and proof.
- `src/git/bisect.rs`, `src/git/work/bisect.rs` and its observation module:
  manual boundaries, current classifications, graph proof and original reset.
- `src/git/integration.rs`, `src/git/work/integration/sequence.rs`: shared actual
  Git family, metadata fingerprints and retained original lineage.
- `src/session/git/refs/sequences.rs`, `src/session/git/refs/bisect.rs`: native
  prompts, complete exact review, source admission and explicit confirmation.
- `tests/git_integration/sequences.rs` and its controls, bisect and presentation
  modules: real Git order/mainlines, repeated stops, controls and both profiles.

## Validation

T9.1-T9.2 development formatting, strict Clippy, all 11 then-existing sequence
cases, 19 shared integration cases, the branch/history/source regression and 21
reference checks passed. T9.3 adds five focused cases: actual first-bad content
in both object formats, skipped-candidate ambiguity, stale and configured-limit
fences, both-profile native review/reset and unsaved-source protection. The core
bisect selection passed in 29.41 seconds, native flow in 11.36 seconds and source
protection in 1.16 seconds. The final expanded fence case passed in 13.24 seconds.
Reference validation checks 539 commands, 268 configuration entries, 76 pages
and 485 local links.

Native fixtures retain full-ID, order, content, ownership and cancellation
assertions and every 5-second deadline. Corrected waits observe actual review
or control availability, and wrapped SHA-256 display assertions retain the full
ID. These checks exposed and corrected source-History reanchoring and background
feedback replacement. A real post-commit hook reproduced unavailable observation
after actual effects; the corrected result remains indeterminate with retained
changing argv. Reference checks also caught missing primary catalog entries.

Successful development commands include:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences::bisect:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences::bisect::bisect_fences -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences::presentation::native_bisect -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration sequences::presentation::unsaved_bisect -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation::refreshed_history_and_source -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Final validation uses the controller for the clean PR-reference HEAD and ordered
command list. The canonical PR Validation section and controller receipt retain
that evidence without a subsequent implementation commit.

## Risks and Follow-ups

Combined PTY/resource acceptance remains Slice 13; the full User Guide remains
Slice 14. Integration and delivery-set closure remain controller-owned.
