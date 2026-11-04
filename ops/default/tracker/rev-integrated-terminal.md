# Integrated Terminal Tracker

Status: Approved
Date: 2026-11-03
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../plan/rev-integrated-terminal.md)
Spec: [Integrated Terminal Sessions](../spec/editor-integrated-terminal.md)
Model: [Integrated Terminal Model](../spec/editor-integrated-terminal-model.md)
Approved: owner, 2026-11-03; explicit request to start the approved terminal delivery through Looper
Active slice: None; all eight slices integrated, combined controller gate pending
Progress owner: Looper checkpoint, canonical PRs and exact-candidate validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Native engine and PTY ownership | `feat/rev-integrated-terminal-01-engine` | `feat(terminal): own bounded Rust terminal sessions` | `ops/default/report/slices/rev-integrated-terminal/slice-1-engine.md` |
| Slice 2 | Lower panel and native commands | `feat/rev-integrated-terminal-02-panel` | `feat(terminal): open interactive shells in the lower panel` | `ops/default/report/slices/rev-integrated-terminal/slice-2-panel.md` |
| Slice 3 | Buffer placement and multiple sessions | `feat/rev-integrated-terminal-03-placement` | `feat(terminal): move live sessions between panel and buffers` | `ops/default/report/slices/rev-integrated-terminal/slice-3-placement.md` |
| Slice 4 | Live terminal interaction | `feat/rev-integrated-terminal-04-interaction` | `feat(terminal): route native input and render interactive tools` | `ops/default/report/slices/rev-integrated-terminal/slice-4-interaction.md` |
| Slice 5 | Retained output and transfer | `feat/rev-integrated-terminal-05-copy` | `feat(terminal): navigate and copy retained terminal output` | `ops/default/report/slices/rev-integrated-terminal/slice-5-copy.md` |
| Slice 6 | Session lifecycle and failure containment | `feat/rev-integrated-terminal-06-lifecycle` | `feat(terminal): preserve sessions and confirm owned shutdown` | `ops/default/report/slices/rev-integrated-terminal/slice-6-lifecycle.md` |
| Slice 7 | Native and PTY acceptance | `feat/rev-integrated-terminal-07-acceptance` | `test(terminal): verify shells tools and bounded session resources` | `ops/default/report/slices/rev-integrated-terminal/slice-7-acceptance.md` |
| Slice 8 | User guidance and joint integration | `feat/rev-integrated-terminal-08-guidance` | `docs(terminal): explain integrated terminal workflows` | `ops/default/report/slices/rev-integrated-terminal/slice-8-guidance.md` |

## Tasks and integration evidence

## Slice 1: Native engine and PTY ownership

Status: delivered
Branch: `feat/rev-integrated-terminal-01-engine`
Expected PR title: `feat(terminal): own bounded Rust terminal sessions`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-1-engine.md`
PR: `#248`

- [x] T1.1: Add the pinned minimal engine and bounded native session/PTY/process owner; establish typed identities, finite queues, grid/extra accounting, readiness and cleanup without exposing command stubs.
- [x] T1.2: Exercise a real shell through a PTY, parsed text/colors/cursor/alternate screen, input, resize, child exit, invalid launch and cleanup; introduce tests/integrated_terminal.rs with meaningful shared fixtures.
- [x] T1.3: Record dependency scope, baseline and candidate release size, allocation sketch and startup/idle observations; preserve the host terminal and existing one-shot process owners.

Validation: controller receipt `VYYVEE7ASRQGTWATZGD3VZMHUX` passed the exact five configured commands on `4aeb2c0659d7b8c638e979bd40cfc168e97ee2eb`, including eight native PTY tests. Focused source receipt `INCMCOMKBAQVJWLATBSO74RKV2` passed on the same head.
Integration: canonical PR #248 merged at `4aeb2c0659d7b8c638e979bd40cfc168e97ee2eb`; local dev and origin/dev match the validated candidate.

Retained incident 1: the operator authorized a minimal same-version source
override for the dependency's failed-spawn SIGCHLD registration leak. The
override preserves version, license, bounds and assertions; the real-PTY
missing-interpreter regression now preserves its baseline descriptor count.
Registry caches are unchanged. The controller retains final candidate validation.

Task evidence: T1.1 `e1e7d7f1`; T1.2 `4fe473a5` passed all eight native PTY
tests, including the unchanged missing-interpreter descriptor assertion. T1.3
records 27 all-target dependency additions, nine Linux additions, a release
increase from 21576728 to 21698848 bytes, the cell/extra allocation sketch and
local 5 ms startup/zero idle CPU-tick observations in the Slice 1 report.

