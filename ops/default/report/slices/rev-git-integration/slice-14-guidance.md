# Slice 14: Git Learning Path and Reference

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-14-guidance`
PR: pending

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

## Validation

Documentation and Spore validation are pending completion of T14.3.
The exact clean PR head and ordered successful commands will be recorded by
controller validation after the PR-reference follow-up commit.

## Risks and Follow-ups

The final delivery-set gate and exact-candidate closure remain controller-owned.
