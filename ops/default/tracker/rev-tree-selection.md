# File Tree Selection Tracker

Status: Ready
Date: 2026-11-07
Delivery set: rev-tree-selection
Approved: product owner; one-slice plan, tracker and Looper delivery
Plan: [File Tree Selection Delivery](../plan/rev-tree-selection.md)
Specification: [File Tree Selection and Active Buffer](../spec/editor-tree-selection.md)
Model: [Tree Selection Policy](../spec/editor-tree-selection-model.md)
Ticket: [TKT-20261031093408](../ticket/ready/20261031093408-highlight-tree-selection-and-sync-active-buffer.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 - Selection and following
Progress owner: retained Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Selection and following | `feat/rev-tree-selection-01-follow` | `feat(tree): retain selection and follow the active buffer` | `ops/default/report/slices/rev-tree-selection/slice-1-follow.md` |

## Slice 1: Selection and following

State: ready
Dependency: approved preparation on clean dev; command-description set closed
Requirements: TSV01-TSV04 and the approved configuration/model policy
Branch: `feat/rev-tree-selection-01-follow`
PR: pending
Report: `ops/default/report/slices/rev-tree-selection/slice-1-follow.md`
Validation: the ten exact ordered commands in the plan and local configuration

- [ ] T1.1: Record actual ticket start and implement default-on configuration,
  separate browsing/active-file identity and event-owned loaded-entry matching.
- [ ] T1.2: Deliver persistent single-row rendering and native acceptance for
  focus, manual navigation, missing targets, overlapping roots and unchanged source.
- [ ] T1.3: Complete Diataxis/Spore and Unreleased guidance; introduce the report,
  record task/commit/PR evidence and move the ticket to reviewing before the gate.

## Final gate and closure

- [ ] Verify the single canonical PR integrated and its report recorded.
- [ ] Run the unchanged selected gate on exact clean dev and record its evidence.
- [ ] Mark the report delivered and the ticket solved with resolution fixed.
- [ ] Complete the plan, tracker, specification and model and reconcile the index.
- [ ] Preserve tested runtime and guide inputs during documentation-only closure.

Preparation makes no claim that the behavior is implemented or that its gate
has passed. Other pending visibility, alias and capability work remains separate.
