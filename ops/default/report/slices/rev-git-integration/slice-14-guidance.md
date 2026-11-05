# Slice 14: Git Learning Path and Reference

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-14-guidance`
PR: [#271](https://forge.adrianpk.com/adrianpk/rev/pulls/271)

## Purpose

Make GIT01-GIT20 discoverable through T14.1-T14.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T14.1 adds an independent three-chapter Git learning path, task guides,
  a control/configuration reference and source/process ownership explanation.
- The local exercise shows saved/index differences, compact Status/diff
  feedback, explicit commit submission, history and local tags without Org.
- Task guides cover exact staging, groups, amend/fixup, explicit fetch and
  integration, reviewed single-branch push, ordered revisions and conflicts.

- T14.2 aligns the Spore book with the independent Git path and extends the
  existing documentation checker to verify its index, chapter links and order.
- Prepared-family procedures describe literal input, separate review/execute,
  non-overwriting output, checked application and unknown arbitrary effects.

- T14.3 consolidates Unreleased Git outcomes, maps every required family/variant
  to existing semantic assertions and records additive baseline/index links.
- Closure fields retain pending exact-candidate/gate identities for the
  controller; historical editor and slice receipts keep their original meaning.

## Implementation Notes

Canonical Markdown remains under the existing Diataxis quadrants. The Git
reference connects the existing complete command, key and preference catalogs
instead of maintaining a second exhaustive inventory. Git guidance is labelled
as current Unreleased behavior alongside the released everyday editing guide.

Verified controller integration of Slice 13 closes its existing report/tracker
at PR #270 and activates the recorded final documentation slice.

## Contracts Added or Changed

User documentation only; native Git, source and process contracts are unchanged.

## Files of Interest

- `docs/tutorials/user-guide/git/index.md` and chapters 17-19: independent exercise.
- `docs/how-to/review-and-commit.md`, `integrate-git-changes.md`: task procedures.
- `docs/reference/git.md`: exact catalog/settings and finite-lifetime entry.
- `docs/explanation/git-ownership.md`: source, observation and job authority.

- `publication/userguide/index.sptree`, `examples/documentation-check.rs`: shared navigation contract.
- `docs/how-to/prepare-git-commands.md`: prepared family and direct routes.

- `ops/default/spec/editor-git-integration-evidence.md`: requirements/variant map and provenance.
- `CHANGELOG.md`: user-facing Unreleased outcomes.

## Validation

Exact ordered commands for the clean PR-reference head:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `cargo fmt --all -- --check`
- `git diff --check`

Coverage-map links and named native assertion owners resolve. Versioned
changelog history is byte-for-byte unchanged. These are documentation
consistency checks; runtime scenarios were not rerun for this slice.

The controller retains the exact head, ordered successful command list and
log in its validation receipt after the PR-reference follow-up. This report
does not substitute for that receipt.

## Risks and Follow-ups

The final delivery-set gate and exact-candidate closure remain controller-owned.

## Integration

[PR #271](https://forge.adrianpk.com/adrianpk/rev/pulls/271) merged through
controller-owned Rebase + Fast-forward into dev at
`71524f02b11264a59a8049d4c10bd64e627017d4`. Controller receipt
`56IS2Q3SWAQ5436XOUMSJIQ3HI` passed the four ordered commands listed above at
that exact head: 85 reachable pages, 550 local links/anchors, 593 commands,
271 settings, successful Spore projection/rendering, formatter and whitespace.
This closes Slice 14 of 14. The selected full gate and exact-candidate
delivery-set closure remain pending with the controller.
