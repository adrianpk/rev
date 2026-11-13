# Tree Viewport Tracker

Status: Completed
Date: 2026-11-12
Delivery set: rev-tree-viewport
Approved: product owner; specification and established Looper delivery workflow
Plan: [Tree Viewport Delivery](../plan/rev-tree-viewport.md)
Specification: [File Tree Width and Horizontal Navigation](../spec/editor-tree-viewport.md)
Model: [Tree Viewport Policy](../spec/editor-tree-viewport-model.md)
Ticket: [TKT-20261108175339](../ticket/solved/20261108175339-improve-file-tree-panel-width-and-horizontal-navigation.md)
Active slice: None - both slices delivered; delivery set completed
Progress owner: native Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Width policy | `feat/rev-tree-viewport-01-width` | `feat(tree): add stable automatic and manual width controls` | `ops/default/report/slices/rev-tree-viewport/slice-1-width.md` |
| Slice 2 | Horizontal navigation | `feat/rev-tree-viewport-02-horizontal` | `feat(tree): pan the shared horizontal viewport` | `ops/default/report/slices/rev-tree-viewport/slice-2-horizontal.md` |

## Slice 1: Width policy

State: delivered
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

State: delivered
Dependency: Slice 1 integrated and closed on dev
Requirements: TVP04, TVP05, TVP06 and joint TVP01-TVP03 acceptance
Branch: `feat/rev-tree-viewport-02-horizontal`
PR: [#31](https://forge.adrianpk.com/rev/rev/pulls/31)
Report: `ops/default/report/slices/rev-tree-viewport/slice-2-horizontal.md`
Validation: the six exact ordered commands in the plan and local configuration

- [x] T2.1: Implement shared one-column viewport, glyph-safe presentation,
  retained position and correct clamping/reset transitions.
- [x] T2.2: Deliver arrow/named routes, retained directory operations and native
  regression protection for keys/hints, pointer, following and context identity.
- [x] T2.3: Complete combined native/PTY acceptance, documentation/Spore and
  Unreleased; report real evidence and move source ticket to reviewing.

## Final gate and closure

- [x] Verify both canonical PRs integrated and reports recorded.
- [x] Run unchanged selected gate on exact clean integrated dev; record candidate,
  commands, receipt, actual costs, execution environment and evidence limits.
- [x] Mark reports delivered; solve the source ticket with resolution fixed.
- [x] Complete specification, model, plan, tracker and operational index.
- [x] Preserve tested runtime/guide inputs during documentation-only closure.
- [x] Remove only this set's clean inactive integrated worktrees without force;
  verify clean synchronized dev and final closure commit.

Slice 1 of 2 is delivered through merged canonical PR #30 at
`03b0d22b322a97b7d374568b7bcbeb807bc38e73`. Exact clean-HEAD receipt
`EXJXXB4JIRODJ7SWQL3EMTUVBW` passed all six ordered commands, including
635 selected native tests. Slice 2 of 2 is delivered through merged canonical
PR #31 at `4a4f89a0bdf8c276d7066413beccce2a474952e3`. Receipt
`6TH5M2YS454DBGIGDPRZSUDQ3V` passed all six slice commands and 639 native tests.
Both reports are delivered. The source ticket is solved with resolution fixed
after the controller passed the fresh integrated gate.

## Integrated gate preparation

Both PR merge states, ancestry and clean synchronized dev were verified against
the canonical forge and Acta. The two owned slice worktrees are removed after
integration; unrelated worktrees and their caches are preserved. Existing hooks
remain active and unchanged. The approved task-owned same-UID PID namespace
runner is intact and its bounded startup check confirms UID 1000, child PID 2
and parent PID 1. Exact wrapper/runner/checksum evidence remains outside versioned
artifacts. Each checkout retains its own Cargo target.

The preparation phase did not run the full gate. The controller subsequently
passed the unchanged combined gate on the clean pushed preparation commit;
that evidence is separate from the earlier slice receipt.

## Delivery-set closure

Tested candidate: `1c98ee49bc6af413b99148f270d1395dc80f9512`.
Successful gate receipt: `DYYC4ATKJZTUCR4UA2GE5UDS43`.
The exact combined command and full evidence are in the
[Slice 2 closure](../report/slices/rev-tree-viewport/slice-2-horizontal.md#delivery-set-closure).

The gate passed full workspace lint, all 639 selected native tests (zero failures
or ignored cases), Better File Tree native/PTY acceptance, 690-command and
300-setting catalogs, 106 documentation pages / 743 local links and the generated
User Guide. All closure tasks are complete. Both PRs are merged; specification,
model, plan and tracker are Completed. The source ticket is solved with
resolution fixed. Owned fixtures finished and cleaned up; no active owned build
or test fixture remained at closure. The two integrated owned worktrees were
removed, preserving unrelated worktrees and their private caches.

Only the configured closure Markdown paths differ from the tested candidate.
The final documentation closure is committed and pushed on clean synchronized
dev. No runtime, test, guide input or acceptance assertion changes follow the gate.
