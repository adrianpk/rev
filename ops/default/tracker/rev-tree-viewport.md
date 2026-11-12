# Tree Viewport Tracker

Status: Approved
Date: 2026-11-12
Delivery set: rev-tree-viewport
Approved: product owner; specification and established Looper delivery workflow
Plan: [Tree Viewport Delivery](../plan/rev-tree-viewport.md)
Specification: [File Tree Width and Horizontal Navigation](../spec/editor-tree-viewport.md)
Model: [Tree Viewport Policy](../spec/editor-tree-viewport-model.md)
Ticket: [TKT-20261108175339](../ticket/in_progress/20261108175339-improve-file-tree-panel-width-and-horizontal-navigation.md)
Active slice: Slice 1 of 2 - Width policy
Progress owner: native Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Width policy | `feat/rev-tree-viewport-01-width` | `feat(tree): add stable automatic and manual width controls` | `ops/default/report/slices/rev-tree-viewport/slice-1-width.md` |
| Slice 2 | Horizontal navigation | `feat/rev-tree-viewport-02-horizontal` | `feat(tree): pan the shared horizontal viewport` | `ops/default/report/slices/rev-tree-viewport/slice-2-horizontal.md` |

## Slice 1: Width policy

State: reviewing
Dependency: approved preparation committed/pushed on clean dev
Requirements: TVP01, TVP02, TVP03 and width-related TVP05, TVP06
Branch: `feat/rev-tree-viewport-01-width`
PR: [#30](https://forge.adrianpk.com/rev/rev/pulls/30)
Report: `ops/default/report/slices/rev-tree-viewport/slice-1-width.md`
Validation: the six exact ordered commands in the plan and local configuration

- [x] T1.1: Record real ticket start; implement complete auto_width configuration,
  expanded-model high-water sizing and desired/actual width lifecycle.
- [x] T1.2: Deliver named/local/prefixed width and mode controls with native
  regressions for policy, geometry, context, Unicode and source preservation.
- [x] T1.3: Deliver width guidance, catalogs/sample, User Guide/Spore, Unreleased,
  existing executable acceptance and canonical report with real commit evidence.

## Slice 2: Horizontal navigation

State: pending
Dependency: Slice 1 integrated and closed on dev
Requirements: TVP04, TVP05, TVP06 and joint TVP01-TVP03 acceptance
Branch: `feat/rev-tree-viewport-02-horizontal`
PR: pending
Report: `ops/default/report/slices/rev-tree-viewport/slice-2-horizontal.md`
Validation: the six exact ordered commands in the plan and local configuration

- [ ] T2.1: Implement shared one-column viewport, glyph-safe presentation,
  retained position and correct clamping/reset transitions.
- [ ] T2.2: Deliver arrow/named routes, retained directory operations and native
  regression protection for keys/hints, pointer, following and context identity.
- [ ] T2.3: Complete combined native/PTY acceptance, documentation/Spore and
  Unreleased; report real evidence and move source ticket to reviewing.

## Final gate and closure

- [ ] Verify both canonical PRs integrated and reports recorded.
- [ ] Run unchanged selected gate on exact clean integrated dev; record candidate,
  commands, receipt, actual costs, execution environment and evidence limits.
- [ ] Mark reports delivered; solve the source ticket with resolution fixed.
- [ ] Complete specification, model, plan, tracker and operational index.
- [ ] Preserve tested runtime/guide inputs during documentation-only closure.
- [ ] Remove only this set's clean inactive integrated worktrees without force;
  verify clean synchronized dev and final closure commit.

Slice 1 is implemented in its dedicated worktree with focused, executable and documentation checks. Canonical PR #30 is open against dev; implementation commit is `77f96b3b69c959fdeff8f0f9590ac074d0c0a7c6`. Final clean-HEAD validation follows the PR-reference commit before controller handoff. Slice 2 and the integrated gate remain pending.
