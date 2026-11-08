# Command Prefix Arguments Tracker

Status: Approved
Date: 2026-11-08
Delivery set: rev-prefix-arguments
Approved: product owner approved the specification and requested plan/slicing
Execution authorized: one four-slice Looper run, 2026-11-08
Plan: [Command Prefix Arguments Delivery](../plan/rev-prefix-arguments.md)
Specification: [Command Prefix Arguments](../spec/editor-prefix-arguments.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 of 4, implementation complete; required focused validation blocked
Progress owner: the authorized Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Prefix entry and movement | `feat/rev-prefix-arguments-01-entry` | `feat(input): add prefix arguments and counted movement` | `ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md` |
| Slice 2 | Counted edits and history | `feat/rev-prefix-arguments-02-editing` | `feat(editing): apply prefix arguments to native edits and history` | `ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md` |
| Slice 3 | Contextual argument behavior | `feat/rev-prefix-arguments-03-contexts` | `feat(input): integrate contextual prefix argument behavior` | `ops/default/report/slices/rev-prefix-arguments/slice-3-contexts.md` |
| Slice 4 | Acceptance and guidance | `test/rev-prefix-arguments-04-acceptance` | `test(input): verify prefix argument workflows and document usage` | `ops/default/report/slices/rev-prefix-arguments/slice-4-acceptance.md` |

## Slice 1: Prefix entry and movement

State: implementation complete; validation blocked
Dependency: approved preparation published; execution authorized.
Branch: `feat/rev-prefix-arguments-01-entry`
PR: pending
Integrated: pending
Report: `ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md`
Validation: formatting, Clippy, commands (22/22), and documentation passed; required wrapping failed (24/26). See the canonical report for commands and baseline evidence.

- [x] T1.1: Define transient argument forms, canonical helpers/alias, complete descriptions, effective bindings and checked entry/cancellation.
- [x] T1.2: Carry arguments through native input and M-x; deliver signed character/word/vertical movement with ordinary wrapping units and preserve existing C-u routes.
- [x] T1.3: Add prefix_arguments native cases, update current catalogs and incremental references, and introduce the canonical report.

### Slice 1 validation blocker

The required wrapping target fails in `org::source_draft` and
`org::table_fields`. Both failures reproduce at the approved preparation
baseline `c268f429e78a73631eb7fddae69d6722bf6109c7` without Slice 1 changes.
The former reports "Source block or view changed during observation" and
retains the parent buffer; the latter leaves Scalar(2) instead of Scalar(21).
The slice report records the focused baseline comparison and exact commands.

No canonical PR has been opened. Required commands and assertions are unchanged;
this unit is not ready for integration. Resolution of these pre-existing
required failures remains outside the delivered prefix entry/movement change.
The retained checkpoint and exact clean-HEAD validation receipt own continuation.

## Slice 2: Counted edits and history

State: pending
Dependency: Slice 1 integrated.
Branch: `feat/rev-prefix-arguments-02-editing`
PR: pending
Integrated: pending
Report: `ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md`
Validation: focused subset defined in the plan

- [ ] T2.1: Implement counted literal/newline/tab/deletion preparation, region precedence, generated-byte admission and one native undo unit.
- [ ] T2.2: Implement logical-line/word kill spans, earlier-entry yank, signed yank rotation and bounded native undo/redo groups.
- [ ] T2.3: Extend real native editing/ring/history fixtures and prefix_arguments command acceptance; update the report and completed user-visible Unreleased outcome.

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

Slice 1 implementation and its focused validation blocker are recorded above.
No canonical integration or delivery-set acceptance is claimed. The retained
execution checkpoint owns live progress and the exact clean-HEAD receipt; later
slices remain pending.
