# Slice 3: Checked Staging Unstaging and Discard

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-03-index`
PR: `#260`

## Purpose

Deliver checked path/hunk staging and unstaging, compatible marked groups and
protected whole-path discard under GIT05, GIT17 and GIT19. The
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md) define T3.1-T3.3.

## Delivered Behavior

T3.1 delivers exact path and hunk actions:

- Index requests bind one whole path or one supported semantic text hunk to a
  complete current comparison and Status generation. Wrong direction, stale,
  unsupported and foreign-parent hunk targets fail admission.
- The worker repeats complete comparison reads before any index change.
  Whole paths use installed Git's literal-path staging or captured-HEAD
  unstaging, including unborn repositories and both sides of staged renames.
- Hunk actions use exact retained patch spans through Git's cached apply/check
  path. Independently reconstructed index bytes define the expected full blob;
  Git's actual post-action object and mode must match that expectation.
- Stage verification respects configured clean filters. No action saves a
  source buffer or changes working-tree bytes. Common storage serializes
  overlapping requests across linked worktrees.
- Records distinguish not-started, possible and verified effects. The changing
  child's argv, exit and diagnostics survive subsequent refresh/verification.
  Failed post-effect execution stays indeterminate and is never replayed.

- Native `git-stage`/`git-unstage` and s/u work in both profiles, from Status
  paths, expanded hunks or diff detail. Asynchronous preparation captures its
  original target; later navigation cannot substitute an adjacent path.
- Modified source buffers, point, split identities and working files survive
  staging and unstaging. A stale retained hunk requires explicit refresh.

T3.2 delivers compatible marked groups:

- Native *, - and 0 commands and the approved x m/x u/x c aliases retain at most
  256 exact comparison targets independently of navigation. Marks share the Git
  admission budget, remain visibly stale after generation changes and require
  explicit removal rather than silent retargeting.
- Whole-path groups use one checked literal NUL input manifest, including mixed
  tracked/untracked staging and unborn unstaging. Text-hunk groups use one
  immutable path/layer comparison and one cached patch with nonoverlapping spans.
- Complete-group admission rejects duplicate/overlapping paths, mixed path/hunk
  units, incompatible directions, foreign hunk parents and stale evidence before
  effects. Every captured comparison is checked before and after preparation;
  an invalid member cannot trigger a smaller action on valid members.
- Expected index entries and independently predicted hunk bytes are verified
  after the single changing child. Unmarked entries and working files remain
  intact in real-Git fixtures.

T3.3 delivers protected whole-path discard:

- k and x d capture one tracked unstaged path with a supported index entry.
  Explicit y/n confirmation is one-shot; Return does not imply yes and C-g
  cancels. Hunks, marked groups, untracked deletion and gitlinks cannot gain
  replacement authority.
- Current canonical path and inode identity find visiting sources, including
  aliases. Unsaved text rejects the request. Source revisions, destination
  evidence and repository context are rechecked when consent is consumed.
- The worker checks complete comparison and index entries before one installed
  Git working-tree restore. It predicts checkout bytes through Git's configured
  filters, then verifies actual bytes, type/mode and unchanged complete index
  entries. HEAD is not substituted as the restoration source.
- File changes enter the existing automatic/ask observation and reload owner;
  the action never saves or directly replaces a source buffer. Failed changing
  children retain their actual exit/diagnostics and possible effects without
  replay. Outstanding cleanup retains its child rather than launching another.

## Implementation Notes

Index work shares the existing worker, cancellation, capture, process-group
cleanup, bounded output and fixed 32 MiB session admission. The configured
operation deadline covers preflight, execution, refresh and verification.
Read-only jobs retain their separate configured observation deadline.

Sealed anonymous stdin staging now belongs to the shared process owner. Shell
commands retain their file-input route; Git uses the same mechanism for literal
patch data and blob hashing. Structured OS arguments preserve exact path bytes.

Hunk prediction compares exact old index spans, including empty ranges and
missing-final-newline markers, before assembling expected bytes. Text-only hunk
patches omit independent executable-mode changes, preserving the existing index
mode in both directions; file-creation/deletion headers remain exact. Truncated
comparison or base-blob evidence cannot authorize a partial operation. Working
tree changes by external processes remain external; editor serialization does
not claim to exclude another Git client.

Group preflight shares complete Status observations across all members while
checking every exact comparison. The NUL manifest admits at most the existing
1 MiB worker scratch ceiling; excess requests reject before effects. Read-only
index verification uses batches of at most twelve paths to preserve the
64-token and 65536-byte argv limits. No operation or resource limit changed.

