# Input, Navigation and Presentation Delivery

Status: Approved
Approved: product owner, 2026-10-27; complete all seven selected topics in one autonomous Looper cycle after reconciling their contracts and slicing. Startup and canonical dev integration requested.
Date: 2026-10-27
Delivery set: rev-input-navigation
Slice strategy: layered
Active slice: Slice 8 - preserve gutter geometry across editor surfaces
Full gate: `scripts/check-all.sh`
Planning baseline: `9e24fa3f7e60ab8a4ff60e3589662631f4a94bae`
Spec: [Line Navigation](../spec/editor-line-navigation.md), [Whitespace Conversion](../spec/editor-whitespace-conversion.md), [Gutter](../spec/editor-gutter.md), [Pointer](../spec/editor-pointer-interaction.md), [Configuration Bootstrap](../spec/editor-configuration-bootstrap.md), [Status Layout](../spec/editor-status-layout.md)
Model: [Gutter Model](../spec/editor-gutter-model.md), [Existing Configuration Model](../spec/editor-configuration-model.md)
Tracker: [Input and Navigation Tracker](../tracker/rev-input-navigation.md)

## Scope and strategy

Complete the seven current work items in this single delivery: responsive
vertical arrows, goto-line, explicit tabs/spaces conversion, line-number gutter,
mouse/trackpad focus/scroll/selection, initial configuration creation and the
two-group status layout. Linked Org, bookmark, autocomplete and LSP proposals do
not add work to these seven tickets. The previous 30-slice set stays closed.

Layered slicing is selected because gutter geometry, view state, terminal input
and settings have shared contracts that must be stabilized before the complete
pointer and bootstrap behavior is activated. Every preparatory slice compiles,
has meaningful focused checks and identifies its boundary; none advertises a
placeholder command or unsupported setting as operational. Later slices complete
and exercise these layers as real native workflows. The count follows these
ownership and verification boundaries, not a target number of milestones.

The approved command conventions are M-g g / M-g M-g and M-x goto-line, with
C-g reserved for cancellation; native tabify/untabify use marked linear regions
and preserve visual columns. No whole-document conversion occurs implicitly.
Pointer scrolling targets the pane beneath the pointer without changing focus;
primary clicks change focus, and a captured drag may autoscroll its source view.
All normal keyboard paths remain available.

## Ticket coverage

| Topic | Ticket | Primary slices |
| --- | --- | --- |
| Arrow responsiveness | [TKT-20261027190521](../ticket/solved/20261027190521-stop-arrow-key-navigation-inertia.md) | 1; repeated at 18-20 |
| Line navigation | [TKT-20261027191322](../ticket/solved/20261027191322-add-emacs-style-goto-line.md) | 2 |
| Whitespace conversion | [TKT-20261027092143](../ticket/solved/20261027092143-convert-tabs-and-spaces.md) | 3 |
| Gutter | [TKT-20261027193500](../ticket/in_progress/20261027193500-deliver-native-line-number-gutter.md) | 4-8 |
| Status layout | [TKT-20261027194421](../ticket/open/20261027194421-group-pane-status-left-and-right.md) | 9 |
| Pointer interaction | [TKT-20261027190629](../ticket/open/20261027190629-mouse-scroll-selection-and-pane-focus.md) | 10-15 |
| Configuration creation | [TKT-20261027193636](../ticket/open/20261027193636-create-missing-user-configuration.md) | 16-17 |

All seven also participate in terminal, resource and final acceptance slices
18-20. Advance a ticket from open to ready and in_progress before its first
implementation. Keep it active until all its primary slices have integrated,
then record actual commits, canonical PRs and reports before solving it. Update
links and any historical ticket-provenance checks coherently with a status move;
never discard an older acceptance assertion just to move a ticket.

## Validation and resource discipline

Each slice uses its exact ordered focused command prefix below. All changes to
rendering or input include display regressions; gutter and pointer test targets
are introduced by Slices 4 and 10 respectively. New user paths require actual
handler/render/filesystem outcomes, not catalog or sample existence alone.
No source, runtime or dependency implementation is included in planning.

