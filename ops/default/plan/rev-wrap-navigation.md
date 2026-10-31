# Wrap and File Navigation

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
Tracker: [Wrap and Navigation Tracker](../tracker/rev-wrap-navigation.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-wrap-navigation.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 6

## Outcome and authority

Deliver word-aware wrapping enabled by default, Emacs Visual Line interaction,
file/directory targets, initially expanded tree roots, selected-candidate Tab
completion and clicked-entry selection/activation. This is the owner's selected
five-topic scope. It uses one ordered delivery and one controller lifecycle.

The request to start authorizes preparation, commits/push to dev, autonomous
implementation and the established controller-owned Rebase + Fast-forward PR
merges into dev. Do not wait for separate human approval on each slice. Main,
release tags, mirrors, installed binaries and website publication are separate.
No other pending capability enters this delivery.

Existing configuration, buffers, commands, checked positions, history, gutter,
Org and pointer support are delivered prerequisites. No LSP, AI, modal editing,
new language grammar, plugin framework or third-party dependency is required.
The specifications resolve the previously open interaction choices using the
owner's Emacs criterion and the selected lightweight file/tree model.

## Delivery strategy

Eight behavior-first slices keep the shared display/input changes reviewable:
wrapping first, then its motion/selection/Org integration, followed by directory
and tree ergonomics, then joint native acceptance and reader guidance. Every
slice must compile and pass its focused checks; do not use placeholder handlers
or intentionally failing committed tests. Later acceptance owners complete the
cross-feature contract without claiming earlier partial work is full delivery.

All prose, config examples and reports remain English. Commits and PR titles
use ordinary Conventional Commits without task-ID suffixes. Reuse native owners
rather than add one framework per topic. Keep the shipped core/Org learning
paths independent, with short commands such as rev ., rev notes.txt and the
relevant shortcut rather than large setup transcripts.

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

Prerequisites: the committed preparation and delivered native foundations.
Acceptance ownership: WRP01, WRP02, WRP06.

Owners to inspect: src/view.rs, src/display.rs, src/coordinates.rs, src/config.rs, src/command.rs and their existing submodules; tests/wrapping.rs.

### Tasks

- T1.1: Implement one checked source-to-visual fragment owner, word-aware/grapheme-safe rendering and point reveal at each pane text width.
- T1.2: Add editor.soft_wrap with compiled true default, complete TOML/sample/reference coverage and the buffer-local visual-line-mode command; create tests/wrapping.rs with actual session/render cases.

### Acceptance

Default and explicit no-wrap modes, whitespace/token breaks, tabs, combining/CJK graphemes, tiny/zero panes, exact save/dirty/undo preservation and equal source identity across split widths.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test display --test configuration --test commands`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 2: Emacs visual-line editing

Prerequisites: Slices 1 through 1 integrated.
Acceptance ownership: WRP03, WRP07.

Owners to inspect: src/view/scroll.rs, src/session scroll/motion/kill owners, native command routing and canonical buffer/history owners.

### Tasks

- T2.1: Resolve arrows, C-n/C-p, C-a/C-e and C-k through the current wrapped view while preserving canonical kill-ring/history and read-only admission.
- T2.2: Integrate goal columns, paging, recenter, goto-line and split point/scroll lifetimes; verify end-of-row/newline/EOF/repeated-kill boundaries against Emacs.

### Acceptance

Visual motion through a single wrapped logical line; one undoable C-k source edit, yank, read-only rejection, disabled-wrap logical behavior, goto source lines and resize/split goal columns.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test editing --test windows --test core --test coordinates`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 3: Wrapped selection and gutter integration

Prerequisites: Slices 1 through 2 integrated.
Acceptance ownership: WRP02, WRP03.

Owners to inspect: src/pointer.rs, src/session/pointer/, src/view/hit.rs, src/gutter.rs, src/search.rs, src/display/selection.rs and src/session/region.rs and their actual owners.

### Tasks

- T3.1: Use the same current mapping for pointer placement/drag/autoscroll, linear selections, search reveal and overlays; keep rectangle semantics in logical source rows/columns.
- T3.2: Render logical gutter labels only on a source line's first fragment, preserve relative logical distances and invalidate stale geometry after source/fold/resize changes.

### Acceptance

SGR/crossterm events after scroll/resize; selections crossing fragments, logical rectangles with tabs/wide cells, both gutter sides, narrow panes and search targets on continuation rows.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test pointer --test rectangles --test search --test gutter --test panels`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 4: Org projection and bounded wrap lifetimes

Prerequisites: Slices 1 through 3 integrated.
Acceptance ownership: WRP04, WRP05; complete WRP01-WRP07.

Owners to inspect: Existing native Org visibility/projection and source/table owners, wrapping layout/cache owner and the ready long-line ticket.

### Tasks

- T4.1: Compose wrapping with existing Org folds, headings, table fields, source blocks and preview projections; retain logical command targets and source-preserving alignment.
- T4.2: Complete buffer-local override/split/close/invalidation and bounded cache/work ownership. Measure large/long-line resize, sustained input/reversal and idle costs; resolve the wrap ticket with exact evidence.

### Acceptance

Visible Org source wraps without implicit table/source-block exceptions; folded content stays hidden, existing field/source operations work, cache eviction does not hide text and sustained input retains prompt stop/reversal.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test org_editing --test org_tables --test org_source --test pointer --test configuration`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 5: Directory targets and initial tree expansion

Prerequisites: Slices 1 through 4 integrated.
Acceptance ownership: NAV01, NAV02, NAV03, NAV06.

Owners to inspect: src/main.rs, src/session/startup.rs, src/session/path.rs, src/tree.rs and native file acceptance/identity owners.

### Tasks

- T5.1: Unify CLI/find-file file-versus-directory admission, Return final directory acceptance, explicit-root precedence and failure-preserving publication.
- T5.2: Initialize only immediate tree children on first presentation/root change, preserve later collapse/hide/show and configuration precedence; resolve both opening/initial-expansion tickets.

### Acceptance

rev ., relative/absolute files/directories, empty/Unicode/spaced/unreadable roots, empty prompt acceptance, no-file directory sessions, cwd policy, explicit roots across nested/outside files and dirty-buffer preservation.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test files --test tree --test path_completion --test configuration --test panels`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 6: Highlighted path completion

Prerequisites: Slices 1 through 5 integrated.
Acceptance ownership: NAV04 and replacement UI03.

Owners to inspect: src/file/completion/navigation/complete.rs and selection/refresh owners, src/session/path.rs and native prompt dispatch.

### Tasks

- T6.1: Make the first result visibly selected and Tab fill that exact candidate, including ambiguous prefixes and explicit list navigation, while Return retains typed/new-file semantics.
- T6.2: Retain directory-component continuation, current-candidate identity across refresh and bounded observation; apply the full UI03 amendment and resolve the path Tab ticket.

### Acceptance

alpha/alpine first-selection Tab, selected alternative and nested continuation, file completion without opening, free new filenames, stale/disappeared candidates, spaces/Unicode, cancellation and clipping.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test path_completion --test files --test commands --test configuration`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 7: Clicked tree entry and activation

Prerequisites: Slices 1 through 6 integrated.
Acceptance ownership: NAV05, NAV06.

Owners to inspect: src/pointer.rs, src/session/pointer.rs, tree viewport/entry/action owners and config mouse catalog.

### Tasks

- T7.1: Carry rendered entry identity through tree hit testing, focus/select that row and apply the existing tree action to it; validate after viewport/listing changes.
- T7.2: Add mouse.tree_activate_on_click default true with false selection-only, complete sample/reference validation and tests; preserve global mouse disablement and resolve the pointer ticket.

### Acceptance

A click targets the rendered row after scroll/resize, default opens files in the remembered editing pane or expands/collapses directories, false selects only, blank clicks only focus and failures preserve dirty text.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test pointer --test tree --test panels --test configuration --test files`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Slice 8: User guidance and joint interaction acceptance

Prerequisites: Slices 1 through 7 integrated.
Acceptance ownership: All WRP01-WRP07, NAV01-NAV06 and UI03 scenarios.

Owners to inspect: docs/tutorials/user-guide/, affected docs/reference/ pages and template copies, CHANGELOG.md, existing native terminal probes and focused Rust/Bash acceptance fixtures.

### Tasks

- T8.1: Deliver native Rust/Bash joint journeys in bash scripts/check-wrap-navigation.sh, using the real event/composition path plus a bounded representative Linux PTY route. Reuse existing probes; add no Python gate or new runtime dependency.
- T8.2: Update the editor User Guide with short visual examples, defaults/commands/configuration and CHANGELOG Unreleased; reconcile affected delivered baseline clauses and complete all five ticket transitions/links before the full gate.

### Acceptance

Open a directory, Tab to a file, click another entry, split Rust/Org views and navigate/kill/select/search a wrapped line. Include resize, cancellation, unwrapped alternative, no leaked fixtures/children and measured long-line input bounds.

### Validation

- `cargo fmt --all -- --check`
- `cargo test --locked --test wrapping --test path_completion --test tree --test pointer --test configuration --test commands`
- `bash scripts/check-wrap-navigation.sh`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

New/changed library helpers also require focused native tests where the integration
fixtures cannot cover their boundary. Do not repeat the full gate per slice.

## Ticket ownership

| Ticket | Closing owner | Earlier implementation inputs |
| --- | --- | --- |
| [Long-line wrapping](../ticket/solved/20261028224018-define-comfortable-long-line-wrapping.md) | Slice 4 | Slices 1-3 deliver the shared projection and interactions. |
| [Directory opening](../ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md) | Slice 5 | Uses the accepted root/Return contract. |
| [Initial tree expansion](../ticket/solved/20261028153707-expand-file-tree-root-on-first-display.md) | Slice 5 | Share first-display initialization with directory opening. |
| [Selected Tab completion](../ticket/ready/20261028170748-complete-highlighted-find-file-path-with-tab.md) | Slice 6 | Uses the same prompt while keeping completion separate from acceptance. |
| [Clicked tree entry](../ticket/ready/20261030231016-select-the-clicked-file-tree-entry.md) | Slice 7 | Reuse current entry identity, geometry and canonical activation. |

Each ticket has one closing slice. Update its lifecycle, implementation commits,
PR and validation in that slice. Complete ready/in_progress/reviewing/solved moves
and repair inbound/outbound links before the final gate. Do not move tickets at
metadata-only closure. Slice 8 verifies the complete set without duplicate tickets.

## Validation selection and cost

The integrated gate is exactly the Full gate command above. Run it once through
the controller against the clean final dev candidate, after all eight PRs.
It includes all native targets because projection, input, coordinates, commands
and configuration are shared across the editor and Org. This is a justified
cross-contract regression check, not every historical Python/PTY/export gate.
The new focused shell gate must use Rust/Bash and available Linux terminal tools,
clean up owned resources, and preserve failures; do not add a Python requirement.
Existing historical check-all tooling and past gate evidence remain unchanged.

Compilation dominates cold execution; expected cost is minutes with a warm
Cargo cache, not an observed benchmark for this candidate. Record actual
elapsed time and long-line/input measurements in the reports. Keep per-slice
checks targeted. No helper may disable wrapping, weaken a valid assertion or
substitute a synthetic stub merely to pass legacy no-wrap expectations.

## Baseline and documentation reconciliation

Slice 1 owns configuration defaults/catalogs and native command naming. Slices
2-4 reconcile the no-wrap display clause in rev-editor R07, the GUT03 clause in editor-gutter and its consolidated copy, plus affected
Org/pointer coordinate assumptions. Inspect the actual requirement headings
before editing; do not invent baseline IDs. Preserve historical closure/gate
records and identify the new delivery separately.

Slice 6 applies the complete modified UI03 in the navigation spec. Slice 8
checks source/default/guide agreement, both full template copies, the ordinary
binding catalog and CHANGELOG Unreleased. Each earlier slice that completes a
user-visible capability also maintains Unreleased under changelog-normalize.
No version number or release tag is selected by this plan.

## Reports, full gate and closure

Each slice creates its one configured report with the required metadata and
Purpose, Delivered Behavior, Implementation Notes, Contracts Added or Changed,
Files of Interest, Validation, Risks and Follow-ups sections. PR bodies have
exactly Summary, Changes and Validation; link the report through its immutable
introduction commit. Looper owns PR admission, exact-HEAD evidence, integration,
full-gate execution and completion checking. Do not launch a second controller.

Only after the exact integrated candidate passes the pinned gate may metadata
closure set plan/tracker to completed, reports to delivered and accepted spec/
model metadata to Completed. The pending index can then identify these topics
as delivered and preserve the other ten capabilities' states. Closure paths
are exact approved documents only; runtime, tests, user docs, templates,
changelog and ticket path/content changes must precede the full gate.

Do not claim completion from eight merged slices alone. Gate failure retains
the same run/checkpoint for diagnosis and the supported recovery workflow;
never weaken its gate, resubmit an uncertain merge or start a successor.
