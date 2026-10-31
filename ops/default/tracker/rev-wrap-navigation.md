# Wrap and File Navigation Tracker

Status: Completed
Date: 2026-10-31
Completed: 2026-10-31
Validated candidate: `6b2afd4e07f269c979cd26faf60b6863314e69e9`
Gate receipt: `F7OQU2CWIOPB3VW234VVECCQNG`
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
Active slice: none; delivery completed
Execution owner: Looper; completed after controller gate and documentation closure

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
Slice 4 of 8 completed; the wrap ticket is solved. Slice 5 subsequently integrated.

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
that exact merge head; the controller recorded its merge in Acta. Slice 6 subsequently integrated.

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
recorded its merge in Acta. The ticket is solved; Slice 7 subsequently integrated. See the [Slice 6 report](../report/slices/rev-wrap-navigation/slice-6-path-tab.md).

## Slice 7: Clicked tree entry and activation

Acceptance: NAV05, NAV06.

- [x] T7.1: Carry rendered entry identity through tree hit testing, focus/select that row and apply the existing tree action to it; validate after viewport/listing changes.
- [x] T7.2: Add mouse.tree_activate_on_click default true with false selection-only, complete sample/reference validation and tests; preserve global mouse disablement and resolve the pointer ticket.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 7 report.

T7.1 commit `65023e2ee5bb5ea94ea2ac34d5c70adcf159ca41` delivers checked tree hits and native activation.
T7.2 adds the startup selection-only preference, strict configuration/sample
coverage, canonical alias and denied-access cases, pointer baseline amendments
and focused native SGR terminal validation. The pointer ticket is solved.
Final configured receipt `N7OR4FAOUUHBTWEANHW5AT3ALC` passed 359 tests at
`754d7af1ed7915e4db1e508b7a3fbbbe73c62c0b`, the exact canonical PR #180 merge
into `dev`. The controller recorded the merge in Acta. Slice 7 is delivered;
Slice 8 subsequently integrated. See the [Slice 7 report](../report/slices/rev-wrap-navigation/slice-7-tree-click.md).

## Slice 8: User guidance and joint interaction acceptance

Acceptance: All WRP01-WRP07, NAV01-NAV06 and UI03 scenarios.

- [x] T8.1: Deliver native Rust/Bash joint journeys in bash scripts/check-wrap-navigation.sh, using the real event/composition path plus a bounded representative Linux PTY route. Reuse existing probes; add no Python gate or new runtime dependency.
- [x] T8.2: Update the editor User Guide with short visual examples, defaults/commands/configuration and CHANGELOG Unreleased; reconcile affected delivered baseline clauses and complete all five ticket transitions/links before the full gate.
- [x] Verify exact-HEAD checks, record the canonical PR/merge and deliver the Slice 8 report.

