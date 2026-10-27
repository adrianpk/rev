# Configuration and Interaction Delivery

Status: Approved
Approved: product owner, 2026-10-27; one combined plan/tracker, all slices and autonomous Looper startup and dev integration authorized.
Delivery set: rev-configuration-ui
Slice strategy: layered
Active slice: Slice 27 - Exercise combined terminal workflows
Full gate: `scripts/check-all.sh`
Spec: [Editor Configuration](../spec/editor-configuration.md)
Model: [Editor Configuration Model](../spec/editor-configuration-model.md)
Concern: [Editor Interaction](../spec/editor-interaction.md)
Tracker: [Configuration and Interaction Tracker](../tracker/rev-configuration-ui.md)

## Outcome and strategy

Deliver startup TOML/XDG configuration of the accepted existing editor surfaces,
coherent panel composition, vertical M-x and find-file completion in one Looper
cycle. The original rev-editor delivery is complete; this set starts Slice 1.
All 30 slices are required. The count follows ownership and review boundaries,
not a curriculum or a fixed target.

Layered strategy is explicit because shared coordinate, configuration and panel
contracts must be reviewed before adapting their consumers. Every layer must
compile and pass focused checks. Candidate schema types may precede executable
startup integration; they are not advertised as delivered controls until their
consumers and Slice 20 are integrated. No failing-test or placeholder-command
slice is permitted. The last ten slices deliver the complete interaction and
acceptance paths on those owners.

## Prerequisites and authority

- Completed 48-slice rev-editor set and validated baseline at planning commit
  `d0b2c8dd26e1721f1413cbfc25816dc1b3441731`.
- Accepted configuration spec/model, pinned palette and observed Emacs
  rectangle tab behavior; all product decisions needed for this set are closed.
- Accepted UI ticket scope and Editor Interaction requirements UI01-UI04.
- Maintainer delegates exactly these slice branches/PRs and controller-owned
  Rebase + Fast-forward integration into dev. Main, tags, releases and public
  mirrors are outside controller authority.
- The primary dev checkout is the integration owner; each slice uses its own
  worktree under the machine's global worktree contract. Preserve other trees.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Model typed configuration | `feat/rev-config-ui-01-config-types` | `feat(slice-1): model typed configuration` | `ops/default/report/slices/rev-configuration-ui/slice-1-config-types.md` |
