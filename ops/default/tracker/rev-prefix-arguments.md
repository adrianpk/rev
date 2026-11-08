# Command Prefix Arguments Tracker

Status: Approved
Date: 2026-11-08
Delivery set: rev-prefix-arguments
Approved: product owner approved the specification and requested plan/slicing
Execution authorized: one four-slice Looper run, 2026-11-08
Plan: [Command Prefix Arguments Delivery](../plan/rev-prefix-arguments.md)
Specification: [Command Prefix Arguments](../spec/editor-prefix-arguments.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 2 of 4, reviewing
Progress owner: the authorized Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Prefix entry and movement | `feat/rev-prefix-arguments-01-entry` | `feat(input): add prefix arguments and counted movement` | `ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md` |
| Slice 2 | Counted edits and history | `feat/rev-prefix-arguments-02-editing` | `feat(editing): apply prefix arguments to native edits and history` | `ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md` |
| Slice 3 | Contextual argument behavior | `feat/rev-prefix-arguments-03-contexts` | `feat(input): integrate contextual prefix argument behavior` | `ops/default/report/slices/rev-prefix-arguments/slice-3-contexts.md` |
| Slice 4 | Acceptance and guidance | `test/rev-prefix-arguments-04-acceptance` | `test(input): verify prefix argument workflows and document usage` | `ops/default/report/slices/rev-prefix-arguments/slice-4-acceptance.md` |

## Slice 1: Prefix entry and movement

State: delivered
Dependency: approved preparation published; execution authorized.
Branch: `feat/rev-prefix-arguments-01-entry`
PR: [#314](https://forge.adrianpk.com/adrianpk/rev/pulls/314)
Integrated: `c558786beb1925902b749dd89f3a2e09810a5412` (canonical PR #314, Rebase + Fast-forward)
Report: `ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md`
Validation: formatting, Clippy, commands (22/22), wrapping (26/26), documentation and whitespace passed. See the canonical report for exact commands; the final clean-HEAD receipt is controller-owned.

- [x] T1.1: Define transient argument forms, canonical helpers/alias, complete descriptions, effective bindings and checked entry/cancellation.
- [x] T1.2: Carry arguments through native input and M-x; deliver signed character/word/vertical movement with ordinary wrapping units and preserve existing C-u routes.
- [x] T1.3: Add prefix_arguments native cases, update current catalogs and incremental references, and introduce the canonical report.

Task commits: `1b47ffb80d805a3625fed42bd6309df5fc03780a` implements
T1.1-T1.3; `5dc05dee3044da581a964b47e9abe68504649ff6` completes the
T1.2 wrapping preflight.

### Slice 1 validation

The shared layout preflight settles initial Org presentation before block/table
workers capture their view. This resolves the two baseline wrapping failures
without changing source/view admission or test assertions. All selected slice
checks pass. Canonical PR #314 is integrated; the delivery-set gate remains pending.

## Slice 2: Counted edits and history

State: reviewing
Dependency: Slice 1 integrated.
Branch: `feat/rev-prefix-arguments-02-editing`
PR: [#315](https://forge.adrianpk.com/adrianpk/rev/pulls/315)
Integrated: pending
Report: `ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md`
Validation: formatting, Clippy, selected commands (24/24), editing (110/110), documentation and whitespace passed. See the canonical report for exact commands; the final clean-HEAD receipt is controller-owned.

- [x] T2.1: Implement counted literal/newline/tab/deletion preparation, region precedence, generated-byte admission and one native undo unit.
- [x] T2.2: Implement logical-line/word kill spans, earlier-entry yank, signed yank rotation and bounded native undo/redo groups.
- [x] T2.3: Extend real native editing/ring/history fixtures and prefix_arguments command acceptance; update the report and completed user-visible Unreleased outcome.

Task commits: `8a34b5ae72b9ef09a0fcebdaa8c245dd9444d9aa` implements
T2.1, T2.2 and T2.3 native acceptance;
`a59daebb930f7dcd5b8c2ffbc6754919d610c6e5` introduces the report and completes
T2.3 references, Unreleased guidance and local validation reporting.

## Slice 3: Contextual argument behavior

State: pending
Dependency: Slice 2 integrated.
Branch: `feat/rev-prefix-arguments-03-contexts`
PR: pending
Integrated: pending
Report: `ops/default/report/slices/rev-prefix-arguments/slice-3-contexts.md`
Validation: focused subset defined in the plan

- [ ] T3.1: Complete line/buffer positioning, row-based supplied scrolling, recentering, direct goto-line and loaded tree-row counts.
- [ ] T3.2: Complete named shell variants, C-u continuation/remapping precedence, input-owner admission and modal count conflict/Escape behavior.
- [ ] T3.3: Verify cooperative work, cancellation, revision/owner invalidation, resource bounds and idle behavior through native command/modal/shell fixtures and record the report.

## Slice 4: Acceptance and guidance

State: pending
Dependency: Slice 3 integrated.
Branch: `test/rev-prefix-arguments-04-acceptance`
PR: pending
Integrated: pending
Report: `ops/default/report/slices/rev-prefix-arguments/slice-4-acceptance.md`
Validation: focused subset defined in the plan; final slice uses the unchanged combined selection

- [ ] T4.1: Complete integrated native acceptance for every command policy and both-profile ownership transitions, with representative large-count and cancellation cases.
- [ ] T4.2: Reconcile complete command/binding/default references and description inventory; add concise User Guide/how-to/reference guidance and build the Markdown-derived Spore guide.
- [ ] T4.3: Finalize reports and task evidence, verify clean canonical integration and prepare the unchanged selected controller gate; closure remains separate.

## Final gate and closure

- [ ] Verify all four canonical integrations and delivered report identities.
- [ ] Run the unchanged ten-command selected gate on exact clean dev.
- [ ] Record tested candidate, controller receipt and actual acceptance results.
- [ ] Complete specification, plan and tracker and reconcile the index.
- [ ] Confirm only the eight configured closure documents changed afterward.

Slice 1 is canonically integrated. Slice 2 is active; Slices 3 and 4 remain
pending. The retained execution checkpoint owns live progress and exact
clean-HEAD receipts. Delivery-set acceptance remains pending.