T8.1 introduces native Rust/composition and Linux PTY joint acceptance plus the
reused five-fixture long-line probe. T8.2 reconciles reader guidance, CLI/defaults,
affected baseline clauses and all five solved tickets. T8.1 commit
`ea088b08e97c1c108fabf81da857f7f6f3fbdbfe` and T8.2 commit
`30945f95121559878db1e9f80c8b848bce59d475` are complete. Controller receipt
`GS6IDBXSHHMCRIIXMPJ6MBN7KW` passed the six recorded commands, 499 tests, native
joint acceptance, bounded PTY/performance probes, documentation and focused Clippy
on the exact clean T8.2 head. Canonical [PR #181](https://forge.adrianpk.com/adrianpk/rev/pulls/181)
merged at `4ec549c581d919b72e3e2e89cb4752507e0b1380`. Final configured receipt
`AUDRUYJUFLGQSXH3OTQIH56NFY` passed the five required commands and 499 tests on
that exact clean head. The controller recorded its merge in Acta as
`01M682VDRQ3BVAVDSPBZAYWXC3`. All eight implementation slices are integrated;
the controller owns the final integrated gate and closure.
See the [Slice 8 report](../report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md).

## Integration evidence

| Delivery slice | State | PR | Integrated commit | Validation/report |
| --- | --- | --- | --- | --- |
| 1 | delivered | #174 | `366efcd384b163e69478c3bf4bda918bd1d7547a` | [Slice 1 report](../report/slices/rev-wrap-navigation/slice-1-word-wrap.md) |
| 2 | delivered | #175 | `70a20019d4bc043f178462eec578fa715c12895e` | [Slice 2 report](../report/slices/rev-wrap-navigation/slice-2-visual-editing.md) |
| 3 | delivered | #176 | `622309c6e57c11aa70dab4ce7e64f8f384a1dd3a` | [Slice 3 report](../report/slices/rev-wrap-navigation/slice-3-wrapped-interaction.md) |
| 4 | delivered | #177 | `279242c0351ce10d2415d97c0aa11f1196b0b63c` | [Slice 4 report](../report/slices/rev-wrap-navigation/slice-4-org-wrap.md) |
| 5 | delivered | #178 | `941faa796047ef7b2dbc809afe495903f690baaa` | [Slice 5 report](../report/slices/rev-wrap-navigation/slice-5-directory-targets.md) |
| 6 | delivered | #179 | `e9aadd9805bcd5a8b418d3575ffcd974bfa080e7` | [Slice 6 report](../report/slices/rev-wrap-navigation/slice-6-path-tab.md) |
| 7 | delivered | #180 | `754d7af1ed7915e4db1e508b7a3fbbbe73c62c0b` | [Slice 7 report](../report/slices/rev-wrap-navigation/slice-7-tree-click.md) |
| 8 | delivered | #181 | `4ec549c581d919b72e3e2e89cb4752507e0b1380` | [Slice 8 report](../report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md) |

Gate repair 1 integrated through [PR #182](https://forge.adrianpk.com/adrianpk/rev/pulls/182)
at `16debf9b98acdac1581a203726041b61ac036774`. Final policy receipt
`VBNOTDVN3ENDHZIBSSQUQ4ERCJ` passed formatting, 36 gutter tests, 74 file tests
and whitespace validation on that exact clean head. The controller recorded
its merge in Acta as `01M68JWDE5DYK7H87XWWFGA9R8`. The repair replaces a
now-valid directory target with invalid UTF-8 for failed-file coverage, retaining
all view/peer/source/history assertions. The prior gate failure at
`90712acf1ba4dc4bb8db091b87285f0929897039` remains identifiable in the
[Slice 8 report](../report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md);
the final exact-candidate pass is recorded below.

Gate repair 2 integrated through [PR #183](https://forge.adrianpk.com/adrianpk/rev/pulls/183)
at `ccfc383bf301425ebb21cd95bb5fb6de32c3e0cc`. Final policy receipt
`XSCA366D6QCOO6S3NT7UIYPBPI` passed formatting, 30 syntax, 26 wrapping, 68 tree
and 41 window tests (165 total), and whitespace validation on that exact clean
head. The controller recorded its merge in Acta as `01M68KC86KT18T7ZKR7F4R1AR7`.
The two horizontal syntax fixtures explicitly select the supported unwrapped
alternative while retaining their clipping, source-row and resource-bound
assertions. The prior gate failure at `ef210e1aef98b4e9d6a907a46767c90f8d2d3ca1`
remains identifiable in the [Slice 8 report](../report/slices/rev-wrap-navigation/slice-8-interaction-acceptance.md);
the final exact-candidate pass is recorded below.

## Delivery acceptance

- [x] Factory word-aware wrap and Emacs visual-line commands work without configuration.
- [x] Source bytes, logical numbering, history, selection and Org semantics survive wrapping.
- [x] CLI/find-file accept files or directories with correct root/visibility precedence.
- [x] First tree presentation exposes immediate entries without repeated state resets.
- [x] Tab completes the visibly selected path without accepting it.
- [x] Click selects the real tree entry and follows the configured native activation policy.
- [x] All five tickets are solved with implementation/PR evidence and valid final links.
- [x] Core guidance, supporting references, samples and Unreleased match delivered behavior.
- [x] All eight configured reports exist and accurately identify focused validation.
- [x] Controller passes the pinned full gate on one exact clean integrated dev candidate.
- [x] Metadata-only closure completes plan/tracker/specs/models/index/reports within allowed paths.

Full-gate preparation verified Rust/Cargo 1.98, rustfmt, Clippy, Bash and the
installed Linux PTY/core utilities, writable owned temporary storage and PTY
device access. `bash -n scripts/check-wrap-navigation.sh` and
`cargo metadata --locked --offline --no-deps --format-version 1` passed.
These prerequisites were rechecked after gate repair 2 integrated.
The joint gate owns fixture creation and cleanup; no persistent fixture or
service is needed. Preparation did not execute the full gate; the controller
subsequently passed it at the immutable candidate recorded below.

## Final gate and closure

The controller passed the pinned full gate on clean integrated candidate
`6b2afd4e07f269c979cd26faf60b6863314e69e9`; immutable receipt
`F7OQU2CWIOPB3VW234VVECCQNG` records this exact command:

```sh
cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-wrap-navigation.sh && cargo run --locked --quiet --example documentation-check && git diff --check
```

All-target testing passed 1,800 tests with zero failures. The native joint
journey passed with built-in, packaged and compiled settings; the Linux PTY
journey completed in 448 ms with 70,025 output bytes, exact saved files, idle
silence and restored terminal attributes. All five long-line probes passed;
maximum measured native step was 1.645 ms, resize 5.837 ms and queued reversal
12.582 ms. These are measurements on this candidate, not latency guarantees.
Documentation passed 305 commands, 180 configuration entries, 58 pages and 331
local links/anchors. No owned persistent fixture remains to stop.

All eight reports are delivered. The selected specification/model metadata is
Completed and the five-topic index records delivery while preserving the ten
remaining proposals. Closure changes only configured operational documentation;
the tested source, fixtures, user guidance, template and Unreleased are intact.
Earlier slice checks and both failed-gate/repair records retain their own hashes.

## Boundaries

The initial planning baseline claimed no implementation or measurements. The
records above identify the completed implementation, integration and validation.
The selected gate and branch/report identities are immutable after startup;
follow native recovery rather than rewriting configuration or checkpoints.