## Slice 2: Lower panel and native commands

Status: delivered
Branch: `feat/rev-integrated-terminal-02-panel`
Expected PR title: `feat(terminal): open interactive shells in the lower panel`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-2-panel.md`
PR: `#249`

- [x] T2.1: Admit the complete [terminal] preference catalog, defaults, sample and candidate validation; retain structured shell argv, startup directory context and no startup process launch.
- [x] T2.2: Open a real interactive terminal in the default lower panel, reuse its stable default session, show/hide with remembered size/focus and render cells through native display ownership.
- [x] T2.3: Admit typed terminal-backed buffers through native navigation and command catalogs; guard source-only file/edit/completion/LSP/Org/AI actions without changing ordinary source buffers, and update Unreleased when the user-visible path works.

Task evidence: T2.1 `47660f76771a` admits all seven startup preferences; T2.2 `c81964d0fa2f` wires native panel commands, cells, wake delivery and typed resource navigation. T2.3 `7d6e91c04df7` completes source admission, bounded cell projection, effective bindings, Unreleased and native fixtures.

Validation: controller receipt `L6AILPA6B5PXJKN455OFAC7EHO` passed the exact five required commands on `6d5d674655998a92e163be7be8248085f24bb287`, including 348 runtime cases. Focused source receipt `GOC2OMTOBVXWI2ONL5BCB7BVKL` passed sixteen files. Owner-approved async cleanup is recorded in `6b2995b519b7` and `6d5d67465599`.
Integration: canonical PR #249 merged into dev at `6d5d674655998a92e163be7be8248085f24bb287`; Slice 2 of 8 completed.

## Slice 3: Buffer placement and multiple sessions

Status: delivered
Branch: `feat/rev-integrated-terminal-03-placement`
Expected PR title: `feat(terminal): move live sessions between panel and buffers`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-3-placement.md`
PR: `#250`

- [x] T3.1: Implement direct terminal-new-buffer and bidirectional move commands; retain process/emulator/buffer identity, working directory, alternate-screen state and output, with atomic geometry failure.
- [x] T3.2: Create multiple sessions only on explicit invocation; implement independent names, selection/list/rename and one selected lower-panel session, retaining hidden sessions and the instance-global default.
- [x] T3.3: Support ordinary pane views with one resize authority and independent copy/viewport state, checked focus changes, tiny-screen recovery, cwd entry commands and other-pane selection; verify no duplicate child on move/toggle.

Task evidence: T3.1 `e039a166383d`; T3.2 `5b3c81588ab0`; T3.3 `c861096dfe81` with correction `6a4f108e2343` completes checked native pane focus, per-attachment viewport state, directory entry and other-pane candidates.

Validation: controller receipt `TTW6SAT24SJSO2KXS5M5RYX2RB` passed the five ordered Slice 3 commands on `d82d9e36628b601efc16cb0ab5e1ca6c9ebfa532`, including 84 runtime cases. Focused receipt `XHWCORNSXMVYXK35HLTMM4SXKM` passed fifteen source files on the same head.
Integration: canonical PR #250 merged into dev at `d82d9e36628b601efc16cb0ab5e1ca6c9ebfa532`; Slice 3 of 8 completed.

## Slice 4: Live terminal interaction

Status: delivered
Branch: `feat/rev-integrated-terminal-04-interaction`
Expected PR title: `feat(terminal): route native input and render interactive tools`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-4-interaction.md`
PR: `#251`

- [x] T4.1: Route printable keys, child control keys, Return/Tab/deletion/arrows/paging, cursor shape and application modes in both startup profiles; preserve host recovery/prefix commands, literal-next and contextual transient cancellation.
- [x] T4.2: Render real shell and full-screen TUI screen/style/cursor/alternate-screen/resize changes without source repaint glitches, raw escape passthrough, focus stealing or a busy idle loop.
- [x] T4.3: Route child-enabled mouse/wheel input and native local selection/scrolling, Shift bypass and explicit validated link activation; deny child clipboard/host effects and keep printable colon/Escape under live child authority.