| Slice 2 | Validate TOML and diagnostics | `feat/rev-config-ui-02-toml-validation` | `feat(slice-2): validate typed TOML settings` | `ops/default/report/slices/rev-configuration-ui/slice-2-toml-validation.md` |
| Slice 3 | Load bounded include graphs | `feat/rev-config-ui-03-config-includes` | `feat(slice-3): load bounded configuration includes` | `ops/default/report/slices/rev-configuration-ui/slice-3-config-includes.md` |
| Slice 4 | Resolve layered settings | `feat/rev-config-ui-04-config-resolution` | `feat(slice-4): resolve configuration precedence` | `ops/default/report/slices/rev-configuration-ui/slice-4-config-resolution.md` |
| Slice 5 | Identify buffer languages | `feat/rev-config-ui-05-file-associations` | `feat(slice-5): resolve file language associations` | `ops/default/report/slices/rev-configuration-ui/slice-5-file-associations.md` |
| Slice 6 | Project buffer tab policies | `feat/rev-config-ui-06-buffer-display-policy` | `feat(slice-6): apply buffer display policies` | `ops/default/report/slices/rev-configuration-ui/slice-6-buffer-display-policy.md` |
| Slice 7 | Apply indentation and new-file defaults | `feat/rev-config-ui-07-indentation-and-format` | `feat(slice-7): apply indentation and new-file defaults` | `ops/default/report/slices/rev-configuration-ui/slice-7-indentation-and-format.md` |
| Slice 8 | Capture Emacs-style rectangles | `fix/rev-config-ui-08-rectangle-capture` | `fix(slice-8): preserve rectangle tab semantics` | `ops/default/report/slices/rev-configuration-ui/slice-8-rectangle-capture.md` |
| Slice 9 | Keep rectangle and path transitions coherent | `feat/rev-config-ui-09-policy-transitions` | `feat(slice-9): retain buffer policies across path changes` | `ops/default/report/slices/rev-configuration-ui/slice-9-policy-transitions.md` |
| Slice 10 | Define panel geometry contracts | `refactor/rev-config-ui-10-panel-contracts` | `refactor(slice-10): define panel geometry contracts` | `ops/default/report/slices/rev-configuration-ui/slice-10-panel-contracts.md` |
| Slice 11 | Compose editing panels | `refactor/rev-config-ui-11-editor-panels` | `refactor(slice-11): compose editing panes through panels` | `ops/default/report/slices/rev-configuration-ui/slice-11-editor-panels.md` |
| Slice 12 | Compose tree and auxiliary panels | `refactor/rev-config-ui-12-tree-panels` | `refactor(slice-12): compose the tree through panels` | `ops/default/report/slices/rev-configuration-ui/slice-12-tree-panels.md` |
| Slice 13 | Resolve palette and syntax colors | `feat/rev-config-ui-13-palette-and-syntax` | `feat(slice-13): apply semantic RGB palette roles` | `ops/default/report/slices/rev-configuration-ui/slice-13-palette-and-syntax.md` |
| Slice 14 | Render UI palette and restore terminal | `feat/rev-config-ui-14-ui-palette` | `feat(slice-14): render UI palette and restore terminal colors` | `ops/default/report/slices/rev-configuration-ui/slice-14-ui-palette.md` |
| Slice 15 | Apply presentation and syntax preferences | `feat/rev-config-ui-15-presentation-preferences` | `feat(slice-15): apply layout tree and syntax preferences` | `ops/default/report/slices/rev-configuration-ui/slice-15-presentation-preferences.md` |
| Slice 16 | Apply bounded history and prompt preferences | `feat/rev-config-ui-16-bounded-preferences` | `feat(slice-16): apply history ring and prompt limits` | `ops/default/report/slices/rev-configuration-ui/slice-16-bounded-preferences.md` |
| Slice 17 | Validate context keymaps | `feat/rev-config-ui-17-keymap-contracts` | `feat(slice-17): validate configurable context keymaps` | `ops/default/report/slices/rev-configuration-ui/slice-17-keymap-contracts.md` |
| Slice 18 | Dispatch configurable editor bindings | `feat/rev-config-ui-18-editor-keymaps` | `feat(slice-18): dispatch configured editor keybindings` | `ops/default/report/slices/rev-configuration-ui/slice-18-editor-keymaps.md` |
| Slice 19 | Dispatch transient and tree bindings | `feat/rev-config-ui-19-context-keymaps` | `feat(slice-19): dispatch prompt and tree keybindings` | `ops/default/report/slices/rev-configuration-ui/slice-19-context-keymaps.md` |
| Slice 20 | Activate startup configuration | `feat/rev-config-ui-20-startup-configuration` | `feat(slice-20): load user configuration at startup` | `ops/default/report/slices/rev-configuration-ui/slice-20-startup-configuration.md` |
| Slice 21 | Present vertical M-x candidates | `feat/rev-config-ui-21-vertical-command-list` | `feat(slice-21): show vertical command completion` | `ops/default/report/slices/rev-configuration-ui/slice-21-vertical-command-list.md` |
| Slice 22 | Navigate and execute command candidates | `feat/rev-config-ui-22-command-selection` | `feat(slice-22): navigate vertical command candidates` | `ops/default/report/slices/rev-configuration-ui/slice-22-command-selection.md` |
| Slice 23 | Observe bounded filesystem candidates | `feat/rev-config-ui-23-path-candidates` | `feat(slice-23): observe filesystem completion candidates` | `ops/default/report/slices/rev-configuration-ui/slice-23-path-candidates.md` |
| Slice 24 | Complete and navigate file paths | `feat/rev-config-ui-24-path-navigation` | `feat(slice-24): complete and navigate file paths` | `ops/default/report/slices/rev-configuration-ui/slice-24-path-navigation.md` |
| Slice 25 | Open selected and new file paths safely | `feat/rev-config-ui-25-find-file-integration` | `feat(slice-25): open completed file paths safely` | `ops/default/report/slices/rev-configuration-ui/slice-25-find-file-integration.md` |
| Slice 26 | Publish exhaustive configuration reference | `docs/rev-config-ui-26-configuration-reference` | `docs(slice-26): document every supported configuration value` | `ops/default/report/slices/rev-configuration-ui/slice-26-configuration-reference.md` |
| Slice 27 | Exercise combined terminal workflows | `test/rev-config-ui-27-configuration-ui-journeys` | `test(slice-27): exercise configuration and UI workflows` | `ops/default/report/slices/rev-configuration-ui/slice-27-configuration-ui-journeys.md` |
| Slice 28 | Harden cross-feature failure boundaries | `test/rev-config-ui-28-configuration-ui-boundaries` | `test(slice-28): verify configuration and UI failure boundaries` | `ops/default/report/slices/rev-configuration-ui/slice-28-configuration-ui-boundaries.md` |
| Slice 29 | Bound startup and completion work | `perf/rev-config-ui-29-configuration-ui-performance` | `perf(slice-29): bound startup and completion work` | `ops/default/report/slices/rev-configuration-ui/slice-29-configuration-ui-performance.md` |
| Slice 30 | Verify delivery acceptance and references | `test/rev-config-ui-30-configuration-ui-acceptance` | `test(slice-30): verify configuration and interaction acceptance` | `ops/default/report/slices/rev-configuration-ui/slice-30-configuration-ui-acceptance.md` |

## Shared implementation rules

The accepted model catalog is exhaustive. Future inventory, runtime reload,
project-local discovery, addons, conversion commands and new grammars do not
become work through sample comments or incidental architecture opportunities.
The explicit future conversion ticket remains open and unscheduled.

Use the existing rope/buffer/view/history/file/command/tree owners. Prefer small
concrete records and direct functions. Preserve safe Rust, shallow modules,
maximum five parameters, cyclomatic complexity at most ten and nesting at most
two, as checked by the existing source gate. No new frontend framework, service,
Node runtime, CLI or generic schema engine is part of this work.