The unchanged full gate is `scripts/check-all.sh`, once on the clean integrated
dev candidate. Its existing eleven stage identities/order and all historical
native, PTY, fidelity and resource protections remain required. Extend the
existing configuration/UI terminal/performance collectors and evidence map for
this delivery; do not create a parallel acceptance framework. Preserve prior
cases and make every new required case/workload mandatory, fresh and zero-skip.
The final evidence must cover every selected requirement, actual canonical
integrations and the same clean candidate. No cached or listing-only result is
runtime acceptance.

The previous complete gate took approximately 71 seconds on a warm candidate.
New input/gutter/drag and bootstrap paths justify adding focused workloads and
one representative combined terminal journey; the warm complete gate should
remain on the order of one to three minutes. Record actual cost and investigate
unexpected expansion rather than raising timeouts or resource limits. Preserve
all existing performance ceilings. Do not run the full gate separately for each
slice or change its command as a recovery shortcut.

Use independent per-worktree build outputs. Every process-based test must own
its HOME and all XDG directories before bootstrap is activated; tests must not
provision or inspect the operator's configuration. Do not stop other builds.

Input scheduling preserves ordered edits/commands, Escape deadlines and C-g.
Autoscroll has no idle activity: a valid out-of-bounds captured drag may tick no
more frequently than every 100 ms, advancing at most the lesser of configured
scroll_lines and the current text-area height; release/cancel is processed first.
Every operation uses checked source revisions and existing shared-view history.
No new dependency, daemon, external input stack or programmable layer is needed.

## Controller authority and bounded recovery

The current startup instruction authorizes implementation and canonical
Rebase + Fast-forward integration of these exact twenty slice PRs into dev,
followed by the complete gate and documentation closure. Read installed
`looper --help`; use one new configuration and worker conversation for this set.
Do not resume or modify the completed configuration/UI set.

Delegate the native bounded-v1 profile before execution: 200 worker turns,
five stalled turns, two diagnostics per family/eight per run, three failed
repairs per family and three environmental retries across the run. Reserve at
most four final-gate repair PRs with prefix fix/rev-input-navigation-validation,
title fix(gate): correct input and navigation acceptance, and existing Slice 20
report identity. Repair focus uses fmt, Clippy, all native tests, source checking
and whitespace; the unchanged full gate remains controller-owned after merge.

Repair paths are bounded to Rust src/tests/examples, docs/reference, the
configuration_ui acceptance/performance/terminal owners, the named existing
PTY/performance/acceptance scripts and the existing Slice 20 report. Exact-file
native derivation may only cover verified files produced by this set's own
integrations. Exclude agent contracts, the approved specs, plan/tracker, Cargo
manifests/lockfile and the full-gate entry point. Grant no new generated-output
producer paths. Recovery may correct implementation or fixture mechanics but
may not weaken assertions, alter approved behavior, change validation commands,
reset counters, kill unrelated processes or create another controller. Genuine
scope/authority changes and exhausted hard budgets stop for the maintainer.

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

## Slice 1: restore responsive vertical navigation

Contracts: R07 R16.
Tickets: TKT-20261027190521.
Branch: `fix/rev-input-01-navigation-responsiveness`. PR title: `fix(slice-1): restore responsive vertical navigation`.
Report: `ops/default/report/slices/rev-input-navigation/slice-1-navigation-responsiveness.md`.

- T1.1: Reproduce the reported repeat-input tail through the existing terminal/event/render path; record the tested build, fixture, input rate and measured cause before changing scheduling.
- T1.2: Correct the demonstrated bottleneck with finite event/render work; preserve every ordered text/command transition, Escape deadlines, C-g, history and both arrow directions. Do not discard input merely to hide a backlog.
- T1.3: Add a meaningful regression, check ordinary/large files and boundary reversal, and record exact focused evidence and the canonical slice report.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands --test display --test windows --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 2: add Emacs-style line navigation

Contracts: NAV01 NAV02.
Tickets: TKT-20261027191322.
Branch: `feat/rev-input-02-goto-line`. PR title: `feat(slice-2): add Emacs-style line navigation`.
Report: `ops/default/report/slices/rev-input-navigation/slice-2-goto-line.md`.

