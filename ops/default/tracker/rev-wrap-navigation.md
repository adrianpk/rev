# Wrap and File Navigation Tracker

Status: approved
Date: 2026-10-31
Delivery set: rev-wrap-navigation
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; five selected topics in one autonomous Looper cycle, Emacs interaction criterion and explicit request to start
Planning baseline: `25039bf27b4c9610371fbd9987fcfb1f91cb9910`
Spec: [Word-Aware Soft Wrapping](../spec/editor-soft-wrap.md)
Model: [Soft Wrap Model](../spec/editor-soft-wrap-model.md)
Spec: [File and Directory Navigation](../spec/editor-file-navigation.md)
Model: [Navigation Model](../spec/editor-file-navigation-model.md)
Concern: [Pending Work Index](../README.md)
Plan: [Wrap and Navigation Plan](../plan/rev-wrap-navigation.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-wrap-navigation.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 7
Execution owner: Looper; Slice 7 implementation active

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Word-aware source display | `feat/rev-wrap-navigation-01-word-wrap` | `feat(wrap): render word-aware source lines` | `ops/default/report/slices/rev-wrap-navigation/slice-1-word-wrap.md` |
| Slice 2 | Emacs visual-line editing | `feat/rev-wrap-navigation-02-visual-editing` | `feat(wrap): follow Emacs visual-line editing` | `ops/default/report/slices/rev-wrap-navigation/slice-2-visual-editing.md` |
| Slice 3 | Wrapped selection and gutter integration | `feat/rev-wrap-navigation-03-wrapped-interaction` | `feat(wrap): map selection and search through visual rows` | `ops/default/report/slices/rev-wrap-navigation/slice-3-wrapped-interaction.md` |
| Slice 4 | Org projection and bounded wrap lifetimes | `feat/rev-wrap-navigation-04-org-wrap` | `feat(wrap): integrate Org projections and bounded layout work` | `ops/default/report/slices/rev-wrap-navigation/slice-4-org-wrap.md` |
| Slice 5 | Directory targets and initial tree expansion | `feat/rev-wrap-navigation-05-directory-targets` | `feat(files): open directory targets with an expanded tree` | `ops/default/report/slices/rev-wrap-navigation/slice-5-directory-targets.md` |
| Slice 6 | Highlighted path completion | `feat/rev-wrap-navigation-06-path-tab` | `fix(files): complete the highlighted path with Tab` | `ops/default/report/slices/rev-wrap-navigation/slice-6-path-tab.md` |
| Slice 7 | Clicked tree entry and activation | `feat/rev-wrap-navigation-07-tree-click` | `feat(tree): select and activate the clicked entry` | `ops/default/report/slices/rev-wrap-navigation/slice-7-tree-click.md` |
| Slice 8 | User guidance and joint interaction acceptance | `feat/rev-wrap-navigation-08-interaction-acceptance` | `test(editor): validate wrap and file navigation together` | `ops/default/report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md` |

## Slice 1: Word-aware source display

Acceptance: WRP01, WRP02, WRP06.

- [x] T1.1: Implement one checked source-to-visual fragment owner, word-aware/grapheme-safe rendering and point reveal at each pane text width.
- [x] T1.2: Add editor.soft_wrap with compiled true default, complete TOML/sample/reference coverage and the buffer-local visual-line-mode command; create tests/wrapping.rs with actual session/render cases.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 1 report.

Implementation commits: T1.1 `aa7c739f30ba097955193c43852508716cf6017e`;
T1.2 `9946e539aaa39b1e81cc78f40e2e339f08cb6536`.
Focused wrapping tests (six native tests) and the command-catalog regression passed.
Canonical receipt `Y7GHL424WJD6IBUZMYYYXSN7K7` passed the configured checks and 339 tests
at `366efcd384b163e69478c3bf4bda918bd1d7547a`. PR #174 is merged at that exact
commit; the controller recorded its merge in Acta. Slice 1 is delivered.

## Slice 2: Emacs visual-line editing

Acceptance: WRP03, WRP07.

- [x] T2.1: Resolve arrows, C-n/C-p, C-a/C-e and C-k through the current wrapped view while preserving canonical kill-ring/history and read-only admission.
- [x] T2.2: Integrate goal columns, paging, recenter, goto-line and split point/scroll lifetimes; verify end-of-row/newline/EOF/repeated-kill boundaries against Emacs.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 2 report.

Implementation commits: T2.1 `36c6253e6c797206f574beda907fea86e6dbfde3`;
T2.2 `85e57005676c2562486d545a0b756836a5c267ce`.
Canonical receipt `MWK2ELBNYFS4FPNKGLAFEHR22Z` passed the configured checks and
180 tests at `70a20019d4bc043f178462eec578fa715c12895e`. Supplemental receipt
`F7DUK4HPS4QFZ4TNSBLRJB23NG` passed library Clippy, 39 display tests and the
documentation checker. PR #175 is merged at that exact commit; the controller
recorded its merge in Acta. Slice 2 is delivered.

## Slice 3: Wrapped selection and gutter integration

Acceptance: WRP02, WRP03.

- [x] T3.1: Use the same current mapping for pointer placement/drag/autoscroll, linear selections, search reveal and overlays; keep rectangle semantics in logical source rows/columns.
- [x] T3.2: Render logical gutter labels only on a source line's first fragment, preserve relative logical distances and invalidate stale geometry after source/fold/resize changes.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 3 report.

Implementation commits: T3.1 `f2e4779891ef87b84bda9a7613bc43012d303507`;
T3.2 `27a8f5c903c890d11e1ed3fcbc5dabae15efb83b`. Native continuation interactions,
paired no-wrap contracts and gutter/fold/resize cases passed focused checks.
Canonical receipt `LQUE67OGIACLOHM7ZOMG55YYUN` passed the configured checks
and 205 tests at `622309c6e57c11aa70dab4ce7e64f8f384a1dd3a`. Supplemental
receipt `SUYGOP5R6BF5ADQOGYP574L7OV` passed all-target Clippy, 39 display tests
and documentation validation (58 pages, 330 local links). PR #176 is merged at
that exact commit; the controller recorded its merge in Acta. Slice 3 is delivered.

## Slice 4: Org projection and bounded wrap lifetimes

Acceptance: WRP04, WRP05; complete WRP01-WRP07.

- [x] T4.1: Compose wrapping with existing Org folds, headings, table fields, source blocks and preview projections; retain logical command targets and source-preserving alignment.
- [x] T4.2: Complete buffer-local override/split/close/invalidation and bounded cache/work ownership. Measure large/long-line resize, sustained input/reversal and idle costs; resolve the wrap ticket with exact evidence.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 4 report.

T4.1 commit `ba7233b763b514be0c0fb0cf205abd2c4ae4c227` delivers wrapped Org
source/field/draft/preview integration. T4.2 completes bounded retention,
resumable coordinate/fragment work, cold command/pointer preparation, finite
ordered input, cancellation and shared frame mapping. Native API and release
PTY probes restore exact source, settle input/reversal and record idle costs.
See the [Slice 4 report](../report/slices/rev-wrap-navigation/slice-4-org-wrap.md)
for work ceilings, measurements and their limits. Override lifetime through
split/close/revisit is covered under both defaults. Implementation candidate
`034b721bb110461e7106cdea970564548934d009` passed all eleven controller
commands and 942 tests in receipt `JNLPDE7N5VZQHLQ33J3U6447EA`. T4.2 commits
`0fe5431e9ee449986bb56c3aadb669513c941331` and
`034b721bb110461e7106cdea970564548934d009` complete native input ownership
and settled Org request admission. Canonical [PR #177](https://forge.adrianpk.com/adrianpk/rev/pulls/177)
merged into `dev` at `279242c0351ce10d2415d97c0aa11f1196b0b63c`. Its final
configured checks passed 596 tests in receipt `K6BO6PQAFLAXYANN27HMXLOHZG`.
Slice 4 of 8 completed; the wrap ticket is solved. Slice 5 is active.

## Slice 5: Directory targets and initial tree expansion

Acceptance: NAV01, NAV02, NAV03, NAV06.

- [x] T5.1: Unify CLI/find-file file-versus-directory admission, Return final directory acceptance, explicit-root precedence and failure-preserving publication.
- [x] T5.2: Initialize only immediate tree children on first presentation/root change, preserve later collapse/hide/show and configuration precedence; resolve both opening/initial-expansion tickets.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 5 report.

T5.1 commit `c5a8568c938772c69018c18fa5f2bb935b003632` delivers shared target
admission and explicit root precedence. T5.2 commit
`18ce4a7f1138511ef060b79fbde16755a8a9c494` adds first-presentation ownership,
bounded immediate listings and retained focus intent. Clean implementation head
`121c167275df9e2f19f5d5470859bdb13bb86b49` passed 336 native tests,
documentation validation and nine CLI/PTY cases in controller receipt
`5BMRLUDPOYEPF7E4V3XQYLLXMZ`. Canonical [PR #178](https://forge.adrianpk.com/adrianpk/rev/pulls/178)
merged into `dev` at `941faa796047ef7b2dbc809afe495903f690baaa`. Both directory
tickets are solved. See the
[Slice 5 report](../report/slices/rev-wrap-navigation/slice-5-directory-targets.md).
Final configured receipt `HAYGF7IDRKHEKYSHCJNEVE2NZW` passed 336 tests on
that exact merge head; the controller recorded its merge in Acta. Slice 6 is active.

## Slice 6: Highlighted path completion

Acceptance: NAV04 and replacement UI03.

- [x] T6.1: Make the first result visibly selected and Tab fill that exact candidate, including ambiguous prefixes and explicit list navigation, while Return retains typed/new-file semantics.
- [x] T6.2: Retain directory-component continuation, current-candidate identity across refresh and bounded observation; apply the full UI03 amendment and resolve the path Tab ticket.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 6 report.

T6.1 commit `28ff1967c7330a6619e144a70487f6dc73d7a3cd` delivers highlighted
native path completion and atomic refresh. T6.2 preserves IO error kinds,
exercises continuation, stale/denied retries and clipping, applies full UI03/R12
reconciliation, and updates reader guidance plus the focused existing PTY probe.
T6.2 commit `1f061dea66754b5090dcbd42b24faad2d0f7a46c` and command/reference
alignment commit `e1122a0ba59b326671c9ab97109be6191c1b801f` passed all six
controller commands, 414 tests and two RGB/NO_COLOR terminal sessions in receipt
`3I26EZ2A36CO2PZGTFKU2M7DGM` at the latter exact clean head. Canonical
[PR #179](https://forge.adrianpk.com/adrianpk/rev/pulls/179) merged into `dev` at
`e9aadd9805bcd5a8b418d3575ffcd974bfa080e7`. Final configured receipt
`AQEZSXJGMWQTINQSMKLKESZQP6` passed 414 tests at that exact head. The controller
recorded its merge in Acta. The ticket is solved; Slice 7 is active. See the [Slice 6 report](../report/slices/rev-wrap-navigation/slice-6-path-tab.md).

## Slice 7: Clicked tree entry and activation

Acceptance: NAV05, NAV06.

- [x] T7.1: Carry rendered entry identity through tree hit testing, focus/select that row and apply the existing tree action to it; validate after viewport/listing changes.
- [ ] T7.2: Add mouse.tree_activate_on_click default true with false selection-only, complete sample/reference validation and tests; preserve global mouse disablement and resolve the pointer ticket.
- [ ] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 7 report.

## Slice 8: User guidance and joint interaction acceptance

Acceptance: All WRP01-WRP07, NAV01-NAV06 and UI03 scenarios.

- [ ] T8.1: Deliver native Rust/Bash joint journeys in bash scripts/check-wrap-navigation.sh, using the real event/composition path plus a bounded representative Linux PTY route. Reuse existing probes; add no Python gate or new runtime dependency.
- [ ] T8.2: Update the editor User Guide with short visual examples, defaults/commands/configuration and CHANGELOG Unreleased; reconcile affected delivered baseline clauses and complete all five ticket transitions/links before the full gate.
- [ ] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 8 report.

## Integration evidence

| Delivery slice | State | PR | Integrated commit | Validation/report |
| --- | --- | --- | --- | --- |
| 1 | delivered | #174 | `366efcd384b163e69478c3bf4bda918bd1d7547a` | [Slice 1 report](../report/slices/rev-wrap-navigation/slice-1-word-wrap.md) |
| 2 | delivered | #175 | `70a20019d4bc043f178462eec578fa715c12895e` | [Slice 2 report](../report/slices/rev-wrap-navigation/slice-2-visual-editing.md) |
| 3 | delivered | #176 | `622309c6e57c11aa70dab4ce7e64f8f384a1dd3a` | [Slice 3 report](../report/slices/rev-wrap-navigation/slice-3-wrapped-interaction.md) |
| 4 | delivered | #177 | `279242c0351ce10d2415d97c0aa11f1196b0b63c` | [Slice 4 report](../report/slices/rev-wrap-navigation/slice-4-org-wrap.md) |
| 5 | delivered | #178 | `941faa796047ef7b2dbc809afe495903f690baaa` | [Slice 5 report](../report/slices/rev-wrap-navigation/slice-5-directory-targets.md) |
| 6 | delivered | #179 | `e9aadd9805bcd5a8b418d3575ffcd974bfa080e7` | [Slice 6 report](../report/slices/rev-wrap-navigation/slice-6-path-tab.md) |
| 7 | pending | pending | pending | pending |
| 8 | pending | pending | pending | pending |

## Delivery acceptance

- [ ] Factory word-aware wrap and Emacs visual-line commands work without configuration.
- [ ] Source bytes, logical numbering, history, selection and Org semantics survive wrapping.
- [ ] CLI/find-file accept files or directories with correct root/visibility precedence.
- [ ] First tree presentation exposes immediate entries without repeated state resets.
- [ ] Tab completes the visibly selected path without accepting it.
- [ ] Click selects the real tree entry and follows the configured native activation policy.
- [ ] All five tickets are solved with implementation/PR evidence and valid final links.
- [ ] Core guidance, supporting references, samples and Unreleased match delivered behavior.
- [ ] All eight configured reports exist and accurately identify focused validation.
- [ ] Controller passes the pinned full gate on one exact clean integrated dev candidate.
- [ ] Metadata-only closure completes plan/tracker/specs/models/index/reports within allowed paths.

## Boundaries

No implemented checks, PRs, commits, merges or timing measurements are claimed
by this initial tracker. Looper updates progress and evidence during execution.
The selected gate and branch/report identities are immutable after startup;
follow native recovery rather than rewriting configuration or checkpoints.