Only toml 1.1.8 and serde 1.0.229 are new direct dependencies selected for this
approved TOML scope; retain GPL-3.0-only for first-party code. Odra is a design
reference, not a framework transplant; no Emacs implementation code is copied.

Follow exact branch/title/report identities. Use Conventional Commits with
existing rev task suffix `(Tn.m)` for task commits; this is a rev-only convention.
One PR per slice targets dev. Follow the global Summary/Changes/Validation body
literally, with an immutable report-introduction commit link. The controller
owns merges. Record nontrivial commits and PR transitions through Acta.

## Task boundaries and acceptance

The ordered task list lives in the single tracker; this section supplies the
technical contract and exact focused validation for each slice. Every slice
also runs format, all-target/all-feature Clippy with warnings denied, the
repository source checker, and git diff --check. Preserve meaningful test
names/comments under repository conventions; avoid duplicated mirror tests.

### Slice 1: Model typed configuration

Contracts: CFG02-CFG06, CFG20. Tickets: TKT-20261027103708.
Prerequisites: committed approved plan/tracker and accepted specs.

- T1.1: Establish a small configuration owner with complete defaults and partial input types for the entire accepted model catalog; absence must survive parsing and language/include merging. Document ownership and numeric semantics without introducing a generic framework.
- T1.2: Pin toml 1.1.8 and serde 1.0.229 with only required parser/derive features; commit Cargo.lock, preserve Rust 1.98 and first-party safe Rust. Create the configuration integration test target and verify explicit false, zero and absent values.
- T1.3: Record source/default/catalog invariants and preparatory status. Do not advertise operational configuration before Slice 20 wires all consumers.

Focused validation:

- `cargo test --test configuration`

### Slice 2: Validate TOML and diagnostics

Contracts: CFG02, CFG04-CFG06, CFG13-CFG14, CFG20. Tickets: TKT-20261027103708.
Prerequisites: Slices 1-1 integrated in order.

- T2.1: Parse UTF-8 TOML 1.1 into typed partial documents, accepting omitted version as 1 and rejecting unknown explicit versions, unknown keys, bad types/enums, invalid RGB and out-of-range/nonfinite numbers.
- T2.2: Validate only accepted language leaves and pattern shapes; retain source/key and spans where available. Reject duplicate keys and non-command binding values without silently defaulting bad input.
- T2.3: Exercise malformed input, Unicode positions, valid empty include arrays and every admitted bound, including soft history limits and zero syntax cutoff. Keep parsing independent of editor mutation.

Focused validation:

- `cargo test --test configuration`

### Slice 3: Load bounded include graphs

Contracts: CFG01, CFG07-CFG08, CFG14, CFG21. Tickets: TKT-20261027103708.
Prerequisites: Slices 1-2 integrated in order.

- T3.1: Resolve independent XDG roots and optional config.toml without creating files/directories; ignore relative XDG values, use HOME fallbacks, and return precise errors when a required root cannot be resolved.
- T3.2: Read explicit includes depth-first in declaration order relative to the declaring source; enforce root-depth zero, depth 16, 64 occurrences, 256 KiB per source and 2 MiB total before unbounded allocation. Detect active-stack symlink/hard-link cycles by file identity.
- T3.3: Test shared acyclic leaves, duplicates counting toward bounds, permission/missing includes, alias cycles and read errors. A missing optional root alone selects defaults; a broken explicit source rejects the candidate.

Focused validation:

- `cargo test --test configuration`

### Slice 4: Resolve layered settings

Contracts: CFG03, CFG07, CFG09, CFG14-CFG15. Tickets: TKT-20261027103708.
Prerequisites: Slices 1-3 integrated in order.

- T4.1: Resolve built-ins, ordered includes and root leaves with table merging and array replacement. Resolve language partials after global layers so a late global value cannot erase an explicit language leaf.
- T4.2: Validate the complete candidate before publishing one immutable startup snapshot; preserve concise per-leaf provenance where useful. A failed candidate yields complete built-ins and one actionable diagnostic, never mixed accepted/rejected values.
- T4.3: Exercise root/include ordering, false/zero/empty values, per-language inheritance and candidate rollback without reading project-local config or evaluating paths.

Focused validation:

- `cargo test --test configuration`

### Slice 5: Identify buffer languages

Contracts: CFG09, CFG18, CFG21. Tickets: TKT-20261027103709.
Prerequisites: Slices 1-4 integrated in order.

- T5.1: Implement the exact built-in basename/suffix map from the model, with user rules overriding built-ins, exact names before suffixes and longest suffix specificity. Identical patterns use normal layer precedence; overlapping extensions are valid.
- T5.2: Keep language identity independent of bundled grammar availability; unknown identities inherit global editing preferences and default to plain syntax. Retain the visible visiting basename when canonical file identity uses an alias.
- T5.3: Cover compound suffixes, .h overrides, exact names, unknown extensions and Go policies without a Go grammar. Association resolution must not read directories or open a new file.

Focused validation:

- `cargo test --test configuration`
- `cargo test --test files`

### Slice 6: Project buffer tab policies

Contracts: CFG09-CFG11. Tickets: TKT-20261027103709.
Prerequisites: Slices 1-5 integrated in order.

