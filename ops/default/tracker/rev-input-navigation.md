# Input, Navigation and Presentation Tracker

Status: Approved
Approved: product owner, 2026-10-27; complete all seven selected topics in one autonomous Looper cycle after reconciling their contracts and slicing. Startup and canonical dev integration requested.
Date: 2026-10-27
Delivery set: rev-input-navigation
Slice strategy: layered
Active slice: Slice 7 - wire gutter settings and native commands
Full gate: `scripts/check-all.sh`
Planning baseline: `9e24fa3f7e60ab8a4ff60e3589662631f4a94bae`
Spec: [Line Navigation](../spec/editor-line-navigation.md), [Whitespace Conversion](../spec/editor-whitespace-conversion.md), [Gutter](../spec/editor-gutter.md), [Pointer](../spec/editor-pointer-interaction.md), [Configuration Bootstrap](../spec/editor-configuration-bootstrap.md), [Status Layout](../spec/editor-status-layout.md)
Model: [Gutter Model](../spec/editor-gutter-model.md), [Existing Configuration Model](../spec/editor-configuration-model.md)
Plan: [Input and Navigation Delivery](../plan/rev-input-navigation.md)

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | restore responsive vertical navigation | `fix/rev-input-01-navigation-responsiveness` | `fix(slice-1): restore responsive vertical navigation` | `ops/default/report/slices/rev-input-navigation/slice-1-navigation-responsiveness.md` |
| Slice 2 | add Emacs-style line navigation | `feat/rev-input-02-goto-line` | `feat(slice-2): add Emacs-style line navigation` | `ops/default/report/slices/rev-input-navigation/slice-2-goto-line.md` |
| Slice 3 | add explicit tabify and untabify | `feat/rev-input-03-whitespace-conversion` | `feat(slice-3): add explicit tabify and untabify` | `ops/default/report/slices/rev-input-navigation/slice-3-whitespace-conversion.md` |
| Slice 4 | define native gutter view policy | `refactor/rev-input-04-gutter-policy` | `refactor(slice-4): define native gutter view policy` | `ops/default/report/slices/rev-input-navigation/slice-4-gutter-policy.md` |
| Slice 5 | render absolute logical line numbers | `feat/rev-input-05-absolute-gutter` | `feat(slice-5): render absolute logical line numbers` | `ops/default/report/slices/rev-input-navigation/slice-5-absolute-gutter.md` |
| Slice 6 | render independent relative numbering | `feat/rev-input-06-relative-gutter` | `feat(slice-6): render independent relative numbering` | `ops/default/report/slices/rev-input-navigation/slice-6-relative-gutter.md` |
| Slice 7 | wire gutter settings and native commands | `feat/rev-input-07-gutter-controls` | `feat(slice-7): wire gutter settings and native commands` | `ops/default/report/slices/rev-input-navigation/slice-7-gutter-controls.md` |
| Slice 8 | preserve gutter geometry across editor surfaces | `fix/rev-input-08-gutter-geometry` | `fix(slice-8): preserve gutter geometry across editor surfaces` | `ops/default/report/slices/rev-input-navigation/slice-8-gutter-geometry.md` |
| Slice 9 | align pane status into left and right groups | `feat/rev-input-09-status-layout` | `feat(slice-9): align pane status into left and right groups` | `ops/default/report/slices/rev-input-navigation/slice-9-status-layout.md` |
| Slice 10 | own native pointer reporting and input state | `feat/rev-input-10-pointer-lifetime` | `feat(slice-10): own native pointer reporting and input state` | `ops/default/report/slices/rev-input-navigation/slice-10-pointer-lifetime.md` |
| Slice 11 | focus panes and place point by pointer | `feat/rev-input-11-pointer-hit-focus` | `feat(slice-11): focus panes and place point by pointer` | `ops/default/report/slices/rev-input-navigation/slice-11-pointer-hit-focus.md` |
| Slice 12 | scroll the pointed pane without changing focus | `feat/rev-input-12-pointer-scroll` | `feat(slice-12): scroll the pointed pane without changing focus` | `ops/default/report/slices/rev-input-navigation/slice-12-pointer-scroll.md` |
| Slice 13 | select canonical text by dragging | `feat/rev-input-13-pointer-selection` | `feat(slice-13): select canonical text by dragging` | `ops/default/report/slices/rev-input-navigation/slice-13-pointer-selection.md` |
| Slice 14 | extend pointer selection with bounded autoscroll | `feat/rev-input-14-drag-autoscroll` | `feat(slice-14): extend pointer selection with bounded autoscroll` | `ops/default/report/slices/rev-input-navigation/slice-14-drag-autoscroll.md` |
| Slice 15 | activate native mouse defaults and runtime control | `feat/rev-input-15-pointer-controls` | `feat(slice-15): activate native mouse defaults and runtime control` | `ops/default/report/slices/rev-input-navigation/slice-15-pointer-controls.md` |
| Slice 16 | package the complete startup configuration template | `feat/rev-input-16-bootstrap-template` | `feat(slice-16): package the complete startup configuration template` | `ops/default/report/slices/rev-input-navigation/slice-16-bootstrap-template.md` |
| Slice 17 | create absent user configuration safely | `feat/rev-input-17-bootstrap-publication` | `feat(slice-17): create absent user configuration safely` | `ops/default/report/slices/rev-input-navigation/slice-17-bootstrap-publication.md` |
| Slice 18 | exercise the complete combined terminal workflow | `test/rev-input-18-combined-terminal` | `test(slice-18): exercise the complete combined terminal workflow` | `ops/default/report/slices/rev-input-navigation/slice-18-combined-terminal.md` |
| Slice 19 | verify bounded input gutter and pointer work | `perf/rev-input-19-bounded-interaction` | `perf(slice-19): verify bounded input gutter and pointer work` | `ops/default/report/slices/rev-input-navigation/slice-19-bounded-interaction.md` |
| Slice 20 | reconcile combined delivery acceptance and references | `test/rev-input-20-delivery-acceptance` | `test(slice-20): reconcile combined delivery acceptance and references` | `ops/default/report/slices/rev-input-navigation/slice-20-delivery-acceptance.md` |

