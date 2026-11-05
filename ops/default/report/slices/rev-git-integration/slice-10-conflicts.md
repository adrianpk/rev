# Slice 10: Native Conflict Resolution

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-10-conflicts`
PR: pending

## Purpose

Deliver GIT12 and its GIT17-GIT19 contracts through T10.1-T10.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T10.1: git-conflicts refreshes actual Git sequence state and exact selectable
  unmerged paths. Operation Return inspects the selected path's available
  Base/Current/Applied stages, full objects, modes and bounded read-only text.
- Roles reflect the actual operation: rebase Current is its destination/onto
  side and Applied is the original replayed commit; revert Base is the reverted
  commit and Applied is its selected parent. Missing lineage remains explicit.
- Missing, binary, structural, non-text and partial sides remain visible.
  Conflict Return visits the canonical editable result through the existing
  file owner, preserving unsaved text. q returns to Operation without changing
  index entries or Git-owned sequence state.
- T10.2 checked replacement/deletion/saved-resolution staging and T10.3 optional
  AI explanation remain pending. Their commands are not registered yet.

## Implementation Notes

The existing repository worker and sequence fingerprint own inspection. Actual
metadata, refs, HEAD, complete index records and working-tree evidence are
checked before and after reading immutable blobs. Raw operating-system paths
and full SHA-1/SHA-256 identities supply authority; escaped labels never do.
Unmerged index entries without an operation marker are also observable without
inventing sequence lineage. Inspection creates no repository mutation.

Available ordinary-file stages share the configured detail byte cap, limited
by the existing 1 MiB scratch bound. Payloads, retained line ranges, path
selection and visible formatting participate in the same 32 MiB Git admission
budget. Existing repository/context counts, owned child lifetime, output caps,
deadlines and 5-second fixture waits remain unchanged. Unsupported modes retain
exact metadata without treating them as editable text. No dependency, provider
or general process owner was added.

Conflict targets travel with wrapped Operation rows into keyboard and pointer
selection. The full-canvas Sequence mapping is distinct from Status's fixed
header offset; painted state includes its viewport and conflict inspection.
Canonical result ingress rejects missing files, symlinks and paths resolving
outside the captured worktree, and reuses native file-buffer ownership.

## Contracts Added or Changed

- Two functional native commands: git-conflicts and git-conflict-visit.
- keys.git_sequence has 33 active entries, including Return inspection.
- keys.git_conflict has 19 active entries for read-only inspection and result
  visits in both interaction profiles.
- Command and configuration catalogs contain 541 commands and 269 entries.

## Files of Interest

- `src/git/conflicts.rs`, `src/git/work/conflicts.rs`: exact roles, stage
  metadata, bounded blob capture and stale observation rejection.
- `src/git/work/integration/sequence.rs`: actual unmerged index evidence even
  when no sequence family marker exists.
- `src/session/git/conflicts.rs`, `src/session/git/refs/integration.rs`: native
  inspection, exact selection and canonical editable result visits.
- `tests/git_integration/conflicts.rs` and `conflicts_presentation.rs`: real
  repository stages, both object formats/profiles and source ownership.

## Validation

T10.1 development passed six conflict cases (19.18 seconds): four actual
operation families in both object formats with a newline/non-UTF-8 leading-dash
path; explicit missing/binary/partial/type-change stages; external unmerged
index without sequence markers; stale disk rejection; both-profile read-only
inspection/unsaved native result visits; and exact refreshed pointer selection.
Six existing native integration cases passed in 25.06 seconds. All 21 command
and configuration reference cases, strict Clippy, formatter, documentation and
whitespace checks passed. Documentation covers 541 commands, 269 configuration
entries, 76 pages and 485 local links.

Development corrections retained the approved assertions and all deadlines:
Git relocates the ordinary-file side of a type change to another exact path,
so the fixture now selects the actual symlink path for its mode assertion.
Pointer coverage exposed and corrected the new Sequence click offset; the test
still clicks a different exact conflict and checks its inspected file.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

Final clean-HEAD validation belongs to the controller after T10.2-T10.3 and the
canonical PR-reference follow-up commit. No final receipt is claimed yet.

## Risks and Follow-ups

T10.2 and T10.3 remain required within this slice. Combined PTY/resource
acceptance remains Slice 13 and full User Guide work remains Slice 14.
Integration and delivery-set closure remain controller-owned.
