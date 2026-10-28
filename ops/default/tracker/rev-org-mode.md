# Native Org Mode Tracker

Status: Approved
Date: 2026-10-28
Delivery set: rev-org-mode
Scope authority: product owner; Org-in-Emacs behavior with the advanced-spreadsheet exception.
Approved: product owner, 2026-10-28; explicitly requested starting this single 64-slice Looper cycle after delegating remaining behavior choices to Org in Emacs.
Slice strategy: layered
Active slice: Slice 7 fold-projection
Plan: [Native Org Plan](../plan/rev-org-mode.md)
Spec: [Org Mode](../spec/editor-org-mode.md)
Model: [Org Mode Model](../spec/editor-org-mode-model.md)
Full gate: `scripts/check-all.sh`

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | define native Org source contracts | `feat/rev-org-01-source-contract` | `feat(slice-1): define native Org source contracts` | `ops/default/report/slices/rev-org-mode/slice-1-source-contract.md` |
| Slice 2 | index Org structure with bounded work | `feat/rev-org-02-incremental-index` | `feat(slice-2): index Org structure with bounded work` | `ops/default/report/slices/rev-org-mode/slice-2-incremental-index.md` |
| Slice 3 | recognize inline Org source forms | `feat/rev-org-03-inline-syntax` | `feat(slice-3): recognize inline Org source forms` | `ops/default/report/slices/rev-org-mode/slice-3-inline-syntax.md` |
| Slice 4 | recognize Org headings metadata and blocks | `feat/rev-org-04-structural-syntax` | `feat(slice-4): recognize Org headings metadata and blocks` | `ops/default/report/slices/rev-org-mode/slice-4-structural-syntax.md` |
| Slice 5 | activate native Org mode and contexts | `feat/rev-org-05-mode-contexts` | `feat(slice-5): activate native Org mode and contexts` | `ops/default/report/slices/rev-org-mode/slice-5-mode-contexts.md` |
| Slice 6 | color native Org source semantically | `feat/rev-org-06-semantic-colors` | `feat(slice-6): color native Org source semantically` | `ops/default/report/slices/rev-org-mode/slice-6-semantic-colors.md` |
| Slice 7 | map folded Org views to canonical source | `feat/rev-org-07-fold-projection` | `feat(slice-7): map folded Org views to canonical source` | `ops/default/report/slices/rev-org-mode/slice-7-fold-projection.md` |
| Slice 8 | deliver native Org visibility controls | `feat/rev-org-08-fold-controls` | `feat(slice-8): deliver native Org visibility controls` | `ops/default/report/slices/rev-org-mode/slice-8-fold-controls.md` |
| Slice 9 | insert and navigate Org headings | `feat/rev-org-09-outline-navigation` | `feat(slice-9): insert and navigate Org headings` | `ops/default/report/slices/rev-org-mode/slice-9-outline-navigation.md` |
| Slice 10 | move and promote coherent Org subtrees | `feat/rev-org-10-subtree-edits` | `feat(slice-10): move and promote coherent Org subtrees` | `ops/default/report/slices/rev-org-mode/slice-10-subtree-edits.md` |
| Slice 11 | edit Org lists and progress cookies | `feat/rev-org-11-lists-checkboxes` | `feat(slice-11): edit Org lists and progress cookies` | `ops/default/report/slices/rev-org-mode/slice-11-lists-checkboxes.md` |
| Slice 12 | edit Org drawers and inherited properties | `feat/rev-org-12-drawers-properties` | `feat(slice-12): edit Org drawers and inherited properties` | `ops/default/report/slices/rev-org-mode/slice-12-drawers-properties.md` |
| Slice 13 | edit and match Org task tags | `feat/rev-org-13-tags-matching` | `feat(slice-13): edit and match Org task tags` | `ops/default/report/slices/rev-org-mode/slice-13-tags-matching.md` |
| Slice 14 | deliver Org task workflows and logging | `feat/rev-org-14-todo-workflows` | `feat(slice-14): deliver Org task workflows and logging` | `ops/default/report/slices/rev-org-mode/slice-14-todo-workflows.md` |
| Slice 15 | record Org effort estimates and priorities | `feat/rev-org-15-effort-priorities` | `feat(slice-15): record Org effort estimates and priorities` | `ops/default/report/slices/rev-org-mode/slice-15-effort-priorities.md` |
| Slice 16 | model Org calendar dates and durations | `feat/rev-org-16-civil-time` | `feat(slice-16): model Org calendar dates and durations` | `ops/default/report/slices/rev-org-mode/slice-16-civil-time.md` |
| Slice 17 | resolve Org local times and zone changes | `feat/rev-org-17-time-zones` | `feat(slice-17): resolve Org local times and zone changes` | `ops/default/report/slices/rev-org-mode/slice-17-time-zones.md` |
| Slice 18 | enter and edit Org timestamps | `feat/rev-org-18-timestamp-entry` | `feat(slice-18): enter and edit Org timestamps` | `ops/default/report/slices/rev-org-mode/slice-18-timestamp-entry.md` |
| Slice 19 | reschedule repeated Org tasks correctly | `feat/rev-org-19-repeaters` | `feat(slice-19): reschedule repeated Org tasks correctly` | `ops/default/report/slices/rev-org-mode/slice-19-repeaters.md` |
| Slice 20 | record native Org task clock intervals | `feat/rev-org-20-clock-records` | `feat(slice-20): record native Org task clock intervals` | `ops/default/report/slices/rev-org-mode/slice-20-clock-records.md` |
| Slice 21 | switch and recover Org task clocks | `feat/rev-org-21-clock-lifecycle` | `feat(slice-21): switch and recover Org task clocks` | `ops/default/report/slices/rev-org-mode/slice-21-clock-lifecycle.md` |
| Slice 22 | compare Org clock totals and effort | `feat/rev-org-22-clock-reports` | `feat(slice-22): compare Org clock totals and effort` | `ops/default/report/slices/rev-org-mode/slice-22-clock-reports.md` |
| Slice 23 | deliver native Org relative and countdown timers | `feat/rev-org-23-timers` | `feat(slice-23): deliver native Org relative and countdown timers` | `ops/default/report/slices/rev-org-mode/slice-23-timers.md` |
| Slice 24 | maintain Org habit schedules and history | `feat/rev-org-24-habit-records` | `feat(slice-24): maintain Org habit schedules and history` | `ops/default/report/slices/rev-org-mode/slice-24-habit-records.md` |
| Slice 25 | render native Org habit consistency graphs | `feat/rev-org-25-habit-graphs` | `feat(slice-25): render native Org habit consistency graphs` | `ops/default/report/slices/rev-org-mode/slice-25-habit-graphs.md` |
| Slice 26 | discover bounded canonical Org agenda sources | `feat/rev-org-26-agenda-sources` | `feat(slice-26): discover bounded canonical Org agenda sources` | `ops/default/report/slices/rev-org-mode/slice-26-agenda-sources.md` |
| Slice 27 | display native day and week Org agendas | `feat/rev-org-27-agenda-calendar` | `feat(slice-27): display native day and week Org agendas` | `ops/default/report/slices/rev-org-mode/slice-27-agenda-calendar.md` |
| Slice 28 | query Org tasks tags and properties | `feat/rev-org-28-agenda-queries` | `feat(slice-28): query Org tasks tags and properties` | `ops/default/report/slices/rev-org-mode/slice-28-agenda-queries.md` |
| Slice 29 | configure native Org agenda views | `feat/rev-org-29-agenda-customization` | `feat(slice-29): configure native Org agenda views` | `ops/default/report/slices/rev-org-mode/slice-29-agenda-customization.md` |
| Slice 30 | edit canonical Org source from the agenda | `feat/rev-org-30-agenda-actions` | `feat(slice-30): edit canonical Org source from the agenda` | `ops/default/report/slices/rev-org-mode/slice-30-agenda-actions.md` |
| Slice 31 | define native Org capture targets and templates | `feat/rev-org-31-capture-templates` | `feat(slice-31): define native Org capture targets and templates` | `ops/default/report/slices/rev-org-mode/slice-31-capture-templates.md` |
| Slice 32 | expand and edit native Org capture drafts | `feat/rev-org-32-capture-drafts` | `feat(slice-32): expand and edit native Org capture drafts` | `ops/default/report/slices/rev-org-mode/slice-32-capture-drafts.md` |
| Slice 33 | finalize or abort Org capture safely | `feat/rev-org-33-capture-finalize` | `feat(slice-33): finalize or abort Org capture safely` | `ops/default/report/slices/rev-org-mode/slice-33-capture-finalize.md` |
| Slice 34 | refile Org subtrees across canonical documents | `feat/rev-org-34-refile` | `feat(slice-34): refile Org subtrees across canonical documents` | `ops/default/report/slices/rev-org-mode/slice-34-refile.md` |
| Slice 35 | archive Org subtrees without silent loss | `feat/rev-org-35-archive` | `feat(slice-35): archive Org subtrees without silent loss` | `ops/default/report/slices/rev-org-mode/slice-35-archive.md` |
| Slice 36 | store and follow checked Org links | `feat/rev-org-36-links` | `feat(slice-36): store and follow checked Org links` | `ops/default/report/slices/rev-org-mode/slice-36-links.md` |
| Slice 37 | manage native Org heading attachments | `feat/rev-org-37-attachments` | `feat(slice-37): manage native Org heading attachments` | `ops/default/report/slices/rev-org-mode/slice-37-attachments.md` |
| Slice 38 | edit native Org footnotes and inline content | `feat/rev-org-38-document-conveniences` | `feat(slice-38): edit native Org footnotes and inline content` | `ops/default/report/slices/rev-org-mode/slice-38-document-conveniences.md` |
| Slice 39 | navigate and align native Org tables | `feat/rev-org-39-table-basics` | `feat(slice-39): navigate and align native Org tables` | `ops/default/report/slices/rev-org-mode/slice-39-table-basics.md` |
| Slice 40 | edit Org table rows columns and references | `feat/rev-org-40-table-structure` | `feat(slice-40): edit Org table rows columns and references` | `ops/default/report/slices/rev-org-mode/slice-40-table-structure.md` |
| Slice 41 | parse the bounded Org formula subset | `feat/rev-org-41-formula-syntax` | `feat(slice-41): parse the bounded Org formula subset` | `ops/default/report/slices/rev-org-mode/slice-41-formula-syntax.md` |
| Slice 42 | evaluate basic Org table formulas safely | `feat/rev-org-42-formula-evaluation` | `feat(slice-42): evaluate basic Org table formulas safely` | `ops/default/report/slices/rev-org-mode/slice-42-formula-evaluation.md` |
| Slice 43 | publish Org recalculation as one transaction | `feat/rev-org-43-formula-publication` | `feat(slice-43): publish Org recalculation as one transaction` | `ops/default/report/slices/rev-org-mode/slice-43-formula-publication.md` |
| Slice 44 | import and export Org table data | `feat/rev-org-44-table-interchange` | `feat(slice-44): import and export Org table data` | `ops/default/report/slices/rev-org-mode/slice-44-table-interchange.md` |
| Slice 45 | display Org property columns and native reports | `feat/rev-org-45-columns-dynamic-blocks` | `feat(slice-45): display Org property columns and native reports` | `ops/default/report/slices/rev-org-mode/slice-45-columns-dynamic-blocks.md` |
| Slice 46 | prepare checked native Org export snapshots | `feat/rev-org-46-export-contract` | `feat(slice-46): prepare checked native Org export snapshots` | `ops/default/report/slices/rev-org-mode/slice-46-export-contract.md` |
| Slice 47 | export Org to text Markdown and Org | `feat/rev-org-47-text-exports` | `feat(slice-47): export Org to text Markdown and Org` | `ops/default/report/slices/rev-org-mode/slice-47-text-exports.md` |
| Slice 48 | export native Org documents to HTML | `feat/rev-org-48-html-export` | `feat(slice-48): export native Org documents to HTML` | `ops/default/report/slices/rev-org-mode/slice-48-html-export.md` |
| Slice 49 | export Org to LaTeX and Beamer | `feat/rev-org-49-tex-exports` | `feat(slice-49): export Org to LaTeX and Beamer` | `ops/default/report/slices/rev-org-mode/slice-49-tex-exports.md` |
| Slice 50 | export native Org to OpenDocument text | `feat/rev-org-50-odt-export` | `feat(slice-50): export native Org to OpenDocument text` | `ops/default/report/slices/rev-org-mode/slice-50-odt-export.md` |
| Slice 51 | export Org to Texinfo and man formats | `feat/rev-org-51-manual-exports` | `feat(slice-51): export Org to Texinfo and man formats` | `ops/default/report/slices/rev-org-mode/slice-51-manual-exports.md` |
| Slice 52 | export Org planning data as iCalendar | `feat/rev-org-52-calendar-export` | `feat(slice-52): export Org planning data as iCalendar` | `ops/default/report/slices/rev-org-mode/slice-52-calendar-export.md` |
| Slice 53 | publish native Org document projects | `feat/rev-org-53-publishing` | `feat(slice-53): publish native Org document projects` | `ops/default/report/slices/rev-org-mode/slice-53-publishing.md` |
| Slice 54 | edit native Org source blocks and headers | `feat/rev-org-54-source-block-editing` | `feat(slice-54): edit native Org source blocks and headers` | `ops/default/report/slices/rev-org-mode/slice-54-source-block-editing.md` |
| Slice 55 | tangle Org source with bounded noweb references | `feat/rev-org-55-tangling` | `feat(slice-55): tangle Org source with bounded noweb references` | `ops/default/report/slices/rev-org-mode/slice-55-tangling.md` |
| Slice 56 | execute selected Org blocks through bounded tools | `feat/rev-org-56-block-execution` | `feat(slice-56): execute selected Org blocks through bounded tools` | `ops/default/report/slices/rev-org-mode/slice-56-block-execution.md` |
| Slice 57 | insert checked Org block and call results | `feat/rev-org-57-block-results` | `feat(slice-57): insert checked Org block and call results` | `ops/default/report/slices/rev-org-mode/slice-57-block-results.md` |
| Slice 58 | import native Org feed entries on demand | `feat/rev-org-58-feeds` | `feat(slice-58): import native Org feed entries on demand` | `ops/default/report/slices/rev-org-mode/slice-58-feeds.md` |
| Slice 59 | preview native Org links and images | `feat/rev-org-59-previews` | `feat(slice-59): preview native Org links and images` | `ops/default/report/slices/rev-org-mode/slice-59-previews.md` |
| Slice 60 | complete the native Org control catalog | `feat/rev-org-60-complete-controls` | `feat(slice-60): complete the native Org control catalog` | `ops/default/report/slices/rev-org-mode/slice-60-complete-controls.md` |
| Slice 61 | verify complete native Org terminal workflows | `test/rev-org-61-combined-terminal` | `test(slice-61): verify complete native Org terminal workflows` | `ops/default/report/slices/rev-org-mode/slice-61-combined-terminal.md` |
| Slice 62 | verify bounded Org work and idle resources | `perf/rev-org-62-bounded-work` | `perf(slice-62): verify bounded Org work and idle resources` | `ops/default/report/slices/rev-org-mode/slice-62-bounded-work.md` |
| Slice 63 | reconcile native Org contracts and references | `docs/rev-org-63-baseline-reconciliation` | `docs(slice-63): reconcile native Org contracts and references` | `ops/default/report/slices/rev-org-mode/slice-63-baseline-reconciliation.md` |
| Slice 64 | prepare complete native Org delivery acceptance | `test/rev-org-64-delivery-acceptance` | `test(slice-64): prepare complete native Org delivery acceptance` | `ops/default/report/slices/rev-org-mode/slice-64-delivery-acceptance.md` |