- T2.1: Add the compiled goto-line identity, bounded signed-integer prompt and M-g g / M-g M-g defaults through existing command/keymap owners; preserve C-g and all prior aliases.
- T2.2: Use checked logical-line lookup and native reveal; match start clamping for nonpositive values and EOF clamping beyond the last line. Preserve other views and cancellation state.
- T2.3: Verify native and prompt dispatch, Unicode/CRLF/empty/final-newline boundaries and malformed/overflow input; update command references and record evidence without claiming general numeric-prefix support.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test commands --test editing --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 3: add explicit tabify and untabify

Contracts: WSP01 WSP02.
Tickets: TKT-20261027092143.
Branch: `feat/rev-input-03-whitespace-conversion`. PR title: `feat(slice-3): add explicit tabify and untabify`.
Report: `ops/default/report/slices/rev-input-navigation/slice-3-whitespace-conversion.md`.

- T3.1: Implement both native M-x operations on the existing admitted linear region; missing regions give feedback and never silently select the whole document.
- T3.2: Compute tab stops from the effective buffer policy and true starting display column, including Unicode prefixes and mixed whitespace; retain all nonselected bytes and line formats.
- T3.3: Publish one checked edit/undo operation, cover redo/no-op/read-only/shared views and document how to select the intended range; retain source fidelity and record focused evidence. Reconcile any historical open-future-ticket guard against the newly verified conversion delivery instead of deleting the provenance check.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test editing --test commands --test coordinates --test files`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 4: define native gutter view policy

Contracts: GUT01 GUT04 GUT11.
Tickets: TKT-20261027193500.
Branch: `refactor/rev-input-04-gutter-policy`. PR title: `refactor(slice-4): define native gutter view policy`.
Report: `ops/default/report/slices/rev-input-navigation/slice-4-gutter-policy.md`.

- T4.1: Introduce the bounded native gutter projection/view-policy records using existing buffer, ViewId, line index and panel owners; add the gutter integration-test target.
- T4.2: Model independent visibility/mode/current-line/side overrides, split inheritance, reset and lifecycle without per-line retained number arrays or new persistent stores.
- T4.3: Verify checked width arithmetic, decimal boundaries and view invalidation; keep preparatory policy distinct from advertised operational TOML/commands and record its contracts.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test panels --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 5: render absolute logical line numbers

Contracts: GUT01 GUT03 GUT04 GUT05 GUT10.
Tickets: TKT-20261027193500.
Branch: `feat/rev-input-05-absolute-gutter`. PR title: `feat(slice-5): render absolute logical line numbers`.
Report: `ops/default/report/slices/rev-input-navigation/slice-5-absolute-gutter.md`.

- T5.1: Render absolute logical line labels with the accepted minimum digits, padding, separator fallback and semantic palette inside each panel.
- T5.2: Give text one checked origin after number allocation; align cursor, tabs, Unicode, syntax/search/regions and horizontal clipping without prefix-string geometry.
- T5.3: Cover empty/final-empty lines, beyond-EOF padding, safe glyphs, decimal-width changes and actual rendered cells; run display regressions with the new gutter cases.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test coordinates --test panels`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 6: render independent relative numbering

Contracts: GUT02 GUT04 GUT11.
Tickets: TKT-20261027193500.
Branch: `feat/rev-input-06-relative-gutter`. PR title: `feat(slice-6): render independent relative numbering`.
Report: `ops/default/report/slices/rev-input-navigation/slice-6-relative-gutter.md`.

- T6.1: Add relative labels from each view's own logical point and absolute/zero/hidden current-line forms; keep digit width stable during origin movement.
- T6.2: Preserve independent points and overrides across shared splits, buffer switches, close and rebind; retain complete logical numbering through edits and history.
- T6.3: Verify both modes, current-line forms and shared-view lifecycle with render-level assertions and record bounded visible-row behavior.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 7: wire gutter settings and native commands

Contracts: GUT10 GUT12 CFG05.
Tickets: TKT-20261027193500.
Branch: `feat/rev-input-07-gutter-controls`. PR title: `feat(slice-7): wire gutter settings and native commands`.
Report: `ops/default/report/slices/rev-input-navigation/slice-7-gutter-controls.md`.

