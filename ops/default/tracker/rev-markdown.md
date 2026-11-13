# Markdown Tracker

Status: Approved
Date: 2026-11-13
Delivery set: rev-markdown
Slice strategy: behavior-first
Approved: product owner, 2026-11-13; approved specification, slicing and established autonomous Looper execution/integration workflow
Plan: [Markdown Delivery](../plan/rev-markdown.md)
Spec: [Markdown Editing and Presentation](../spec/editor-markdown-mode.md)
Model: [Markdown Policy and Actions](../spec/editor-markdown-mode-model.md)
Active slice: Slice 1 - Recognition and colors
Progress owner: native Looper checkpoint, canonical PR state and exact-candidate validation receipts
Parser decision: owner accepted pulldown-cmark 0.13.6 subject to compatible licensing; exact package and minimal graph verified

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Recognition and colors | `feat/rev-markdown-01-recognition` | `feat(markdown): recognize and color native Markdown` | `ops/default/report/slices/rev-markdown/slice-1-recognition.md` |
| Slice 2 | Heading presentation | `feat/rev-markdown-02-headings` | `feat(markdown): present level-colored heading symbols` | `ops/default/report/slices/rev-markdown/slice-2-headings.md` |
| Slice 3 | List presentation and editing | `feat/rev-markdown-03-lists` | `feat(markdown): present and edit nested lists` | `ops/default/report/slices/rev-markdown/slice-3-lists.md` |
| Slice 4 | Table creation and field editing | `feat/rev-markdown-04-tables` | `feat(markdown): create align and navigate GFM tables` | `ops/default/report/slices/rev-markdown/slice-4-tables.md` |
| Slice 5 | Table row and column operations | `feat/rev-markdown-05-table-structure` | `feat(markdown): edit table rows and columns safely` | `ops/default/report/slices/rev-markdown/slice-5-table-structure.md` |
| Slice 6 | Acceptance and guidance | `feat/rev-markdown-06-acceptance` | `test(markdown): verify native editing and document workflows` | `ops/default/report/slices/rev-markdown/slice-6-acceptance.md` |

## Slice 1: Recognition and colors

State: reviewing
Requirements: MD01, MD02 semantic colors, MD06
Branch: `feat/rev-markdown-01-recognition`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-1-recognition.md`
Validation: the exact ordered checks for Slice 1 in the plan and frozen local configuration

- [x] T1.1: Implement a private, source-range Markdown observation owner and revision-qualified worker integration; add only the accepted pinned parser and its reviewed minimal transitive graph; retain their upstream notices. Preserve literal blocks, CommonMark context, bounded snapshots and cancellation admission.
- [x] T1.2: Add automatic .md/.markdown recognition, markdown-mode/Text-mode transitions, markdown.enabled and the six semantic palette leaves with full parsing, validation, merge/provenance, sample and configuration catalog coverage. Keep LSP optional and existing modes unaffected.
- [x] T1.3: Create tests/markdown.rs with named source/context/offline/invalidation cases, including native palette output and disabled coloring. Update affected references/Unreleased for the usable behavior and record the canonical report.

Task commits: T1.1-T1.2 and native regressions at `98cda39921c31c00997b4400cc967ec9bad4abad`;
T1.3 report introduction is recorded with the PR references.

Development evidence: 205 command and 166 configuration tests passed; the
corrected focused Markdown/syntax batch passed 12 and 30 tests. Full first-party
lint and the exact 691-command/314-setting catalogs pass. The frozen combined
batch and final clean PR HEAD receipt remain required before worker handoff.

## Slice 2: Heading presentation

State: pending
Requirements: MD02, MD05, heading-related MD01/MD06
Branch: `feat/rev-markdown-02-headings`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-2-headings.md`
Validation: the exact ordered checks for Slice 2 in the plan and frozen local configuration

- [ ] T2.1: Add the six heading palette leaves and markdown.pretty; project only admitted ATX source prefixes with the exact approved marker/color profile. Color Setext titles while retaining underlines; preserve literal source and canonical width/cursor mappings.
- [ ] T2.2: Implement markdown-pretty-mode and markdown-reset-presentation with view-local overrides, split inheritance, buffer switches, explicit source view and read-only presentation. Keep structural recognition independent of color policy.
- [ ] T2.3: Cover real terminal cells, wide/combining source, clipped/wrapped prefixes, selection/search precedence, copy/save/history, Text-mode suspension and existing Org projection regressions. Complete references, catalogs and the slice report.

Task commits: pending.

## Slice 3: List presentation and editing

