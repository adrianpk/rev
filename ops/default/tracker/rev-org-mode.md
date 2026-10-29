# Native Org Mode Tracker

Status: Approved
Date: 2026-10-28
Delivery set: rev-org-mode
Scope authority: product owner; Org-in-Emacs behavior with the advanced-spreadsheet exception.
Approved: product owner, 2026-10-28; explicitly requested starting this single 64-slice Looper cycle after delegating remaining behavior choices to Org in Emacs.
Slice strategy: layered
Active slice: Slice 45 columns and dynamic blocks
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

- [x] T7.1: Add view-local visibility projection over source ranges, retaining independent shared-buffer views.
- [x] T7.2: Map cursor, gutter, search, mouse and selections through hidden ranges without aliasing source positions.
- [x] T7.3: Verify tiny panes, Unicode, hit testing, stale folds and edit-driven reveal.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#108](https://forge.adrianpk.com/adrianpk/rev/pulls/108).
Integrated into `dev` at `40b0399c60144a1b5b95f0709ed63c10f3124e0c`.
Controller receipt `ZLMY2P2DZQMCCWWZZ6MDWQNZQ3` passed the required five-command
batch with 147 tests and zero skips. Canonical title/body/report/head verified.

### Slice 8: deliver native Org visibility controls

Contracts: ORG03 ORG17.
Details and exact checks: [Slice 8](../plan/rev-org-mode.md#slice-8-deliver-native-org-visibility-controls).

- [x] T8.1: Implement heading TAB cycle, whole-view S-TAB cycle and direct visibility/drawer commands.
- [x] T8.2: Integrate hierarchy-aware reveal with ordinary motion, searches and saved independent view state.
- [x] T8.3: Exercise native keys, M-x, cancellation and source immutability in an owned outline PTY journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#109](https://forge.adrianpk.com/adrianpk/rev/pulls/109).
Report: [Slice 8](../report/slices/rev-org-mode/slice-8-fold-controls.md).
Integrated into `dev` at `72c490f46bd417923ad9cde762ccae7ae2f9d729`.
Controller receipt `I27ZJKPOPSFRW75KQXZ5VOSJXJ` passed the required six checks,
218 Rust tests and two owned PTY sessions with zero skips. Focused receipt
`YH2OYN7TGRX2CR6THR2ISRXYF3` passed 96 configuration tests and reference/template
checks at the same head. Canonical title/body/report/head verified.

### Slice 9: insert and navigate Org headings

Contracts: ORG04 ORG17.
Details and exact checks: [Slice 9](../plan/rev-org-mode.md#slice-9-insert-and-navigate-org-headings).

- [x] T9.1: Deliver sibling insertion and next/previous/same-level/parent heading movement.
- [x] T9.2: Handle contextual M-RET while preserving attached sections, blank lines and final newline state.
- [x] T9.3: Verify read-only navigation, inactive context behavior, undo and logical source bounds.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#110](https://forge.adrianpk.com/adrianpk/rev/pulls/110).
Report: [Slice 9](../report/slices/rev-org-mode/slice-9-outline-navigation.md).
Integrated into `dev` at `32ba8f34d6310409baa2d89a932e1898dcd87ef4`.
Controller receipt `OZCI4OGWRNKNBDR25TU2HLZDMI` passed the required five checks
and 293 Rust tests with zero skips. Focused receipt `SDBZKA3ESTQHII7XEV7YX2ROR5`
passed 96 configuration tests, reference/template checks and two owned PTY
sessions at the same head. Canonical title/body/report/head verified.

### Slice 10: move and promote coherent Org subtrees

Contracts: ORG04 ORG02.
Details and exact checks: [Slice 10](../plan/rev-org-mode.md#slice-10-move-and-promote-coherent-org-subtrees).

- [x] T10.1: Implement checked promote/demote and sibling subtree moves over complete owned source.
- [x] T10.2: Retain descendants, affiliated metadata, marks and independent views through one atomic history change.
- [x] T10.3: Verify boundary rejections, malformed parents, regional edits and full undo/redo fidelity.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

PR: [#111](https://forge.adrianpk.com/adrianpk/rev/pulls/111).
Report: [Slice 10](../report/slices/rev-org-mode/slice-10-subtree-edits.md).
Task commits: T10.1 `108eb180cc51e646143b0b07336449375b3eba93`,
T10.2 `442f5de871c7faf32f091a8764ae2fa3fae03530`,
T10.3 `ad4c1a46af165a10f9f20e01e5e34a491f17a6ce`.
Integrated at `243764fbeaa687596c1be9869df6e974d626c709` through PR #111.
Required receipt `NMHRAJ25SXERYSWRLIORRQHZDU`: five checks, 192 Rust tests,
zero skipped. Focused receipt `A3YXJEV7CJZC2H3Q7IX2PWGEQN`: 249 tests,
reference/template checks and two owned PTY sessions at the same HEAD.
Canonical title/body/report/head and exact dev integration verified.

### Slice 11: edit Org lists and progress cookies

Contracts: ORG04 ORG06 ORG30.
Details and exact checks: [Slice 11](../plan/rev-org-mode.md#slice-11-edit-org-lists-and-progress-cookies).

- [x] T11.1: Continue, indent, outdent, reorder and renumber ordered/unordered/description lists contextually.
- [x] T11.2: Implement ordinary/radio checkboxes and coherent list/task progress cookies through checked edits.
- [x] T11.3: Verify nested lists, timer-item contexts, literal blocks and one-operation undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

PR: [#112](https://forge.adrianpk.com/adrianpk/rev/pulls/112).
Report: [Slice 11](../report/slices/rev-org-mode/slice-11-lists-checkboxes.md).
Task commits: T11.1 `548ff5a88f7b6e74c56cf0e8390291c3df30bee6`,
T11.2 `5a9ad119c441d3e473b56536a009f5a014300461`,
T11.3 `97150fd852bdc708e4c7adc137824f7a037dcbee`.
Required receipt `MCMTZD7MYMKYYP6FW5HPHHENXA` passed all five commands and
324 Rust tests. Focused receipt `5N6KVB7GJ5RCPK7UWY4YRTPFNM` passed 137 tests,
configuration references and two owned terminal sessions with zero skipped.
Canonical PR #112 merged at `88b53cf3c8a8d78987f8b51534ca476ac830f5fe`;
local and canonical `dev` match that validated candidate.

### Slice 12: edit Org drawers and inherited properties

Contracts: ORG05 ORG17.
Details and exact checks: [Slice 12](../plan/rev-org-mode.md#slice-12-edit-org-drawers-and-inherited-properties).

- [x] T12.1: Implement property set/remove/inspect and drawer folding without confusing ordinary and property drawers.
- [x] T12.2: Resolve local/additive/inherited values with source-owner provenance and safe directive precedence.
- [x] T12.3: Verify selective inheritance, unknown properties and edits to the actual owner.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

PR: [#113](https://forge.adrianpk.com/adrianpk/rev/pulls/113).
Report: [Slice 12](../report/slices/rev-org-mode/slice-12-drawers-properties.md).
Task commits: T12.1 `725c0fd9c5a633d878e8793d1bd745aae4caad6b`,
T12.2 `fa51a5569d2cc301c23b369a5a137562b2f20c53`,
T12.3 `5de7b9d26d8d64e9b6de8f76b3bde6d90e4edad2`.
Required receipt `BAOIBKP7ZP3THSKGOQAS7572LO` passed all five commands and
239 Rust tests. Focused receipt `NM33OJCTV54QWA7NWVJEANDEHM` passed 154 command
tests, configuration references and two owned terminal sessions with zero skipped.
Canonical PR #113 merged at `6c7052a31fbbe76b90f892a34c37223fe767c9d1`;
local and canonical `dev` match the validated candidate.

### Slice 13: edit and match Org task tags

Contracts: ORG05 ORG23.
Details and exact checks: [Slice 13](../plan/rev-org-mode.md#slice-13-edit-and-match-org-task-tags).

- [x] T13.1: Deliver tag entry, removal, completion and local/inherited membership inspection.
- [x] T13.2: Implement the declared Org tag/property match grammar with typed comparisons and bounded parsing.
- [x] T13.3: Verify case sensitivity, escaping, malformed queries and unchanged unrelated source.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

PR: [#114](https://forge.adrianpk.com/adrianpk/rev/pulls/114).
Report: [Slice 13](../report/slices/rev-org-mode/slice-13-tags-matching.md).
Task commits: T13.1 `22fa841590398823ddbba052379bc5d436ccd365`,
T13.2 `cc6bc81ce9d5f8374f004aec52bd29aa8be42c20`,
T13.3 `b264270aad53655e0b22772380e4cc03bc9bc118`.
Required receipt `Z3FB7EH4EYOYLNJTXEGM5EBJKV` passed all five commands and
312 Rust tests. Focused receipt `6EIW7RMFBU4IUTWXW26OJUDRZY` passed 99 config
tests, references and two owned terminal sessions with zero skipped.
Canonical PR #114 merged at `2f81bd7547f3a0ae3eb4d4cb9f9c34476a66d7c3`;
local and canonical `dev` match the validated candidate.

### Slice 14: deliver Org task workflows and logging

Contracts: ORG06 ORG17.
Details and exact checks: [Slice 14](../plan/rev-org-mode.md#slice-14-deliver-org-task-workflows-and-logging).

- [x] T14.1: Admit TODO/SEQ_TODO/TYP_TODO declarations and complete native state transitions with selected logging.
- [x] T14.2: Integrate priorities, blocking rules, CLOSED/state notes and cookies into coherent transactions.
- [x] T14.3: Verify key/M-x equivalence, read-only behavior and task/metadata history in an owned PTY journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR [#115](https://forge.adrianpk.com/adrianpk/rev/pulls/115) merged
into dev at `56d11d40f15dbbe7ec148b23fb8410d0fc060841`;
[Slice 14 report](../report/slices/rev-org-mode/slice-14-todo-workflows.md) is delivered.
Required receipt `PUQ3NTFOSJSZSRFRNOEFDZRSR3` passed 6/6 (365 integration tests,
four task PTYs). Focused `Y4GSG26OGTESIZSZ7G77A7PZMA` passed 5/5 (two clock
and 59 source tests, reference and two outline PTYs), all on that exact HEAD.

### Slice 15: record Org effort estimates and priorities

Contracts: ORG06 ORG21.
Details and exact checks: [Slice 15](../plan/rev-org-mode.md#slice-15-record-org-effort-estimates-and-priorities).

- [x] T15.1: Deliver native priority and EFFORT editing with Org duration forms and factory choices.
- [x] T15.2: Keep workflow metadata compatible and expose operational commands in Org and agenda contexts.
- [x] T15.3: Verify numeric/duration rejection, source spelling preservation and atomic undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR [#116](https://forge.adrianpk.com/adrianpk/rev/pulls/116) targets dev;
[Slice 15 report](../report/slices/rev-org-mode/slice-15-effort-priorities.md) is delivered.
Task commits:
- T15.1: `06ba3495029d20fdab3fedf63469b005d862297d`.
- T15.2: `d6f6d45f850a79abcbe8aa91cb709205da75a4fb`.
- T15.3: `53813b06e6f1ddefcb5ca064948f2fcee5e8b047`.

Integration: PR #116 merged into dev at `9fc8e30500b6916f3dc2119dd8da69962fa54d24`.
Required receipt `YEVUSWWZWDPVUCPGHHFWNR7ES7` passed all five ordered checks,
including 383 integration tests. Focused receipt `CR64ENK5CQM66T6BCG4E2XSW37`
passed 59 source tests, configuration reference checks and four owned task PTYs.
Canonical Forgejo, controller merge evidence and Acta were verified before closure.

### Slice 16: model Org calendar dates and durations

Contracts: ORG07 ORG18 ORG21.
Details and exact checks: [Slice 16](../plan/rev-org-mode.md#slice-16-model-org-calendar-dates-and-durations).

- [x] T16.1: Introduce org_time with checked Gregorian civil dates, intervals and Org duration units.
- [x] T16.2: Separate scheduling calendar arithmetic from elapsed work durations and formatting.
- [x] T16.3: Verify leap years, month ends, overflow, duration forms and an injected operation clock.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR [#117](https://forge.adrianpk.com/adrianpk/rev/pulls/117) targets dev;
[Slice 16 report](../report/slices/rev-org-mode/slice-16-civil-time.md) is delivered.
Task commits:
- T16.1: `3673caad6299791f0dc6031e0e2ae373d033e184`.
- T16.2: `b28f51447f890482d4e4dc3cb6df4483b950a8a6`.
- T16.3: `dd34455e73e76770e9e3139e92811c282f864f53`.

Integration: PR #117 merged into dev at `b056990f1c1e2231ba02a0c0117be574b3ac1bb7`.
Required receipt `I5IHIQRKVH4XSTVI4DKS2WSUY7` passed all five ordered checks,
including 78 source/time tests. Focused receipt `RBECIIFXDEG6Q5M2GG6LI56SDE`
passed 126 editing tests and four owned task PTYs. Canonical Forgejo, controller
merge evidence and Acta were verified before closure.

### Slice 17: resolve Org local times and zone changes

Contracts: ORG07 ORG18.
Details and exact checks: [Slice 17](../plan/rev-org-mode.md#slice-17-resolve-org-local-times-and-zone-changes).

- [x] T17.1: Implement bounded native TZif/POSIX-zone observations and system/local/UTC/IANA selection.
- [x] T17.2: Capture effective date/zone once per operation; resolve folds/gaps explicitly without guessing.
- [x] T17.3: Verify DST transitions, invalid zones and external zone changes with owned fixtures.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR [#118](https://forge.adrianpk.com/adrianpk/rev/pulls/118) targets dev;
[Slice 17 report](../report/slices/rev-org-mode/slice-17-time-zones.md) is delivered.
Task commits:
- T17.1: `2aa2110ec2f78dbc9136f72becaba82c5ea12431`.
- T17.2: `1384f20c768c19a05a992197162ddee3f8299850`.
- T17.3: `cdef5c9ae706b0e61a03c596ae72300d5b79ca95`.

Integration: PR #118 merged into dev at `5e6a50d5b0e0c68aa2ad3cc36f09c247860edba7`.
Required receipt `KHVWYQBX4J3NFOKDGJFSKFH3UP` passed all five ordered checks,
including 144 configuration/time tests. Focused `6Y3W4L6CVCY6WMAUEYZFCT3OBF`
passed 126 editing tests, configuration reference agreement and four owned
task PTYs. Canonical Forgejo, controller merge evidence and Acta were verified.

### Slice 18: enter and edit Org timestamps

Contracts: ORG07 ORG17.
Details and exact checks: [Slice 18](../plan/rev-org-mode.md#slice-18-enter-and-edit-org-timestamps).

- [x] T18.1: Deliver native date/time/calendar prompts for active/inactive timestamps, ranges, schedule and deadline.
- [x] T18.2: Use familiar context actions and cancellation, retaining weekday/source formatting where untouched.
- [x] T18.3: Verify invalid dates, ambiguous local times, Unicode prompts and one-command undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T18.1: `3da11bf31957afb061b8a9b8c487a74ce4b801a1`; focused clean-head receipt
`Z7ZXSK5BB3YHKS4ZJ5ED5K7JAA` passed all four commands. T18.2 development
checks passed 27 focused timestamp tests, library Clippy, configuration reference
agreement and changed-source quality. T18.3 passed 22 editing, 12 time and two
display timestamp tests plus four owned RGB/NO_COLOR task PTYs.
T18.2: `7b6dff79c380`; T18.3: `538691d2e1ae`.
[Canonical PR #119](https://forge.adrianpk.com/adrianpk/rev/pulls/119) and
[Slice 18 report](../report/slices/rev-org-mode/slice-18-timestamp-entry.md)
were delivered by verified PR #119 integration at
`ff8de433dd5e386b69ca54c9f19e5acc22e80b2a`. Required receipt
`RKDUEWDIX7SBPT536C7LBTRHD6` passed all five commands and 389 tests; focused
`3LIVUVB4UOG7DUKV4DHLQVZIIZ` passed 105 configuration tests, reference agreement
and four owned task PTYs. Canonical Forgejo, controller evidence and Acta agree. Final configuration
regression exposed calendar inherited-action availability; focused default-map,
host-role and native prompt regressions pass after the admission correction.
Required command tests also exposed incomplete binding-reference tables; both
existing complete catalog/context assertions pass after reconciliation.

### Slice 19: reschedule repeated Org tasks correctly

Contracts: ORG07 ORG20.
Details and exact checks: [Slice 19](../plan/rev-org-mode.md#slice-19-reschedule-repeated-org-tasks-correctly).

- [x] T19.1: Implement distinct +, ++ and .+ recurrence, warning delays and habit interval ranges.
- [x] T19.2: Apply repeat completion, state history, LAST_REPEAT and planning changes coherently.
- [x] T19.3: Verify overdue/backlog, calendar boundaries, duplicate completion dates and full undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T19.1: `b0479ab5f842c5117530f18f52cf28c3754df8f3`; clean-head receipt
`BJXI7CB4LF5H5CV54ZPCNAEUGB` passed 59 time tests, 23 timestamp editing tests
and `git diff --check`. T19.2 passed 21 focused repeat/time tests, 15 shared
task/configuration regressions, library Clippy, configuration reference agreement
and changed-source quality. T19.2: `553f20ea028f`. T19.3 passed five additional
native boundary/save checks and four owned RGB/NO_COLOR task PTYs with exact
source/save/undo assertions. T19.3: `73ecd1db9ecd`.
[Canonical PR #120](https://forge.adrianpk.com/adrianpk/rev/pulls/120) and the
[Slice 19 report](../report/slices/rev-org-mode/slice-19-repeaters.md) are
delivered at `aa5fad3ef7870ae1b29ecc31661f09a440f3ba20`. Required receipt
`EYXTNPTDAGE6TVSCEV3CHCCKLT` passed 5/5 checks and 224 native tests. Focused
`WOSCCVGU5ZFLCL3WOYW7EX2I22` passed 295 configuration/command/display tests,
reference agreement and four owned task PTYs. Canonical merge and exact dev
integration were verified. Slice 19 of 64 completed.

### Slice 20: record native Org task clock intervals

Contracts: ORG19 ORG21.
Details and exact checks: [Slice 20](../plan/rev-org-mode.md#slice-20-record-native-org-task-clock-intervals).

- [x] T20.1: Deliver clock-in/out/cancel/goto with compatible CLOCK source and task-owned interval durations.
- [x] T20.2: Preserve document transactions and remove or retain zero intervals according to the frozen defaults.
- [x] T20.3: Verify normal save/read-only/history boundaries and no per-second idle source updates.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T20.2: `8395add8fb79`; T20.3: `4cc89289a740`. T20.3 passed five additional native boundary/save/deadline
cases and four owned task PTYs, with affected-test Clippy and source quality.

T20.1: `8a6a6d74d279`; native interval/command checks, library Clippy,
configuration reference and source quality passed. T20.2 passed eight focused
clock editing/configuration tests, typed policy provenance/bounds, library and
affected-test Clippy, reference agreement and changed-source quality.

[Canonical PR #121](https://forge.adrianpk.com/adrianpk/rev/pulls/121) and the
[Slice 20 report](../report/slices/rev-org-mode/slice-20-clock-records.md) are
delivered at `e9ef28d9500d9cb31bc997afb93c76dbc5bc647e`. Required receipt
`A7FX2E7IS7S3U2MYC7CISUDDQT` passed 5/5 checks and 391 tests. Focused
`PZSSQFVN42UJEFUYPGEXV52QCA` passed 143 configuration/display tests, reference
agreement and four owned task PTYs. Canonical merge and exact dev integration
were verified. Slice 20 of 64 completed.

### Slice 21: switch and recover Org task clocks

Contracts: ORG19 ORG18.
Details and exact checks: [Slice 21](../plan/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks).

- [x] T21.1: Enforce one active session clock, checked task switching and completion-triggered clock-out.
- [x] T21.2: Handle quit/close, stale edits, undo and explicit incomplete-clock resolution without losing source.
- [x] T21.3: Introduce org_io and verify optional state persistence, failed recovery and no unsolicited startup scans.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T21.1: `b22d7b7236875c57860d43ed2ce3368ff7000c88`; 16 clock tests,
two configuration cases and 137 editing/window regressions passed. T21.2 passed
native source/time/recovery boundaries, exact 64/65 record admission, complete
catalog/key parity, affected Clippy/source quality and four owned task PTYs.

T21.2: `a1df24f58a3c`; T21.3: `998e7d26d93b`. T21.3 passed eleven org_io cases, bounded recent-task
and policy cases, affected all-target Clippy/source quality, configuration
reference agreement and eight owned task/restart PTYs. Final exact-head required
and focused receipts remain tied to the PR-reference head.

[Canonical PR #122](https://forge.adrianpk.com/adrianpk/rev/pulls/122) merged
into dev at `55f5a763eac4817f1eb8560211beed27ece30455`.
Required receipt `74YUR2Z7JMGBCWZ6BDBWNKUXF4` passed five checks and 95 tests.
Focused receipt `KDLOGSJDKZERJ3EPAAZ32HPBLV` passed 623 tests, configuration
reference agreement and eight owned PTYs. The Slice 21 report is delivered.

### Slice 22: compare Org clock totals and effort

Contracts: ORG19 ORG21 ORG23.
Details and exact checks: [Slice 22](../plan/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort).

- [x] T22.1: Deliver bounded task/subtree/file/date-range clock totals and effort comparisons.
- [x] T22.2: Generate compatible clock-table reports only on explicit refresh and preserve unrelated dynamic-block source.
- [x] T22.3: Introduce org_agenda contracts and verify overlaps, open intervals and transactional report replacement.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T22.1: `4d45e7957e9c`; T22.2: `3ca4d3d4461a`. Native comparisons, explicit
clocktable publication and shared-job contracts pass focused checks. T22.3
adds ten org_agenda cases, exact source/record/output bounds, checked shared-view
history and eight owned task/restart PTYs. Final exact-head validation and
canonical PR reference follow below; verified integration remains open.

T22.3: `e270b1e2859e2d02a039d30ff7e8c17f7ce365f1`.
[Canonical PR #123](https://forge.adrianpk.com/adrianpk/rev/pulls/123) merged
into dev at `608e64a51443e11931cc4a3897a948fe8f1ec072`.
Required receipt `DNBOQFBOGGFETXKTGCZVKLOZDT` passed five checks and 75 tests.
Focused receipt `KIMDANVL7LO66QYSIDUOONWPFO` passed 579 tests, reference agreement
and eight owned PTYs. The Slice 22 report is delivered.

### Slice 23: deliver native Org relative and countdown timers

Contracts: ORG22 ORG18.
Details and exact checks: [Slice 23](../plan/rev-org-mode.md#slice-23-deliver-native-org-relative-and-countdown-timers).

- [x] T23.1: Implement relative time insertion/list items, pause/resume/stop and countdown entry with familiar bindings.
- [x] T23.2: Arm only selected deadlines in the existing native event owner; cancel before due work and retain terminal cleanup.
- [x] T23.3: Verify one-shot expiry, restart, exit and no inactive wakeups in the owned Org time journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T23.1: `37ee7a7ac5c3`; T23.2: `899689081195`. Native timers, checked
source notes and selected deadlines pass focused core/command/display/recovery
checks. T23.3 adds nine owned timer PTYs, actual output-error cleanup, restart,
inactive waiting and stale replacement guards. Final exact-head validation and
canonical PR reference follow below; verified integration remains open.

T23.3: `2777a34834d158595fa7b64359f4b4a452fbe17a`. [Canonical PR #124](https://forge.adrianpk.com/adrianpk/rev/pulls/124)
and the [Slice 23 report](../report/slices/rev-org-mode/slice-23-timers.md) are
delivered. Final required/focused controller receipts bind the merged head;
verified dev integration is recorded below.

PR #124 merged into dev at `2cc50fe81a469178fb44a2c361b40d10558158a3`,
verified against the canonical forge and remote. Required receipt
`IKLOANP42NTQUAQD7IJ7TBDALH` passed six checks, 287 tests and nine owned PTYs.
Focused receipt `RJGSEJETDJE6ST7ZY4DOD3CB5N` passed 340 tests and configuration
reference agreement, zero skips. The Slice 23 report is delivered.

### Slice 24: maintain Org habit schedules and history

Contracts: ORG20 ORG07.
Details and exact checks: [Slice 24](../plan/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history).

- [x] T24.1: Recognize STYLE habit and integrate logged completion, interval ranges and Org day semantics.
- [x] T24.2: Preserve compatible histories and distinguish missing observations from actual failed/completed days.
- [x] T24.3: Verify recurrence types, source-local declarations and coherent completion undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T24.1: `5adee7cd157a`. Habit source admission, checked calendar ranges and
Org day recurrence pass focused arithmetic, configuration and native editing
checks. T24.2 adds bounded canonical history observations without source writes.

T24.2: `3a273633634370622a9eca19ed2c5bd43165d275`. T24.3 verifies early-day
recurrence, local declarations, preserved history, read-only/stale guards, native
file bytes, shared anchors and coherent undo. Fifteen selected habit cases pass,
zero skips. Canonical PR reference and final exact-head receipts follow.

T24.3: `922b9101d7f49beb1ec167b030c17e7043c13110`. [Canonical PR #125](https://forge.adrianpk.com/adrianpk/rev/pulls/125)
and the [Slice 24 report](../report/slices/rev-org-mode/slice-24-habit-records.md) are
delivered. Final controller receipts bind the merged head; verified integration
is recorded below.

PR #125 merged into dev at `411ba869fa1d4e2f3852f6caa8a0761e731e3f45`,
verified against the canonical forge and remote. Required receipt
`5E75KWPZSAPBNQ7VNSTOQ74DWJ` passed five checks and 271 tests. Focused receipt
`M5DOBIW7MT5UQRT7EN4P473YWM` passed 119 tests and configuration reference
agreement, zero skips. The Slice 24 report is delivered.

### Slice 25: render native Org habit consistency graphs

Contracts: ORG20 ORG16.
Details and exact checks: [Slice 25](../plan/rev-org-mode.md#slice-25-render-native-org-habit-consistency-graphs).

- [x] T25.1: Render bounded habit graphs with verified defaults, completion/today markers and semantic palette states.
- [x] T25.2: Deliver agenda visibility, all-today/today-only and graph controls through native commands.
- [x] T25.3: Verify narrow panes, late completions, dirty source observations and idle immutability.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T25.1: `60428ea7e251`. Bounded graph data, verified preferences and semantic
pane rendering pass fifteen selected checks. T25.2 adds the explicit native
habit agenda and compiled controls using the existing report worker.

T25.2: `78bbf7e8d2c3db3f74366863fb5e951b12a4a56e`. T25.3 verifies native
narrow/peer geometry, late status, exact source bytes, NO_COLOR, dirty completion
refresh, new-day capture, no idle graph work and 128/129 row admission. Twenty-one
selected cases pass, zero skips. Canonical PR and final receipts follow.

T25.3: `5f32a30d3fe59bd71d7636f74205fad7eccae829`. [Canonical PR #126](https://forge.adrianpk.com/adrianpk/rev/pulls/126)
and the [Slice 25 report](../report/slices/rev-org-mode/slice-25-habit-graphs.md) are
reviewing. Final required/focused controller receipts bind the clean PR-reference
head; verified dev integration remains pending.

Canonical PR #126 merged into dev at
`f1603b6e934166845ad4739ed8d727e55b7ffd88`. Required receipt
`SYVH5LSTFUPY5N2AAHCNV2DPHB` passed five ordered checks and 245 tests.
Focused receipt `37I2IPU4S4F5PJFTYT2JDRMGKX` passed 377 tests and the
configuration reference, zero skips. The Slice 25 report is delivered.

### Slice 26: discover bounded canonical Org agenda sources

Contracts: ORG08 ORG18.
Details and exact checks: [Slice 26](../plan/rev-org-mode.md#slice-26-discover-bounded-canonical-org-agenda-sources).

- [x] T26.1: Resolve explicitly selected files/directories without scanning the home or project implicitly.
- [x] T26.2: Prefer current buffers, deduplicate real file identities and observe disk sources through cancellable owned work.
- [x] T26.3: Verify unreadable sources, cycles, limits and truthful partial results while keeping editing responsive.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T26.1 delivers typed explicit source selection, non-recursive directory scope,
canonical identity and whole-source partial-result observations. Four focused
IO/configuration checks pass; native session ownership follows in T26.2.

T26.1: `09ff6bd06713`. T26.2 connects explicit C-c a/M-x source selection to
one shared native worker, revalidates each buffer stamp, and preserves editing.
Eight focused filesystem/native cases and nine command/reference checks pass.
The complete configuration reference now contains 192 bindings.

T26.2: `42c96d7e0be0`. T26.3 verifies real unreadable and special-file
failures, whole-file budget boundaries including failed reads, missing/unsaved
open sources, directory cycles, cancelled results and buffer-only count limits.
Sixteen selected native/IO/configuration checks pass; focused Clippy and source
quality preserve their original limits. Final canonical PR and receipts follow.

T26.3: `61ae680593ef1c0c33aac0521eeefa243c5594e0`.
[Canonical PR #127](https://forge.adrianpk.com/adrianpk/rev/pulls/127) and the
[Slice 26 report](../report/slices/rev-org-mode/slice-26-agenda-sources.md) are
delivered. PR #127 merged into dev at
`499077927bf1fc35ca73e01975e19b29a2aaa210`. Required receipt
`6LLRE3NDUBRD6AINGMU56GBW5H` passed five ordered checks and 152 tests.
Focused receipt `JWDJTHQ2JQ653WINLZQNMAMMYX` passed 412 tests and the
configuration reference, zero skips.

### Slice 27: display native day and week Org agendas

Contracts: ORG08 ORG09.
Details and exact checks: [Slice 27](../plan/rev-org-mode.md#slice-27-display-native-day-and-week-org-agendas).

- [x] T27.1: Deliver read-only agenda rows, time grid, overdue/scheduled/deadline sections and day/week navigation.
- [x] T27.2: Use revision-qualified source locators and the captured effective date/zone for every row.
- [x] T27.3: Verify real refresh, source return, shared buffers and no agenda-as-document mutation.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T27.1 delivers bounded day/week facts, planning sections and time grids with
captured date/zone context. Four focused model/configuration cases, focused
Clippy, source quality and the complete reference pass. Native ownership and
checked source navigation are delivered in T27.2. Two native calendar journeys
and fifteen existing source/agenda cases pass; focused Clippy, changed-source
quality and the complete configuration reference agree. T27.3 verifies real
refresh, captured clocks, checked source return, shared-pane ownership, endpoint
and hourly-range times, zone ambiguity and retained cancellation incompleteness.
Canonical [PR #128](https://forge.adrianpk.com/adrianpk/rev/pulls/128) merged
into dev at `2af9d3bd9c8e79b69a6cf97a440902e3b18aebc9`; report delivered.
Required receipt `32KTKDZP5RE2OA5E7RC2N4NIKE` passed five ordered checks and
112 tests. Focused receipt `P2RIIU7SARFXIGDQWNFMJOMQP6` passed 622 tests and
the configuration reference; zero failures or ignored tests.

### Slice 28: query Org tasks tags and properties

Contracts: ORG08 ORG23.
Details and exact checks: [Slice 28](../plan/rev-org-mode.md#slice-28-query-org-tasks-tags-and-properties).

- [x] T28.1: Deliver TODO/search/tag/property views with declared Boolean/comparison grammar and source scopes.
- [x] T28.2: Evaluate bounded parsed queries against canonical metadata, including inheritance and effort.
- [x] T28.3: Verify escaping, invalid operators, completeness and stable result identity.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T28.1 delivers typed query plans and the bounded Boolean/text/regexp language.
Two parameterized grammar cases, focused Clippy, changed-source quality and the
complete configuration reference pass. T28.2 adds canonical metadata evaluation,
qualified row locations and truthful partial results. Four query cases and
fourteen existing property cases pass. T28.3 verifies native query controls,
invalid drafts, scope retention, canonical refresh/return, Unicode shared panes,
regexp bounds and owned cancellation. Nine query cases plus the real cancellation
case pass; focused Clippy, source quality and the complete reference agree.
Canonical [PR #129](https://forge.adrianpk.com/adrianpk/rev/pulls/129) targets
dev; the report is reviewing. Controller evidence binds its final clean pushed
head. Integration remains controller-owned.

PR #129 merged into dev at `64e056b8d31138624adc1df9a8e7149df2862849`, verified on the canonical forge and origin/dev. Required receipt `CTMBFH6CJKUU33YSBF6REIRKAV` and focused receipt `NXJTUZYJMNPLIYNQCEOKSSBNZK` passed 665 tests with zero failures or ignored tests. Slice 28 is delivered.

## Slice 29: configure native Org agenda views

Contracts: ORG23 ORG17.
Details and exact checks: [Slice 29](../plan/rev-org-mode.md#slice-29-configure-native-org-agenda-views).

- [x] T29.1: Deliver typed custom/block views, columns, filtering, sorting and declared date spans.
- [x] T29.2: Expose compiled view/refresh controls and native agenda context bindings without a scripting registry.
- [x] T29.3: Verify combination limits, visibility/reset behavior and consistent exhaustive sample defaults.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #130](https://forge.adrianpk.com/adrianpk/rev/pulls/130) targets dev; the Slice 29 report is reviewing. T29.1/T29.2/T29.3 commits are `d6626fc1a2d1`, `0cf592cb0ab3` and `8fd93c685e46`. Thirteen custom cases, no-color rendering, focused Clippy and unchanged source-quality limits pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

T29.1 delivers admitted typed blocks and canonical observations. T29.2 delivers native presentation controls. T29.3 verifies limits, source scopes, preset reset, pane lifecycle and exhaustive defaults.

PR #130 merged into dev at `99125c9f9cc7c0ce774e7aa8b97556e30e225aaa`, verified against Forgejo and origin/dev. Required receipt `CF2NCOJATDVH4V6E7ITQJTNETK` and focused receipt `RVS434VRIFBRQQK3UOFBGJ6CTL` passed 679 tests with zero failures or ignored tests. Slice 29 is delivered.

## Slice 30: edit canonical Org source from the agenda

Contracts: ORG09 ORG23.
Details and exact checks: [Slice 30](../plan/rev-org-mode.md#slice-30-edit-canonical-org-source-from-the-agenda).

- [x] T30.1: Route agenda task/date/tag/priority/effort/clock actions through checked source handlers.
- [x] T30.2: Deliver explicit selection, bulk operations and per-target outcomes with revision re-resolution.
- [x] T30.3: Exercise dirty buffers, cancellation, failure and undo in an owned multi-source agenda journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #131](https://forge.adrianpk.com/adrianpk/rev/pulls/131) targets dev; the Slice 30 report is reviewing. T30.1/T30.2/T30.3 commits are `ecce41921d6f`, `a65745954df3` and `7bfb98cd0408`. Seven source cases, five bulk cases, six lifecycle cases and three focused RGB terminal scenarios pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

T30.1 routes checked agenda source actions through existing native handlers. Seven focused source cases pass, including cancellation, stale/read-only rejection, dirty canonical text, undo/redo, clock ownership and mapped shared views.

PR #131 merged into dev at `41648989959e0e01bb338f258c03624db99cd207`, verified against Forgejo and origin/dev. Required receipt `SY4DYDUC7OZY3AICUCNVXEVB22` and focused receipt `FQDAFBADYGNOJL76FRONONV2M5` passed 697 tests and six owned terminal sessions with zero failures or ignored tests. Slice 30 is delivered.

## Slice 31: define native Org capture targets and templates

Contracts: ORG10 ORG17.
Details and exact checks: [Slice 31](../plan/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates).

- [x] T31.1: Admit typed native capture templates/targets and the source-compatible factory fallback.
- [x] T31.2: Resolve file/headline/outline/date-tree/table/list targets without writing them during discovery.
- [x] T31.3: Verify explicit destination entry, missing targets and template/default/sample coherence.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #132](https://forge.adrianpk.com/adrianpk/rev/pulls/132) targets dev; the Slice 31 report is reviewing. T31.1 is `39f9929bc439`; T31.2 uses reader move `cf1810e5ee25` and implementation `768d2a9fd695`; T31.3 is `3a35303cc0e1`. Four configuration and nine file-backed capture workflows, focused Clippy and unchanged source-quality limits pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

PR #132 merged into dev at `f7fb5fbbea3c7df2fb4f3792f77f5a3db2d94222`, verified against Forgejo and origin/dev. Required receipt `YVZVSLCQXGTAJ5NCB6FT4JK5OB` and focused receipt `PSITFVEX7B3SW54RVYEJ5JK5ZX` passed 435 tests with zero failures or ignored tests. Slice 31 is delivered.

## Slice 32: expand and edit native Org capture drafts

Contracts: ORG10 ORG02.
Details and exact checks: [Slice 32](../plan/rev-org-mode.md#slice-32-expand-and-edit-native-org-capture-drafts).

- [x] T32.1: Expand the declared bounded text substitutions, prompts and checked origin links without evaluating Lisp.
- [x] T32.2: Retain the editable capture draft in its own transient owner with familiar context bindings.
- [x] T32.3: Verify malformed expansion, literal preservation, cancellation and return to the originating view.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #133](https://forge.adrianpk.com/adrianpk/rev/pulls/133) targets dev; the Slice 32 report is reviewing. T32.1 is `75774e457d81`, T32.2 is `bbfabe138789` and T32.3 is `d7a9c1e40cd5`. Five expansion cases, nine native session workflows, backend save/mode admission and two owned RGB/NO_COLOR terminal sessions pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

PR #133 merged into dev at `e4e81f12c16a4c1c9ed687e04f3b4c3231d49221`, verified against Forgejo and origin/dev. Required receipt `MMEG2LYSXODJ4G3A47XFYR565F` and focused receipt `HBFNNHQQRBF2GSAUSGCOFQY23R` passed 665 tests and two owned terminal sessions with zero failures or ignored tests. Slice 32 is delivered.

## Slice 33: finalize or abort Org capture safely

Contracts: ORG10 ORG18.
Details and exact checks: [Slice 33](../plan/rev-org-mode.md#slice-33-finalize-or-abort-org-capture-safely).

- [x] T33.1: Revalidate destinations and publish capture once through ordinary source/file safeguards.
- [x] T33.2: Preserve a recoverable draft after stale, read-only, save or filesystem failure; abort leaves targets unchanged.
- [x] T33.3: Verify file/heading/list/table/date-tree capture and failed publication with actual owned files.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #134](https://forge.adrianpk.com/adrianpk/rev/pulls/134) targets dev; the Slice 33 report is reviewing. T33.1 is `0cdd4a33a0f3`, T33.2 is `7396cdeaa1fa` and T33.3 is `85a4a9279e7c`. Checked publication, ordinary undo/format preservation, recoverable drafts, pinned unresolved-save retry and owned RGB/NO_COLOR save/rejection cases pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

PR #134 merged into dev at `29fa9bc4417f7a1f38757f6daab5b390adc21201`, verified against Forgejo and origin/dev. Required receipt `4GVVTFNMHYYD3XHTN5MKZ2FIGC` and focused receipt `DFIW4EIK5RW2ZKVQO4A6SLWQQP` passed 753 tests and six owned terminal sessions with zero failures or ignored tests. Slice 33 is delivered.

## Slice 34: refile Org subtrees across canonical documents

Contracts: ORG11 ORG02.
Details and exact checks: [Slice 34](../plan/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents).

- [x] T34.1: Prepare cross-buffer subtree moves and destination insertion as one checked semantic operation.
- [x] T34.2: Keep source/destination revisions, history groups and durable-save outcomes explicit and recoverable.
- [x] T34.3: Verify duplicate/stale targets, read-only buffers, external file replacement and partial publication truth.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical [PR #135](https://forge.adrianpk.com/adrianpk/rev/pulls/135) targets dev; the Slice 34 report is reviewing. T34.1 is `f907554f0469`, T34.2 is `c01a962955ec` and T34.3 is `806728b058be`. Checked refile/copy, coherent canonical publication, shared positions, stale/duplicate/read-only targets, per-file save recovery and owned RGB/NO_COLOR workflows pass. Final controller evidence binds the clean pushed PR head; integration remains controller-owned.

PR #135 merged into dev at `eae5238f770073727d2c543958713f331cd635f0`, verified against Forgejo and origin/dev. Required receipt `ZNK2FZH7WA6P7KTOVUE7DA2XZE` and focused receipt `XPBAA7HO6WQSNHHYSCJ2T4WQWJ` passed 739 tests and six owned terminal sessions with zero failures or ignored tests. Slice 34 is delivered.

## Slice 35: archive Org subtrees without silent loss

Contracts: ORG11 ORG09.
Details and exact checks: [Slice 35](../plan/rev-org-mode.md#slice-35-archive-org-subtrees-without-silent-loss).

- [x] T35.1: Deliver archive tag, archive sibling and %s_archive destination behavior with Org metadata.
- [x] T35.2: Reuse checked multi-document moves and retain archived source discoverability under selected agenda policy.
- [x] T35.3: Exercise capture/refile/archive, undo and failed destinations in one owned terminal journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

T35.1 is `1ca449bbe1e2`; T35.2 is `c6f0246d7266`. Native archive plans,
metadata, exact target admission, archived agenda policy, retained per-file
save outcomes and the combined owned RGB/NO_COLOR journey pass focused checks.
T35.3 is `d807b2fd7d03`. Canonical [PR #136](https://forge.adrianpk.com/adrianpk/rev/pulls/136)
targets dev; the existing Slice 35 report is reviewing. Final controller receipts
bind the clean pushed head after PR references. Integration remains controller-owned.

PR #136 merged into dev at `ea25d56b2f02af66d9c3d2c34b5839081f8c1c83`, verified against Forgejo and origin/dev. Required receipt `7TT4OOJQLCEMAY5ZEMLEIAWGKF` and focused receipt `HCDOP5M2CL3CUNIC27O6YO3TZG` passed 788 tests and eight owned terminal sessions with zero failures, ignored tests or skipped sessions. Slice 35 is delivered.

## Slice 36: store and follow checked Org links

Contracts: ORG15 ORG02.
Details and exact checks: [Slice 36](../plan/rev-org-mode.md#slice-36-store-and-follow-checked-org-links).
Canonical PR: [#137](https://forge.adrianpk.com/adrianpk/rev/pulls/137).

- [x] T36.1: Deliver internal targets, file/ID/attachment links, insertion, storage and navigation.
- [x] T36.2: Re-resolve source and dispatch external resources through the bounded owned job path only after explicit follow; introduce the shared child-lifetime primitive here.
- [x] T36.3: Verify ambiguity, encoded paths, read-only documents and source-preserving failure.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.


Canonical PR #137 merged into dev at `be826a5a2d2b11dc6ef67c8cbd8b2956a5b0234c`, verified against Forgejo and origin/dev. Required receipt `BCKPCVCG6A42ETM253MRSYVMM5` passed five ordered checks and 473 tests. Focused receipt `Z75SNTPHWVOEUGB3KVNZVM6JXI` passed 309 tests and six owned terminal sessions. Zero failures, ignored tests or skipped sessions. Slice 36 is delivered.

## Slice 37: manage native Org heading attachments

Contracts: ORG24 ORG15.
Details and exact checks: [Slice 37](../plan/rev-org-mode.md#slice-37-manage-native-org-heading-attachments).
Canonical PR: [#138](https://forge.adrianpk.com/adrianpk/rev/pulls/138).

- [x] T37.1: Deliver ID/DIR attachment locations, linking, copy/move/list/open and reviewed dispatcher controls.
- [x] T37.2: Recheck real paths and collisions and retain operation evidence across partial file/source failure.
- [x] T37.3: Verify directory traversal, symlinks, unreadable files and that visiting a heading performs no attachment IO.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR #138 merged into dev at `c568a14d90d932eb58918666798bfaaa796df0ac`, verified against Forgejo and origin/dev. Required receipt `E3W242KXPLDXVGEULP7P7ZV7RT` passed five ordered checks and 252 tests. Focused receipt `G7VP3AJIZRUYE3RYOSUOL3R6CA` passed 340 tests and six owned terminal sessions. Zero failures, ignored tests or skipped sessions. Slice 37 is delivered.

## Slice 38: edit native Org footnotes and inline content

Contracts: ORG27 ORG30 ORG02.
Canonical PR: [#139](https://forge.adrianpk.com/adrianpk/rev/pulls/139).
Details and exact checks: [Slice 38](../plan/rev-org-mode.md#slice-38-edit-native-org-footnotes-and-inline-content).

- [x] T38.1: Deliver footnote create/follow/renumber, entities, text markup and source-preserving affiliated forms.
- [x] T38.2: Keep literal blocks and unknown forms distinct during structured editing and context actions.
- [x] T38.3: Verify Unicode, malformed references, radio targets and coherent edit/history behavior.
Task commits: T38.1: `34374ac5f4b97bc735e4f554bc1d311a672e0035`, T38.2: `508aee2c666ba74aae01bc8e39b5875dc61fa4aa`, T38.3: `c70f78a1add18ea785603dc364ce7490912e55e2`.

- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR #139 merged into dev at `c754558125d33a9b400ed58921b62cba41ce5ad5`, verified against Forgejo and origin/dev. Required receipt `G2QMJXDYBO73K2K2US5HK7EWIH` passed five ordered checks and 319 tests. Focused receipt `GLOQUWSDGVCFE722MLZZJUBY2E` passed 304 tests and six owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions; prior diagnostics remain in the report. Slice 38 is delivered.

## Slice 39: navigate and align native Org tables

Contracts: ORG12 ORG17.
Details and exact checks: [Slice 39](../plan/rev-org-mode.md#slice-39-navigate-and-align-native-org-tables).

- [x] T39.1: Introduce org_tables and native cell/row navigation, insertion and alignment over original cell source.
- [x] T39.2: Retain separators, names, captions, formulas and non-table text while preserving displayed columns.
- [x] T39.3: Verify tabs, wide graphemes, CRLF, empty cells and one-operation undo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#140](https://forge.adrianpk.com/adrianpk/rev/pulls/140).
Task commits: T39.1 `e9f5e768b05b5b76f14725c0922963b14a54370b`, T39.2 `7b7da9152943c2426df8dc03cce5598fa70e0214`, T39.3 `a1777bf863707211f7b884dbacd2f6d2b6f974e2`.
Canonical PR #140 merged into dev at `140adb91c9f7f716d7989b7405033a111f821553`, verified against Forgejo and origin/dev. Required receipt `MVEID2P3N26WG3IFGUYYGYPYTC` passed five ordered checks and 261 tests. Focused receipt `ZVEACD2WH5NQ6NSGW5SR5CHMZ3` passed 305 tests and four owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions; the pre-existing outline timer assertion diagnostic remains in the report. Slice 39 is delivered.

## Slice 40: edit Org table rows columns and references

Contracts: ORG12 ORG14.
Details and exact checks: [Slice 40](../plan/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references).

- [x] T40.1: Deliver row/column insert/delete/move, sorting and list/table conversion with checked ownership.
- [x] T40.2: Rewrite only supported formula references and reject ambiguous transformations before source mutation.
- [x] T40.3: Verify cell value fidelity, separator boundaries and full structural undo/redo.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#141](https://forge.adrianpk.com/adrianpk/rev/pulls/141).
Task commits: T40.1 `f1fc1cf7ecf203aca3fc60579a91293ba7981a51`, T40.2 `fe1580557d2eef0aaf126385a3d95659e3191768`, T40.3 `10a4c277d4e2bdc8313fbb9134838fa263b91987`.

Canonical PR #141 merged into dev at `e0f3401ad564847c2286ceafaba23c9114b39147`, verified against Forgejo and origin/dev. Required receipt `U3Q43PKH3AXV25XJSCHQRWP5E2` passed five ordered checks and 256 tests. Focused receipt `7YB63JTRCI5JXC6V7RGZMGMORG` passed 341 tests and two owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions; retained diagnostics remain in the report. Slice 40 is delivered.

## Slice 41: parse the bounded Org formula subset

Contracts: ORG13 ORG14.
Details and exact checks: [Slice 41](../plan/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset).

- [x] T41.1: Parse declared arithmetic, field/column/range references, functions and compatible TBLFM assignments.
- [x] T41.2: Enforce precedence, right-associative exponentiation and bounded syntax/depth before evaluation.
- [x] T41.3: Verify unsupported Calc/Lisp preservation, malformed references and precise diagnostics.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#142](https://forge.adrianpk.com/adrianpk/rev/pulls/142).
Task commits: T41.1 `9f599f4cd663572378ee0820c8e242204642bd35`, T41.2 `cab7833a5be8c819c9b17548504b09ca19805bbe`, T41.3 `3872b96e962cd03dfd3700f44f763e5528492065`.

Canonical PR #142 merged into dev at `f65f56a4198b08b64b79c7a929d7fddfa37ef5bd`, verified against Forgejo and origin/dev. Required receipt `QNH4ZCQ4XYRJHBJF6NRALLEKAR` passed five ordered checks and 106 tests. Focused receipt `Q5TWMB2RETPEXLMFNXLROATRIQ` passed 48 library tests and two owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions. Slice 41 is delivered.

## Slice 42: evaluate basic Org table formulas safely

Contracts: ORG13 ORG14.
Details and exact checks: [Slice 42](../plan/rev-org-mode.md#slice-42-evaluate-basic-org-table-formulas-safely).

- [x] T42.1: Evaluate dependency graphs, aggregates, rounding and formatting under the frozen numeric policy.
- [x] T42.2: Detect cycles, empty/text misuse, division by zero, non-finite values and work-budget exhaustion.
- [x] T42.3: Verify expected arithmetic independently, including negative half rounding and reference direction.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#143](https://forge.adrianpk.com/adrianpk/rev/pulls/143).
Task commits: T42.1 `9b64692c22d779254de9e0bc673a835ed37ccf4d`, T42.2 `a82d0bafe9b0c99def549d265c6a87a2bc401712`, T42.3 `858a37838e90fee2cc6d83cc6da767c542a19ac4`.

Canonical PR #143 merged into dev at `04e2f4d8fe5b932eecf9f767c1a2b9035a0947e0`, verified against Forgejo and origin/dev. Required receipt `LRKE3EYHBXLIIWBJDLGJGLO4NZ` passed five ordered checks and 138 tests. Focused receipt `ZCW5ICYYNSCDKALWG77QG5INED` passed 111 library/source tests and two owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions. Slice 42 is delivered.

## Slice 43: publish Org recalculation as one transaction

Contracts: ORG13 ORG14 ORG17.
Details and exact checks: [Slice 43](../plan/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction).

- [x] T43.1: Prepare all admitted results before applying the selected recalculation as one checked history unit.
- [x] T43.2: Expose table/cell/column recalculation through contextual commands and M-x without implicit evaluation.
- [x] T43.3: Exercise success, unsupported formulas, failure preservation and undo in the owned table journey.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#144](https://forge.adrianpk.com/adrianpk/rev/pulls/144).
Task commits: T43.1 `bf412f7dd09ec93ed767fc6430a59ba1f496142e`, T43.2 `7b952916a66f05bfdf861475fd65ea5c335a416f`, T43.3 `060cee2e12457b034ca260bc4077b9b3e74c5407`.

Canonical PR #144 merged into dev at `9698833f2061afda469515ee7f1fd06bb406f309`, verified against Forgejo and origin/dev. Required receipt `PVVHBQWSO3JDXA4CAUZX75RHJU` passed six ordered checks, 445 tests and two owned terminal sessions. Focused receipt `WJYTUOCOFJ3XUC6KGT6FZHZEOB` passed 231 tests and the configuration reference. Final receipts have zero failures, ignored tests or skipped sessions. Slice 43 is delivered.

## Slice 44: import and export Org table data

Contracts: ORG12 ORG27.
Details and exact checks: [Slice 44](../plan/rev-org-mode.md#slice-44-import-and-export-org-table-data).

- [x] T44.1: Deliver bounded CSV/TSV import/export and list/table conversion through native commands.
- [x] T44.2: Preserve quoting and representable cell values; reject embedded line breaks or unsupported transformations before mutation rather than inventing lossy escapes.
- [x] T44.3: Verify actual file round trips and rejected malformed input without lossy recovery.
- [x] Record exact-head validation, canonical PR/report and verified dev integration.

Canonical PR: [#145](https://forge.adrianpk.com/adrianpk/rev/pulls/145).
Task commits: T44.1 `f9207a63c2fe94924dcdee632c4f886108c35840`, T44.2 `2ff48bf6f4eb649c09f7e09324b80aa8177cb92f`, T44.3 `bf2882b2a749ee353efdb3689bf61af90768bddf`.

Canonical PR #145 merged into dev at `009bdd6b5cde9894bc8ba158e398a1b4872c10e2`, verified against Forgejo and origin/dev. Required receipt `WNCPD2FFP52735Z2N3ZRXQBNDK` passed five ordered checks and 211 tests. Focused receipt `2Q7VDJ4I6P726SAKRZGHM54DDQ` passed 158 command tests and two owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions. Slice 44 is delivered.

## Slice 45: display Org property columns and native reports

Contracts: ORG21 ORG23 ORG27.
Details and exact checks: [Slice 45](../plan/rev-org-mode.md#slice-45-display-org-property-columns-and-native-reports).

- [x] T45.1: Deliver property column views and declared native dynamic report blocks, including effort and clock totals.
- [x] T45.2: Keep projections separate from source; replace only recognized explicit report bodies on update.
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