- T7.1: Wire every accepted gutter/model key, palette role, type/bound and default into the existing partial/resolved settings path without a second preference store.
- T7.2: Implement all eight accepted compiled gutter command identities, per-view overrides, inspect and reset; no placeholder handlers or new factory key conflicts.
- T7.3: Update the complete sample/default references only with supported keys and verify includes, invalid candidates, no-config defaults, runtime resets and no document edits.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test configuration --test commands --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 8: preserve gutter geometry across editor surfaces

Contracts: GUT05 GUT06 GUT13 GUT14.
Tickets: TKT-20261027193500.
Branch: `fix/rev-input-08-gutter-geometry`. PR title: `fix(slice-8): preserve gutter geometry across editor surfaces`.
Report: `ops/default/report/slices/rev-input-navigation/slice-8-gutter-geometry.md`.

- T8.1: Complete narrow/zero/resized geometry with the accepted degradation order, left/right placement, nested splits, tree and completion-region coexistence.
- T8.2: Audit every display-to-text consumer including rectangles, hit-test preparation, search overlays and status bounds; avoid independent coordinate recipes.
- T8.3: Exercise actual editing/reveal/save/history and mode switches, verify no idle gutter work or retained per-document label arrays, and close the gutter ticket only after its assigned integrations.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test panels --test windows --test search --test path_completion`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 9: align pane status into left and right groups

Contracts: R05 STA01 STA02.
Tickets: TKT-20261027194421.
Branch: `feat/rev-input-09-status-layout`. PR title: `feat(slice-9): align pane status into left and right groups`.
Report: `ops/default/report/slices/rev-input-navigation/slice-9-status-layout.md`.

- T9.1: Compose the existing identity/dirty/access/position facts on the left and syntax/Text on the right inside each pane's status area; retain one row and active styling.
- T9.2: Use cell-width-aware flexible space and the accepted compact/omit/clip order; preserve truthful independent points, RO/RW and clean/modified markers.
- T9.3: Adapt existing native/PTY status expectations to semantic two-group rendering while preserving their substantive assertions; run display regressions and the affected historical acceptance journeys.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test display --test panels --test windows --test commands`
- `python3 scripts/check-pty.py --suite acceptance`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 10: own native pointer reporting and input state

Contracts: PTR01 PTR05.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-10-pointer-lifetime`. PR title: `feat(slice-10): own native pointer reporting and input state`.
Report: `ops/default/report/slices/rev-input-navigation/slice-10-pointer-lifetime.md`.

- T10.1: Add the pointer integration-test target, minimal native event/capture records and terminal reporting ownership using the existing crossterm decoder and cleanup owner.
- T10.2: Admit only required press/release/button-motion/scroll reporting, avoiding all-motion hover traffic and a device/plugin abstraction; undo partially acquired modes on failure.
- T10.3: Verify enable/disable/unwind/idempotent restoration and preserve keyboard/prefix deadlines. Keep factory activation until the working handlers and settings are integrated.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test commands --test panels --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 11: focus panes and place point by pointer

Contracts: PTR02 PTR03.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-11-pointer-hit-focus`. PR title: `feat(slice-11): focus panes and place point by pointer`.
Report: `ops/default/report/slices/rev-input-navigation/slice-11-pointer-hit-focus.md`.

- T11.1: Resolve primary clicks through current composed pane/gutter/text geometry and reuse the ordinary focus transition and checked movement owner.
- T11.2: Map tabs, graphemes, wide glyphs, scrolling, clipping and EOF accurately; pane-owned decorations may focus without fabricating text positions.
- T11.3: Verify tree and multiple shared/different-buffer panes, stale/resize geometry and prompt ownership; preserve file bytes, history and active-region movement rules.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test panels --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 12: scroll the pointed pane without changing focus