State: pending
Requirements: MD03 and list-related MD05/MD06
Branch: `feat/rev-markdown-03-lists`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-3-lists.md`
Validation: the exact ordered checks for Slice 3 in the plan and frozen local configuration

- [ ] T3.1: Decorate unordered source markers by actual nesting depth using the approved three-marker cycle and the shared view pretty flag. Retain ordered source numbers, continuation paragraphs, indentation and literal-code ownership.
- [ ] T3.2: Implement continue-item, empty-item level exit and checked indent/outdent with child/continuation retention, ordered increment and one-action undo. Introduce the Markdown editing keymap overlay and approved Return/Meta-Return dispatch without capturing modal Normal/Visual or completion-popup ownership.
- [ ] T3.3: Deliver both-profile named commands, effective key hints, override inheritance, read-only/cancellation cases, Unicode and shared-point regressions. Update catalogs, list guidance, Unreleased and the canonical report.

Task commits: pending.

## Slice 4: Table creation and field editing

State: pending
Requirements: MD04 creation/alignment/navigation, MD05, MD06
Branch: `feat/rev-markdown-04-tables`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-4-tables.md`
Validation: the exact ordered checks for Slice 4 in the plan and frozen local configuration

- [ ] T4.1: Observe GFM tables from admitted parser context with original source cell spans, delimiter alignments and the fixed finite table profile. Accept optional outer pipes and escaped pipes; refuse structured operations that would discard extra body cells or depend on stale/incomplete context.
- [ ] T4.2: Implement explicit table creation, alignment, next/previous field and next-row actions, skipping delimiters and appending body rows when required. Preserve exact cell source, newline identity and existing canonical transaction authority; aligned navigation adds no history.
- [ ] T4.3: Wire contextual table routes in both editing profiles and named selectors. Cover header-only tables, all alignments, literal code, ragged/invalid shapes, read-only refusal, limits and exact undo/redo; document behavior and report acceptance.

Task commits: pending.

## Slice 5: Table row and column operations

State: pending
Requirements: MD04 structure, shared-point MD05 and MD06
Branch: `feat/rev-markdown-05-table-structure`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-5-table-structure.md`
Validation: the exact ordered checks for Slice 5 in the plan and frozen local configuration

- [ ] T5.1: Implement body-row and whole-column insertion/deletion with the model-defined placement. Retain the header/delimiter, minimum one column, alignment metadata and all surviving cell text; reject unsupported targets before mutation.
- [ ] T5.2: Map retained cell source, initiating point and other view marks/points through the existing transaction owner, including Unicode and escaped/code content. Keep read-only, stale revision, canceled work and failure paths atomic with one operation per undo unit.
- [ ] T5.3: Finish structural commands/bindings and discoverability in both profiles; protect inherited remaps, modal state boundaries and Org table behavior. Record table geometry/fidelity regressions, documentation and the canonical report.

Task commits: pending.

## Slice 6: Acceptance and guidance

State: pending
Requirements: Joint MD01-MD06 and the complete reproducibility matrix
Branch: `feat/rev-markdown-06-acceptance`
PR: pending
Report: `ops/default/report/slices/rev-markdown/slice-6-acceptance.md`
Validation: the exact ordered checks for Slice 6 in the plan and frozen local configuration

- [ ] T6.1: Add scripts/check-markdown.sh using existing owned native/PTY helpers and one finite fixture matrix. Verify actual cells, source/pretty views, both profiles, NO_COLOR, BOM/CRLF, Unicode, copy/save, list/table commands and terminal restoration with owned children and temporary roots.
- [ ] T6.2: Complete the Markdown reference and User Guide lesson, exact command/configuration/binding catalogs, packaged sample, learning-path navigation, Spore projection and Unreleased. Keep canonical Markdown as the documentation source and retain all existing guide/reference assertions.
- [ ] T6.3: Record native/PTY evidence and complete the report. Prepare the clean published integrated candidate for the controller-owned full gate; after success close only configured Markdown artifacts and remove this set's clean inactive integrated worktrees.

Task commits: pending.

## Integration and closure

- [x] Confirm the parser dependency decision and compatible MIT licensing without changing the approved behavior.
- [x] Commit/push the approved scope, six-slice plan/tracker, model and index at 8767a298; record the subsequent dependency decision before startup.
- [ ] Generate/register the final native configuration, pass preflight and start one detached Looper owner.
- [ ] Integrate all six canonical PRs through controller-owned Rebase + Fast-forward into dev.
- [ ] Remove each clean inactive integrated task worktree without force; preserve pre-existing worktrees and branches.
- [ ] Run the frozen Full gate on the exact clean integrated candidate and record its receipt and actual cost.
- [ ] Close only configured Markdown paths, with reports delivered and spec/model/plan/tracker Completed.
- [ ] Verify clean synchronized dev, unchanged tested runtime/guide inputs and final Acta/commit evidence.

No runtime or validation outcome is claimed by preparation. The native
configuration will pin the plan's slice check prefixes and unchanged Full gate
before the first checkpoint. Only the explicitly accepted parser and its reviewed graph may be introduced;
additional dependencies still require separate approval.