Task evidence: T4.1 `5f3ae4f4d345` and cancellation correction `9671a9d28f0d`; the raw-PTY cases pass in both profiles and input modes. Dispatch correction: `580bd8538d2a`. T4.2 `7da3f48dff23` resolves child palette colors and adds controlled full-screen PTY coverage.
Focused receipts: `NZ6I2BQ7YVM7IJBMWFHBBJYCIE` passed T4.1 input and six source files on `580bd8538d2a7b566d37d07e54803f0d7f266d59`; `SCGY7G3XHNXBZ74CDIPCGGQK5Q` passed both interaction scenarios and renderer source checks on `7da3f48dff23ff0470fc0fd1d97b359d4215c9d0`.

T4.3 task evidence: `a819a54562a6`, with fixture correction `ef72e033fb2f`.
Controller receipt `2MF3LO7YJWYNHK5YRRN7T5RZK7` passed four real-PTY pointer/link scenarios, six host mouse restoration cases, five configured-viewer regressions, seven source files and whitespace on `ef72e033fb2ffea99d62280631cb546c83154c4a`.
Validation: controller receipt `YORJCCZZS3WYURZ75HWQNZSX6D` passed the six ordered Slice 4 commands on `ef59e61d3a152d52faf23e8e05288ff02f84f9ba`, including 148 runtime cases and documentation.
Integration: canonical PR #251 merged into dev at `ef59e61d3a152d52faf23e8e05288ff02f84f9ba`; Slice 4 of 8 completed.

## Slice 5: Retained output and transfer

Status: delivered
Branch: `feat/rev-integrated-terminal-05-copy`
Expected PR title: `feat(terminal): navigate and copy retained terminal output`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-5-copy.md`
PR: `#252`

- [x] T5.1: Provide bounded scrollback, stable per-view copy mode, selection/search, copy with retained viewport, tail-follow suspension/resumption and the specified Return/C-g exits while output continues.
- [x] T5.2: Implement native terminal yank/yank-pop with checked replaceability and bracketed paste, separate clear-screen/clear-scrollback operations, line/cursor navigation and explicit send/undo operations.
- [x] T5.3: Complete prompt/error/link navigation using actual admitted evidence such as OSC 133 and recognized locations; report unavailable evidence honestly. Complete the finite terminal-send-* command inventory without placeholders.

T5.1 is complete. Receipt `NBFYD5DMKMI6RM4NRLBO7KFQLI` passed all 21 panel
PTY tests and sixteen source files at `976b0dcb5ade370b536dbfb22a7b8a7b6e3fa6b4`.
T5.2/T5.3 implementation and focused evidence are recorded below and in the report.

Validation: controller receipt `DA64MDHVV5YRRY3SUYKXH3JKAZ` passed all five ordered Slice 5 commands on `fa2ca69da2384e4f45257ce449dfb3d35ccb8100`, including 110 runtime tests and both 474-command inventories.
Integration: canonical PR #252 merged into dev at `fa2ca69da2384e4f45257ce449dfb3d35ccb8100`; Slice 5 of 8 completed.

T5.2 evidence: implementation `61a16c018292` and cursor-owner refactor
`cc1a07055a1f`; queue fixture correction `a892e8e60ad6`; checked policy/catalog
`a5daf0519d9f`. Receipt `MCZY75A524KKYSJWPA7Z35TKP4` passed 34 real-PTY tests,
twelve source-quality files, both 470-command documentation inventories and diff
checking on the exact clean T5.2 head. Final validation is recorded above.

T5.3 evidence: `e37ec1eef7d0` and `1b394b3eb6b2` implement bounded actual
prompt reports, frozen URI targets and checked compiler-location visits. The
printf-data correction `77416996db98` preserves the exact URI assertion. Receipt
`CMCFOMPGVG7Z2XCBTMMBJSNSU6` passed prompt and evidence/capacity tests on
`c9fbc6df3c1c674a3e3261147485547de20cca9b`; both documentation inventories now
contain all 474 commands. Final exact-PR validation passed as recorded above.

## Slice 6: Session lifecycle and failure containment

Status: delivered
Branch: `feat/rev-integrated-terminal-06-lifecycle`
Expected PR title: `feat(terminal): preserve sessions and confirm owned shutdown`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-6-lifecycle.md`
PR: `#253`

- [x] T6.1: Keep visibility separate from process lifetime, retain actual exited status/output, replace an exited default without erasing its buffer and release capacity only after explicit closure.
- [x] T6.2: Confirm live-buffer termination and editor-wide session shutdown once, preserve declined choices, own process-group cleanup/reaping and handle failure without losing unsaved source or unrelated sessions.
- [x] T6.3: Exercise queue/grid/extra/session admission, output floods, blocked input, incomplete controls, rapid resize/close, failed relocations and native shutdown/error paths under finite deadlines and fair readiness service.