- T6.1: Give each buffer one effective editing/display policy from a supplied settings snapshot; replace hardcoded tab assumptions in coordinate conversion, viewport projection, vertical goal columns, selection and search geometry.
- T6.2: Use buffer policy coherently across shared views and invalidate display-derived caches without changing canonical text positions, rope revisions, saved-state identity or history.
- T6.3: Test widths 1, 2, 4, 8 and 32, mixed wide/combining/ZWJ clusters, chunk seams, CRLF and clipped views. Default tab width must preserve baseline geometry.

Focused validation:

- `cargo test --test coordinates`
- `cargo test --test display`
- `cargo test --test windows`

### Slice 7: Apply indentation and new-file defaults

Contracts: CFG10, CFG16. Tickets: TKT-20261027103709.
Prerequisites: Slices 1-6 integrated in order.

- T7.1: Route Tab through the effective policy: literal U+0009 for tabs; enough spaces for the next indent_width stop for spaces. Bracketed paste remains literal and one normal undo operation.
- T7.2: Apply configured LF/CRLF and BOM only when creating a new buffer. Existing files retain recognized BOM, mixed/newline fidelity and final-newline absence through save.
- T7.3: Test nonaligned columns, tabs already in text, wide Unicode, read-only refusal, undo/redo and existing-file save fidelity. Do not implement conversion commands.

Focused validation:

- `cargo test --test editing`
- `cargo test --test files`
- `cargo test --test commands`

### Slice 8: Capture Emacs-style rectangles

Contracts: CFG11, modified R20. Tickets: TKT-20261027103709.
Prerequisites: Slices 1-7 integrated in order.

- T8.1: Separate copy and kill rectangle capture semantics: copy expands retained tabs without source mutation; kill preserves whole interior tabs and materializes only partial edges as required.
- T8.2: Use the source buffer policy for display columns, retain whole-grapheme boundaries and baseline short-line/no-right-padding behavior, and keep linear and rectangular stores distinct.
- T8.3: Port the observed Emacs behavior into original fixtures, not source code: a-tab-b and c-tab-d at width 4, partial edges, wide clusters and a one-transaction kill/undo. Update contradictory copy tests deliberately.

Focused validation:

- `cargo test --test rectangles`
- `cargo test --test coordinates`

### Slice 9: Keep rectangle and path transitions coherent

Contracts: CFG09, CFG11, CFG16-CFG17. Tickets: TKT-20261027103709.
Prerequisites: Slices 1-8 integrated in order.

- T9.1: Yank retained rectangle rows at one left display column under destination tab policy; preserve retained literal tabs rather than forcing the source right edge. Pad only required left gaps and publish one reversible operation.
- T9.2: Resolve policy after successful save-as/file reassociation and tree rename, including visited descendants of a renamed directory, from the same startup snapshot. Failed/cancelled operations retain prior policy.
- T9.3: Test cross-width copy versus kill/yank, shared views, dirty saved-state truth, undo/redo, renamed directories, and existing file identity/fidelity; preserve all path-change guarantees.

Focused validation:

- `cargo test --test rectangles`
- `cargo test --test tree`
- `cargo test --test files`

### Slice 10: Define panel geometry contracts

Contracts: UI01, baseline R05-R07, R14-R15. Tickets: TKT-20261027103711, TKT-20261027085102.
Prerequisites: Slices 1-9 integrated in order.

- T10.1: Introduce the smallest common presentation records for panel outer/content bounds, reserved separator/status cells, clipping and active state; keep Layout, ViewId and buffer ownership authoritative.
- T10.2: Derive separator topology and junctions for nested horizontal/vertical splits, including minimum geometry and future completion-area reservation. Avoid a second window tree or general UI registry.
- T10.3: Create native panel-contract tests for no overlap, bounded coordinates, zero/tiny frames and stable identities. This preparatory layer must compile without changing input semantics.

Focused validation:

- `cargo test --test panels`
- `cargo test --test windows`

### Slice 11: Compose editing panels

Contracts: UI01, baseline R05-R07, R14-R15. Tickets: TKT-20261027103711, TKT-20261027085102.
Prerequisites: Slices 1-10 integrated in order.

- T11.1: Migrate editor rendering to panel content bounds and continuous shared Unicode separators/junctions, with no per-pane outer frame. Reserve horizontal border rows and status cells explicitly in layout minima.
- T11.2: Derive text, cursor, syntax, search, selection and status translations from the same geometry owner; remove repeated ASCII-pipe painting as an independent placement convention.
- T11.3: Test nested split directions, active/inactive panes, same-buffer independent points, clipped Unicode and resize. Keep the baseline default split proportion and focus traversal.

Focused validation:

- `cargo test --test panels`
- `cargo test --test display`
- `cargo test --test windows`

### Slice 12: Compose tree and auxiliary panels

Contracts: UI01, CFG19. Tickets: TKT-20261027103711, TKT-20261027085102.
Prerequisites: Slices 1-11 integrated in order.

