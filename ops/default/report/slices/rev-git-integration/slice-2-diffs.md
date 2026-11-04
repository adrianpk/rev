# Slice 2: Changes Hunks and Source Visits

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-02-diffs`
PR: pending

## Purpose

Deliver exact staged, unstaged and untracked comparisons, native expansion and
read-only detail, and canonical current-source visits under GIT04 and
GIT17-GIT19. The [approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md) define T2.1-T2.3.

## Delivered Behavior

T2.1 adds the comparison backend:

- One exact path and layer are admitted from a confirmed Status generation.
  Staged detail compares HEAD with the index, including the empty unborn
  baseline. Unstaged detail compares the index with working-tree bytes;
  untracked preview uses an explicit empty baseline.
- Installed Git receives structured OS arguments and literal pathspec policy.
  Comparison reads disable external diff/textconv helpers, color and inherited
  diff formatting while retaining exact raw patch evidence and path bytes.
- Complete comparisons are read twice, with repository/HEAD/path checks before
  and after. Changed bytes, index facts or generation reject publication and
  retain the previous inspection. A Status refresh invalidates older detail.
- Immutable comparisons expose exact old/new ranges and patch spans for
  semantic text hunks. Binary, rename, type-change, submodule, unmerged,
  malformed and truncated detail retain explicit classifications. A staged
  rename does not hide eligible text hunks in its separate unstaged layer.
- Reached detail, aggregate output or session admission is observable;
  incomplete or malformed content has no supported hunk authority. Failed
  detail does not replace confirmed Status or its last retained inspection.

T2.2 native presentation and T2.3 source/navigation acceptance remain pending.

## Implementation Notes

Comparisons share Slice 1's installed-Git worker, stable operation identities,
finite observation deadline, capture/cleanup owner and process-record limits.
The operation kind distinguishes Status from diff work. No implicit shell
parses path arguments, and no display label is reconstructed as a target.

Raw comparison bytes, worker-indexed display chunks and semantic hunks are
admitted before allocation under the shared 32 MiB budget. Contexts retain one
latest inspection; external Arc readers keep replaced observations and detail
charged until released. Display chunks are bounded to 2048 input bytes.

Hunk identities bind an immutable parent comparison and its exact patch unit.
Parser success alone grants no current-state authority: callers must also check
the captured Status generation and comparison completeness. Future changing
operations must revalidate the complete comparison before effects.

## Contracts Added or Changed

- Added typed diff baselines, classifications, completeness, raw evidence,
  bounded display indices and semantic hunk identities under the Git owner.
- Added generation-bound comparison admission and separate retained-detail
  failure state without replacing confirmed Status.
- Added typed operation kinds to progress, completion and process records.
- Dependency and persistent storage contracts are unchanged.

## Files of Interest

- `src/git/diff.rs`: comparison/hunk records, bounded indexing and parsing.
- `src/git/work/detail.rs`: exact Git argv, paired reads and target verification.
- `src/git.rs`: comparison admission, immutable publication and current checks.
- `tests/git_integration/diffs.rs`: real layer, path, special-file and stale-data
  fixtures, including controlled output corruption and concurrent file change.

## Validation

Development checks establish the backend foundation. The earlier combined Git
run passed 27 tests; the focused comparison rerun covers the layer-specific
rename correction. The controller's final clean-HEAD receipt follows complete
T2.2-T2.3 implementation and PR-reference follow-up.

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs:: -- --test-threads=1`: 6 passed after the layer-specific follow-up, about 34 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- --test-threads=1`: 27 passed before the layer-specific follow-up, about 66 seconds.
- `git diff --check`: passed.

The first bounded-detail fixture requested 1024 bytes and was rejected by the
approved 4096-byte minimum. The corrected fixture uses that minimum and retains
its assertions on explicit truncation, byte admission and unsupported hunks.

## Risks and Follow-ups

Complete T2.2 native expansion, read-only detail and canonical source visits,
then T2.3 both-profile navigation and ownership acceptance before preparing the
Slice 2 PR. Combined PTY and resource measurements remain assigned to Slice 13.
