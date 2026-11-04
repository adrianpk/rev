# Slice 1: Repository Context and Native Status

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-01-status`
PR: pending

## Purpose

Deliver the repository and process ownership needed by native Status, then its
full-canvas presentation and guarded lifetime. The
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md) define T1.1-T1.3 and the
remaining delivery set.

## Delivered Behavior

T1.1 implements the backend foundation:

- Explicit asynchronous discovery validates working-tree, per-worktree Git and
  common directories through installed Git. Bare repositories remain distinct
  from unavailable repositories and have no working-tree Status.
- Immutable porcelain-v2 observations retain unborn, branch and detached HEAD,
  full SHA-1/SHA-256 identities, tracking counts, exact path bytes, layer facts,
  rename origins, modes and objects. Reached path admission is explicit.
- A failed refresh preserves the last confirmed observation and marks its
  context stale. Re-entering a subdirectory reuses the validated identity;
  linked worktrees retain separate identities over common storage.
- Native jobs retain exit and bounded output facts on failure, cancellation,
  timeout and output exhaustion. Cancellation targets one operation identity;
  completed cancellation cannot retarget a later job.
- The complete startup `[git]` catalog, factory sample, include provenance and
  whole-candidate validation are in place. Loading configuration does not run
  Git or require it to be installed.

Native commands, Status rendering/input and application-exit integration are
pending in T1.2-T1.3. Slice 1 is not ready for its pull request.

## Implementation Notes

The Git owner uses bounded worker-owned installed-Git observations. Workers
receive captured directories and structured OS arguments; display conversion
does not reconstruct an argument. Existing nonblocking stream and process-group
observation primitives remain shared with text-tool callers. Their existing
success/error interface remains intact.

At most four repository contexts remain retained. Unknown-context discovery is
serialized, and a known context cannot start overlapping work. Active and
recent process count/byte limits apply before launch. Owned capacities,
temporary candidates and external readers of old observations share the fixed
32 MiB admission budget. Workers admit actual candidate paths before parsing;
large configured ceilings do not preallocate empty worst-case lists.

Capture uses bounded stream readiness and a high-level observation deadline.
Cleanup sends TERM, allows 500 ms grace, then escalates within a two-second
deadline. Failed cleanup retains ownership for an explicit retry. No sequence
abort or repository rollback is inferred from child termination.

## Contracts Added or Changed

- Added the typed native Git observation, operation identity, process record
  and capacity contracts under `src/git.rs` and `src/git/`.
- Added exact OS argv capture with retained failure facts under the existing
  process boundary. Existing text-tool callers keep their interface.
- Added all fifteen approved startup preferences with per-source provenance,
  documented defaults/ranges and complete-candidate cross-field validation.
- No dependency or persistent repository store was added.

## Files of Interest

- `src/git.rs`: repository contexts, immutable publication, process ownership
  and bounded retention.
- `src/git/status.rs`: structured porcelain-v2 parsing and exact layer targets.
- `src/git/work.rs`: installed-Git capability/discovery and HEAD verification.
- `src/process/capture.rs`: OS arguments, bounded capture and owned cleanup.
- `src/config/git.rs`: typed startup policy and validation.
- `tests/git_integration/status.rs`: real isolated repository contracts.

## Validation

T1.1 development checks pass. These checks do not establish native
presentation, the complete slice, or the final delivery gate. The controller's
clean-HEAD slice receipt is pending T1.2-T1.3.

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status:: -- --test-threads=1`: 10 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1`: 10 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration reference -- --test-threads=1`: 11 passed.
- `git diff --check`: passed.

## Risks and Follow-ups

Complete T1.2 native presentation and T1.3 process inspection, checked UI
cancellation, guarded application exit and remaining failure fixtures before
opening the canonical Slice 1 pull request. Joint PTY and resource measurements
remain assigned to Slice 13 by the approved plan.
