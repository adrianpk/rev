# Startup Modal Editing Delivery

Status: approved
Date: 2026-11-01
Delivery set: rev-modal-editing
Slice strategy: behavior-first
Approved: product owner, 2026-11-01; eight-slice plan and autonomous Looper execution with delegated integration into dev
Capability approval: product owner, 2026-11-01; explicit approval of Modal Editing and its paired model
Planning baseline: `9af6cd59d36d333b69cf228d1c4ed54b63d6d672`
Spec: [Modal Editing](../spec/editor-modal-editing.md)
Model: [Modal Editing Model](../spec/editor-modal-editing-model.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-modal.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 2 reviewing; Slice 1 delivered through PR #212
Tracker: [Modal Editing Tracker](../tracker/rev-modal-editing.md)

## Outcome and authority

Deliver startup-selected Emacs/Vim interaction over rev's native Rust owners.
Emacs remains the default; profile changes take effect only on restart. Match
Odra's shipped behavior for shared modal movements, selections, operators,
replacement, repetition and colon entry. Preserve the explicit additional rev
requirements, including checked jump history and Insert C-o continuation.

The owner approved the capability, this eight-slice implementation plan and
controller startup on 2026-11-01. Looper may execute the recorded tasks and
integrate one canonical PR per slice into dev using Rebase + Fast-forward.
This authority covers the configured integrated gate and documentation closure.
The controller checkpoint owns runtime progress; this tracker is its durable
task record, not a second execution controller.

Use one ordered Looper delivery set for all eight slices, its exact-candidate
full gate and closure. Main promotion, releases, installation and public-site
deployment remain separate instructions. Ordinary Conventional Commit subjects
must not contain task IDs or slice-number suffixes. Keep source/config/docs English.

## Strategy and prerequisites

Eight behavior-first increments keep useful contracts reviewable without
inventing an additional runtime, command registry, text store or undo engine.
Each slice must compile, have meaningful passing tests and expose only commands
it actually supports. State maps and documentation grow with implemented behavior;
an intermediate profile is not advertised as complete Vim compatibility.

Delivered prerequisites are native editing/history, coordinates, Emacs input,
typed startup TOML and complete keymaps, vertical command/path prompts,
rectangles, splits, wrapping/gutter/pointer projection, native Org, bookmarks,
native Tree-sitter completion and the optional LSP client from v0.2.0.
Discussion item 10 is temporarily deferred, not a dependency or cancellation.
No new runtime, package, grammar or external Vim dependency is authorized.

Use the actual Odra reference pinned by the approved spec. Read current modal
reference/guide and follow-up code rather than treating the original core
spec's historical Replace/repeat/Block exclusions as current behavior. Static
reference inspection is not an executed Odra test. Do not change Odra's checkout.

## Finite interaction and configuration matrix

| Surface | Finite behavior and preservation case | Delivery |
| --- | --- | --- |
| Startup | `editor.interaction_profile = "emacs" / "vim"`; omission means Emacs. Invalid profile/map rejects the whole candidate into complete native defaults. No runtime or per-buffer profile setter. | Slice 1 |
| Keymaps | Keep `keys.editor` for Emacs and introduce `keys.vim_normal`, `vim_insert`, `vim_replace`, `vim_visual_character`, `vim_visual_line`, `vim_visual_block` under the existing `keys` owner. Local component maps retain priority; inactive profiles do not shadow active keys. | Slices 1, 4, 6, 7 |
| States | Normal, Insert, Replace, Visual Character/Line/Block. i/a/I/A/o/O enter native insertion; R enters Replace; Escape/C-[ returns to Normal when editing owns input. Unmatched Normal/Visual text never inserts. | Slices 1, 4, 6 |
| Character/line/file movement | h/j/k/l, arrows, 0/^/$, gg/G and counted line destinations. Odra's canonical grapheme/endpoint policy is normative for the shared subset. Vim line targets are logical despite wrapping; Emacs visual-line behavior is unchanged. | Slice 2 |
| Words and local targets | w/b/e, W/B/E, ge/gE; f/F/t/T and ;/,; paragraphs { / } and matching delimiter %. WORD/end variants and counted destinations remain the prior explicit rev scope where absent from Odra. | Slice 2 |
| View movement | C-f/C-b full page, C-d/C-u half page, Page Up/Down, H/M/L and zt/zz/zb through native geometry; no private viewport or folded-body offsets. | Slice 2 |
| Search | / and ?, n/N, * and # use native literal search, smart case, no wrapping and origin-preserving cancellation. Word-at-point adds checked word boundaries through the same search owner, not Vim regexps. | Slice 2 |
| Operators | d/c/y, </>, multiplied counts, dd/cc/yy, D/C/Y, x/X, r<char>, p/P and J. Endpoint, inclusivity and linewise intent resolve once before a single atomic publication. | Slice 3 |
| Text objects | iw/aw, iW/aW, ip/ap; inner/around (), [], {}, single/double/backtick quotes. Closing-delimiter spellings are aliases of their matching pair. Missing/invalid targets preserve text and history. | Slice 3 |
| Visual | v/V/C-v enter, convert or exit the corresponding shape; o exchanges endpoints. Character is destination-inclusive; Line uses whole logical lines; Vim Block uses logical grapheme columns while Emacs rectangles keep display cells. | Slice 4 |
| Jumps | Normal C-o/C-i, including terminal Tab aliases, traverse checked view-local locations. Insert C-o admits one complete Normal command then resumes only a compatible insertion context. A local popup/prompt keeps Tab. | Slice 5 |
| Replacement and dot | Whole-grapheme R/r replacement; newline-safe EOL insertion; native undo/redo. Dot retains semantic intent, resolves current targets and supports an explicit count override, never raw event/provider/consent replay. | Slices 3, 6 |
| Commands and panes | :canonical-name shares M-x query/completion/history/arguments/availability/dispatch. No additional command filter, :w/:q aliases or Ex parser. C-w h/j/k/l/s/v/c/o/= uses native pane operations; compatible existing modified bindings remain context-checked. | Slice 7 |
| Completion and local contexts | Insert/Replace native or optional LSP completion keeps the v0.2 popup, colors, acceptance/imports/undo and stale-result checks. Normal/Visual motion input is not typing. Tree, prompts, decisions, bookmarks and native Org contexts own input before modal grammar. | Slice 7 |
| Guidance and acceptance | Canonical Markdown is the source for Spore. Guide chapter order, assets, images and local routes remain in sync; real PTY tests cover both startup profiles and terminal restoration. | Slice 8 |

Before committing a state map, cross-check every binding against this table and
its requirement. Add canonical profile-neutral semantic operations where rev
lacks a target; do not duplicate existing functions with Vim-only equivalents.
Normal C-r is redo and C-w is the pane prefix; Insert retains compatible
direct meanings such as backward search and kill-region. An unhandled or
incompatible composition clears safely with a concise message.

## Bounds, focus and lifetime contract

- Match Odra's checked u32 count ceiling, 16 pending strokes and 96-byte pending
  character-target storage. A large count must not drive a count-sized UI loop:
  bound actual work by source/view traversal and reject arithmetic overflow
  before movement/edit publication.
- Retain one last-change record with at most 1,024 text bytes, following the
  Odra finite repeat record. Oversized or unsupported changes remain ordinary
  valid edits with normal undo; never publish a truncated replay as complete.
- Keep at most 256 jump entries per view. Record nonlocal navigation and confirmed
  destinations, not each ordinary character/word step or traversal itself.
  New jumps discard a forward branch. Resolve a candidate before committing
  point/index; stale, missing or ambiguous evidence preserves the current view.
- A split copies the current bounded jump observation, then evolves independently.
  Closing a view drops its transient grammar/continuation; buffer replacement
  cannot retarget a pending operator, selection, completion or repeat record.
- Enter Normal from insertion through the inspected Odra transition semantics,
  not an assumed byte decrement. State transitions and focus restoration use
  checked native positions; no separate modal cursor is authoritative.
- Components retain their own Escape/Tab/Enter meanings before ordinary editing
  grammar. Preserve protected C-g and M-x, terminal alias truth and bracketed
  paste ownership. A cancelled result cannot later reopen a popup or edit text.
- Keep actual shared search, transfer, history, buffers and location owners.
  Jump history is session-local and does not create a bookmark persistence store.
  No modal idle scans, permanent service, extra parser/index or dropped input.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Startup profile and Insert state | `feat/rev-modal-editing-01-profile` | `feat(input): select startup editing profiles` | `ops/default/report/slices/rev-modal-editing/slice-1-profile.md` |
| Slice 2 | Logical navigation, counts and search | `feat/rev-modal-editing-02-navigation` | `feat(motion): add checked modal navigation` | `ops/default/report/slices/rev-modal-editing/slice-2-navigation.md` |
| Slice 3 | Operators and shared text transfer | `feat/rev-modal-editing-03-operators` | `feat(editing): compose modal operators and targets` | `ops/default/report/slices/rev-modal-editing/slice-3-operators.md` |
| Slice 4 | Visual selections and block geometry | `feat/rev-modal-editing-04-visual` | `feat(selection): add modal visual states` | `ops/default/report/slices/rev-modal-editing/slice-4-visual.md` |
| Slice 5 | Jump history and Insert continuation | `feat/rev-modal-editing-05-jumps` | `feat(navigation): add checked modal jump history` | `ops/default/report/slices/rev-modal-editing/slice-5-jumps.md` |
| Slice 6 | Replace state and semantic repeat | `feat/rev-modal-editing-06-repeat` | `feat(editing): add modal replacement and repeat` | `ops/default/report/slices/rev-modal-editing/slice-6-repeat.md` |
| Slice 7 | Colon commands and completion ownership | `feat/rev-modal-editing-07-contexts` | `feat(input): share colon commands and native completion` | `ops/default/report/slices/rev-modal-editing/slice-7-contexts.md` |
| Slice 8 | Terminal acceptance, resources and guidance | `feat/rev-modal-editing-08-acceptance` | `test(modal): validate startup profiles and native workflows` | `ops/default/report/slices/rev-modal-editing/slice-8-acceptance.md` |

## Slice 1: Startup profile and Insert state

Requirements: MOD01-MOD03, MOD09-MOD11.
Dependencies: delivered v0.2.0 native owners.

- T1.1: Add the typed editor.interaction_profile enum with emacs as the default, immutable session admission, startup-only application and whole-candidate fallback. Create tests/modal.rs using owned configuration fixtures; preserve startup provisioning and existing configuration precedence.
- T1.2: Introduce one native session/view-coordinated modal state owner and safe Normal/Insert transitions. Normal rejects unmatched printable input; i/a/I/A/o/O and Escape/C-[ use checked native edits and cursor positions. Keep M-x and C-g recovery, source/history ownership and terminal restoration intact.
- T1.3: Extend the existing keys tables with profile/state-scoped Vim maps. Keep keys.editor as the Emacs factory surface and preserve local component precedence; admit only implemented canonical commands, never scripts or unbinding sentinels. Distinguish compatible modified Insert bindings from conflicting Normal bindings.
- T1.4: Show NORMAL/INSERT through the existing status layout and terminal cursor API, resetting cursor presentation on exit. Update the complete supported sample, defaults, command/key inventories and targeted R01/R24/CFG12-CFG15 wording without rewriting historical delivery evidence.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test configuration --test core --test commands --test display --test windows
git diff --check
```

## Slice 2: Logical navigation, counts and search

Requirements: MOD04-MOD05, MOD08-MOD09.
Dependencies: Slice 1.

- T2.1: Implement shared semantic targets for h/j/k/l, arrows, 0/^/$, w/b/e, W/B/E, ge/gE, gg/G and counted logical-line destinations. Pin Odra endpoint behavior for the shared subset and label the previously requested extra word/count forms as rev requirements; preserve graphemes and Emacs motion results.
- T2.2: Add logical-line f/F/t/T and ;/, paragraph { / } and matching-delimiter % targets. Resolve Unicode character targets and missing matches without partial movement, stale offsets or unbounded count loops; keep target descriptors reusable by operators and Visual states.
- T2.3: Implement checked count composition, page/half-page controls, H/M/L and zt/zz/zb through native view geometry. Route /, ?, n/N and word-at-point search through the canonical literal/smart-case/nonwrapping search lifecycle, with cancellation restoring the search origin.
- T2.4: Exercise logical versus wrapped rows, empty/CRLF files, EOF, tabs, combining/wide graphemes, hidden Org bodies, terminal aliases and arithmetic overflow. Prove large counts are clamped/resolved through bounded source traversal rather than repeating billions of primitive commands.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test coordinates --test search --test wrapping --test gutter
git diff --check
```

## Slice 3: Operators and shared text transfer

Requirements: MOD05-MOD06, MOD12.
Dependencies: Slice 2.

- T3.1: Compose d/c/y and </> with one checked target, including multiplied operator/motion counts and doubled logical-line operators. Resolve inclusivity and linewise intent before publication; pending grammar must not move point or mutate text.
- T3.2: Implement the finite text-object set: inner/around word and WORD, paragraph, paired parentheses/brackets/braces, and single/double/backtick quotes. Test nested delimiters, escapes, absent targets and boundary counts without introducing a language parser or regexp dialect for text objects.
- T3.3: Deliver dd/cc/yy, D/C/Y, x/X, r<char>, p/P, line joining J and indentation through shared prepared edits, kill/yank ownership and history. Preserve linewise transfer intent without creating a modal register store; linewise paste handles EOF and newline conventions.
- T3.4: Verify examples such as dw on 'one two' leaving 'two', de preserving the following separator, 2d3w as one checked six-word target and one undo, plus read-only/failed preparation, Unicode and multi-view restoration. Update inventories for each new supported semantic command.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test editing --test rectangles --test recovery
git diff --check
```

## Slice 4: Visual selections and block geometry

Requirements: MOD06, MOD09-MOD12.
Dependencies: Slice 3.

- T4.1: Implement v/V selection entry, inclusive destination graphemes, complete logical lines, shape conversion, repeated-shape exit and o active-end exchange. Use canonical point/mark ownership and preserve valid cancellation and shared-buffer view semantics.
- T4.2: Extend native rectangular selection ownership with an explicit logical-grapheme column policy for Vim C-v while preserving the existing display-cell policy for Emacs rectangles. Pin tab, wide/combining grapheme, short-line and EOL cases; geometry must remain shared with wrapping/gutter/pointer projection.
- T4.3: Apply Visual d/c/y to character, line and block selections as one atomic native operation; change enters Insert and successful copy/delete returns Normal. Keep Visual indentation/text-object availability aligned with the finite Odra subset and preserve rectangular transfer shape.
- T4.4: Cover mouse selection, split views, buffer switches, Org folds, resize, source revision retirement and one-step undo/redo. Prove no modal selection can target a different buffer and no block failure commits earlier lines.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test rectangles --test pointer --test windows --test wrapping --test org_editing
git diff --check
```

## Slice 5: Jump history and Insert continuation

Requirements: MOD08-MOD09, MOD12-MOD13.
Dependencies: Slice 4.

- T5.1: Add a bounded session-local jump history owned per view, with native buffer/file identity and checked source-location evidence. Rebase live destinations through canonical edits/undo and preserve rename identity; reuse location primitives without coupling storage to the bookmark collection.
- T5.2: Record admitted nonlocal destinations: explicit line/buffer/paragraph/viewport jumps, confirmed searches and cross-file visits. Do not record each h/j/k/l, word step, rendering pass or history traversal. New jumps after backward traversal discard the forward branch; failed navigation does not advance its index.
- T5.3: Implement counted C-o/C-i backward/forward traversal in Normal and the requested one-Normal-command C-o continuation in Insert. A count/operator or colon argument workflow is one semantic command lifetime, not one raw keystroke; return to Insert only when its checked editing context survives.
- T5.4: Test closed/missing/replaced files, dirty-buffer reuse, edits, rename, read-only visits, split inheritance and view deletion. A split starts a bounded copy of current jump state, then evolves independently. Tab/C-i remains prompt/completion Tab while a component owns input; persistence and bookmarks remain unchanged.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test bookmarks --test files --test windows --test recovery
git diff --check
```

## Slice 6: Replace state and semantic repeat

Requirements: MOD03, MOD05, MOD12, MOD14.
Dependencies: Slice 5.

- T6.1: Implement persistent R replacement over whole graphemes with underline cursor feedback and compatible Insert controls. At logical EOL, insert before the line break rather than consuming the next line. Escape/C-[ restores Normal and ordinary history owns grouping and undo.
- T6.2: Retain a finite semantic last-change record for supported inserts, changes, replacement, delete, paste and indentation. Store resolved operation intent rather than raw terminal events; never replay prompts, file/save consent, network providers or command workflows.
- T6.3: Implement dot against current native targets, using the recorded count unless an explicit dot count overrides it as in Odra. Group repeat as one undoable edit, preserve linear/linewise/block intent and validate source/view/read-only evidence before publishing.
- T6.4: Bound retained text and handle oversized or unsupported changes without truncating a replay or rejecting an already admitted edit. Test Insert/Replace session grouping, stale/failed repeats, cancellation and canonical history across panes; update state/command/sample references.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test editing --test recovery --test coordinates
git diff --check
```

## Slice 7: Colon commands and completion ownership

Requirements: MOD07-MOD12.
Dependencies: Slice 6.

- T7.1: Open the existing vertical command selector from : in Normal and Visual, using exactly the canonical names, history, availability, argument prompts and dispatch of M-x. Keep the full shared interactive catalog initially; no :w/:q aliases, Ex arguments, ranges or colon-only command filter. Add the supported C-w pane grammar through native window commands.
- T7.2: Integrate automatic native/LSP completion with Insert/Replace and manual canonical invocation. Preserve 0.2 popup keys, colored kinds, syntax colors, checked acceptance/imports and one-step undo. Retire pending suggestions/resolution on incompatible modal transitions, and reject late replies without reopening a popup.
- T7.3: Give tree, search, argument/decision prompts, inventories, bookmarks and native Org contexts priority over the editing grammar. Compatible colon cancellation restores the original Visual state; changed buffer/view/revision retires it. Org source drafts inherit the session profile and use their existing edit/save/cancel ownership.
- T7.4: Test Escape/Meta disambiguation, protected C-g/M-x, component-owned Tab/C-i, held Return, bracketed paste and pointer focus. Popup cancellation is local before ordinary state transitions; Normal/Visual letters never schedule automatic completion. Keep literal colon/slash input ordinary inside Insert and argument prompts.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test completion --test lsp --test core --test commands --test panels --test path_completion --test org_source
git diff --check
```

## Slice 8: Terminal acceptance, resources and guidance

Requirements: MOD01-MOD14.
Dependencies: Slice 7.

- T8.1: Complete approachable canonical Markdown guidance for startup profile selection, ordinary modal editing, selections, colon commands and completion, with small visible examples. Keep Emacs as the introductory default. Update the guide order and Spore tree together, references, full sample/catalogs, baseline/model reconciliation and Unreleased through changelog-normalize.
- T8.2: Create a native Rust acceptance example and scripts/check-modal.sh using the existing owned-fixture/controlling-PTY approach. Cover Emacs startup isolation, Vim editing/operators/Visual/jumps/repeat, colon/M-x equivalence, native plus one installed rust-analyzer semantic completion path, undo and clean terminal/child restoration. No new Python harness or project dependency.
- T8.3: Measure large-file/count/block behavior, a finite input burst, executable bytes under the same normal release flags and idle work. Preserve bounded worker ownership and no modal idle polling; protect first-party safe Rust, source quality and unchanged input batching without unrelated cleanup or tuning.
- T8.4: Run focused acceptance, then hand clean integrated dev to the controller's unchanged full gate and closure. Verify all requirement/coverage rows, reports, actual docs/Spore output and preservation cases before marking delivered; no controller checkpoint edits, fake receipts or untested closure claims.

Required ordered checks:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test configuration --test commands --test display --example documentation-check --example userguide-publication
bash scripts/check-modal.sh
cargo run --locked --quiet --example documentation-check
git diff --check
```

## Integrated gate and cost

After all eight canonical PRs integrate into clean dev, the controller runs:

```sh
cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-modal.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check
```

This protects both profiles, shared edits/coordinates/history, local focus and
native/semantic completion. The single modal journey owns the representative
native/LSP/PTY/resource cases; do not append every historical delivery journey.
Spore is already an established publication prerequisite, and rust-analyzer is
the existing optional-server acceptance prerequisite. Neither becomes an editor
runtime dependency or an automatic acquisition step.

Expected warm cost is several minutes, approximately 5-10 minutes including the
all-target suite and release/PTy journey; cold compilation can take longer.
Record actual cost and affected behavior rather than silently raising budgets.
Run focused slice checks during development, then the full gate once at the
integrated candidate. A real correction or changed candidate determines retry
scope under the validation contract. Do not disable checks or alter a frozen gate.

## Closure and handoff

The exact-HEAD gate, eight canonical PR merges and eight delivered reports must
agree before completion. Closure may update only this plan/tracker, the modal
spec/model, the pending-work index and the eight report paths. Baseline R01/R24,
R33 and CFG12-CFG15 reconciliation belongs in the implementation slices before
the gate, preserving historical receipts and existing source/reference ownership.
Any required integration repair follows Looper's bounded, explicitly delegated
repair policy; do not edit checkpoints, fabricate evidence or add hidden slices.

Mark the capability/model Completed only after that evidence exists. Preserve
the deferred item 10 and unrelated tickets. Publication to main, SemVer and
public guide deployment require their own later authorization.