T6.1: `ea41ea80cf39` passes receipt `4YZKUDYB6PWBIR4MIKAPNVDJTR`: readable
actual live/copy status, final-view detachment, exited-buffer/default retention
and capacity release only after closure, with both profiles and three source files.

T6.2: `fbaff04c09f8` passes receipt `PPD4GYIBWP77SFKH7JFLMK3L4K`: four
native lifecycle scenarios across both profiles, actual owned background-group
cleanup, source revision guards, and the unresolved-save continuation regression.

T6.3: `1b0fb2569df0` passes receipt `OSU7364GZSOJZEX53J6V4ZMEZP`: eight
resource scenarios plus native extras-failure containment in both profiles;
finite flood fairness, blocked input, exact FD checks and owned cleanup retained.

Validation: final receipt `DHHPXEKLN57H44VO5X5ELFNGQL` passed all six configured commands at `e48e4efdc6305167e6aa38753217e0c952923396`: 201 integration tests, seven process-owner tests, format, all-target Clippy, documentation and whitespace.
Integration: canonical [PR #253](https://forge.adrianpk.com/adrianpk/rev/pulls/253) merged into dev unchanged at `e48e4efdc6305167e6aa38753217e0c952923396`; checkpoint records merge activity.

## Slice 7: Native and PTY acceptance

Status: delivered
Branch: `feat/rev-integrated-terminal-07-acceptance`
Expected PR title: `test(terminal): verify shells tools and bounded session resources`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-7-acceptance.md`
PR: [#254](https://forge.adrianpk.com/adrianpk/rev/pulls/254)

- [x] T7.1: Add examples/integrated-terminal-acceptance.rs and scripts/check-integrated-terminal.sh using existing Rust/Linux PTY support and isolated fixtures; exercise real shells and installed lazygit in a disposable local repository without network or user Git state.
- [x] T7.2: Verify both profiles, multiple sessions, panel/main-buffer movement, continued child state, alternate screen, focus/literal-next, mouse, copy/paste, protected source admission and complete host restoration through real terminal journeys.
- [x] T7.3: Measure binary delta, idle/active CPU, RSS, bounded retained output and queue/flood behavior, process/FD cleanup and fairness. Add focused source-quality coverage, share fixture setup and record real observations instead of guessed performance claims.

T7.1: `8f0a16c510ce` passes focused receipt `PGKUQXPYN44HYS2BXYNCZKH7VC`: actual interactive Bash/lazygit and identity-preserving relocation in both profiles, with source quality and whitespace.

T7.2/T7.3: `dee525cecdd8` passed focused receipt
`TF7COIF6DUMGCENUZH4237HRMM`: both native/host profiles, real tool relocation,
exact literal/mouse/paste bytes, normal/error restoration, declined/confirmed
closure and resource flood/fairness. Fresh active-shell prompt generations and
actual tool exit zero replace ambiguous restored-screen readiness. Interrupted
zero-time exit polls defer within the existing service bound (`4e9bbcbdd69b`).
Shared shell PTY regression passed in `7MUL6VQNUYUSO4W434E6KGL6NL` at
`0555c3af914d`. The report records measured binary, CPU, RSS, queue, stream and
FD/thread results and the explicit host-key transport boundary.

Validation: final canonical receipt `C4HWXE56WCZ757B54BGMDKQ333` passed all six configured commands at `47bc4911ee8a6bd1886d153ddb87d8e5b8b8c6d2`: format, all-target Clippy, 45 tests, complete native/PTY/resource acceptance, documentation and whitespace.
Integration: canonical PR #254 merged into dev unchanged at `47bc4911ee8a6bd1886d153ddb87d8e5b8b8c6d2`; checkpoint records merge activity.

## Slice 8: User guidance and joint integration

Status: delivered
Branch: `feat/rev-integrated-terminal-08-guidance`
Expected PR title: `docs(terminal): explain integrated terminal workflows`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-8-guidance.md`
PR: [#255](https://forge.adrianpk.com/adrianpk/rev/pulls/255)

- [x] T8.1: Add a short ordinary terminal chapter to the editor learning path, a task guide and exact command/configuration/input/lifecycle reference; explain panel versus buffer, moving a live lazygit session, multiple terminals and returning to the editor with small visual examples.
- [x] T8.2: Keep Markdown the source of truth for Spore, update indexes/catalog checks and both learning-path navigation, document shell-command/filter distinction and finalize user-facing Unreleased notes without a version bump.
- [x] T8.3: Reconcile accepted baseline/model clauses and all requirement evidence, run the focused slice checks and prepare clean dev for the controller-owned combined gate; do not claim completion before that receipt.

T8.1: `ef908f3a7155` adds the terminal chapter, task guide, exact reference and
small placement diagrams. T8.2: `ae3fb57269c5` connects Markdown/Spore navigation,
checks live terminal catalogs and finalizes the terminal Unreleased entry.
T8.3: `bcbdbbab9114` reconciles baseline/model typed ownership and TRM01-TRM11
evidence without changing accepted requirement/scenario text.

Validation: focused receipt `Y2WNEWSV2PFZYVWAYCQOPNJ7A7` passed documentation,
Spore strict links/render and whitespace at `10dbde18199a`. The unchanged path
check caught a Next: format mismatch, corrected in that candidate.
Final canonical receipt `IT5NYQHLV2YQN43Q46TFNMIGTD` passed all six configured
commands at `56862f0225b718b82c57d1c0ccd3c24fa97e1ab9`: format, all-target
Clippy, 361 integration tests, documentation, local guide generation and whitespace.
Integration: canonical PR #255 merged into dev unchanged at
`56862f0225b718b82c57d1c0ccd3c24fa97e1ab9`; Slice 8 of 8 completed.
The checkpoint records merge activity for all eight slices.

## Final gate and closure

All eight canonical PRs #248-#255 are merged unchanged into dev and all slice
reports are delivered. Combined acceptance and delivery-set closure remain pending.

Gate prerequisites reverified after repair integration on 2026-11-04: Bash
5.3.15, Git 2.55.0, lazygit
0.64.1, Rust/Cargo 1.98.0, rustfmt, Clippy, Spore and the fixture utilities are
available. Locked offline Linux metadata resolves 91 available package manifests.
Linux procfs and PTY allocation work; the primary target filesystem has over
108 GiB free against the fixed 1 GiB prerequisite. No primary build or validator
is active. The acceptance script creates its isolated HOME/XDG, local Git
repository and finite PTY fixtures when invoked; the guide builder owns its
temporary projection and target/userguide output. No persistent fixture is needed.

The unchanged plan Full gate failed at
`c22102280c1cb514fc1eca1e663c126e9a12e063`, receipt
`DIMJP2WKAA5IZ4ZYCRY75TCV4W`, during the non-modal real-PTY lazygit relocation
journey's second M-x prompt. Earlier native batches passed. Successful combined
acceptance and restricted documentation closure remain controller-owned.

Gate repair 1 uses `fix/rev-integrated-terminal-validation-1`, the pinned title
`fix(terminal): repair approved integrated terminal acceptance`, and the existing
Slice 8 report. Focused receipt `TBCKOI6QAOOQ2H3X5H5U2O3XZL` passed unchanged
PTY journeys on the failed base. A controlled PTY probe then reproduced signal
readiness consuming the Mio edge while the eight-byte Alt-x event remains unread.
The owner approved incident 3 and the controller granted the exact source archive,
readiness patch and vendor README. The same-version/license override retains
unconsumed readiness tokens and checks queued TTY bytes before another read.
All dependency versions and graph edges are unchanged. The new real-PTY
`readiness::signal_keys` regression fails on the original source and passes
signal-first, input-first and multi-read burst cases with the correction.
Canonical repair PR: [#256](https://forge.adrianpk.com/adrianpk/rev/pulls/256).
Focused receipt `O7FTJPLHVDD6O4KVWBHSORX3IN` passed format, the regression and
whitespace on `27fff3c81e60a46b78def641ad47b679af88097c`.
Final canonical receipt `BCMQANMXVO5IJO5KC7TXAZVSEM` passed all five configured
repair commands at `6390608f045e36d24e3510763054777067407a6f`, including 461 tests
and documentation. Focused receipt `EZFW2VPLAEB6I5MVBWYNLAZX6Y` passed both
profiles' real-PTY journeys and normal/error restoration on the same head.

Integration: canonical PR #256 merged into dev unchanged at
`6390608f045e36d24e3510763054777067407a6f`; checkpoint merge activity is recorded.
All slice reports and this repair are delivered. The unchanged combined gate
has not rerun after repair integration. The controller retains that gate and
final delivery-set closure; slice-local evidence does not mark the set Completed.
