# Slice 1: Repository Context and Native Status

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-01-status`
PR: `#258`
Merged commit: `2a6181a062428d91dd4192467b5cbb9735bed6d1`
Controller validation: `ODY3HME22HUL3MRUW4F2VYWHPM` (9 ordered checks passed for the merged commit)

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

T1.2 adds the native presentation:

- `C-x g`, M-x and modal colon enter the same read-only full-canvas Status in
  both startup profiles, using the visiting file, chosen directory or launch
  directory as the discovery origin.
- Only nonempty Unmerged, Untracked, Unstaged and Staged groups appear. HEAD,
  upstream counts, clean, unborn, detached, bare, checking, stale and truncated
  states are explicit. Display labels escape exact operating-system bytes.
- Refresh preserves compatible path/group selection. Mouse targets require the
  painted context, generation, dimensions, focus and navigation state; a stale
  hit rejects before choosing another row.
- Native prompts and the available-action dispatcher preserve source text,
  unsaved revisions, window identities, modal state, tree focus and the same
  live terminal. Returning with `q` leaves started Git work owned.
- `keys.git` supplies validated navigation bindings independently of source
  maps. Retained frames participate in the same 32 MiB budget as observations.

T1.3 completes owned-work inspection and exit:

- `$` and `git-processes` show active observations before bounded recent records,
  including after Status is hidden. Return opens the selected identity's phase,
  duration, actual exit evidence and available captured diagnostics; q restores
  the originating list or Status selection.
- Contextual C-g and `git-process-cancel` cancel an explicitly selected live
  identity. Menus, prompts and pending prefixes retain first refusal. Escape,
  inspection and view exit do not stop a child; expired records never retarget a
  replacement operation.
- Guarded application exit identifies active Git observations, captures consent
  for those identities and gathers terminal consent before termination starts.
  Decline preserves running work and unsaved sources. Accepted exit awaits
  bounded cleanup; cancelling exit after consent keeps cleanup owned without
  exiting. Failed cleanup retains an inspectable owner and explicit retry.
- Real isolated repositories cover tracking divergence and unmerged entries.
  Controlled malformed output, a real intervening HEAD change and deadline
  exhaustion retain the confirmed observation with truthful failure outcomes.

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

Process detail borrows retained bytes and escapes bounded visible chunks.
Lines containing URLs, user-info or credential markers are withheld in full,
including separated authorization values. Diagnostic labels remain display
text and never become repository arguments. Active capture stays on its worker;
completed or retained-cleanup records provide the available byte evidence.

The documentation schema collector reads the new Git model alongside the other
configuration modules. Its existing exact comparison still rejects missing,
extra or duplicate preferences and mismatched packaged defaults.

## Contracts Added or Changed

- Added the typed native Git observation, operation identity, process record
  and capacity contracts under `src/git.rs` and `src/git/`.
- Added exact OS argv capture with retained failure facts under the existing
  process boundary. Existing text-tool callers keep their interface.
- Added all fifteen approved startup preferences with per-source provenance,
  documented defaults/ranges and complete-candidate cross-field validation.
- Added native Git command identities and a validated `keys.git` context, with
  full-canvas frame admission and source/terminal input ownership boundaries.
- Extended the existing guarded exit coordinator with captured Git operation
  consent and an asynchronous cleanup continuation, alongside source/terminal
  decisions.
- No dependency or persistent repository store was added.

## Files of Interest

- `src/git.rs`: repository contexts, immutable publication, process ownership
  and bounded retention.
- `src/git/status.rs`: structured porcelain-v2 parsing and exact layer targets.
- `src/git/work.rs`: installed-Git capability/discovery and HEAD verification.
- `src/process/capture.rs`: OS arguments, bounded capture and owned cleanup.
- `src/config/git.rs`: typed startup policy and validation.
- `src/session/git.rs`: retained native Status navigation and checked input.
- `src/session/git/processes.rs`: identity-bound inspection and cancellation.
- `src/session/git/shutdown.rs`: consent and guarded exit continuation.
- `src/display/git.rs`: safe read-only projection under shared admission.
- `tests/git_integration/status.rs`: real isolated repository contracts.
- `tests/git_integration/presentation.rs`: both-profile navigation, terminal
  retention, pointer generations, configuration and display bounds.
- `tests/git_integration/lifetime.rs`: real child retention, selected cancellation,
  guarded exit and credential-safe diagnostic presentation.

## Validation

T1.1-T1.3 development checks pass. The 21 native Git tests finish in about
31 seconds on the development host. Controlled wrappers isolate delays and
failure bytes around installed Git; they do not substitute normal repository
semantics. No user remote, live credentials or external network is exercised.
The controller records the final clean-HEAD ordered validation separately after
the PR-reference commit. Shared window/panel checks below ran during T1.2;
combined PTY/resource acceptance remains assigned to Slice 13.

Initial final-candidate validation passed checks 1-7, then the documentation
check rejected the missing Git schema source. Adding that source corrected the
collector; the same documentation command then passed with 482 commands, 263
configuration entries, 76 pages and 485 local links/anchors. The final ordered
receipt is regenerated for the corrected clean candidate.

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status:: -- --test-threads=1`: 21 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1`: 10 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration reference -- --test-threads=1`: 11 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`: 10 command and 11 configuration checks passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test windows --test panels -- --test-threads=1`: 41 window and 19 panel checks passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::quit -- --test-threads=1`: 1 passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::lifecycle::shutdown -- --test-threads=1`: 1 passed, covering both profiles.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`: passed after correcting the Git schema source inventory.
- `git diff --check`: passed.

## Risks and Follow-ups

Joint PTY, binary/RSS/idle measurements and combined resource acceptance remain
assigned to Slice 13 by the approved plan. The full delivery gate follows all
approved implementation slices under controller ownership.