- T12.1: Migrate tree content, separator, status and selection to the panel contract; remove string-prefix geometry and duplicated syntax/selection offset arithmetic.
- T12.2: Reserve an auxiliary completion panel through the same composition interface while keeping tree focus and semantic state separate. Hiding a pane must not destroy buffers or pending prompts.
- T12.3: Test tree plus nested splits, narrow hiding, focus repair, reopen/resize and exact cell clipping. No Odra registry/framework or unrelated tree feature is required.

Focused validation:

- `cargo test --test panels`
- `cargo test --test tree`
- `cargo test --test windows`

### Slice 13: Resolve palette and syntax colors

Contracts: CFG04, CFG18, UI01. Tickets: TKT-20261027103710, TKT-20261027085102.
Prerequisites: Slices 1-12 integrated in order.

- T13.1: Implement all 25 selected palette roles using the pinned Tokyo Night Neovim night values, including all seven existing syntax classes. Every present leaf overrides only itself.
- T13.2: Replace hardcoded syntax ANSI indices with configured RGB style resolution through existing captures. Preserve capture mapping, incremental parsing and plain-text fallback; add no grammars.
- T13.3: Test all default values, single-role overrides, invalid RGB rejection, selected text precedence over syntax and Rust/plain policy changes without touching document/history.

Focused validation:

- `cargo test --test configuration`
- `cargo test --test syntax`
- `cargo test --test display`

### Slice 14: Render UI palette and restore terminal

Contracts: CFG04, CFG14, UI01, baseline R03. Tickets: TKT-20261027103710, TKT-20261027085102.
Prerequisites: Slices 1-13 integrated in order.

- T14.1: Apply background/foreground, status, border/focus, tree, region, search, prompt, warning and error roles through composition. Ensure every selected role has a real consumer.
- T14.2: Apply cursor color through supported terminal sequences, with a documented capability fallback; restore cursor/text colors and terminal modes on normal exit and failures through the existing lifecycle owner.
- T14.3: Test safe escape emission, no style leakage between panels, readable selections and restoration after errors. Keep idle rendering event-driven and do not add blinking.

Focused validation:

- `cargo test --test display`
- `cargo test --test panels`
- `cargo test --test recovery`

### Slice 15: Apply presentation and syntax preferences

Contracts: CFG18-CFG19, UI01. Tickets: TKT-20261027103711.
Prerequisites: Slices 1-14 integrated in order.

- T15.1: Apply page_overlap with clamped forward progress, preferred split_ratio with geometry minima, tree visibility/width/max_fraction/root, hidden entry filtering and directory-first ordering with deterministic filename ties.
- T15.2: Apply syntax enabled/max_bytes and supported per-language rust/plain grammar selection; crossing a cutoff invalidates stale styling without preventing edits or changing text/history.
- T15.3: Test default equivalence, valid extremes, tiny geometry, dotfiles, initial file versus scratch root, retained tree focus and no filesystem reads during drawing. Do not add natural sorting or extra display controls.

Focused validation:

- `cargo test --test windows`
- `cargo test --test tree`
- `cargo test --test syntax`

### Slice 16: Apply bounded history and prompt preferences

Contracts: CFG17, CFG20, CFG22. Tickets: TKT-20261027103712.
Prerequisites: Slices 1-15 integrated in order.

- T16.1: Apply whole-group history count and soft payload target through the existing history owner, preserving the newest coherent transaction and truthful saved state.
- T16.2: Apply positive linear-ring capacity and directional coalescing; apply command history/input limits through existing prompt owners. No session persistence or time-based grouping is introduced.
- T16.3: Test retention below one compound rectangle, ring eviction, coalescing disabled, bounded Unicode prompt input and cancellation after an over-limit error. Avoid tests that only mirror field assignment.

Focused validation:

- `cargo test --test editing`
- `cargo test --test commands`
- `cargo test --test rectangles`

### Slice 17: Validate context keymaps

Contracts: CFG06, CFG12-CFG13, modified R22. Tickets: TKT-20261027103713.
Prerequisites: Slices 1-16 integrated in order.

- T17.1: Audit the real editor/minibuffer/tree default maps and fixed action identities. Parse configured chord sequences through one normalization vocabulary including terminal aliases and prefix sequences.
- T17.2: Validate inherited-plus-user maps, exact replacement, command availability in context and leaf/prefix conflicts before activation. Existing prompt/tree actions may gain stable catalog names, but configuration never supplies executable bodies.
- T17.3: Keep cancellation reachable: preserve C-g as the transient cancel chord and reject attempts that remove that escape path. Reject non-command/unbinding values and explain terminal-indistinguishable chords. Document every default mapping.

Focused validation:

- `cargo test --test configuration`
- `cargo test --test commands`

### Slice 18: Dispatch configurable editor bindings

Contracts: CFG12-CFG13, modified R22. Tickets: TKT-20261027103713.
Prerequisites: Slices 1-17 integrated in order.

- T18.1: Route editor chords and prefixes through the validated effective map and the same Rust handler identities as M-x. Retain built-in aliases, Meta/Escape timing and literal paste semantics.
- T18.2: Apply exact user reassignment without deleting any inherited binding implicitly; commenting an override restores its built-in. Preserve prefix cancellation and reset across context changes.
- T18.3: Test aliases such as M-j to next-line, replacement, multikey sequences, unknown commands, prefix collisions, terminal aliases, read-only refusal and exactly-once dispatch.