Contracts: PTR01 PTR02.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-12-pointer-scroll`. PR title: `feat(slice-12): scroll the pointed pane without changing focus`.
Report: `ops/default/report/slices/rev-input-navigation/slice-12-pointer-scroll.md`.

- T12.1: Route vertical wheel/trackpad events to the document view or tree beneath the pointer without stealing keyboard focus.
- T12.2: Clamp native scroll bounds and adjust only necessary target-view state using existing view contracts; keep other view points and shared canonical text coherent.
- T12.3: Verify both directions, inactive targets, large/empty documents, tree and nested panes; add no simulated inertia, idle loop or competing scrolling owner.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test tree --test windows --test display --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 13: select canonical text by dragging

Contracts: PTR03 PTR04.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-13-pointer-selection`. PR title: `feat(slice-13): select canonical text by dragging`.
Report: `ops/default/report/slices/rev-input-navigation/slice-13-pointer-selection.md`.

- T13.1: Use primary-button press/drag/release to establish and extend the existing linear mark/region; a click without drag only moves point.
- T13.2: Keep capture on the originating view across gutter, status and neighboring panes; cancel on C-g, invalidation, mode disable and reported focus loss.
- T13.3: Verify selection/copy/kill/undo integration, Unicode/tab endpoints and byte fidelity; retain selection after release without implicit clipboard writes.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test editing --test commands --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 14: extend pointer selection with bounded autoscroll

Contracts: PTR04 PTR05.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-14-drag-autoscroll`. PR title: `feat(slice-14): extend pointer selection with bounded autoscroll`.
Report: `ops/default/report/slices/rev-input-navigation/slice-14-drag-autoscroll.md`.

- T14.1: Implement source-view autoscroll above/below its visible text only during a valid captured drag; pause inside and stop on release/cancellation/invalidation.
- T14.2: Use a minimum 100 ms tick and at most min(mouse.scroll_lines, text-area height) rows per tick; process cancellation before due work and never accumulate timer/motion debt.
- T14.3: Use controlled-clock native checks for bounds, pause/stop and target ownership plus representative terminal events; show no timer or repaint work remains at idle.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test commands --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 15: activate native mouse defaults and runtime control

Contracts: PTR01 PTR06 CFG05.
Tickets: TKT-20261027190629.
Branch: `feat/rev-input-15-pointer-controls`. PR title: `feat(slice-15): activate native mouse defaults and runtime control`.
Report: `ops/default/report/slices/rev-input-navigation/slice-15-pointer-controls.md`.

- T15.1: Wire mouse.enabled=true and mouse.scroll_lines=3 with bounds 1..100, plus the compiled M-x mouse-mode session toggle, through the existing typed configuration/catalog.
- T15.2: Activate only the complete event/cleanup path, preserve startup-only snapshots and no-write runtime overrides, and add accurate exhaustive sample/default documentation.
- T15.3: Verify startup on/off, runtime enable/disable during capture, restoration and terminal-owned selection bypass guidance; close the pointer ticket after its complete integrations.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test configuration --test commands --test display --test recovery`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 16: package the complete startup configuration template

Contracts: CFG23 CFG05.
Tickets: TKT-20261027193636.
Branch: `feat/rev-input-16-bootstrap-template`. PR title: `feat(slice-16): package the complete startup configuration template`.
Report: `ops/default/report/slices/rev-input-navigation/slice-16-bootstrap-template.md`.

- T16.1: Embed the supported complete sample in the binary and expose a bounded template source without needing a checkout, documentation file or network at runtime.
- T16.2: Ensure every active generated leaf equals compiled defaults, explicit palette values are complete and unused optional examples stay commented.
- T16.3: Extend existing catalog/reference guards to cover packaged/reference/default equality including delivered gutter/mouse controls; do not create user files in pure loading helpers.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test gutter --test pointer`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 17: create absent user configuration safely

Contracts: CFG01 CFG14 CFG15 CFG24.
Tickets: TKT-20261027193636.
Branch: `feat/rev-input-17-bootstrap-publication`. PR title: `feat(slice-17): create absent user configuration safely`.
Report: `ops/default/report/slices/rev-input-navigation/slice-17-bootstrap-publication.md`.

- T17.1: Provision the implicit user root during normal editor startup only when no destination entry exists; honor existing XDG resolution and create only necessary configuration parents.
- T17.2: Publish a complete template without clobbering concurrent winners, links, invalid/empty files or other entries; keep pure resolution/loading read-only and retain usable defaults with truthful diagnostics on failure.
- T17.3: Verify real filesystem races and fault paths, isolate HOME/all XDG roots in every process-based fixture, and adapt missing-root startup assertions while retaining their recovery/fidelity guarantees.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test files --test recovery --test commands`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 18: exercise the complete combined terminal workflow

