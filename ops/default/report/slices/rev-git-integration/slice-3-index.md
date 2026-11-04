# Slice 3: Checked Staging Unstaging and Discard

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-03-index`
PR: pending

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

T3.3 protected discard remains pending. No complete Slice 3 delivery or PR is
claimed.

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

## Contracts Added or Changed

- Added typed index direction and immutable path/hunk request targets.
- Added explicit effect states and indeterminate mutation outcomes to records.
- Added index operation kinds and truthful process inspection labels.
- Added borrowed, sealed input at the existing process boundary.
- Registered working stage/unstage commands, both-profile s/u bindings and their
  complete sample/default-reference entries. T3.2 adds mark/unmark/clear and
  their direct and x-family aliases (25 Git bindings, 489 commands).
- No new dependency or persistent application storage was introduced.

## Files of Interest

- `src/git/index.rs`: request admission and common-storage serialization.
- `src/git/work/index.rs`: exact Git operations, hunk prediction and proof.
- `src/git/work.rs`: mutation lifetime and separate child/effect evidence.
- `src/session/git/index.rs`: native preparation, captured targets and completion.
- `src/process/input.rs`: sealed anonymous input shared with shell commands.
- `tests/git_integration/index.rs`: real index, path, hunk and failure fixtures.
- `tests/git_integration/index_groups.rs`: complete-group effects and rejection.
- `tests/git_integration/index_presentation.rs`: both-profile captured marks.

## Validation

Development validation covers real repository effects and shared input/UI
contracts. The initial backend run passed nine tests (about 85 seconds); the
expanded eleven-case run passed both-profile native paths/hunks and took about
134 seconds. Three later focused cases cover executable-mode preservation, initialized
gitlinks and whole-file creation/deletion modes. Final clean-HEAD controller validation follows complete
T3.1-T3.3 delivery and the PR-reference follow-up.

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`: passed after the mode/gitlink follow-ups and again after T3.2.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index:: -- --test-threads=1`: 11 passed before the executable-mode follow-up.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::hunk_mode -- --test-threads=1`: regression reproduced a postcondition mismatch, then passed both directions after separating mode metadata from text hunks.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::gitlink_path -- --test-threads=1`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::hunk_file_modes -- --test-threads=1`: regression reproduced the staged-deletion mode mismatch, then passed deletion/creation cases after deriving the restoration mode from HEAD.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process::capture:: -- --test-threads=1`: 3 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands regions:: -- --test-threads=1`: 3 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`: 21 passed in T3.1 and again after T3.2 catalog correction.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::presentation -- --test-threads=1`: 9 passed again after T3.2.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs::presentation -- --test-threads=1`: 5 passed again after T3.2.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`: passed in T3.1 and after T3.2; the unchanged checker now covers 489 command names, 25 Git bindings, 263 settings and 76 pages.
- `git diff --check`: passed.

T3.2 development checks add three backend group cases (about 39 seconds) and
four native index cases (about 45 seconds, including both existing single-target
journeys). The path-group case passed again after extending its born-repository
variant to mix tracked and untracked members. These remain development evidence;
no final exact-HEAD controller receipt is claimed yet.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::groups -- --test-threads=1`: 3 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::groups::paths -- --test-threads=1`: extended mixed-layer case passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration index::presentation:: -- --test-threads=1`: 4 passed across both profiles.

The first strict check rejected complex internal tuple types; named prepared
request state resolved that finding. The first documentation check rejected
an outdated 17-key defaults row after s/u were added; the reference now matches
the packaged map. The first native mark fixture attempted to visit binary input;
using an unchanged text source for entry retained the binary mutation targets.
Reference validation then caught mark commands missing from the top-level binding
inventory; the catalog entries were added without changing either checker.
No assertions or configured limits were weakened.

## Risks and Follow-ups

Complete T3.3 confirmed discard with modified-buffer protection. Combined PTY
and resource acceptance remain assigned to Slice 13.