Focused validation:

- `cargo test --test commands`
- `cargo test --test editing`

### Slice 19: Dispatch transient and tree bindings

Contracts: CFG12-CFG13, UI02-UI03. Tickets: TKT-20261027103713.
Prerequisites: Slices 1-18 integrated in order.

- T19.1: Wire validated minibuffer/tree maps to their existing action owners without bypassing argument, search, replace, confirmation or tree mutation state machines.
- T19.2: Keep C-g cancellation reachable and destructive confirmations explicit. Reassignment in one context cannot leak into editor text or change unrelated context defaults.
- T19.3: Test context isolation, prefix reset, prompt Unicode editing, tree focus/navigation and cancellation through find-file, M-x, query-replace and pending filesystem decisions.

Focused validation:

- `cargo test --test commands`
- `cargo test --test tree`
- `cargo test --test search`

### Slice 20: Activate startup configuration

Contracts: CFG01-CFG22, modified R01, R17, R20, R22. Tickets: TKT-20261027103708.
Prerequisites: Slices 1-19 integrated in order.

- T20.1: Wire XDG discovery, bounded loading, whole-candidate validation and one immutable snapshot into the actual executable/session before buffer creation; connect every accepted consumer from prior slices.
- T20.2: On invalid or unreadable config, enter a usable editor with complete built-ins and a visible source/key diagnostic. Missing config stays silent. Never write config, create empty XDG roots or load project-local files.
- T20.3: Test new startup versus unchanged running session after disk edits, palette/tab/key behavior through session boundaries and rollback of otherwise valid settings when one leaf is invalid. No reload command, polling or watcher.

Focused validation:

- `cargo test --test configuration`
- `cargo test --test commands`
- `cargo test --test files`
- `cargo test --test recovery`

### Slice 21: Present vertical M-x candidates

Contracts: UI02, CFG04, CFG19, CFG22. Tickets: TKT-20261027084156.
Prerequisites: Slices 1-20 integrated in order.

- T21.1: Replace horizontal joined hints with query-plus-vertical candidate rows in the auxiliary panel. Apply configured row limit and matching mode with stable rank/order, safe Unicode clipping and explicit empty results.
- T21.2: Retain selected row and scroll origin independently of editable query; draw selected command clearly using semantic palette roles, without rendering-side filesystem reads or dispatch.
- T21.3: Test many matches, query filtering, zero available candidate rows, tree/nested panes, resize and draft/selection retention. Preserve the underlying editing viewport contract.

Focused validation:

- `cargo test --test commands`
- `cargo test --test panels`
- `cargo test --test display`

### Slice 22: Navigate and execute command candidates

Contracts: UI02, CFG12-CFG13, baseline R23. Tickets: TKT-20261027084156.
Prerequisites: Slices 1-21 integrated in order.

- T22.1: Bind Up/Down and C-p/C-n to candidates, M-p/M-n to command history, Tab to complete the selected name and Return to the same named handler exactly once.
- T22.2: Define exact/no-match/empty-query behavior, reset selection coherently after query changes and keep the selected row visible. Continue argument prompts normally; C-g restores the previous editing/tree focus.
- T22.3: Test history versus candidate navigation, no accidental first-command execution, custom context aliases, cancellation and selected handler invocation with arguments.

Focused validation:

- `cargo test --test commands`
- `cargo test --test panels`

### Slice 23: Observe bounded filesystem candidates

Contracts: UI03-UI04, baseline R10-R13. Tickets: TKT-20261027085234.
Prerequisites: Slices 1-22 integrated in order.

- T23.1: Create a filesystem candidate provider for one current directory component with 4096-entry and 512-KiB-name-byte limits, safe retained path identities, directories-first byte-name ordering and cached filtering.
- T23.2: Handle relative/absolute paths, leading ~/, spaces and Unicode without shell expansion; separate lossless path identity from clipped/control-safe labels. Report non-UTF-8 names explicitly instead of opening lossy paths.
- T23.3: Create provider tests for ambiguous/common/unique prefixes, hidden names, symlinks, permission/disappearance errors and limits. No recursion, watcher or reads during rendering/filtering.

Focused validation:

- `cargo test --test path_completion`

### Slice 24: Complete and navigate file paths

Contracts: UI03-UI04, CFG12-CFG13. Tickets: TKT-20261027085234.
Prerequisites: Slices 1-23 integrated in order.

- T24.1: Initialize find-file in the current visiting file parent or cwd, and integrate the provider with the common vertical completion panel; show directory markers without modifying the actual path bytes.
- T24.2: Implement Tab unique/shared-prefix completion, directory descent, editable parent navigation and candidate selection. Invalidate cached observations only for directory changes or explicit completion refresh.
- T24.3: Test ./, ../, ~/, absolute names, spaces, Unicode boundaries, trailing separators, changing prefixes, incomplete lists, tiny resize and C-g with draft/focus retained.

Focused validation:

- `cargo test --test path_completion`
- `cargo test --test commands`
- `cargo test --test panels`

### Slice 25: Open selected and new file paths safely

Contracts: UI03-UI04, CFG09, CFG16. Tickets: TKT-20261027085234.
Prerequisites: Slices 1-24 integrated in order.

