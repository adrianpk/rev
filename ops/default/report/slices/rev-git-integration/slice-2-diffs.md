# Slice 2: Changes Hunks and Source Visits

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-02-diffs`
PR: `#259`

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

T2.2 adds native inspection and visits:

- Tab toggles a Status group, exact path comparison or semantic hunk body.
  One path and one hunk body expand at a time; navigation skips display-only
  metadata and resolves hunk headers through immutable comparison identities.
- `git-diff` opens full-canvas read-only patch detail. Refresh replaces Status
  and the same comparison, preserving a surviving exact patch unit; failed or
  incomplete detail remains explicitly stale/unverified or truncated.
- Return visits the actual current file through normal canonical ingress,
  retaining an existing modified buffer. Missing files and directories remain
  inspection targets without creating a new empty source buffer.
- q returns one presentation level. Process inspection retains its Status or
  Diff origin and distinguishes operation kinds. C-g in diff navigation does
  not cancel a process; process cancellation retains its explicit context.
- Pointer receipts include comparison identity, expansion and viewport state.
  Generated content remains outside writable source, Org, LSP and save paths.

T2.3 verifies both startup profiles through native key, command, pointer and
source owners: distinct layers and multiple hunks, exact byte paths, refresh
selection, stale receipts and resize, failed detail retention, modified source
visits, retained split identities and live terminal return. Staged unusual-name
fixtures also exercise literal pathspecs in SHA-1 and SHA-256 repositories.
Hunk lookup uses its private ordinal and full immutable identity, so walking
bounded display rows does not repeatedly scan the complete hunk list.

## Implementation Notes

Implementation commits: `11a9b7c87ab940764b18b15dec849bf5c655d898`
(T2.1), `c23fb12f2daa75a9856c1db9f59aeb9fd27f3f57` (T2.2) and
`805723003834aa2022ed5cda2003697954835f27` (T2.3).
[PR #259](https://forge.adrianpk.com/adrianpk/rev/pulls/259) targets dev;
controller-owned Rebase + Fast-forward integrated it into dev at
`78e4c43cf79792337006786f8752a093badc05d6`. Controller validation
`45HQQNKFYXODWKRVVQBKHITJD5` passed all seven ordered checks for that exact
commit, including 32 Git and 21 reference tests, documentation and whitespace.
Slice 2 of 14 completed.

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
- Registered working `git-diff` and `git-visit-file` commands; Return visits
  current source in Status/Diff and inspects a selected operation in Processes.
- Dependency and persistent storage contracts are unchanged.

## Files of Interest

- `src/git/diff.rs`: comparison/hunk records, bounded indexing and parsing.
- `src/git/work/detail.rs`: exact Git argv, paired reads and target verification.
- `src/git.rs`: comparison admission, immutable publication and current checks.
- `src/session/git/comparisons.rs`: native comparison state and canonical visits.
- `src/session/git.rs`: semantic expansion, navigation and pointer receipts.
- `tests/git_integration/diffs.rs`: real layer, path, special-file and stale-data
  fixtures, including controlled output corruption and concurrent file change.

## Validation

Development checks establish the backend foundation. The earlier combined Git
run passed 27 tests; the focused comparison rerun covers the layer-specific
rename correction. Nine native Status/presentation/process regression checks
pass after T2.2. T2.3's five native acceptance cases pass in both profiles;
the extended staged unusual-name fixture passes in both object formats.
The final ordered clean-HEAD validation is recorded through the controller
after the PR-reference follow-up; development results below are not that receipt.

- `cargo fmt --all -- --check`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs:: -- --test-threads=1`: 6 passed after the layer-specific follow-up, about 34 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration -- --test-threads=1`: 27 passed before the layer-specific follow-up, about 66 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::presentation -- --test-threads=1`: 9 passed after T2.2.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs::presentation -- --test-threads=1`: initial 4 native cases passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs::presentation::failed_and_hidden_detail -- --test-threads=1`: passed after adding failed replacement and refresh-return acceptance.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration diffs::unborn_and_path_bytes -- --test-threads=1`: extended literal staged-path fixtures passed.
- `git diff --check`: passed.

The first controller candidate `680e73237042a0f06a9124fd6b7f930e8b0a1162`
passed formatting, strict Clippy, all 11 diff tests and all 21 Status tests.
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
then failed the bindings catalog assertion: the new visit command appeared twice
and the retained process-show command lacked its own catalog entry. The table
now documents each live command exactly once while retaining Return's actual
contextual dispatch. Checks and native behavior are unchanged; the complete
ordered receipt must be regenerated for the corrected candidate.

The first bounded-detail fixture requested 1024 bytes and was rejected by the
approved 4096-byte minimum. The corrected fixture uses that minimum and retains
its assertions on explicit truncation, byte admission and unsupported hunks.

## Risks and Follow-ups

Combined PTY and resource measurements remain assigned to Slice 13. This slice
prepares exact read-only hunk identities; changing index and discard behavior
remains assigned to Slice 3.
