# Slice 6: Branches and Both Tag Kinds

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-06-refs`
PR: pending

## Purpose

Deliver GIT08 and its GIT17-GIT19 integration through the approved T6.1-T6.3
[plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

T6.1 adds exact ref inspection and local branch requests:

- b or y in Status opens native Refs in both profiles. Local branches,
  remote-tracking refs, symbolic refs, tags, configured remote names and
  available full upstream/tracking facts retain exact byte identities.
  Annotated tag objects and their peeled targets remain separate.
- HEAD stays explicitly symbolic, unborn or detached, including read-only
  bare-repository observations. Ref and remote count limits produce a visible
  truncated state that cannot authorize mutation. Remote URL values are not
  retained in public ref observations.
- n collects a new local name at the selected exact commit, S switches the
  selected local branch, R collects its replacement name and D requests local
  deletion. A scrollable native review shows the full target and names. Only
  explicit y accepts; Return is not acceptance, n or C-g cancels.
- Git validates names. Creation does not enable tracking implicitly. Existing
  names, stale refs/HEAD, current-branch deletion and unmerged deletion reject.
  Git retains its non-forced deletion and linked-worktree protections.
- Switch admission rejects incomplete or dirty index/worktree observations and
  modified repository-backed buffers. Source stamps are checked again at
  confirmation. Started changes refresh compatible Status and Refs together;
  source file observations use the existing automatic-or-ask reload owner.
- q returns from Refs to Status and the retained source layout. View exit
  retains started work. Failed observation leaves confirmed facts visible and
  stale; g can establish a fresh compatible observation.

## Implementation Notes

The existing repository/process owner runs structured installed-Git argv,
serializes shared storage for mutation and retains changing-child diagnostics
across refresh and verification. No command reparses a displayed ref name or
abbreviated object. A complete captured ref set and HEAD are checked before
execution; index/worktree cleanliness is rechecked for switch. A postcondition
checks the accepted ref/object and HEAD relation, with uncertain effects
remaining explicit instead of replaying the child.

Ref metadata fields retain the 4096-byte bound. Raw output, parsed candidates,
source stamps, review text and public observations use the existing shared
32 MiB admission. Counts use `git.max_refs` and `git.max_remotes`; observation
and changing-action deadlines remain the configured existing limits.

## Contracts Added or Changed

- `git-refs`, `git-branch-create`, `git-branch-switch`, `git-branch-rename` and
  `git-branch-delete` have native handlers, catalog/reference entries and
  both-profile contextual bindings.
- `keys.git_refs` has an independent map and complete active sample. n owns
  branch creation; navigation uses Up/Down or C-p/C-n.
- Ref refresh publishes Status and Refs with one compatible observation;
  incompatible old targets cannot supply changing authority.
- No dependency or persistent repository store was added.

## Files of Interest

- `src/git/refs.rs`, `src/git/work/refs.rs`: exact observations, bounded
  installed-Git reads, branch preconditions, execution and postconditions.
- `src/session/git/refs.rs`: native list selection, names, source admission,
  explicit scrollable review and file-owner reconciliation.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: actual factory
  bindings and complete configuration sample.
- `tests/git_integration/refs.rs`, `refs_presentation.rs`: real object/ref effects,
  stale and dirty-state rejection, and both-profile native interaction.

## Validation

T6.1 development validation passed. Final exact-HEAD validation will
run through the controller after T6.2-T6.3 and PR-reference follow-up.
The focused ref fixtures use serial native execution, isolated installed Git,
neutral identity/configuration and the existing unchanged five-second fixture
wait. The first six tests passed in 17.70 seconds after the new worktree's
initial build. A missing process-presentation enum arm found by the first
compile was corrected before that successful test run.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs:: -- --test-threads=1` passed all six tests in 17.15 seconds after scrollable review integration.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration refs::presentation::native_branches -- --test-threads=1` passed after adding full SHA-256 review and review-scrolling assertions in both profiles.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1` passed 21 tests.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` passed: 508 commands, 267 configuration entries, 76 reachable pages and 485 links.
- `git diff --check`

## Risks and Follow-ups

T6.2 tag creation/deletion and T6.3 acceptance remain pending. Combined native
PTY/resource acceptance remains Slice 13; the complete User Guide remains
Slice 14. Revalidation detects changed Git facts and does not claim an exclusive
transaction against external Git clients.