- T25.1: Unify C-x C-f and M-x find-file: explicit selected-candidate Return accepts that identity; freely typed Return accepts the draft/new filename; an accepted directory descends without visiting a text buffer.
- T25.2: Open through existing canonical visit/file ownership, reusing dirty buffers and applying startup language/new-file policies. Candidate observation/completion cannot create files or publish saves.
- T25.3: Test revisit through aliases, relative canonical reuse, nonexistent file creation only in memory, failed/unreadable paths retaining prompt and active buffer, symlink destinations and no accidental save. Scope stays find-file.

Focused validation:

- `cargo test --test path_completion`
- `cargo test --test files`
- `cargo test --test commands`
- `cargo test --test recovery`

### Slice 26: Publish exhaustive configuration reference

Contracts: CFG05-CFG06, CFG20, CFG22, UI01-UI04. Tickets: TKT-20261027103714.
Prerequisites: Slices 1-25 integrated in order.

- T26.1: Create docs/reference/config.toml with every accepted option, all 25 palette values active, complete default bindings for all three contexts and commented supported optional include/language/association examples. Keep all comments English.
- T26.2: Document types, bounds, units, scopes, defaults, precedence, invalid-config fallback, startup-only semantics, terminal limitations and the three new UI flows in existing docs. Keep future controls out of the operational sample.
- T26.3: Add deterministic schema/default/sample and command-map consistency checks. Validate actual runtime effects separately; parsing a sample is not UI acceptance. Preserve the concise README product/build style.

Focused validation:

- `cargo test --test configuration`
- `python3 scripts/check-config-reference.py`

### Slice 27: Exercise combined terminal workflows

Contracts: CFG22, UI01-UI04, baseline R03-R07. Tickets: TKT-20261027103715.
Prerequisites: Slices 1-26 integrated in order.

- T27.1: Add real isolated PTY journeys for startup palette/tab/key overrides, missing and invalid config, M-x candidate/history selection, find-file directory/Unicode completion and new filename behavior.
- T27.2: Combine tree/nested panels, shared buffer policies, horizontal and vertical boundaries, focus and tiny/growing resize. Assert visible behavior and retained document text, not only escape substring presence.
- T27.3: Use bounded fixtures/timeouts, zero skips and explicit child cleanup after completion. Verify normal/error terminal color/mode restoration and no disk entry created by observation.

Focused validation:

- `python3 scripts/check-configuration-ui.py --suite terminal`
- `cargo test --test configuration`
- `cargo test --test path_completion`

### Slice 28: Harden cross-feature failure boundaries

Contracts: CFG01-CFG22, UI01-UI04. Tickets: TKT-20261027103715.
Prerequisites: Slices 1-27 integrated in order.

- T28.1: Complete adversarial coverage of include cycles/limits/read failures, layered key collisions, association overlaps, whole-candidate rollback and candidate-list stale filesystem state.
- T28.2: Exercise cross-width rectangle copy/kill/yank, successful and failed file/directory rename policies, CRLF/BOM fidelity, shared views and undo/saved-state truth with configured limits.
- T28.3: Repair only bounded defects in approved owners revealed by these checks; no new features or duplicated blanket test suite. Map each failure boundary to the requirement it can falsify.

Focused validation:

- `cargo test --test configuration`
- `cargo test --test path_completion`
- `cargo test --test rectangles`
- `cargo test --test recovery`

### Slice 29: Bound startup and completion work

Contracts: CFG08, CFG17-CFG19, UI04, baseline R32-R35. Tickets: TKT-20261027103715.
Prerequisites: Slices 1-28 integrated in order.

- T29.1: Extend reproducible performance fixtures for maximal admitted include graphs, directory candidate limits/filter reuse, panel redraw with tabbed Unicode and syntax cutoff behavior.
- T29.2: Check event-driven idle behavior, bounded config/candidate allocations and absence of recursive directory work or filesystem reads on draw. Retain the original load/search/edit/scroll/syntax budgets and fixtures.
- T29.3: Record environment, workload sizes, measured values and enforced budgets. Fix demonstrated regressions within scope; do not add speculative tuning or weaken a failing budget.

Focused validation:

- `python3 scripts/check-configuration-ui.py --suite performance`
- `python3 scripts/check-performance.py`

### Slice 30: Verify delivery acceptance and references

Contracts: CFG01-CFG22, modified R01/R17/R20/R22, UI01-UI04, baseline A01-A20. Tickets: TKT-20261027103715.
Prerequisites: Slices 1-29 integrated in order.

- T30.1: Add a requirement-to-native/PTY/performance evidence map for this delivery and require it from scripts/check-all.sh, retaining all original eight checks and A01-A20 evidence protections. Detect missing cases, skipped suites, stale candidate identity or absent slice reports.
- T30.2: Reconcile selected baseline statements/default tables and rectangle references with accepted config/interaction contracts, keeping baseline history distinct from the new implementation evidence. Verify sample, docs and all eleven ticket outcomes agree with actual delivered behavior.
- T30.3: Prepare all 30 reports, resolved ticket paths and acceptance prerequisites; do not run the full gate in the worker. After controller-owned exact-dev full-gate success, close only approved artifacts using that evidence, record Acta outcome and stop without main promotion.