Contracts: GUT14 PTR01-PTR06 NAV01-NAV02 WSP01-WSP02 CFG23-CFG24 STA01-STA02.
Tickets: TKT-20261027190521, TKT-20261027190629, TKT-20261027191322, TKT-20261027092143, TKT-20261027193500, TKT-20261027193636, TKT-20261027194421.
Branch: `test/rev-input-18-combined-terminal`. PR title: `test(slice-18): exercise the complete combined terminal workflow`.
Report: `ops/default/report/slices/rev-input-navigation/slice-18-combined-terminal.md`.

- T18.1: Extend the existing configuration/UI PTY harness with a named combined journey covering first-start creation, splits/tree, both gutter modes, status groups, line navigation, whitespace conversion and pointer events.
- T18.2: Use actual decoded terminal screen cells and file/history outcomes, including cancel, resize, capture cleanup and reported failures; no schema-only success or skipped path counts as acceptance.
- T18.3: Extend the existing evidence map/checker for the admitted additional terminal cases while preserving all prior cases and assertions; keep the unchanged full-gate command sequence coherent.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test configuration --test recovery`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 19: verify bounded input gutter and pointer work

Contracts: GUT13 PTR05 R07 CFG23.
Tickets: TKT-20261027190521, TKT-20261027190629, TKT-20261027193500, TKT-20261027193636.
Branch: `perf/rev-input-19-bounded-interaction`. PR title: `perf(slice-19): verify bounded input gutter and pointer work`.
Report: `ops/default/report/slices/rev-input-navigation/slice-19-bounded-interaction.md`.

- T19.1: Extend the existing native performance example/collector with large-buffer gutter, repeat-input release/stop and active-drag/idle workloads; reuse its process/fixture measurement ownership.
- T19.2: Record latency, CPU, memory, iterations and candidate identity; preserve the existing ceilings and give new workloads finite measured bounds without raising limits to mask failures.
- T19.3: Check hidden-gutter and mouse-disabled paths, no idle ticks and one-time bootstrap work; add guard coverage so missing/skipped/stale workloads cannot establish acceptance.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test configuration`
- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-source.py`
- `git diff --check`

## Slice 20: reconcile combined delivery acceptance and references

Contracts: All seven selected work items.
Tickets: TKT-20261027190521, TKT-20261027190629, TKT-20261027191322, TKT-20261027092143, TKT-20261027193500, TKT-20261027193636, TKT-20261027194421.
Branch: `test/rev-input-20-delivery-acceptance`. PR title: `test(slice-20): reconcile combined delivery acceptance and references`.
Report: `ops/default/report/slices/rev-input-navigation/slice-20-delivery-acceptance.md`.

- T20.1: Reconcile complete approved specification replacements/additions into their actual baseline, configuration model and user references without rewriting historical validation provenance.
- T20.2: Extend existing acceptance evidence for all new requirements and the current plan/ticket/report ancestry; preserve original protections, zero-skip checks and same-clean-candidate evidence.
- T20.3: Verify all seven tickets and all prior integrations, prepare only approved closure metadata, and hand the clean dev candidate to the controller for the unchanged full gate and final closure.

Required validation, exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test configuration --test panels --test windows --test editing`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-source.py`
- `git diff --check`

## Final integration and closure

After Slice 20 integrates, run the full gate on clean pushed dev. A failed gate
keeps the set open; the delegated repair route does not add numbered slices.
After the exact candidate passes, close only documentation: plan/tracker,
configured reports, selected spec/model metadata and the solved paths of these
seven tickets. Preserve the tested SHA, validation identity, historical results
and later documentation-only closure SHA as distinct facts. No runtime/script,
manifest or validation requirement may change after that gate without new
validation. All twenty reports must be delivered and every task reconciled.

Delivery ends on dev. No main promotion, tag, release, public-mirror push or
separate successor run is included in this startup.
