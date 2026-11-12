# Standalone Acceptance Correction

Status: In progress
Date: 2026-11-12
Delivery set: rev-architecture-quality-acceptance
Parent delivery set: rev-architecture-quality
Slice strategy: behavior-first
Approved: owner explicitly requires all recorded acceptance debt to be corrected before closing the parent set and requested Looper visibility
Active slice: Slice 1, Standalone acceptance debt
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Concern: [Pending Editor Work](../README.md)
Tracker: [Standalone Acceptance Tracker](../tracker/rev-architecture-quality-acceptance.md)
Parent plan: [Architecture Quality Delivery](rev-architecture-quality.md)

## Outcome and authority

Repay the five recorded standalone acceptance failures before closing the reopened
parent delivery. One corrective PR targets dev. The completed original Looper run
retains its configuration, receipts, conversation and counters. This bounded run
owns the additional correction and must close the parent artifacts only after its
expanded exact-candidate gate passes. Do not start a successor.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Standalone acceptance debt | `fix/rev-architecture-quality-acceptance` | `fix(acceptance): resolve standalone editor journeys` | `ops/default/report/slices/rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md` |

## Slice 1: Standalone acceptance debt

- T1.1: Correct file-tree native and both-profile PTY acceptance, including Unicode labels and the explicit parent navigation row.
- T1.2: Correct command-ranking native acceptance against the current command inventory; retain restart and paired-publisher checks.
- T1.3: Correct keybinding-hint native and both-profile PTY acceptance against the current theme and complete command-row contract.
- T1.4: Correct wrapping native directory identity while retaining the full PTY and release performance/resource probes.
- T1.5: Verify the complete Projects/Workspaces native and both-profile PTY journeys; correct only a reproduced defect.
- T1.6: Integrate the correction with focused receipts, then let the controller run the expanded gate and close both this correction and the parent delivery.

These map to parent T9.4-T9.8 and T10.4. Correct fixtures when their expectations
contradict established product contracts; correct runtime behavior when those
contracts are violated. Source areas are the five existing example families,
their five existing check scripts, and the directly affected runtime/test owners
only if a reproduced product defect requires them. Preserve Unicode inputs,
negative checks, exact bytes/history/state, color and frame assertions, idle and
terminal-restoration checks, process ownership and all time/count/byte bounds.
No check may be removed, made optional or skipped to obtain a pass. Keep existing
one-job build settings and private per-worktree targets/caches.

## Baseline evidence

On ce14022c, the complete file-tree script fails its Unicode suffix assertion,
command-ranking fails its empty-query order, keybinding hints fail the native
foreground role, and wrapping fails its directory BufferId expectation.
Projects/Workspaces passes its complete native, non-modal and modal journeys;
retain that coverage and do not add a speculative fix. Reproduce/fix later phases
through each original entrypoint instead of bypassing an earlier failure.

## Frozen ordered focused checks

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope examples`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-file-tree.sh`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-command-ranking.sh`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-command-keybinding-hints.sh`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-wrap-navigation.sh`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-projects-workspaces.sh`
7. `git diff --check`

## Expanded integrated gate

First run all five complete acceptance entrypoints above, in that order. Then run
the parent plan's unchanged eight-stage frozen integrated gate. Both parts must
pass on the exact clean integrated candidate. The controller owns this gate.
A cold release build and the existing wrapping performance matrix are expected
costs. Retain the original deadlines rather than increasing them for host load.

## Closure

After successful gate evidence, mark this plan/tracker Completed and the report
delivered. Close parent T9.4-T9.8 and T10.4, the parent plan/tracker/spec/review,
the Slice 9/10 reports and concern index with the fresh receipt. Replace the
current standalone-debt deferrals with verified correction evidence while keeping
the historical failure/acceptance record explicit. Only configured Markdown
closure paths may differ from the tested candidate. Main, releases and publication
keep their existing approval boundaries.