Focused validation:

- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 scripts/check-config-reference.py`

## Tickets and acceptance ownership

The following eleven tickets are the complete current delivery input. Each is
promoted from ready through its lifecycle as its assigned slices progress. A
ticket spanning several slices records multiple PRs and is solved only when all
assigned work has integrated; do not invent a separate ticket branch. Once the
status directory changes, update references in this delivery accordingly.

| Ticket | Requirements | Required slices |
| --- | --- | --- |
| [TKT-20261027103708](../ticket/ready/20261027103708-configuration-loading.md) | CFG01-CFG03, CFG07-CFG08, CFG14-CFG15, CFG21 | 1, 2, 3, 4, 20 |
| [TKT-20261027103709](../ticket/ready/20261027103709-configuration-buffer-policies.md) | CFG09-CFG11, CFG16, modified R17/R20 | 5, 6, 7, 8, 9 |
| [TKT-20261027103710](../ticket/ready/20261027103710-configuration-palette.md) | CFG04, CFG18 | 13, 14 |
| [TKT-20261027103711](../ticket/ready/20261027103711-configuration-presentation.md) | CFG18-CFG20 | 10, 11, 12, 15 |
| [TKT-20261027103712](../ticket/ready/20261027103712-configuration-retention.md) | CFG17, CFG20 | 16 |
| [TKT-20261027103713](../ticket/ready/20261027103713-configuration-keybindings.md) | CFG06, CFG12-CFG13, modified R22 | 17, 18, 19 |
| [TKT-20261027103714](../ticket/ready/20261027103714-configuration-reference.md) | CFG05-CFG06, CFG20 | 26 |
| [TKT-20261027103715](../ticket/ready/20261027103715-configuration-acceptance.md) | CFG22, UI01-UI04 | 27, 28, 29, 30 |
| [TKT-20261027084156](../ticket/ready/20261027084156-vertical-command-completion.md) | UI02 | 21, 22 |
| [TKT-20261027085102](../ticket/ready/20261027085102-panel-presentation-abstraction.md) | UI01 | 10, 11, 12, 13, 14 |
| [TKT-20261027085234](../ticket/ready/20261027085234-file-path-completion.md) | UI03-UI04 | 23, 24, 25 |

Each CFG01-CFG22 and UI01-UI04 requirement must map to concrete native tests and,
where observable in a terminal, PTY assertions in the final acceptance map.
Every MODIFIED baseline requirement in the configuration delta must be reconciled
as a complete requirement, not a partial replacement of its prior guarantees.
Keep R05-R07/R12/R14-R15/R23 consistent with UI01-UI04 without claiming additional
grammars, packages or extensibility. Preserve baseline R02-R40 and A01-A20 checks
except deliberate accepted changes to defaults, tab/rectangle behavior and UI.

## Delivery gate and cost

The frozen full-gate command is scripts/check-all.sh. Extend its owned acceptance
orchestration in Slice 30 to require both original baseline evidence and the new
configuration/UI evidence, on one clean exact dev candidate. Retain its original
format, Clippy, full native tests, source checks, original PTY acceptance,
performance, acceptance-evidence and whitespace checks. Add the new sample
consistency, terminal and bounded-work suites without making earlier evidence
or nonempty assertions optional. Existing acceptance indices/identity handling
must be adapted explicitly when adding stages, not bypassed.

New required commands are:

- python3 scripts/check-config-reference.py
- python3 scripts/check-configuration-ui.py --suite terminal
- python3 scripts/check-configuration-ui.py --suite performance
- python3 scripts/check-configuration-ui.py --suite prerequisites

The full gate runs once after Slice 30 is integrated, through the controller;
workers prepare prerequisites and do not duplicate it. Existing baseline cost
is several minutes including builds, 455 native tests, four PTY journeys and
two performance fixtures; this set adds targeted suites and bounded directory/
configuration fixtures. Actual runtime is recorded, not estimated as duration.

Per-slice checks are exactly the controller's required_commands prefix, followed
by git diff --check. Focused native targets avoid repeating cargo test --all and
the complete historical terminal/performance matrix for every slice. Reuse
validation only for an unchanged clean HEAD and the exact ordered commands.
No skipped cases, disabled gates, result caching across changed heads or
unsupported schema keys count as successful validation.

If the final gate reveals an integration defect, the standard bounded validation
repair workflow may be proposed under the same scope; the controller cannot
invent repair PR authority. A gate failure blocks closure until explicitly
authorized repair/retry evidence succeeds. Do not claim success from 30 merges.

## Closure

After the unchanged required full gate reports its exact dev candidate green,
close this plan, tracker, accepted spec/model and interaction spec and record
the gate identity/candidate evidence. All 30 delivered slice reports, eleven
solved tickets, sample/catalog consistency and requirement evidence are required.
The gate candidate and any later documentation-only closure commit must remain
distinguishable. Historical rev-editor reports and its completed run remain
unchanged; configuration/UI closure is a separate outcome. The future conversion
ticket stays open. Main promotion and public mirror publication wait for their
own maintainer instruction.
