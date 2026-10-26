# rev Editor Delivery

Status: approved
Approved: product owner, 2026-10-26; complete baseline implementation and Looper startup requested.
Delivery set: rev-editor
Slice strategy: behavior-first
Active slice: Slice 20 - Handle tiny and changing terminal geometry
Full gate: `scripts/check-all.sh`
Spec: [rev Editor](../spec/rev-editor.md)
Model: [rev Editor Model](../spec/rev-editor-model.md)
Tracker: [rev Editor Tracker](../tracker/rev-editor.md)

## Outcome and strategy

Deliver the entire accepted terminal editor, satisfying R01-R40 and A01-A20,
through one approved Looper run. The 48 slices follow text ownership, editing,
file safety, views, interaction, tree, syntax, and real acceptance boundaries.
Initial native contracts support executable behavior immediately; later slices
add complete capabilities rather than disconnected scaffolding. The slice count
follows those reviewable boundaries and is not a fixed curriculum or a quota.

Prerequisites delivered: Git bootstrap, canonical origin, accepted specification,
accepted state model, and owner authorization. The runtime package and validation
scripts are intentionally implemented by Slice 1. No external product decision
or optional Ruru/Ruby capability blocks this baseline.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Own terminal lifetime | `feat/rev-01-terminal` | `feat(slice-1): own terminal lifetime` | `ops/default/report/slices/rev-editor/slice-1-terminal.md` |
| Slice 2 | Introduce canonical rope buffers | `feat/rev-02-buffers` | `feat(slice-2): introduce canonical rope buffers` | `ops/default/report/slices/rev-editor/slice-2-buffers.md` |
| Slice 3 | Resolve Unicode and display coordinates | `feat/rev-03-coordinates` | `feat(slice-3): resolve unicode and display coordinates` | `ops/default/report/slices/rev-editor/slice-3-coordinates.md` |
| Slice 4 | Publish coherent text edits | `feat/rev-04-edits` | `feat(slice-4): publish coherent text edits` | `ops/default/report/slices/rev-editor/slice-4-edits.md` |
| Slice 5 | Decode prefixes and focused prompts | `feat/rev-05-input` | `feat(slice-5): decode prefixes and focused prompts` | `ops/default/report/slices/rev-editor/slice-5-input.md` |
| Slice 6 | Render one rope-backed viewport | `feat/rev-06-display` | `feat(slice-6): render one rope-backed viewport` | `ops/default/report/slices/rev-editor/slice-6-display.md` |
| Slice 7 | Edit printable text and graphemes | `feat/rev-07-insertion` | `feat(slice-7): edit printable text and graphemes` | `ops/default/report/slices/rev-editor/slice-7-insertion.md` |
| Slice 8 | Implement character and word motion | `feat/rev-08-motion` | `feat(slice-8): implement character and word motion` | `ops/default/report/slices/rev-editor/slice-8-motion.md` |
| Slice 9 | Preserve vertical motion and scrolling | `feat/rev-09-scrolling` | `feat(slice-9): preserve vertical motion and scrolling` | `ops/default/report/slices/rev-editor/slice-9-scrolling.md` |
| Slice 10 | Restore per-buffer undo and redo | `feat/rev-10-history` | `feat(slice-10): restore per-buffer undo and redo` | `ops/default/report/slices/rev-editor/slice-10-history.md` |
| Slice 11 | Group typing and compound history | `feat/rev-11-grouping` | `feat(slice-11): group typing and compound history` | `ops/default/report/slices/rev-editor/slice-11-grouping.md` |
| Slice 12 | Load exact UTF-8 file content | `feat/rev-12-file-ingress` | `feat(slice-12): load exact utf-8 file content` | `ops/default/report/slices/rev-editor/slice-12-file-ingress.md` |
| Slice 13 | Reuse canonical visited files | `feat/rev-13-file-visits` | `feat(slice-13): reuse canonical visited files` | `ops/default/report/slices/rev-editor/slice-13-file-visits.md` |
| Slice 14 | Publish safe same-directory saves | `feat/rev-14-saving` | `feat(slice-14): publish safe same-directory saves` | `ops/default/report/slices/rev-editor/slice-14-saving.md` |
| Slice 15 | Confirm overwrite and external changes | `feat/rev-15-save-decisions` | `feat(slice-15): confirm overwrite and external changes` | `ops/default/report/slices/rev-editor/slice-15-save-decisions.md` |
| Slice 16 | Switch list and close buffers | `feat/rev-16-buffer-commands` | `feat(slice-16): switch list and close buffers` | `ops/default/report/slices/rev-editor/slice-16-buffer-commands.md` |
| Slice 17 | Create nested editing splits | `feat/rev-17-splits` | `feat(slice-17): create nested editing splits` | `ops/default/report/slices/rev-editor/slice-17-splits.md` |
| Slice 18 | Delete and cycle editing windows | `feat/rev-18-window-commands` | `feat(slice-18): delete and cycle editing windows` | `ops/default/report/slices/rev-editor/slice-18-window-commands.md` |
| Slice 19 | Map shared-buffer edits across views | `feat/rev-19-shared-views` | `feat(slice-19): map shared-buffer edits across views` | `ops/default/report/slices/rev-editor/slice-19-shared-views.md` |
| Slice 20 | Handle tiny and changing terminal geometry | `feat/rev-20-resize` | `feat(slice-20): handle tiny and changing terminal geometry` | `ops/default/report/slices/rev-editor/slice-20-resize.md` |
| Slice 21 | Activate and render linear regions | `feat/rev-21-regions` | `feat(slice-21): activate and render linear regions` | `ops/default/report/slices/rev-editor/slice-21-regions.md` |
| Slice 22 | Kill lines and words directionally | `feat/rev-22-kills` | `feat(slice-22): kill lines and words directionally` | `ops/default/report/slices/rev-editor/slice-22-kills.md` |
| Slice 23 | Copy kill and yank linear text | `feat/rev-23-kill-ring` | `feat(slice-23): copy kill and yank linear text` | `ops/default/report/slices/rev-editor/slice-23-kill-ring.md` |
| Slice 24 | Rotate only the current yank range | `feat/rev-24-yank-pop` | `feat(slice-24): rotate only the current yank range` | `ops/default/report/slices/rev-editor/slice-24-yank-pop.md` |
| Slice 25 | Resolve Meta Escape and undo aliases | `feat/rev-25-terminal-keys` | `feat(slice-25): resolve meta escape and undo aliases` | `ops/default/report/slices/rev-editor/slice-25-terminal-keys.md` |
| Slice 26 | Insert bracketed Unicode paste atomically | `feat/rev-26-paste` | `feat(slice-26): insert bracketed unicode paste atomically` | `ops/default/report/slices/rev-editor/slice-26-paste.md` |
| Slice 27 | Run discoverable commands through M-x | `feat/rev-27-command-palette` | `feat(slice-27): run discoverable commands through m-x` | `ops/default/report/slices/rev-editor/slice-27-command-palette.md` |
| Slice 28 | Complete focused argument and cancel flows | `feat/rev-28-prompt-flows` | `feat(slice-28): complete focused argument and cancel flows` | `ops/default/report/slices/rev-editor/slice-28-prompt-flows.md` |
| Slice 29 | Search forward across rope chunks | `feat/rev-29-forward-search` | `feat(slice-29): search forward across rope chunks` | `ops/default/report/slices/rev-editor/slice-29-forward-search.md` |
| Slice 30 | Search backward with smart case and cancel | `feat/rev-30-search-control` | `feat(slice-30): search backward with smart case and cancel` | `ops/default/report/slices/rev-editor/slice-30-search-control.md` |
| Slice 31 | Confirm literal replacements interactively | `feat/rev-31-query-replace` | `feat(slice-31): confirm literal replacements interactively` | `ops/default/report/slices/rev-editor/slice-31-query-replace.md` |
| Slice 32 | Select and copy display-column rectangles | `feat/rev-32-rectangle-copy` | `feat(slice-32): select and copy display-column rectangles` | `ops/default/report/slices/rev-editor/slice-32-rectangle-copy.md` |
| Slice 33 | Kill rectangles as one transaction | `feat/rev-33-rectangle-kill` | `feat(slice-33): kill rectangles as one transaction` | `ops/default/report/slices/rev-editor/slice-33-rectangle-kill.md` |
| Slice 34 | Yank blocks with coherent padding and undo | `feat/rev-34-rectangle-yank` | `feat(slice-34): yank blocks with coherent padding and undo` | `ops/default/report/slices/rev-editor/slice-34-rectangle-yank.md` |
| Slice 35 | Render and focus the left file tree | `feat/rev-35-tree-pane` | `feat(slice-35): render and focus the left file tree` | `ops/default/report/slices/rev-editor/slice-35-tree-pane.md` |
| Slice 36 | Traverse directories lazily and safely | `feat/rev-36-tree-navigation` | `feat(slice-36): traverse directories lazily and safely` | `ops/default/report/slices/rev-editor/slice-36-tree-navigation.md` |
| Slice 37 | Open and create tree entries | `feat/rev-37-tree-create` | `feat(slice-37): open and create tree entries` | `ops/default/report/slices/rev-editor/slice-37-tree-create.md` |
| Slice 38 | Reconcile visited paths after tree rename | `feat/rev-38-tree-rename` | `feat(slice-38): reconcile visited paths after tree rename` | `ops/default/report/slices/rev-editor/slice-38-tree-rename.md` |
| Slice 39 | Confirm deletion while preserving open text | `feat/rev-39-tree-delete` | `feat(slice-39): confirm deletion while preserving open text` | `ops/default/report/slices/rev-editor/slice-39-tree-delete.md` |
| Slice 40 | Bundle Rust syntax highlighting | `feat/rev-40-rust-highlights` | `feat(slice-40): bundle rust syntax highlighting` | `ops/default/report/slices/rev-editor/slice-40-rust-highlights.md` |
| Slice 41 | Update syntax from exact rope edits | `feat/rev-41-incremental-syntax` | `feat(slice-41): update syntax from exact rope edits` | `ops/default/report/slices/rev-editor/slice-41-incremental-syntax.md` |
| Slice 42 | Complete guarded multi-buffer session exit | `feat/rev-42-session-quit` | `feat(slice-42): complete guarded multi-buffer session exit` | `ops/default/report/slices/rev-editor/slice-42-session-quit.md` |
| Slice 43 | Harden ordinary failure boundaries | `fix/rev-43-failure-recovery` | `fix(slice-43): harden ordinary failure boundaries` | `ops/default/report/slices/rev-editor/slice-43-failure-recovery.md` |
| Slice 44 | Drive user journeys through a PTY | `test/rev-44-terminal-journeys` | `test(slice-44): drive user journeys through a pty` | `ops/default/report/slices/rev-editor/slice-44-terminal-journeys.md` |
| Slice 45 | Exercise cross-feature Unicode and file fidelity | `test/rev-45-adversarial-text` | `test(slice-45): exercise cross-feature unicode and file fidelity` | `ops/default/report/slices/rev-editor/slice-45-adversarial-text.md` |
| Slice 46 | Measure representative editing workloads | `perf/rev-46-performance` | `perf(slice-46): measure representative editing workloads` | `ops/default/report/slices/rev-editor/slice-46-performance.md` |
| Slice 47 | Audit source quality and product reference | `refactor/rev-47-quality-reference` | `refactor(slice-47): audit source quality and product reference` | `ops/default/report/slices/rev-editor/slice-47-quality-reference.md` |
| Slice 48 | Verify the complete integrated editor contract | `test/rev-48-acceptance` | `test(slice-48): verify the complete integrated editor contract` | `ops/default/report/slices/rev-editor/slice-48-acceptance.md` |