## Task state

Unchecked means not implemented or verified. Keep one Conventional Commit per
tracker task where practical and preserve rev's existing (Tn.m) suffix only in
this repository. Each report has the seven required sections and its actual
canonical PR and validation identities. Do not create delivered reports in
advance or claim prerequisites are runtime evidence.

### Slice 1: define native Org source contracts

Contracts: ORG01 ORG02 ORG18.
Details and exact checks: [Slice 1](../plan/rev-org-mode.md#slice-1-define-native-org-source-contracts).

- [x] T1.1: Introduce the in-house Org owner and revision-qualified source ranges; keep canonical rope bytes authoritative.
- [x] T1.2: Represent recognized and unknown forms without executing directives or retaining a second editable document.
- [x] T1.3: Introduce the org_source target and verify UTF-8, CRLF, malformed input and checked-range failures.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Task commits: T1.1 `78dac679f1cfda33e32bb841ed6c4220bb75aa88`,
T1.2 `a30dfbe6f753bf8b6d444c59d8658dfadd4c263e`,
T1.3 `1cb2d944a403a5059f559b64844ab7c7564e3bac`.
Canonical PR: [#102](https://forge.adrianpk.com/adrianpk/rev/pulls/102).
Report: [Slice 1](../report/slices/rev-org-mode/slice-1-source-contract.md).
Integration: PR #102 merged into dev at `466b5356cfd1a633b2b37f946009dfe1f242dda0`.
Validation: controller receipt `BPV6RCGRQUGEUWI4ASIDLTG22H` passed all five
ordered checks at that exact candidate, including 30 tests with zero skips.

### Slice 2: index Org structure with bounded work

Contracts: ORG02 ORG18.
Details and exact checks: [Slice 2](../plan/rev-org-mode.md#slice-2-index-org-structure-with-bounded-work).

- [x] T2.1: Build bounded line/ancestor indexes with explicit parse observations and truthful incomplete states.
- [x] T2.2: Invalidate affected scopes after canonical edits and introduce the shared bounded cancellable observation-job owner; never copy the full rope per key.
- [x] T2.3: Verify large edits, stale observations and bounded worker cancellation while preserving ordinary editing.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Task commits: T2.1 `37dbf80bc6cbc268ba508e5e0c17a85db0e6f278`,
T2.2 `f7e2c648754453805ad03572223953f26bddd83d`,
T2.3 `5bf669cb3f2cf826983f729bf0f8248df3948d6f`.
Canonical PR: [#103](https://forge.adrianpk.com/adrianpk/rev/pulls/103).
Report: [Slice 2](../report/slices/rev-org-mode/slice-2-incremental-index.md).
Integration: PR #103 merged into dev at `69ef1aa1c44ba013487b16b94a8c87dcb2fdee1d`.
Validation: controller receipt `UVSJY7HF7ZPWX5FJ2AIZY4UCUR` passed all five
ordered checks at that exact candidate, including 118 tests with zero skips.

### Slice 3: recognize inline Org source forms

Contracts: ORG02 ORG15 ORG16 ORG30.
Details and exact checks: [Slice 3](../plan/rev-org-mode.md#slice-3-recognize-inline-org-source-forms).

- [x] T3.1: Recognize emphasis, code, entities, links, targets and footnotes with exact source spans.
- [x] T3.2: Keep escaping, malformed delimiters and literal/code contexts distinct from structural markup.
- [x] T3.3: Verify Unicode boundaries, nested forms and recovery without normalizing unknown source.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Task commits: T3.1 `31c01cebd212d488717d5503fd41bdaa9d20c836`,
T3.2 `7a186ffd60ec59746527474732e63b3d9a084a37`,
T3.3 `5e4cd90dae99bed02f5070e66d62f3ab3e682267`.
Canonical PR: [#104](https://forge.adrianpk.com/adrianpk/rev/pulls/104).
Report: [Slice 3](../report/slices/rev-org-mode/slice-3-inline-syntax.md).
Integration: PR #104 merged into dev at `6f286b086857056fe3c09e9eb0aca2eec3586ddd`.
Validation: controller receipt `WVGRL3W5TZU3RFM6UXYBPFTHXQ` passed all five
ordered checks at that exact candidate, including 56 tests with zero skips.

### Slice 4: recognize Org headings metadata and blocks

Contracts: ORG02 ORG04 ORG05 ORG12.
Details and exact checks: [Slice 4](../plan/rev-org-mode.md#slice-4-recognize-org-headings-metadata-and-blocks).

- [x] T4.1: Recognize headings, sections, lists, drawers, planning lines, affiliated keywords and tables.
- [x] T4.2: Treat source/example/export blocks as owned literal regions with valid delimiter recovery.
- [x] T4.3: Verify subtree ownership, malformed drawers and apparent headings inside blocks.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Task commits: T4.1 `0b6bf91c2f3bc1260cd9a60d85311facd89b2288`,
T4.2 `08c400135626913eb197cc2cbaa3e4b3429b5b5d`,
T4.3 `f6277f82d149f93e5049040e2a09980c51b98514`.
Canonical PR: [#105](https://forge.adrianpk.com/adrianpk/rev/pulls/105).
Report: [Slice 4](../report/slices/rev-org-mode/slice-4-structural-syntax.md).
Integration: PR #105 merged into dev at `4bc3e524e5088d592c4127215ab6c4e70f6e71b0`.
Validation: controller receipt `E5AMDDY6ER5WIALLDGYXL5Y2CS` passed all five
ordered checks at that exact candidate, including 70 tests with zero skips.

### Slice 5: activate native Org mode and contexts

Contracts: ORG01 ORG17.
Details and exact checks: [Slice 5](../plan/rev-org-mode.md#slice-5-activate-native-org-mode-and-contexts).

- [x] T5.1: Activate the native Org mode by visiting suffix or explicit compiled mode command.
- [x] T5.2: Add Org editing, agenda, capture, calendar and special-edit context routing with prompt precedence.
- [x] T5.3: Introduce org_editing; expose only operational commands and verify context cancellation and plain-text fallback.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Task commits: T5.1 `22e1a1e48e26216844a2ee8aa1698c7cc449be2b`,
T5.2 `f0194fce53e68b203f2575bcf0492192efbc8ea2`,
T5.3 `a1b81966608eed6ef1a08a7358011105d78360b9`.
Canonical PR: [#106](https://forge.adrianpk.com/adrianpk/rev/pulls/106).
Report: [Slice 5](../report/slices/rev-org-mode/slice-5-mode-contexts.md).
Integration: PR #106 merged into dev at `0136e7b8b7ee97491c5c75138531c7ebff4b5c71`.
Validation: controller receipt `BR5SRPHXR34SQCE3JDM75GDU6U` passed all five
ordered checks at that exact candidate, including 301 tests with zero skips.

### Slice 6: color native Org source semantically

Contracts: ORG16 ORG17.
Details and exact checks: [Slice 6](../plan/rev-org-mode.md#slice-6-color-native-org-source-semantically).

- [x] T6.1: Project native Org spans through existing palette and display owners, including literal-block boundaries.
- [x] T6.2: Define typed semantic Org roles with complete factory/sample coverage as each role activates.
- [x] T6.3: Verify incremental coloring, shared views, malformed source and usable no-color rendering.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#107](https://forge.adrianpk.com/adrianpk/rev/pulls/107).
Integrated into `dev` at `c8c2189b1a09e90b3847461f8b2aab35eb6cd0c6`.
Controller receipt `FN2QUFG2U6DIDUYUTCEJKAG52X` passed the required five-command
batch and 128 tests; focused receipt `ZEW4YBTP577HOQTYVDBZIMJJJH` passed
96 configuration tests and reference/template checks at that same head.

### Slice 7: map folded Org views to canonical source

Contracts: ORG03 ORG18.
Details and exact checks: [Slice 7](../plan/rev-org-mode.md#slice-7-map-folded-org-views-to-canonical-source).

- [ ] T7.1: Add view-local visibility projection over source ranges, retaining independent shared-buffer views.
- [ ] T7.2: Map cursor, gutter, search, mouse and selections through hidden ranges without aliasing source positions.
- [ ] T7.3: Verify tiny panes, Unicode, hit testing, stale folds and edit-driven reveal.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 8: deliver native Org visibility controls

Contracts: ORG03 ORG17.
Details and exact checks: [Slice 8](../plan/rev-org-mode.md#slice-8-deliver-native-org-visibility-controls).

- [ ] T8.1: Implement heading TAB cycle, whole-view S-TAB cycle and direct visibility/drawer commands.
- [ ] T8.2: Integrate hierarchy-aware reveal with ordinary motion, searches and saved independent view state.
- [ ] T8.3: Exercise native keys, M-x, cancellation and source immutability in an owned outline PTY journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 9: insert and navigate Org headings

Contracts: ORG04 ORG17.
Details and exact checks: [Slice 9](../plan/rev-org-mode.md#slice-9-insert-and-navigate-org-headings).

- [ ] T9.1: Deliver sibling insertion and next/previous/same-level/parent heading movement.
- [ ] T9.2: Handle contextual M-RET while preserving attached sections, blank lines and final newline state.
- [ ] T9.3: Verify read-only navigation, inactive context behavior, undo and logical source bounds.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 10: move and promote coherent Org subtrees

Contracts: ORG04 ORG02.
Details and exact checks: [Slice 10](../plan/rev-org-mode.md#slice-10-move-and-promote-coherent-org-subtrees).

- [ ] T10.1: Implement checked promote/demote and sibling subtree moves over complete owned source.
- [ ] T10.2: Retain descendants, affiliated metadata, marks and independent views through one atomic history change.
- [ ] T10.3: Verify boundary rejections, malformed parents, regional edits and full undo/redo fidelity.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 11: edit Org lists and progress cookies

Contracts: ORG04 ORG06 ORG30.
Details and exact checks: [Slice 11](../plan/rev-org-mode.md#slice-11-edit-org-lists-and-progress-cookies).

- [ ] T11.1: Continue, indent, outdent, reorder and renumber ordered/unordered/description lists contextually.
- [ ] T11.2: Implement ordinary/radio checkboxes and coherent list/task progress cookies through checked edits.
- [ ] T11.3: Verify nested lists, timer-item contexts, literal blocks and one-operation undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 12: edit Org drawers and inherited properties

Contracts: ORG05 ORG17.
Details and exact checks: [Slice 12](../plan/rev-org-mode.md#slice-12-edit-org-drawers-and-inherited-properties).

- [ ] T12.1: Implement property set/remove/inspect and drawer folding without confusing ordinary and property drawers.
- [ ] T12.2: Resolve local/additive/inherited values with source-owner provenance and safe directive precedence.
- [ ] T12.3: Verify selective inheritance, unknown properties and edits to the actual owner.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 13: edit and match Org task tags

Contracts: ORG05 ORG23.
Details and exact checks: [Slice 13](../plan/rev-org-mode.md#slice-13-edit-and-match-org-task-tags).

- [ ] T13.1: Deliver tag entry, removal, completion and local/inherited membership inspection.
- [ ] T13.2: Implement the declared Org tag/property match grammar with typed comparisons and bounded parsing.
- [ ] T13.3: Verify case sensitivity, escaping, malformed queries and unchanged unrelated source.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 14: deliver Org task workflows and logging

Contracts: ORG06 ORG17.
Details and exact checks: [Slice 14](../plan/rev-org-mode.md#slice-14-deliver-org-task-workflows-and-logging).

- [ ] T14.1: Admit TODO/SEQ_TODO/TYP_TODO declarations and complete native state transitions with selected logging.
- [ ] T14.2: Integrate priorities, blocking rules, CLOSED/state notes and cookies into coherent transactions.
- [ ] T14.3: Verify key/M-x equivalence, read-only behavior and task/metadata history in an owned PTY journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 15: record Org effort estimates and priorities

Contracts: ORG06 ORG21.
Details and exact checks: [Slice 15](../plan/rev-org-mode.md#slice-15-record-org-effort-estimates-and-priorities).

- [ ] T15.1: Deliver native priority and EFFORT editing with Org duration forms and factory choices.
- [ ] T15.2: Keep workflow metadata compatible and expose operational commands in Org and agenda contexts.
- [ ] T15.3: Verify numeric/duration rejection, source spelling preservation and atomic undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 16: model Org calendar dates and durations

Contracts: ORG07 ORG18 ORG21.
Details and exact checks: [Slice 16](../plan/rev-org-mode.md#slice-16-model-org-calendar-dates-and-durations).

- [ ] T16.1: Introduce org_time with checked Gregorian civil dates, intervals and Org duration units.
- [ ] T16.2: Separate scheduling calendar arithmetic from elapsed work durations and formatting.
- [ ] T16.3: Verify leap years, month ends, overflow, duration forms and an injected operation clock.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 17: resolve Org local times and zone changes

Contracts: ORG07 ORG18.
Details and exact checks: [Slice 17](../plan/rev-org-mode.md#slice-17-resolve-org-local-times-and-zone-changes).

- [ ] T17.1: Implement bounded native TZif/POSIX-zone observations and system/local/UTC/IANA selection.
- [ ] T17.2: Capture effective date/zone once per operation; resolve folds/gaps explicitly without guessing.
- [ ] T17.3: Verify DST transitions, invalid zones and external zone changes with owned fixtures.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 18: enter and edit Org timestamps

Contracts: ORG07 ORG17.
Details and exact checks: [Slice 18](../plan/rev-org-mode.md#slice-18-enter-and-edit-org-timestamps).

- [ ] T18.1: Deliver native date/time/calendar prompts for active/inactive timestamps, ranges, schedule and deadline.
- [ ] T18.2: Use familiar context actions and cancellation, retaining weekday/source formatting where untouched.
- [ ] T18.3: Verify invalid dates, ambiguous local times, Unicode prompts and one-command undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 19: reschedule repeated Org tasks correctly

Contracts: ORG07 ORG20.
Details and exact checks: [Slice 19](../plan/rev-org-mode.md#slice-19-reschedule-repeated-org-tasks-correctly).

- [ ] T19.1: Implement distinct +, ++ and .+ recurrence, warning delays and habit interval ranges.
- [ ] T19.2: Apply repeat completion, state history, LAST_REPEAT and planning changes coherently.
- [ ] T19.3: Verify overdue/backlog, calendar boundaries, duplicate completion dates and full undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 20: record native Org task clock intervals

Contracts: ORG19 ORG21.
Details and exact checks: [Slice 20](../plan/rev-org-mode.md#slice-20-record-native-org-task-clock-intervals).

- [ ] T20.1: Deliver clock-in/out/cancel/goto with compatible CLOCK source and task-owned interval durations.
- [ ] T20.2: Preserve document transactions and remove or retain zero intervals according to the frozen defaults.
- [ ] T20.3: Verify normal save/read-only/history boundaries and no per-second idle source updates.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 21: switch and recover Org task clocks

Contracts: ORG19 ORG18.
Details and exact checks: [Slice 21](../plan/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks).

- [ ] T21.1: Enforce one active session clock, checked task switching and completion-triggered clock-out.
- [ ] T21.2: Handle quit/close, stale edits, undo and explicit incomplete-clock resolution without losing source.
- [ ] T21.3: Introduce org_io and verify optional state persistence, failed recovery and no unsolicited startup scans.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 22: compare Org clock totals and effort

Contracts: ORG19 ORG21 ORG23.
Details and exact checks: [Slice 22](../plan/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort).

- [ ] T22.1: Deliver bounded task/subtree/file/date-range clock totals and effort comparisons.
- [ ] T22.2: Generate compatible clock-table reports only on explicit refresh and preserve unrelated dynamic-block source.
- [ ] T22.3: Introduce org_agenda contracts and verify overlaps, open intervals and transactional report replacement.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 23: deliver native Org relative and countdown timers

Contracts: ORG22 ORG18.
Details and exact checks: [Slice 23](../plan/rev-org-mode.md#slice-23-deliver-native-org-relative-and-countdown-timers).

- [ ] T23.1: Implement relative time insertion/list items, pause/resume/stop and countdown entry with familiar bindings.
- [ ] T23.2: Arm only selected deadlines in the existing native event owner; cancel before due work and retain terminal cleanup.
- [ ] T23.3: Verify one-shot expiry, restart, exit and no inactive wakeups in the owned Org time journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 24: maintain Org habit schedules and history

Contracts: ORG20 ORG07.
Details and exact checks: [Slice 24](../plan/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history).

- [ ] T24.1: Recognize STYLE habit and integrate logged completion, interval ranges and Org day semantics.
- [ ] T24.2: Preserve compatible histories and distinguish missing observations from actual failed/completed days.
- [ ] T24.3: Verify recurrence types, source-local declarations and coherent completion undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 25: render native Org habit consistency graphs

Contracts: ORG20 ORG16.
Details and exact checks: [Slice 25](../plan/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs).

- [ ] T25.1: Render bounded habit graphs with verified defaults, completion/today markers and semantic palette states.
- [ ] T25.2: Deliver agenda visibility, all-today/today-only and graph controls through native commands.
- [ ] T25.3: Verify narrow panes, late completions, dirty source observations and idle immutability.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 26: discover bounded canonical Org agenda sources

Contracts: ORG08 ORG18.
Details and exact checks: [Slice 26](../plan/rev-org-mode.md#slice-26-discover-bounded-canonical-org-agenda-sources).

- [ ] T26.1: Resolve explicitly selected files/directories without scanning the home or project implicitly.
- [ ] T26.2: Prefer current buffers, deduplicate real file identities and observe disk sources through cancellable owned work.
- [ ] T26.3: Verify unreadable sources, cycles, limits and truthful partial results while keeping editing responsive.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 27: display native day and week Org agendas

Contracts: ORG08 ORG09.
Details and exact checks: [Slice 27](../plan/rev-org-mode.md#slice-27-display-native-day-and-week-org-agendas).

- [ ] T27.1: Deliver read-only agenda rows, time grid, overdue/scheduled/deadline sections and day/week navigation.
- [ ] T27.2: Use revision-qualified source locators and the captured effective date/zone for every row.
- [ ] T27.3: Verify real refresh, source return, shared buffers and no agenda-as-document mutation.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 28: query Org tasks tags and properties

Contracts: ORG08 ORG23.
Details and exact checks: [Slice 28](../plan/rev-org-mode.md#slice-28-query-org-tasks-tags-and-properties).

- [ ] T28.1: Deliver TODO/search/tag/property views with declared Boolean/comparison grammar and source scopes.
- [ ] T28.2: Evaluate bounded parsed queries against canonical metadata, including inheritance and effort.
- [ ] T28.3: Verify escaping, invalid operators, completeness and stable result identity.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 29: configure native Org agenda views

Contracts: ORG23 ORG17.
Details and exact checks: [Slice 29](../plan/rev-org-mode.md#slice-29-configure-native-org-agenda-views).

- [ ] T29.1: Deliver typed custom/block views, columns, filtering, sorting and declared date spans.
- [ ] T29.2: Expose compiled view/refresh controls and native agenda context bindings without a scripting registry.
- [ ] T29.3: Verify combination limits, visibility/reset behavior and consistent exhaustive sample defaults.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 30: edit canonical Org source from the agenda

Contracts: ORG09 ORG23.
Details and exact checks: [Slice 30](../plan/rev-org-mode.md#slice-30-edit-canonical-org-source-from-the-agenda).

- [ ] T30.1: Route agenda task/date/tag/priority/effort/clock actions through checked source handlers.
- [ ] T30.2: Deliver explicit selection, bulk operations and per-target outcomes with revision re-resolution.
- [ ] T30.3: Exercise dirty buffers, cancellation, failure and undo in an owned multi-source agenda journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 31: define native Org capture targets and templates

Contracts: ORG10 ORG17.
Details and exact checks: [Slice 31](../plan/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates).

- [ ] T31.1: Admit typed native capture templates/targets and the source-compatible factory fallback.
- [ ] T31.2: Resolve file/headline/outline/date-tree/table/list targets without writing them during discovery.
- [ ] T31.3: Verify explicit destination entry, missing targets and template/default/sample coherence.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 32: expand and edit native Org capture drafts

Contracts: ORG10 ORG02.
Details and exact checks: [Slice 32](../plan/rev-org-mode.md#slice-32-expand-and-edit-native-org-capture-drafts).

- [ ] T32.1: Expand the declared bounded text substitutions, prompts and checked origin links without evaluating Lisp.
- [ ] T32.2: Retain the editable capture draft in its own transient owner with familiar context bindings.
- [ ] T32.3: Verify malformed expansion, literal preservation, cancellation and return to the originating view.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 33: finalize or abort Org capture safely

Contracts: ORG10 ORG18.
Details and exact checks: [Slice 33](../plan/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely).

- [ ] T33.1: Revalidate destinations and publish capture once through ordinary source/file safeguards.
- [ ] T33.2: Preserve a recoverable draft after stale, read-only, save or filesystem failure; abort leaves targets unchanged.
- [ ] T33.3: Verify file/heading/list/table/date-tree capture and failed publication with actual owned files.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 34: refile Org subtrees across canonical documents

Contracts: ORG11 ORG02.
Details and exact checks: [Slice 34](../plan/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents).

- [ ] T34.1: Prepare cross-buffer subtree moves and destination insertion as one checked semantic operation.
- [ ] T34.2: Keep source/destination revisions, history groups and durable-save outcomes explicit and recoverable.
- [ ] T34.3: Verify duplicate/stale targets, read-only buffers, external file replacement and partial publication truth.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 35: archive Org subtrees without silent loss

Contracts: ORG11 ORG09.
Details and exact checks: [Slice 35](../plan/rev-org-mode.md#slice-35-archive-org-subtrees-without-silent-loss).

- [ ] T35.1: Deliver archive tag, archive sibling and %s_archive destination behavior with Org metadata.
- [ ] T35.2: Reuse checked multi-document moves and retain archived source discoverability under selected agenda policy.
- [ ] T35.3: Exercise capture/refile/archive, undo and failed destinations in one owned terminal journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 36: store and follow checked Org links

Contracts: ORG15 ORG02.
Details and exact checks: [Slice 36](../plan/rev-org-mode.md#slice-36-store-and-follow-checked-org-links).

- [ ] T36.1: Deliver internal targets, file/ID/attachment links, insertion, storage and navigation.
- [ ] T36.2: Re-resolve source and dispatch external resources through the bounded owned job path only after explicit follow; introduce the shared child-lifetime primitive here.
- [ ] T36.3: Verify ambiguity, encoded paths, read-only documents and source-preserving failure.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 37: manage native Org heading attachments

Contracts: ORG24 ORG15.
Details and exact checks: [Slice 37](../plan/rev-org-mode.md#slice-37-manage-native-org-heading-attachments).

- [ ] T37.1: Deliver ID/DIR attachment locations, linking, copy/move/list/open and reviewed dispatcher controls.
- [ ] T37.2: Recheck real paths and collisions and retain operation evidence across partial file/source failure.
- [ ] T37.3: Verify directory traversal, symlinks, unreadable files and that visiting a heading performs no attachment IO.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 38: edit native Org footnotes and inline content

Contracts: ORG27 ORG30 ORG02.
Details and exact checks: [Slice 38](../plan/rev-org-mode.md#slice-38-edit-native-org-footnotes-and-inline-content).

- [ ] T38.1: Deliver footnote create/follow/renumber, entities, text markup and source-preserving affiliated forms.
- [ ] T38.2: Keep literal blocks and unknown forms distinct during structured editing and context actions.
- [ ] T38.3: Verify Unicode, malformed references, radio targets and coherent edit/history behavior.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 39: navigate and align native Org tables

Contracts: ORG12 ORG17.
Details and exact checks: [Slice 39](../plan/rev-org-mode.md#slice-39-navigate-and-align-native-org-tables).

- [ ] T39.1: Introduce org_tables and native cell/row navigation, insertion and alignment over original cell source.
- [ ] T39.2: Retain separators, names, captions, formulas and non-table text while preserving displayed columns.
- [ ] T39.3: Verify tabs, wide graphemes, CRLF, empty cells and one-operation undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 40: edit Org table rows columns and references

Contracts: ORG12 ORG14.
Details and exact checks: [Slice 40](../plan/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references).

- [ ] T40.1: Deliver row/column insert/delete/move, sorting and list/table conversion with checked ownership.
- [ ] T40.2: Rewrite only supported formula references and reject ambiguous transformations before source mutation.
- [ ] T40.3: Verify cell value fidelity, separator boundaries and full structural undo/redo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 41: parse the bounded Org formula subset

Contracts: ORG13 ORG14.
Details and exact checks: [Slice 41](../plan/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset).

- [ ] T41.1: Parse declared arithmetic, field/column/range references, functions and compatible TBLFM assignments.
- [ ] T41.2: Enforce precedence, right-associative exponentiation and bounded syntax/depth before evaluation.
- [ ] T41.3: Verify unsupported Calc/Lisp preservation, malformed references and precise diagnostics.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 42: evaluate basic Org table formulas safely

Contracts: ORG13 ORG14.
Details and exact checks: [Slice 42](../plan/rev-org-mode.md#slice-42-evaluate-basic-org-table-formulas-safely).

- [ ] T42.1: Evaluate dependency graphs, aggregates, rounding and formatting under the frozen numeric policy.
- [ ] T42.2: Detect cycles, empty/text misuse, division by zero, non-finite values and work-budget exhaustion.
- [ ] T42.3: Verify expected arithmetic independently, including negative half rounding and reference direction.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 43: publish Org recalculation as one transaction

Contracts: ORG13 ORG14 ORG17.
Details and exact checks: [Slice 43](../plan/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction).

- [ ] T43.1: Prepare all admitted results before applying the selected recalculation as one checked history unit.
- [ ] T43.2: Expose table/cell/column recalculation through contextual commands and M-x without implicit evaluation.
- [ ] T43.3: Exercise success, unsupported formulas, failure preservation and undo in the owned table journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 44: import and export Org table data

Contracts: ORG12 ORG27.
Details and exact checks: [Slice 44](../plan/rev-org-mode.md#slice-44-import-and-export-org-table-data).

- [ ] T44.1: Deliver bounded CSV/TSV import/export and list/table conversion through native commands.
- [ ] T44.2: Preserve quoting and representable cell values; reject embedded line breaks or unsupported transformations before mutation rather than inventing lossy escapes.
- [ ] T44.3: Verify actual file round trips and rejected malformed input without lossy recovery.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 45: display Org property columns and native reports

Contracts: ORG21 ORG23 ORG27.
Details and exact checks: [Slice 45](../plan/rev-org-mode.md#slice-45-display-org-property-columns-and-native-reports).

- [ ] T45.1: Deliver property column views and declared native dynamic report blocks, including effort and clock totals.
- [ ] T45.2: Keep projections separate from source; replace only recognized explicit report bodies on update.
- [ ] T45.3: Verify aggregates, source provenance, unknown dynamic blocks and atomic update undo.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 46: prepare checked native Org export snapshots

Contracts: ORG25 ORG02.
Details and exact checks: [Slice 46](../plan/rev-org-mode.md#slice-46-prepare-checked-native-org-export-snapshots).

- [ ] T46.1: Freeze an export snapshot and backend/options/include/macro contracts over native source forms.
- [ ] T46.2: Resolve bounded includes and text-only macros without executing blocks or file-local Lisp.
- [ ] T46.3: Verify cycles, absent includes, unsaved source, export scope and destination preservation.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 47: export Org to text Markdown and Org

Contracts: ORG25 ORG27.
Details and exact checks: [Slice 47](../plan/rev-org-mode.md#slice-47-export-org-to-text-markdown-and-org).

- [ ] T47.1: Deliver ASCII/UTF-8, Markdown and Org outputs retaining structure, tables, links and footnotes.
- [ ] T47.2: Apply selected subtree/region/options and explicit destination writes from checked snapshots.
- [ ] T47.3: Verify independent semantic fixtures, Unicode conversion diagnostics and no source mutations.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 48: export native Org documents to HTML

Contracts: ORG25 ORG30.
Details and exact checks: [Slice 48](../plan/rev-org-mode.md#slice-48-export-native-org-documents-to-html).

- [ ] T48.1: Deliver native HTML structure, escaping, metadata, links, images, tables, notes and source blocks.
- [ ] T48.2: Implement declared attributes/options and safe relative asset references without executing contents.
- [ ] T48.3: Verify generated markup using independent HTML parsing and representative full-document fixtures.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 49: export Org to LaTeX and Beamer

Contracts: ORG25 ORG30.
Details and exact checks: [Slice 49](../plan/rev-org-mode.md#slice-49-export-org-to-latex-and-beamer).

- [ ] T49.1: Deliver native LaTeX/Beamer rendering with declared headings, lists, tables, notes and math/source forms.
- [ ] T49.2: Support explicit optional compiler jobs for PDF through the bounded external owner, never on export discovery.
- [ ] T49.3: Verify escaped output and truthful absent-tool behavior without requiring a TeX installation.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 50: export native Org to OpenDocument text

Contracts: ORG25 ORG18.
Details and exact checks: [Slice 50](../plan/rev-org-mode.md#slice-50-export-native-org-to-opendocument-text).

- [ ] T50.1: Deliver bounded native ODT XML and ZIP packaging without adding a runtime service or crate.
- [ ] T50.2: Preserve document structure, Unicode, links, lists and tables with validated package relationships.
- [ ] T50.3: Verify generated archives with Python zipfile/XML tooling and truthful unsupported-rich-content handling.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 51: export Org to Texinfo and man formats

Contracts: ORG25 ORG27.
Details and exact checks: [Slice 51](../plan/rev-org-mode.md#slice-51-export-org-to-texinfo-and-man-formats).

- [ ] T51.1: Deliver native Texinfo and man renderers over the shared export snapshot.
- [ ] T51.2: Retain declared hierarchy, references, preformatted source and escaping without invoking tools implicitly.
- [ ] T51.3: Verify structural golden fixtures and ordinary save/cancellation safeguards.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 52: export Org planning data as iCalendar

Contracts: ORG25 ORG07.
Details and exact checks: [Slice 52](../plan/rev-org-mode.md#slice-52-export-org-planning-data-as-icalendar).

- [ ] T52.1: Deliver native iCalendar events/tasks with checked IDs, timestamps, time zones and recurrence conversion.
- [ ] T52.2: Preserve unsupported recurrence source and report conversion limits rather than inventing dates.
- [ ] T52.3: Verify independent calendar fixtures, line folding, escaping and stable refresh output.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 53: publish native Org document projects

Contracts: ORG26 ORG25.
Details and exact checks: [Slice 53](../plan/rev-org-mode.md#slice-53-publish-native-org-document-projects).

- [ ] T53.1: Deliver typed project selection, bounded source discovery, export and attachment-copy publication.
- [ ] T53.2: Honor explicit output ownership, collision consent and per-file outcomes; optional upload uses declared external tools.
- [ ] T53.3: Exercise actual exports/publishing, failure and source immutability through owned terminal/filesystem fixtures.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 54: edit native Org source blocks and headers

Contracts: ORG28 ORG02.
Details and exact checks: [Slice 54](../plan/rev-org-mode.md#slice-54-edit-native-org-source-blocks-and-headers).

- [ ] T54.1: Admit declared source block/name/header structure and native special-edit entry/return.
- [ ] T54.2: Reuse available language presentation while keeping embedded Org-looking text literal and source authoritative.
- [ ] T54.3: Verify header precedence, malformed blocks, edit cancellation and checked reinsertion.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 55: tangle Org source with bounded noweb references

Contracts: ORG28 ORG18.
Details and exact checks: [Slice 55](../plan/rev-org-mode.md#slice-55-tangle-org-source-with-bounded-noweb-references).

- [ ] T55.1: Expand declared named references and tangle selected blocks into explicit checked destinations.
- [ ] T55.2: Bound expansion/cycles and reject unsafe or unresolved targets before overwriting any file.
- [ ] T55.3: Verify separate-file outputs, indentation/source fidelity and truthful multi-file failures.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 56: execute selected Org blocks through bounded tools

Contracts: ORG28 ORG18.
Details and exact checks: [Slice 56](../plan/rev-org-mode.md#slice-56-execute-selected-org-blocks-through-bounded-tools).

- [ ] T56.1: Deliver explicit execution confirmation and typed external program adapters without an embedded interpreter.
- [ ] T56.2: Own child groups, stdin/out/err, temporary files, result ceilings, cancellation and finite deadlines.
- [ ] T56.3: Verify real owned shell jobs, missing tools, timeout and reaping; visiting/exporting source never executes it.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 57: insert checked Org block and call results

Contracts: ORG28 ORG14.
Details and exact checks: [Slice 57](../plan/rev-org-mode.md#slice-57-insert-checked-org-block-and-call-results).

- [ ] T57.1: Deliver named calls, declared Library of Babel observations and results insertion/replacement as checked edits.
- [ ] T57.2: Retain header/argument semantics, source stamps and confirmation across related block resolution.
- [ ] T57.3: Verify output/value/table results and reject stale/failed publication without disturbing prior results.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 58: import native Org feed entries on demand

Contracts: ORG29 ORG18.
Details and exact checks: [Slice 58](../plan/rev-org-mode.md#slice-58-import-native-org-feed-entries-on-demand).

- [ ] T58.1: Deliver bounded native RSS/Atom parsing, declared feed targets and compatible FEEDSTATUS deduplication.
- [ ] T58.2: Fetch only explicit updates using configured optional transport, with no background poller or new service.
- [ ] T58.3: Verify local XML fixtures, repeat imports, hostile entity rejection and failed fetch/destination preservation.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 59: preview native Org links and images

Contracts: ORG30 ORG15.
Details and exact checks: [Slice 59](../plan/rev-org-mode.md#slice-59-preview-native-org-links-and-images).

- [ ] T59.1: Deliver bounded link/image preview controls over explicit source/resource observations.
- [ ] T59.2: Use admitted terminal PNG presentation when supported and explicit external viewers otherwise; retain readable fallback.
- [ ] T59.3: Verify clipping, terminal restoration, source immutability and child cleanup in the owned external-feature journey.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 60: complete the native Org control catalog

Contracts: ORG17 ORG31.
Details and exact checks: [Slice 60](../plan/rev-org-mode.md#slice-60-complete-the-native-org-control-catalog).

- [ ] T60.1: Reconcile every shipped command, default context binding, typed preference and palette role with the catalog.
- [ ] T60.2: Extend the complete sample and embedded template with actual supported Org controls and optional tool examples.
- [ ] T60.3: Verify no inert controls, no unimplemented M-x entries and exact parser/default/template/reference agreement.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 61: verify complete native Org terminal workflows

Contracts: ORG01 ORG02 ORG17 ORG18.
Details and exact checks: [Slice 61](../plan/rev-org-mode.md#slice-61-verify-complete-native-org-terminal-workflows).

- [ ] T61.1: Extend existing terminal admission with the declared representative Org journeys and same-head receipts.
- [ ] T61.2: Exercise canonical editing, capture/refile, clocks/habits, agenda, tables and optional-tool failures together.
- [ ] T61.3: Retain every earlier journey and verify zero skips, isolated HOME/XDG roots and owned process cleanup.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 62: verify bounded Org work and idle resources

Contracts: ORG18 ORG31.
Details and exact checks: [Slice 62](../plan/rev-org-mode.md#slice-62-verify-bounded-org-work-and-idle-resources).

- [ ] T62.1: Extend existing performance collection for large Org source, many files, habit graphs, formulas and exports.
- [ ] T62.2: Measure responsive cancellation and inactive/armed timer behavior under the frozen resource ceilings.
- [ ] T62.3: Retain baseline ceilings, same-candidate provenance and explicit warm gate cost; investigate regressions.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 63: reconcile native Org contracts and references

Contracts: ORG01 ORG17 ORG31.
Details and exact checks: [Slice 63](../plan/rev-org-mode.md#slice-63-reconcile-native-org-contracts-and-references).

- [ ] T63.1: Reconcile delivered Org mode with editor/configuration/view baselines and complete user references.
- [ ] T63.2: Preserve all previous tested candidates and make the new capability matrix and optional-tool boundaries explicit.
- [ ] T63.3: Verify every requirement/scenario links to native, real-terminal and resource evidence; keep historical closures intact.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

### Slice 64: prepare complete native Org delivery acceptance

Contracts: ORG01 ORG18 ORG31.
Details and exact checks: [Slice 64](../plan/rev-org-mode.md#slice-64-prepare-complete-native-org-delivery-acceptance).

- [ ] T64.1: Admit all sixty-four canonical integrations, reports and agreed requirement mappings without declaring prerequisites runtime proof.
- [ ] T64.2: Prepare clean integrated dev for the unchanged controller full gate and exact-path documentation closure.
- [ ] T64.3: Verify coherent plan/model/tracker state, zero skips and the distinction between tested head and later closure head.
- [ ] Record exact-head validation, canonical PR/report and verified dev integration.

## Delivery closure

- [ ] All 64 canonical slice integrations and 192 task outcomes are verified.
- [ ] All ORG01-ORG31 requirements have actual native, terminal and resource evidence.
- [ ] Complete operational command/binding/configuration/template/reference coverage agrees.
- [ ] Previous editor/configuration/input acceptance and historical provenance are preserved.
- [ ] The single controller full gate passes the exact clean integrated dev candidate with zero skips.
- [ ] Later documentation closure stays within exact granted paths and retains tested-head provenance.
- [ ] Final plan/model/tracker/report state, canonical push and Acta outcome match actual delivery.

No controller has been started by preparing this tracker. The product owner approved this exact single-cycle startup on 2026-10-28.
The planning set must be committed before native execution begins.