Discard snapshots complete staged entries within the existing detail/output
caps and admits checkout, verification and source-guard payloads before they are
retained. Regular-file proof opens without following a substituted symlink and
checks identity/metadata around a bounded byte read; symlink proof checks exact
link bytes and stable metadata. Source confirmation uses the existing native
decision owner and routes cancellation before generic Git navigation.

## Contracts Added or Changed

- Added typed index direction and immutable path/hunk request targets.
- Added explicit effect states and indeterminate mutation outcomes to records.
- Added index operation kinds and truthful process inspection labels.
- Added borrowed, sealed input at the existing process boundary.
- Registered working stage/unstage commands, both-profile s/u bindings and their
  complete sample/default-reference entries. T3.2 adds mark/unmark/clear and
  their direct and x-family aliases; T3.3 adds confirmed discard (27 Git bindings,
  490 commands).
- No new dependency or persistent application storage was introduced.

## Files of Interest

- `src/git/index.rs`: request admission and common-storage serialization.
- `src/git/work/index.rs`: exact Git operations, hunk prediction and proof.
- `src/git/work.rs`: mutation lifetime and separate child/effect evidence.
- `src/session/git/index.rs`: native preparation, captured targets and completion.
- `src/process/input.rs`: sealed anonymous input shared with shell commands.
- `tests/git_integration/index.rs`: real index, path, hunk and failure fixtures.
- `tests/git_integration/index_groups.rs`: complete-group effects and rejection.
- `tests/git_integration/index_presentation.rs`: both-profile marks, consent and source protection.
- `src/git/work/discard.rs`: bounded checkout prediction, index and file proof.
- `src/session/git/discard.rs`: canonical source admission and one-shot consent.
- `tests/git_integration/index_discard.rs`: actual restores, filters and failure effects.

## Validation

Controller receipt `WQFGAZWVDQUWWTWHRKJ5VA3LDW` passed all nine ordered checks
for `d14a59b0b274bc5a376594e375e4d288c8f71594`, including 24 index, 9 Status,
5 diff, 21 reference tests and the shared reload test. The canonical PR contract
was verified. [PR #260](https://forge.adrianpk.com/adrianpk/rev/pulls/260) merged
through controller-owned Rebase + Fast-forward into dev at that exact commit.
Slice 3 of 14 completed.

Development checks passed for formatting, strict focused Clippy, three backend
marked-group cases, four native index cases, five discard cases and the shared
reference/presentation checks. The group path fixture also passed after adding
mixed tracked/untracked staging. Discard cases cover actual text/binary, type,
mode, symlink and missing-file results; checkout filters; stale rejection;
post-effect exit failure; both-profile cancellation and dirty canonical aliases;
and automatic versus ask reload.

The three group cases took about 39 seconds, four native index cases about
45 seconds and five discard cases about 46 seconds. These bounded development
runs add observable index/file/source protection without running the delivery
set gate. The controller receipt for the final clean PR head supplies exact-HEAD
outcomes for this ordered command list after PR-reference follow-up:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::presentation -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs::presentation -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test external_changes observation::shared_kernel_reload -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Additional successful development selections:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::groups -- --test-threads=1`: 3 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::groups::paths -- --test-threads=1`: extended mixed-layer case passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::presentation:: -- --test-threads=1`: 4 passed before the two discard journeys were added.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration discard -- --test-threads=1`: 5 passed.

Regression development corrected independent executable-mode headers in text
hunk patches and HEAD-mode restoration for staged deletion. Reference checks
caught outdated key counts and missing top-level mark catalog entries; the
references were corrected without changing the checkers. The discard C-g test
caught generic Git cancellation preceding the consent owner; cancellation now
consumes the captured decision through its own route. No assertion or configured
limit was weakened.

## Risks and Follow-ups

External clients remain outside editor serialization. Non-deterministic checkout
filters or concurrent post-effect changes can prevent exact file verification;
such results retain possible effects rather than success. Complete index or
checkout evidence beyond existing caps rejects before replacement. Combined PTY
and resource acceptance remain assigned to Slice 13.

## Delivery-set closure

The controller passed the unchanged selected full gate at immutable candidate
`343850fb6914bf801dfa248b8cdd0e1c34ae1fef`, receipt `D6BU4EM7IWRUA3UHGDBE432UK3`.
All fourteen slices and repair #272 are integrated; this report remains delivered.
Its earlier slice evidence retains its own candidate. The
[Slice 14 closure](slice-14-guidance.md#delivery-set-closure) records final
counts, measurements and evidence limits.
