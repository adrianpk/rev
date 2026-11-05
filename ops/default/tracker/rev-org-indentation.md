# Org Indentation Tracker

Status: In progress
Date: 2026-11-05
Delivery set: rev-org-indentation
Plan: [Org Indentation Delivery](../plan/rev-org-indentation.md)
Spec: [Org Visual Indentation](../spec/editor-org-indentation.md)
Model: [Org Visual Indentation Model](../spec/editor-org-indentation-model.md)
Approved: product owner, 2026-11-05; specification and explicit autonomous four-slice Looper startup through dev
Active slice: None - all four slices integrated; combined gate pending
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

State: delivered
Dependency: Slice 1 integrated
Requirements: OIN01-OIN03, OIN05
Implementation: `66e6654eeba004dd997c74144c55253aca5bc9b6` (T2.1-T2.3)
PR: `#277`
Report: [Slice 2](../report/slices/rev-org-indentation/slice-2-presentation.md)
Integrated: `8e839d21cdb029012d6f5c6400c6f5abec6cbff3`
Validation: all five required commands passed through Looper for that exact clean HEAD; 346 selected native tests and 17 additional view regressions passed locally. Documentation validates 593 commands and 281 configuration entries. Canonical PR #277 is merged.

- [x] T2.1: Admit the boolean, complete sample and provenance through startup configuration.
- [x] T2.2: Integrate the same mapping across rendering, editing, navigation, overlays and pointer input.
- [x] T2.3: Verify enabled/disabled real sessions and retain honest legacy geometry coverage.

## Slice 3: Buffer-local runtime commands

State: delivered
Dependency: Slice 2 integrated
Requirements: OIN02-OIN05
Implementation: `e6dc1c0f1b07e046309911867b9782d87b24008e` (T3.1-T3.3)
PR: `#278`
Report: [Slice 3](../report/slices/rev-org-indentation/slice-3-controls.md)
Integrated: `660b97410fd96f6cfb3b664b48dceee7b37ddfc3`
Validation: all six required commands passed through Looper for that exact clean HEAD; 356 selected native tests, 20 modal contexts and one additional readonly result-owner case passed locally. The catalog contains 596 commands. Canonical PR #278 is merged.

- [x] T3.1: Deliver native toggle/enable/disable through M-x, colon and configurable keymaps.
- [x] T3.2: Preserve policy and source anchors through views, edits, reload and Org lifecycle.
- [x] T3.3: Verify both profiles and policy independence; reconcile catalogs and Unreleased.

## Slice 4: Joint acceptance and guidance

State: delivered
Dependency: Slice 3 integrated
Requirements: OIN01-OIN05
Implementation: `da5fc782b23df9d5d360ba306e7fd8629006b7a8` (T4.1-T4.3)
PR: `#279`
Report: [Slice 4](../report/slices/rev-org-indentation/slice-4-acceptance.md)
Integrated: `6c010648a49847b54f914063a2261e7403dd8c34`
Validation: all six required commands passed through Looper for that exact clean HEAD; 32 selected native tests and eight focused cache/work cases passed locally. Development native and real PTY journeys passed both profiles; three normal-release resource cases passed with original bounds. Documentation validates 596 commands, 281 settings and 560 links/anchors. Canonical PR #279 is merged; the combined gate and strict Spore build remain controller-owned.

- [x] T4.1: Deliver representative native/resource/PTY acceptance with the Rust/Bash entrypoint.
- [x] T4.2: Update Markdown-derived Spore guidance, references and affected specification clauses.
- [x] T4.3: Record reports and hand off the exact integrated candidate for gate and closure.

## Final gate and closure

Gate prerequisites are ready: Rust/Cargo 1.98, the required shell/PTY utilities
and Spore are available. Offline locked Cargo metadata resolves all selected
targets; acceptance helpers and publication inputs are present. Both gate
entrypoint scripts pass Bash syntax checks. The combined gate and strict Spore
build have not run during this preparation.

The plan owns the ordered slice and final-gate commands. All four slices are
integrated and their reports are delivered. The combined delivery gate remains
pending. Controller identity and live execution progress belong to Looper; it
validates the exact clean dev candidate. Record its hash, receipt, successful
commands, report introductions and evidence limits here, then mark the paired
specification/model, plan and tracker Completed. A failed gate remains open;
follow native recovery and the authorized repair workflow rather than starting
a replacement delivery set. The owner capped this set at six slices; this map
contains four and does not grant authority to add more automatically.