## Detailed implementation inputs

- [Part 1: Core and terminal](rev-editor/01-core-and-terminal.md), covering 1-8.
- [Part 2: Files and history](rev-editor/02-files-and-history.md), covering 9-16.
- [Part 3: Windows and regions](rev-editor/03-windows-and-regions.md), covering 17-24.
- [Part 4: Commands and search](rev-editor/04-commands-and-search.md), covering 25-32.
- [Part 5: Rectangles and tree](rev-editor/05-rectangles-and-tree.md), covering 33-40.
- [Part 6: Syntax and acceptance](rev-editor/06-syntax-and-acceptance.md), covering 41-48.

Read the current part and active slice section before implementation. Read the
shared decisions and relevant specification/model contracts; do not load the
entire reference corpus or duplicate these identities into another tracker.

## Pinned implementation decisions

These resolve the paired model's planning inputs within the approved scope.

| ID | Decision |
| --- | --- |
| P01 | Rust edition 2024, compiler target 1.98.0, one executable/library package. Pin direct dependencies exactly and commit Cargo.lock. |
| P02 | `crossterm = "=0.29.0"`, `ropey = "=1.6.1"`, `unicode-segmentation = "=1.13.3"`, `unicode-width = "=0.2.2"`, `tree-sitter = "=0.27.1"`, `tree-sitter-rust = "=0.24.2"`. Versions were checked against the primary crates.io API on 2026-10-26. Use safe APIs and Tree-sitter's reexported StreamingIterator; no extra direct iterator/highlighter dependency is needed. |
| P03 | Existing visits canonicalize to the referent. An absent file uses a normalized absolute intended path, resolving its existing parent where possible; opening does not create missing parents. Save through an existing symlink targets its referent and preserves the link entry. Confirm an occupied write-file target; disclose a different resolved referent. Refuse write-file onto a different live visiting buffer's canonical target. |
| P04 | Strict UTF-8 and NUL-based conservative binary rejection. Keep original LF/CRLF bytes in the rope; inserted separators follow the prevailing count, with LF on ties and for new files. Recognized leading BOM is format metadata and emitted exactly once. Preserve final-newline absence unless edited. |
| P05 | Point uses right insertion affinity, mark uses left affinity, and positions within removed spans collapse to the replacement boundary with valid grapheme adjustment. The initiating edit sets its intended cursor explicitly; all other live views map independently. Revisions qualify prepared ranges; logical history states qualify clean save points. |
| P06 | D04 includes whole touched wide graphemes. A partially touched tab expands only the touched tab into its existing display spaces for precise copy/kill; untouched tabs retain bytes. Copy adds no cells beyond source EOL. Rectangle insertion snaps to the start of a touched target grapheme, pads only the short target prefix, and creates required lines in one transaction. |
| P07 | Conventional per-buffer undo/redo. New edit clears redo. Group adjacent typing only with valid command/view continuity. Retain up to 512 complete groups with a soft 64 MiB logical payload target; prune complete oldest groups, always retaining the newest coherent operation even if it alone exceeds that soft target. Preserve correctness of the saved-state marker after pruning. |
| P08 | Synchronous same-directory temporary save is sufficient. Finish required writes/flush/sync before replacement; sync the parent where supported for a durability claim. Distinguish no publication, complete publication, and published-but-unresolved outcomes. Only complete saves satisfy required lifecycle saving; retain actual path/publication facts after later failures. |
| P09 | Normalize reported Alt directly. A standalone Escape begins a finite 35 ms Meta-disambiguation window serviced by event polling, not a blocking sleep. C-SPC may normalize reportable NUL/C-@ aliases. Support reportable C-/, C-_, and C-x u; document actual PTY/terminal encodings without weakening required semantic tests. |
| P10 | Four-cell tab stops, no wrap, control text rendered visibly without raw escape execution, status uses one-based logical line and display column. LF ends logical lines; CRLF remains one separator. Lone CR and other controls remain visible within a line. Ropey uses `default-features = false` with `simd`, since its default Unicode/CR line features turn controls into line breaks. A normal editing pane needs at least three text columns and one text row plus its status; zero/tiny geometry suspends/clips presentation safely. Tree width is at most 24 columns and at most one third of the frame, hidden below 32 frame columns. Traverse editing leaves in deterministic first-child order; tree focus is separate. |
| P11 | Literal Unicode smart-case search compares case-insensitively when the query has no uppercase character and maps matches to original rope positions. Scan chunks without a whole-file folded copy. At search boundaries show failure without implicit wrap; repeat moves within the chosen direction. Each accepted query-replace occurrence is one undoable transaction. |
| P12 | A session-local 32-entry linear ring and a separate rectangle store. Prompt history is bounded at 128 entries; prompt input is bounded at 4096 Unicode scalars with a visible rejection rather than silent truncation. No session persistence, OS clipboard integration, language completion, dynamic theme, or extension runtime. |
| P13 | Tree roots follow the accepted default. Sort entries deterministically, load children lazily, treat symlink directories as leaves, and never recurse through symlink referents during deletion. Update visited descendant paths after successful directory rename. Removed visiting paths retain their ropes and require an explicit later save to recreate files. |
| P14 | Bundle Rust LANGUAGE and HIGHLIGHTS_QUERY from the pinned grammar. Use retained Parser/Tree plus safe query capture APIs, incremental edits in bytes, visible-range highlighting, revision-qualified caches, and safe plain-text fallback. Never put a full-rope String clone in the per-key path. |
| P15 | Use the Rust test runner and Python standard library for PTY/fault/performance orchestration. Do not add external TUI, async, snapshot-test, mocking, AST-analysis, or service dependencies. Source-quality checks must tokenize strings/comments safely, check first-party unsafe/parameter/nesting rules, and measure cyclomatic paths or accompany explicit disciplined branch-count evidence. Clippy cognitive complexity alone is not a cyclomatic proof. |