## Task state

All unchecked tasks remain work to do; checked tasks have evidence below.
Use one Conventional Commit per task when practical, retaining this repository's existing (Tn.m) suffix convention without exporting it to other projects.

### Slice 1: restore responsive vertical navigation

Contracts: R07 R16.
Details and exact checks: [Slice 1](../plan/rev-input-navigation.md#slice-1-restore-responsive-vertical-navigation).

- [x] T1.1: Reproduce the reported repeat-input tail through the existing terminal/event/render path; record the tested build, fixture, input rate and measured cause before changing scheduling.
- [x] T1.2: Correct the demonstrated bottleneck with finite event/render work; preserve every ordered text/command transition, Escape deadlines, C-g, history and both arrow directions. Do not discard input merely to hide a backlog.
- [x] T1.3: Add a meaningful regression, check ordinary/large files and boundary reversal, and record exact focused evidence and the canonical slice report.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 1 of 20 completed. The [canonical report](../report/slices/rev-input-navigation/slice-1-navigation-responsiveness.md) records baseline fixtures/rates, the rendering backlog and the corrected native/PTY regressions. T1.1: `db32befb12db284ed56958d719ac3843c3cb71e5`. T1.2: `6ae33afb14c6e0d6973d2bf9238970708e1b91cf`. T1.3: `ef908a5081e64d8030062405e95e6293ff95e7c3`. Canonical PR: [#81](https://forge.adrianpk.com/adrianpk/rev/pulls/81), targeting dev. Canonical five-command evidence `JAGKIAXYUR5X37CCUFQZDRF2BG` passed at integrated dev commit `44edc37bcb15d5bb9ff1ed2456c5f5381515ad26`; focused evidence `HPKPBCV7JJPONAAG554JFKMZ55` also passed on that candidate. Slice 2 was activated.

### Slice 2: add Emacs-style line navigation

Contracts: NAV01 NAV02.
Details and exact checks: [Slice 2](../plan/rev-input-navigation.md#slice-2-add-emacs-style-line-navigation).

- [x] T2.1: Add the compiled goto-line identity, bounded signed-integer prompt and M-g g / M-g M-g defaults through existing command/keymap owners; preserve C-g and all prior aliases.
- [x] T2.2: Use checked logical-line lookup and native reveal; match start clamping for nonpositive values and EOF clamping beyond the last line. Preserve other views and cancellation state.
- [x] T2.3: Verify native and prompt dispatch, Unicode/CRLF/empty/final-newline boundaries and malformed/overflow input; update command references and record evidence without claiming general numeric-prefix support.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 2 of 20 completed. The [canonical report](../report/slices/rev-input-navigation/slice-2-goto-line.md) records native command/prompt dispatch, numeric and source bounds, independent views and real PTY outcomes. T2.1: `645d5b45c8344e6a0033c0bf6679430235db8343`. T2.2: `4cf7b662a1b8200282800df1e38b0338e6f5969b`. T2.3: `dcd195bbea7ec7a55a2fc5958052664e094ed0a9`. Canonical PR: [#82](https://forge.adrianpk.com/adrianpk/rev/pulls/82), targeting dev. Canonical five-command evidence `JBTSUPG2UDHIZXF55AEKYU5NHQ` and focused evidence `YLCNU3AI5PBBZ2R5BFSZLRKMC6` passed at integrated dev commit `9fdd5ab5c0e66a85c560060c586823f2abbeaacb`. Slice 3 is active.

### Slice 3: add explicit tabify and untabify

Contracts: WSP01 WSP02.
Details and exact checks: [Slice 3](../plan/rev-input-navigation.md#slice-3-add-explicit-tabify-and-untabify).

- [x] T3.1: Implement both native M-x operations on the existing admitted linear region; missing regions give feedback and never silently select the whole document.
- [x] T3.2: Compute tab stops from the effective buffer policy and true starting display column, including Unicode prefixes and mixed whitespace; retain all nonselected bytes and line formats.
- [x] T3.3: Publish one checked edit/undo operation, cover redo/no-op/read-only/shared views and document how to select the intended range; retain source fidelity and record focused evidence. Reconcile any historical open-future-ticket guard against the newly verified conversion delivery instead of deleting the provenance check.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 3 of 20 completed. Canonical PR [#83](https://forge.adrianpk.com/adrianpk/rev/pulls/83) integrated into dev at `bc94816edd96ef245310b892df24fe57edc8589a`. Canonical five-command evidence `ZH4LE3KVA6JWX6JFD3WHREEGRO` and focused evidence `JUHABZU63SQ2BX4KTIRZQHHWLB` passed on that candidate. Slice 4 is active. The [canonical report](../report/slices/rev-input-navigation/slice-3-whitespace-conversion.md) records native conversion, history/fidelity, PTY outcomes and retained provenance checks. T3.1: `e015dcfb6104872696571df7f0e361209774c2f7`. T3.2: `28be1f4d52fd44c50ab92c9d586a4fe54b648d60`. T3.3: `86874577229ca5ec52eb00547f63a62a46d3da9f`.

### Slice 4: define native gutter view policy

Contracts: GUT01 GUT04 GUT11.
Details and exact checks: [Slice 4](../plan/rev-input-navigation.md#slice-4-define-native-gutter-view-policy).

- [x] T4.1: Introduce the bounded native gutter projection/view-policy records using existing buffer, ViewId, line index and panel owners; add the gutter integration-test target.
- [x] T4.2: Model independent visibility/mode/current-line/side overrides, split inheritance, reset and lifecycle without per-line retained number arrays or new persistent stores.
- [x] T4.3: Verify checked width arithmetic, decimal boundaries and view invalidation; keep preparatory policy distinct from advertised operational TOML/commands and record its contracts.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 4 of 20 completed. Canonical PR [#84](https://forge.adrianpk.com/adrianpk/rev/pulls/84) integrated into dev at `db5006cb038a644b4786d015f80d83afeb33daaf`. Canonical five-command evidence `AVX4W4B2M4OCHU3UXSQUUIYFON` and focused evidence `PZDKOF6JKENREIFMCNYTWMY3KP` passed on that candidate. Slice 5 is active. The [canonical report](../report/slices/rev-input-navigation/slice-4-gutter-policy.md) records policy/lifecycle, checked geometry and invalidation evidence. T4.1: `3762cf7f1f26a5a8244f5ce904214d57ef5d645a`. T4.2: `e3364ba61da2cb8fb8fbd8ef295f7079dc87b8ca`. T4.3: `3e83ab48b1d45bbc1b2f1d10630bca2b9dd9586c`.

### Slice 5: render absolute logical line numbers

Contracts: GUT01 GUT03 GUT04 GUT05 GUT10.
Details and exact checks: [Slice 5](../plan/rev-input-navigation.md#slice-5-render-absolute-logical-line-numbers).

- [x] T5.1: Render absolute logical line labels with the accepted minimum digits, padding, separator fallback and semantic palette inside each panel.
- [x] T5.2: Give text one checked origin after number allocation; align cursor, tabs, Unicode, syntax/search/regions and horizontal clipping without prefix-string geometry.
- [x] T5.3: Cover empty/final-empty lines, beyond-EOF padding, safe glyphs, decimal-width changes and actual rendered cells; run display regressions with the new gutter cases.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 5 of 20 completed. Canonical PR [#85](https://forge.adrianpk.com/adrianpk/rev/pulls/85) integrated into dev at `d4707f0381b0a20b4c0558724c321710b3ef8f20`. Canonical five-command evidence `IEKFELNPRVBKZWGWRGSE5HA5SC` and focused evidence `6NOINAR4Q7ZBHZM5UCKOBJGA7J` passed on that candidate. Slice 6 is active. The [canonical report](../report/slices/rev-input-navigation/slice-5-absolute-gutter.md) records native rendered cells, final-origin regressions and owned terminal outcomes. T5.1: `469b7f0a07fea889a24c94cb1f54e8316f84747b`. T5.2: `35c6006d2634764a37059590946d78b67a03cc44`. T5.3: `a90d4e9a12a3a7b6314a20dd058bde9dba04e2aa`.

### Slice 6: render independent relative numbering

Contracts: GUT02 GUT04 GUT11.
Details and exact checks: [Slice 6](../plan/rev-input-navigation.md#slice-6-render-independent-relative-numbering).

- [x] T6.1: Add relative labels from each view's own logical point and absolute/zero/hidden current-line forms; keep digit width stable during origin movement.
- [x] T6.2: Preserve independent points and overrides across shared splits, buffer switches, close and rebind; retain complete logical numbering through edits and history.
- [x] T6.3: Verify both modes, current-line forms and shared-view lifecycle with render-level assertions and record bounded visible-row behavior.
- [x] Record exact clean-head validation, canonical PR, report and verified dev integration.

Slice 6 of 20 completed. Canonical PR [#86](https://forge.adrianpk.com/adrianpk/rev/pulls/86) integrated into dev at `3e1f4b9651fed9306949ca2679c16859a5ed916d`. Canonical evidence `F47M2RWK5WRIAP3PNKT3GJTDUI` and focused evidence `X5QA4V5IQIY5WXOUYBMXEZ2WOP` passed on that candidate. Slice 7 is active. The [canonical report](../report/slices/rev-input-navigation/slice-6-relative-gutter.md) records logical relative labels, all current-line forms and native shared-view lifecycle. T6.1: `6a1d15d697d76622bb3b397b1c3b50def73bced4`. T6.2: `9f491efaf72c48d9770ccfe4060ddd17bfa68c12`. T6.3: `cf4ebcd56703127cfd5cb8c24f1fe91e77c2126e`.

### Slice 7: wire gutter settings and native commands

Contracts: GUT10 GUT12 CFG05.
Details and exact checks: [Slice 7](../plan/rev-input-navigation.md#slice-7-wire-gutter-settings-and-native-commands).

- [x] T7.1: Wire every accepted gutter/model key, palette role, type/bound and default into the existing partial/resolved settings path without a second preference store.
- [x] T7.2: Implement all eight accepted compiled gutter command identities, per-view overrides, inspect and reset; no placeholder handlers or new factory key conflicts.
- [ ] T7.3: Update the complete sample/default references only with supported keys and verify includes, invalid candidates, no-config defaults, runtime resets and no document edits.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 8: preserve gutter geometry across editor surfaces

Contracts: GUT05 GUT06 GUT13 GUT14.
Details and exact checks: [Slice 8](../plan/rev-input-navigation.md#slice-8-preserve-gutter-geometry-across-editor-surfaces).

- [ ] T8.1: Complete narrow/zero/resized geometry with the accepted degradation order, left/right placement, nested splits, tree and completion-region coexistence.
- [ ] T8.2: Audit every display-to-text consumer including rectangles, hit-test preparation, search overlays and status bounds; avoid independent coordinate recipes.
- [ ] T8.3: Exercise actual editing/reveal/save/history and mode switches, verify no idle gutter work or retained per-document label arrays, and close the gutter ticket only after its assigned integrations.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 9: align pane status into left and right groups

Contracts: R05 STA01 STA02.
Details and exact checks: [Slice 9](../plan/rev-input-navigation.md#slice-9-align-pane-status-into-left-and-right-groups).

- [ ] T9.1: Compose the existing identity/dirty/access/position facts on the left and syntax/Text on the right inside each pane's status area; retain one row and active styling.
- [ ] T9.2: Use cell-width-aware flexible space and the accepted compact/omit/clip order; preserve truthful independent points, RO/RW and clean/modified markers.
- [ ] T9.3: Adapt existing native/PTY status expectations to semantic two-group rendering while preserving their substantive assertions; run display regressions and the affected historical acceptance journeys.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 10: own native pointer reporting and input state

Contracts: PTR01 PTR05.
Details and exact checks: [Slice 10](../plan/rev-input-navigation.md#slice-10-own-native-pointer-reporting-and-input-state).

- [ ] T10.1: Add the pointer integration-test target, minimal native event/capture records and terminal reporting ownership using the existing crossterm decoder and cleanup owner.
- [ ] T10.2: Admit only required press/release/button-motion/scroll reporting, avoiding all-motion hover traffic and a device/plugin abstraction; undo partially acquired modes on failure.
- [ ] T10.3: Verify enable/disable/unwind/idempotent restoration and preserve keyboard/prefix deadlines. Keep factory activation until the working handlers and settings are integrated.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 11: focus panes and place point by pointer

Contracts: PTR02 PTR03.
Details and exact checks: [Slice 11](../plan/rev-input-navigation.md#slice-11-focus-panes-and-place-point-by-pointer).

- [ ] T11.1: Resolve primary clicks through current composed pane/gutter/text geometry and reuse the ordinary focus transition and checked movement owner.
- [ ] T11.2: Map tabs, graphemes, wide glyphs, scrolling, clipping and EOF accurately; pane-owned decorations may focus without fabricating text positions.
- [ ] T11.3: Verify tree and multiple shared/different-buffer panes, stale/resize geometry and prompt ownership; preserve file bytes, history and active-region movement rules.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 12: scroll the pointed pane without changing focus

Contracts: PTR01 PTR02.
Details and exact checks: [Slice 12](../plan/rev-input-navigation.md#slice-12-scroll-the-pointed-pane-without-changing-focus).

- [ ] T12.1: Route vertical wheel/trackpad events to the document view or tree beneath the pointer without stealing keyboard focus.
- [ ] T12.2: Clamp native scroll bounds and adjust only necessary target-view state using existing view contracts; keep other view points and shared canonical text coherent.
- [ ] T12.3: Verify both directions, inactive targets, large/empty documents, tree and nested panes; add no simulated inertia, idle loop or competing scrolling owner.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 13: select canonical text by dragging

Contracts: PTR03 PTR04.
Details and exact checks: [Slice 13](../plan/rev-input-navigation.md#slice-13-select-canonical-text-by-dragging).

- [ ] T13.1: Use primary-button press/drag/release to establish and extend the existing linear mark/region; a click without drag only moves point.
- [ ] T13.2: Keep capture on the originating view across gutter, status and neighboring panes; cancel on C-g, invalidation, mode disable and reported focus loss.
- [ ] T13.3: Verify selection/copy/kill/undo integration, Unicode/tab endpoints and byte fidelity; retain selection after release without implicit clipboard writes.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 14: extend pointer selection with bounded autoscroll

Contracts: PTR04 PTR05.
Details and exact checks: [Slice 14](../plan/rev-input-navigation.md#slice-14-extend-pointer-selection-with-bounded-autoscroll).

- [ ] T14.1: Implement source-view autoscroll above/below its visible text only during a valid captured drag; pause inside and stop on release/cancellation/invalidation.
- [ ] T14.2: Use a minimum 100 ms tick and at most min(mouse.scroll_lines, text-area height) rows per tick; process cancellation before due work and never accumulate timer/motion debt.
- [ ] T14.3: Use controlled-clock native checks for bounds, pause/stop and target ownership plus representative terminal events; show no timer or repaint work remains at idle.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 15: activate native mouse defaults and runtime control

Contracts: PTR01 PTR06 CFG05.
Details and exact checks: [Slice 15](../plan/rev-input-navigation.md#slice-15-activate-native-mouse-defaults-and-runtime-control).

- [ ] T15.1: Wire mouse.enabled=true and mouse.scroll_lines=3 with bounds 1..100, plus the compiled M-x mouse-mode session toggle, through the existing typed configuration/catalog.
- [ ] T15.2: Activate only the complete event/cleanup path, preserve startup-only snapshots and no-write runtime overrides, and add accurate exhaustive sample/default documentation.
- [ ] T15.3: Verify startup on/off, runtime enable/disable during capture, restoration and terminal-owned selection bypass guidance; close the pointer ticket after its complete integrations.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 16: package the complete startup configuration template

Contracts: CFG23 CFG05.
Details and exact checks: [Slice 16](../plan/rev-input-navigation.md#slice-16-package-the-complete-startup-configuration-template).

- [ ] T16.1: Embed the supported complete sample in the binary and expose a bounded template source without needing a checkout, documentation file or network at runtime.
- [ ] T16.2: Ensure every active generated leaf equals compiled defaults, explicit palette values are complete and unused optional examples stay commented.
- [ ] T16.3: Extend existing catalog/reference guards to cover packaged/reference/default equality including delivered gutter/mouse controls; do not create user files in pure loading helpers.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 17: create absent user configuration safely

Contracts: CFG01 CFG14 CFG15 CFG24.
Details and exact checks: [Slice 17](../plan/rev-input-navigation.md#slice-17-create-absent-user-configuration-safely).

- [ ] T17.1: Provision the implicit user root during normal editor startup only when no destination entry exists; honor existing XDG resolution and create only necessary configuration parents.
- [ ] T17.2: Publish a complete template without clobbering concurrent winners, links, invalid/empty files or other entries; keep pure resolution/loading read-only and retain usable defaults with truthful diagnostics on failure.
- [ ] T17.3: Verify real filesystem races and fault paths, isolate HOME/all XDG roots in every process-based fixture, and adapt missing-root startup assertions while retaining their recovery/fidelity guarantees.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 18: exercise the complete combined terminal workflow

Contracts: GUT14 PTR01-PTR06 NAV01-NAV02 WSP01-WSP02 CFG23-CFG24 STA01-STA02.
Details and exact checks: [Slice 18](../plan/rev-input-navigation.md#slice-18-exercise-the-complete-combined-terminal-workflow).

- [ ] T18.1: Extend the existing configuration/UI PTY harness with a named combined journey covering first-start creation, splits/tree, both gutter modes, status groups, line navigation, whitespace conversion and pointer events.
- [ ] T18.2: Use actual decoded terminal screen cells and file/history outcomes, including cancel, resize, capture cleanup and reported failures; no schema-only success or skipped path counts as acceptance.
- [ ] T18.3: Extend the existing evidence map/checker for the admitted additional terminal cases while preserving all prior cases and assertions; keep the unchanged full-gate command sequence coherent.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 19: verify bounded input gutter and pointer work

Contracts: GUT13 PTR05 R07 CFG23.
Details and exact checks: [Slice 19](../plan/rev-input-navigation.md#slice-19-verify-bounded-input-gutter-and-pointer-work).

- [ ] T19.1: Extend the existing native performance example/collector with large-buffer gutter, repeat-input release/stop and active-drag/idle workloads; reuse its process/fixture measurement ownership.
- [ ] T19.2: Record latency, CPU, memory, iterations and candidate identity; preserve the existing ceilings and give new workloads finite measured bounds without raising limits to mask failures.
- [ ] T19.3: Check hidden-gutter and mouse-disabled paths, no idle ticks and one-time bootstrap work; add guard coverage so missing/skipped/stale workloads cannot establish acceptance.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

### Slice 20: reconcile combined delivery acceptance and references

Contracts: All seven selected work items.
Details and exact checks: [Slice 20](../plan/rev-input-navigation.md#slice-20-reconcile-combined-delivery-acceptance-and-references).

- [ ] T20.1: Reconcile complete approved specification replacements/additions into their actual baseline, configuration model and user references without rewriting historical validation provenance.
- [ ] T20.2: Extend existing acceptance evidence for all new requirements and the current plan/ticket/report ancestry; preserve original protections, zero-skip checks and same-clean-candidate evidence.
- [ ] T20.3: Verify all seven tickets and all prior integrations, prepare only approved closure metadata, and hand the clean dev candidate to the controller for the unchanged full gate and final closure.
- [ ] Record exact clean-head validation, canonical PR, report and verified dev integration.

## Delivery closure

- [ ] All seven tickets reflect verified primary-slice integration and actual implementation evidence.
- [ ] Every assigned slice report is delivered; source/model/reference links and complete baseline amendments agree.
- [ ] Controller full gate passed the exact clean integrated dev candidate, including new real terminal/resource evidence and retained protections.
- [ ] Only approved documentation closure paths changed afterward; record candidate and validation identity without relabeling evidence.
- [ ] Final plan/tracker/spec/model metadata, canonical push and Acta outcome match actual completion.
