# File Tree Selection Tracker

Status: In progress
Date: 2026-11-07
Delivery set: rev-tree-selection
Approved: product owner; one-slice plan, tracker and Looper delivery
Plan: [File Tree Selection Delivery](../plan/rev-tree-selection.md)
Specification: [File Tree Selection and Active Buffer](../spec/editor-tree-selection.md)
Model: [Tree Selection Policy](../spec/editor-tree-selection-model.md)
Ticket: [TKT-20261031093408](../ticket/reviewing/20261031093408-highlight-tree-selection-and-sync-active-buffer.md)
Concern: [Pending Editor Work](../README.md)
Active slice: None - Slice 1 of 1 integrated; final gate pending
Progress owner: retained Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Selection and following | `feat/rev-tree-selection-01-follow` | `feat(tree): retain selection and follow the active buffer` | `ops/default/report/slices/rev-tree-selection/slice-1-follow.md` |

## Slice 1: Selection and following

State: delivered
Dependency: approved preparation on clean dev; command-description set closed
Requirements: TSV01-TSV04 and the approved configuration/model policy
Branch: `feat/rev-tree-selection-01-follow`
PR: `#308`
Integrated: `b3e77ecb57a3dfd3d141a911ae319ccf72ab2ef8`
Report: `ops/default/report/slices/rev-tree-selection/slice-1-follow.md`
Validation: the ten exact ordered commands in the plan and local configuration

- [x] T1.1: Record actual ticket start and implement default-on configuration,
  separate browsing/active-file identity and event-owned loaded-entry matching.
- [x] T1.2: Deliver persistent single-row rendering and native acceptance for
  focus, manual navigation, missing targets, overlapping roots and unchanged source.
- [x] T1.3: Complete Diataxis/Spore and Unreleased guidance; introduce the report,
  record task/commit/PR evidence and move the ticket to reviewing before the gate.

## Final gate and closure

- [x] Verify the single canonical PR integrated and its report recorded.
- [ ] Run the unchanged selected gate on exact clean dev and record its evidence.
- [ ] Mark the report delivered and the ticket solved with resolution fixed.
- [ ] Complete the plan, tracker, specification and model and reconcile the index.
- [ ] Preserve tested runtime and guide inputs during documentation-only closure.

Slice 1 of 1 is integrated through canonical PR #308 and its report is delivered.
The ten ordered slice checks passed through the controller validator on the
clean PR-reference and integrated commit `b3e77ecb57a3dfd3d141a911ae319ccf72ab2ef8`.

Gate preparation verified locked offline metadata for 118 packages, Rust/Cargo
1.98.0, Spore and guide inputs, independent target directories and no active
primary-checkout build. Only report/tracker delivery metadata changes after
integration. The controller will run the unchanged ten-command gate against
the prepared clean, pushed dev HEAD. The source ticket remains reviewing; final
gate evidence, ticket resolution and delivery-set closure are pending.
Other pending visibility, alias and capability work remains separate.

Task commits: T1.1 `0b5e987a20a02bbef1d7c53a1ae91749ab42b813`;
T1.2 `88087201423c6ea4e331d0328fd295ae94942607`;
T1.3 `750da5ece50597b114c3fff02150793219bb2ab2`.
