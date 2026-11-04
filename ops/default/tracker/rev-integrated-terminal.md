# Integrated Terminal Tracker

Status: Approved
Date: 2026-11-03
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../plan/rev-integrated-terminal.md)
Spec: [Integrated Terminal Sessions](../spec/editor-integrated-terminal.md)
Model: [Integrated Terminal Model](../spec/editor-integrated-terminal-model.md)
Approved: owner, 2026-11-03; explicit request to start the approved terminal delivery through Looper
Active slice: Slice 4 - Live terminal interaction
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

Status: active
Branch: `feat/rev-integrated-terminal-04-interaction`
Expected PR title: `feat(terminal): route native input and render interactive tools`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-4-interaction.md`
PR: pending

- [x] T4.1: Route printable keys, child control keys, Return/Tab/deletion/arrows/paging, cursor shape and application modes in both startup profiles; preserve host recovery/prefix commands, literal-next and contextual transient cancellation.
- [ ] T4.2: Render real shell and full-screen TUI screen/style/cursor/alternate-screen/resize changes without source repaint glitches, raw escape passthrough, focus stealing or a busy idle loop.
- [ ] T4.3: Route child-enabled mouse/wheel input and native local selection/scrolling, Shift bypass and explicit validated link activation; deny child clipboard/host effects and keep printable colon/Escape under live child authority.

Task evidence: T4.1 `5f3ae4f4d345` and cancellation correction `9671a9d28f0d`; the raw-PTY cases pass in both profiles and input modes.

Validation: pending; use the exact ordered Slice 4 commands from the plan and frozen controller configuration.
Integration: pending canonical PR and controller verification.

## Slice 5: Retained output and transfer

Status: pending
Branch: `feat/rev-integrated-terminal-05-copy`
Expected PR title: `feat(terminal): navigate and copy retained terminal output`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-5-copy.md`
PR: pending

- [ ] T5.1: Provide bounded scrollback, stable per-view copy mode, selection/search, copy with retained viewport, tail-follow suspension/resumption and the specified Return/C-g exits while output continues.
- [ ] T5.2: Implement native terminal yank/yank-pop with checked replaceability and bracketed paste, separate clear-screen/clear-scrollback operations, line/cursor navigation and explicit send/undo operations.
- [ ] T5.3: Complete prompt/error/link navigation using actual admitted evidence such as OSC 133 and recognized locations; report unavailable evidence honestly. Complete the finite terminal-send-* command inventory without placeholders.

Validation: pending; use the exact ordered Slice 5 commands from the plan and frozen controller configuration.
Integration: pending canonical PR and controller verification.

## Slice 6: Session lifecycle and failure containment

Status: pending
Branch: `feat/rev-integrated-terminal-06-lifecycle`
Expected PR title: `feat(terminal): preserve sessions and confirm owned shutdown`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-6-lifecycle.md`
PR: pending

- [ ] T6.1: Keep visibility separate from process lifetime, retain actual exited status/output, replace an exited default without erasing its buffer and release capacity only after explicit closure.
- [ ] T6.2: Confirm live-buffer termination and editor-wide session shutdown once, preserve declined choices, own process-group cleanup/reaping and handle failure without losing unsaved source or unrelated sessions.
- [ ] T6.3: Exercise queue/grid/extra/session admission, output floods, blocked input, incomplete controls, rapid resize/close, failed relocations and native shutdown/error paths under finite deadlines and fair readiness service.

Validation: pending; use the exact ordered Slice 6 commands from the plan and frozen controller configuration.
Integration: pending canonical PR and controller verification.

## Slice 7: Native and PTY acceptance

Status: pending
Branch: `feat/rev-integrated-terminal-07-acceptance`
Expected PR title: `test(terminal): verify shells tools and bounded session resources`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-7-acceptance.md`
PR: pending

- [ ] T7.1: Add examples/integrated-terminal-acceptance.rs and scripts/check-integrated-terminal.sh using existing Rust/Linux PTY support and isolated fixtures; exercise real shells and installed lazygit in a disposable local repository without network or user Git state.
- [ ] T7.2: Verify both profiles, multiple sessions, panel/main-buffer movement, continued child state, alternate screen, focus/literal-next, mouse, copy/paste, protected source admission and complete host restoration through real terminal journeys.
- [ ] T7.3: Measure binary delta, idle/active CPU, RSS, bounded retained output and queue/flood behavior, process/FD cleanup and fairness. Add focused source-quality coverage, share fixture setup and record real observations instead of guessed performance claims.

Validation: pending; use the exact ordered Slice 7 commands from the plan and frozen controller configuration.
Integration: pending canonical PR and controller verification.

## Slice 8: User guidance and joint integration

Status: pending
Branch: `feat/rev-integrated-terminal-08-guidance`
Expected PR title: `docs(terminal): explain integrated terminal workflows`
Report: `ops/default/report/slices/rev-integrated-terminal/slice-8-guidance.md`
PR: pending

- [ ] T8.1: Add a short ordinary terminal chapter to the editor learning path, a task guide and exact command/configuration/input/lifecycle reference; explain panel versus buffer, moving a live lazygit session, multiple terminals and returning to the editor with small visual examples.
- [ ] T8.2: Keep Markdown the source of truth for Spore, update indexes/catalog checks and both learning-path navigation, document shell-command/filter distinction and finalize user-facing Unreleased notes without a version bump.
- [ ] T8.3: Reconcile accepted baseline/model clauses and all requirement evidence, run the focused slice checks and prepare clean dev for the controller-owned combined gate; do not claim completion before that receipt.

Validation: pending; use the exact ordered Slice 8 commands from the plan and frozen controller configuration.
Integration: pending canonical PR and controller verification.

## Final gate and closure

Pending eight canonical integrations, the unchanged plan Full gate, an exact
clean-dev receipt, and the restricted documentation closure. The checkpoint is
the execution owner; this tracker records human-readable evidence rather than
creating a second controller. Do not mark terminal delivery Completed from
slice-local checks or move main, tags, mirrors or public sites in this run.
