# Org Indentation Tracker

Status: In progress
Date: 2026-11-05
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../plan/rev-org-indentation.md)
Spec: [Org Visual Indentation](../spec/editor-org-indentation.md)
Model: [Org Visual Indentation Model](../spec/editor-org-indentation-model.md)
Approved: product owner, 2026-11-05; specification and explicit autonomous four-slice Looper startup through dev
Active slice: Slice 2 - Configured editing presentation
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Projection and checked mappings | `feat/rev-org-indentation-01-projection` | `feat(org): model virtual indentation in checked layout` | `ops/default/report/slices/rev-org-indentation/slice-1-projection.md` |
| Slice 2 | Configured editing presentation | `feat/rev-org-indentation-02-presentation` | `feat(org): align heading bodies with configured visual indentation` | `ops/default/report/slices/rev-org-indentation/slice-2-presentation.md` |
| Slice 3 | Buffer-local runtime commands | `feat/rev-org-indentation-03-controls` | `feat(org): control visual indentation per buffer` | `ops/default/report/slices/rev-org-indentation/slice-3-controls.md` |
| Slice 4 | Joint acceptance and guidance | `test/rev-org-indentation-04-acceptance` | `test(org): verify visual indentation and document controls` | `ops/default/report/slices/rev-org-indentation/slice-4-acceptance.md` |

## Slice 1: Projection and checked mappings

State: delivered
Dependency: approved preparation on clean dev
Requirements: OIN01, OIN03, OIN05
Implementation: `54395f4feea63a7cc0de34582f4d18c3840c09de` (T1.1-T1.3)
PR: `#276`
Report: [Slice 1](../report/slices/rev-org-indentation/slice-1-projection.md)
Integrated: `3849381e48c4951a33cba391201fd0bc7cb4f840`
Validation: all five required commands passed through Looper for that exact clean HEAD; 17 library-filter, 16 heading and 26 wrapping tests passed, plus 11 focused structural regressions during implementation. Canonical PR #276 is merged.

- [x] T1.1: Extend native enclosing-heading evidence and checked virtual-margin mappings.
- [x] T1.2: Handle structural ownership, continuations, tabs/Unicode, clipping, caps and invalidation.
- [x] T1.3: Verify projection, source round trips and bounded cases at native owner boundaries.

## Slice 2: Configured editing presentation

State: reviewing
Dependency: Slice 1 integrated
Requirements: OIN01-OIN03, OIN05
Implementation: `66e6654eeba004dd997c74144c55253aca5bc9b6` (T2.1-T2.3)
PR: `#277`
Report: [Slice 2](../report/slices/rev-org-indentation/slice-2-presentation.md)
Validation: all five required commands passed locally; 346 selected native tests and 17 additional view regressions passed. Documentation validates 593 commands and 281 configuration entries. Looper records exact clean-HEAD evidence after the PR-reference commit.

- [x] T2.1: Admit the boolean, complete sample and provenance through startup configuration.
- [x] T2.2: Integrate the same mapping across rendering, editing, navigation, overlays and pointer input.
- [x] T2.3: Verify enabled/disabled real sessions and retain honest legacy geometry coverage.

## Slice 3: Buffer-local runtime commands

State: pending
Dependency: Slice 2 integrated
Requirements: OIN02-OIN05

- [ ] T3.1: Deliver native toggle/enable/disable through M-x, colon and configurable keymaps.
- [ ] T3.2: Preserve policy and source anchors through views, edits, reload and Org lifecycle.
- [ ] T3.3: Verify both profiles and policy independence; reconcile catalogs and Unreleased.

## Slice 4: Joint acceptance and guidance

State: pending
Dependency: Slice 3 integrated
Requirements: OIN01-OIN05

- [ ] T4.1: Deliver representative native/resource/PTY acceptance with the Rust/Bash entrypoint.
- [ ] T4.2: Update Markdown-derived Spore guidance, references and affected specification clauses.
- [ ] T4.3: Record reports and hand off the exact integrated candidate for gate and closure.

## Final gate and closure

The plan owns the ordered slice and final-gate commands. Slice 1 is integrated;
Slice 2 is implemented for review. Later slices and the combined delivery gate remain pending. Controller identity
and live execution progress belong to Looper. After all four slices integrate, it
validates the exact clean dev candidate. Record its hash, receipt, successful
commands, report introductions and evidence limits here, then mark the paired
specification/model, plan and tracker Completed. A failed gate remains open;
follow native recovery and the authorized repair workflow rather than starting
a replacement delivery set. The owner capped this set at six slices; this map
contains four and does not grant authority to add more automatically.