Primary dependency references: [Crossterm](https://docs.rs/crossterm/0.29.0/),
[Ropey](https://docs.rs/ropey/1.6.1/),
[Unicode segmentation](https://docs.rs/unicode-segmentation/1.13.3/),
[Unicode width](https://docs.rs/unicode-width/0.2.2/),
[Tree-sitter](https://docs.rs/tree-sitter/0.27.1/), and
[Rust grammar](https://docs.rs/tree-sitter-rust/0.24.2/).

Do not copy Ruru's custom storage, Ruby bridges, seven-parameter allowance,
branching history, surface caps, or quota framework. The accepted spec records
the reviewed Rust invariants; each decision above keeps rev's smaller owners.
Any demonstrated dependency incompatibility must be diagnosed and resolved
within the approved required capability, recorded in the plan before use,
and must not broaden dependencies or silently weaken safe-Rust/quality limits.

## Validation contract

Every completed implementation increment and every slice PR must successfully
run this exact ordered prefix, also pinned in the controller configuration:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
python3 scripts/check-source.py
```

Run the active dossier's focused command while developing and record it after
the prefix in the PR. Inspect that it ran real cases; zero-test success does
not prove a behavior. `git diff --check` is last. Required commands remain in
the PR body as exact backticked command lines, without prose substitutions.
The first slice creates the scripts before its PR is considered ready.

`scripts/check-source.py` must enforce `#![forbid(unsafe_code)]`, first-party
unsafe exclusion, parameter counts including self, shallow nesting, and
production cyclomatic maximum 10 with target 5. Explicit review also checks
concise one/two-word identifiers, restrained meaningful comments, cohesive
ownership, test value, and no fake command options. Do not build a generic
lint framework or lower the accepted caps to make a candidate pass.

The full gate is frozen as `scripts/check-all.sh`. It must run the same prefix,
the complete real-executable acceptance PTY suite, the representative
performance command, and acceptance-evidence completeness checks. It must
fail on missing scripts, required skipped cases, stale/missing evidence,
unsafe publication, or violated required behavior. Do not make absent tools
or unsupported test chords silently pass; semantic fallback bindings still
need real coverage. No network is needed by the normal running editor.

The 20 acceptance identities in the specification are the gate's required
evidence map. Rust suites cover deterministic variants at the owning boundary;
PTY journeys establish actual input/output/lifetime behavior. Fault injection
is limited to real filesystem, terminal, and parser boundaries and must state
what is simulated. Performance records a deterministic tens-of-thousands-line
mixed-width fixture and actual hardware model/architecture without hostnames.
It checks startup, insert/delete, scroll, split redraw, highlighting, and idle
work; do not invent numeric latency guarantees.

Expected final-gate cost is minutes on a warm build, plus an initial Cargo
build where needed; this is a planning estimate, not a measured result. Record
actual timing during implementation. Reuse native suite setup and condition
waits. Diagnose failures narrowly before repeating an expensive integrated
gate. Mandatory per-increment Cargo commands remain required by R38.

## Execution and closure authority

The product owner approved the paired baseline and requested controller startup
on 2026-10-26. This authorizes the full ordered set, necessary declared
dependencies, task worktrees, commits/pushes, one canonical PR per slice, and
Looper-owned Rebase + Fast-forward merges into dev. No separate approval is
needed between slices of this set. Keep the primary checkout on clean dev;
perform implementation in canonically named task worktrees.

Each slice stays within its recorded tasks and adds behavior tests at the first
owning boundary. Later hardening slices do not permit knowingly unsafe saves,
broken Unicode, or fake commands in earlier accepted increments. Every slice
must compile and pass its required commands. Do not commit intentionally
failing or disabled acceptance tests to represent future work.

Use one Conventional Commit per tracker task where practical, with the task
identity in its message. Every slice has exactly one report at the table path,
with required metadata and the global report sections. Set it to reviewing
before opening its PR; mark delivered after verified canonical integration.
Use the exact Summary / Changes / Validation PR headings and an immutable
report link as the final Changes item. Preserve the configured branch, title,
report path, and slice numbering throughout the run.

Maintain a reader-facing Unreleased changelog using changelog-normalize when
a complete user-visible behavior is first delivered. Product reference/how-to
documentation belongs under the corresponding Diataxis directories; delivery
evidence remains under ops. Do not copy global agent rules into this repository.

The controller owns full-gate execution after all 48 PRs integrate. The worker
leaves dev clean/pushed and waits for that exact-candidate outcome. After a
passing gate, closure changes only the registered plan, tracker, paired
spec/model status metadata, and canonical reports. Mark plan/tracker completed,
all reports delivered, and every task checked only after its outcome is real.
No numbered successor, reduced scope, main promotion, tag, or release is part
of this set.

Keep one logical checkpoint if interrupted. The launch budget is 1600 worker
turns for the whole 48-slice set and its recoveries, with the default five
consecutive stalled turns. These are ceilings, not targets to consume.
Do not change frozen budgets, gates, identities, or counters after startup.
A final-gate failure must remain visible and use the same run's supported
bounded repair workflow; this plan does not manufacture an initial repair
grant that the installed controller cannot pin before slice integration.

## Requirement coverage

| Requirement | Owning slice numbers |
| --- | --- |
| R01 | 1, 44, 47, 48 |
| R02 | 1, 7, 12, 13, 48 |
| R03 | 1, 20, 42, 43, 44, 48 |
| R04 | 7, 10, 14, 15, 16, 28, 39, 42, 43, 44, 48 |
| R05 | 6, 16, 20, 35, 40, 48 |
| R06 | 6, 17, 20, 35, 43, 48 |
| R07 | 3, 6, 8, 9, 17, 20, 21, 29, 32, 41, 45, 46, 48 |
| R08 | 2, 4, 6, 7, 19, 46, 48 |
| R09 | 2, 3, 4, 8, 12, 19, 29, 32, 33, 41, 45, 48 |
| R10 | 3, 12, 14, 26, 34, 43, 45, 48 |
| R11 | 2, 10, 12, 13, 16, 19, 37, 38, 39, 42, 48 |
| R12 | 13, 15, 16, 28, 37, 44, 48 |
| R13 | 14, 15, 38, 42, 43, 48 |
| R14 | 17, 18, 19, 20, 44, 48 |
| R15 | 18, 44, 48 |
| R16 | 3, 8, 9, 48 |
| R17 | 4, 7, 8, 19, 22, 26, 48 |
| R18 | 21, 23, 44, 48 |
| R19 | 11, 22, 23, 24, 44, 45, 48 |
| R20 | 11, 32, 33, 34, 43, 44, 45, 48 |
| R21 | 4, 10, 11, 14, 19, 22, 23, 24, 25, 26, 31, 33, 34, 42, 45, 48 |
| R22 | 5, 27, 44, 47, 48 |
| R23 | 5, 13, 15, 16, 27, 28, 31, 48 |
| R24 | 5, 21, 25, 28, 30, 42, 48 |
| R25 | 5, 7, 11, 25, 26, 43, 44, 47, 48 |
| R26 | 29, 30, 44, 45, 46, 48 |
| R27 | 31, 44, 48 |
| R28 | 35, 36, 37, 44, 48 |
| R29 | 36, 39, 48 |
| R30 | 37, 38, 39, 43, 45, 48 |
| R31 | 40, 44, 48 |
| R32 | 41, 43, 45, 46, 48 |
| R33 | 2, 4, 5, 17, 47, 48 |
| R34 | 1, 40, 47, 48 |
| R35 | 1, 47, 48 |
| R36 | 47, 48 |
| R37 | 12, 43, 47, 48 |
| R38 | 1, 47, 48 |
| R39 | 43, 44, 45, 48 |
| R40 | 41, 46, 48 |
