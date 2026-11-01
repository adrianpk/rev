# Startup Modal Editing Tracker

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
Active slice: Slice 1 reviewing; canonical integration pending
Plan: [Modal Editing Plan](../plan/rev-modal-editing.md)
Execution owner: Looper; startup and canonical slice integration into dev authorized on 2026-11-01; checkpoint and canonical merge evidence own runtime progress

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

Acceptance: MOD01-MOD03, MOD09-MOD11.

- [x] T1.1: Add the typed editor.interaction_profile enum with emacs as the default, immutable session admission, startup-only application and whole-candidate fallback. Create tests/modal.rs using owned configuration fixtures; preserve startup provisioning and existing configuration precedence.
- [x] T1.2: Introduce one native session/view-coordinated modal state owner and safe Normal/Insert transitions. Normal rejects unmatched printable input; i/a/I/A/o/O and Escape/C-[ use checked native edits and cursor positions. Keep M-x and C-g recovery, source/history ownership and terminal restoration intact.
- [x] T1.3: Extend the existing keys tables with profile/state-scoped Vim maps. Keep keys.editor as the Emacs factory surface and preserve local component precedence; admit only implemented canonical commands, never scripts or unbinding sentinels. Distinguish compatible modified Insert bindings from conflicting Normal bindings.
- [x] T1.4: Show NORMAL/INSERT through the existing status layout and terminal cursor API, resetting cursor presentation on exit. Update the complete supported sample, defaults, command/key inventories and targeted R01/R24/CFG12-CFG15 wording without rewriting historical delivery evidence.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 2: Logical navigation, counts and search

Acceptance: MOD04-MOD05, MOD08-MOD09.

- [ ] T2.1: Implement shared semantic targets for h/j/k/l, arrows, 0/^/$, w/b/e, W/B/E, ge/gE, gg/G and counted logical-line destinations. Pin Odra endpoint behavior for the shared subset and label the previously requested extra word/count forms as rev requirements; preserve graphemes and Emacs motion results.
- [ ] T2.2: Add logical-line f/F/t/T and ;/, paragraph { / } and matching-delimiter % targets. Resolve Unicode character targets and missing matches without partial movement, stale offsets or unbounded count loops; keep target descriptors reusable by operators and Visual states.
- [ ] T2.3: Implement checked count composition, page/half-page controls, H/M/L and zt/zz/zb through native view geometry. Route /, ?, n/N and word-at-point search through the canonical literal/smart-case/nonwrapping search lifecycle, with cancellation restoring the search origin.
- [ ] T2.4: Exercise logical versus wrapped rows, empty/CRLF files, EOF, tabs, combining/wide graphemes, hidden Org bodies, terminal aliases and arithmetic overflow. Prove large counts are clamped/resolved through bounded source traversal rather than repeating billions of primitive commands.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 3: Operators and shared text transfer

Acceptance: MOD05-MOD06, MOD12.

- [ ] T3.1: Compose d/c/y and </> with one checked target, including multiplied operator/motion counts and doubled logical-line operators. Resolve inclusivity and linewise intent before publication; pending grammar must not move point or mutate text.
- [ ] T3.2: Implement the finite text-object set: inner/around word and WORD, paragraph, paired parentheses/brackets/braces, and single/double/backtick quotes. Test nested delimiters, escapes, absent targets and boundary counts without introducing a language parser or regexp dialect for text objects.
- [ ] T3.3: Deliver dd/cc/yy, D/C/Y, x/X, r<char>, p/P, line joining J and indentation through shared prepared edits, kill/yank ownership and history. Preserve linewise transfer intent without creating a modal register store; linewise paste handles EOF and newline conventions.
- [ ] T3.4: Verify examples such as dw on 'one two' leaving 'two', de preserving the following separator, 2d3w as one checked six-word target and one undo, plus read-only/failed preparation, Unicode and multi-view restoration. Update inventories for each new supported semantic command.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 4: Visual selections and block geometry

Acceptance: MOD06, MOD09-MOD12.

- [ ] T4.1: Implement v/V selection entry, inclusive destination graphemes, complete logical lines, shape conversion, repeated-shape exit and o active-end exchange. Use canonical point/mark ownership and preserve valid cancellation and shared-buffer view semantics.
- [ ] T4.2: Extend native rectangular selection ownership with an explicit logical-grapheme column policy for Vim C-v while preserving the existing display-cell policy for Emacs rectangles. Pin tab, wide/combining grapheme, short-line and EOL cases; geometry must remain shared with wrapping/gutter/pointer projection.
- [ ] T4.3: Apply Visual d/c/y to character, line and block selections as one atomic native operation; change enters Insert and successful copy/delete returns Normal. Keep Visual indentation/text-object availability aligned with the finite Odra subset and preserve rectangular transfer shape.
- [ ] T4.4: Cover mouse selection, split views, buffer switches, Org folds, resize, source revision retirement and one-step undo/redo. Prove no modal selection can target a different buffer and no block failure commits earlier lines.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 5: Jump history and Insert continuation

Acceptance: MOD08-MOD09, MOD12-MOD13.

- [ ] T5.1: Add a bounded session-local jump history owned per view, with native buffer/file identity and checked source-location evidence. Rebase live destinations through canonical edits/undo and preserve rename identity; reuse location primitives without coupling storage to the bookmark collection.
- [ ] T5.2: Record admitted nonlocal destinations: explicit line/buffer/paragraph/viewport jumps, confirmed searches and cross-file visits. Do not record each h/j/k/l, word step, rendering pass or history traversal. New jumps after backward traversal discard the forward branch; failed navigation does not advance its index.
- [ ] T5.3: Implement counted C-o/C-i backward/forward traversal in Normal and the requested one-Normal-command C-o continuation in Insert. A count/operator or colon argument workflow is one semantic command lifetime, not one raw keystroke; return to Insert only when its checked editing context survives.
- [ ] T5.4: Test closed/missing/replaced files, dirty-buffer reuse, edits, rename, read-only visits, split inheritance and view deletion. A split starts a bounded copy of current jump state, then evolves independently. Tab/C-i remains prompt/completion Tab while a component owns input; persistence and bookmarks remain unchanged.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 6: Replace state and semantic repeat

Acceptance: MOD03, MOD05, MOD12, MOD14.

- [ ] T6.1: Implement persistent R replacement over whole graphemes with underline cursor feedback and compatible Insert controls. At logical EOL, insert before the line break rather than consuming the next line. Escape/C-[ restores Normal and ordinary history owns grouping and undo.
- [ ] T6.2: Retain a finite semantic last-change record for supported inserts, changes, replacement, delete, paste and indentation. Store resolved operation intent rather than raw terminal events; never replay prompts, file/save consent, network providers or command workflows.
- [ ] T6.3: Implement dot against current native targets, using the recorded count unless an explicit dot count overrides it as in Odra. Group repeat as one undoable edit, preserve linear/linewise/block intent and validate source/view/read-only evidence before publishing.
- [ ] T6.4: Bound retained text and handle oversized or unsupported changes without truncating a replay or rejecting an already admitted edit. Test Insert/Replace session grouping, stale/failed repeats, cancellation and canonical history across panes; update state/command/sample references.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 7: Colon commands and completion ownership

Acceptance: MOD07-MOD12.

- [ ] T7.1: Open the existing vertical command selector from : in Normal and Visual, using exactly the canonical names, history, availability, argument prompts and dispatch of M-x. Keep the full shared interactive catalog initially; no :w/:q aliases, Ex arguments, ranges or colon-only command filter. Add the supported C-w pane grammar through native window commands.
- [ ] T7.2: Integrate automatic native/LSP completion with Insert/Replace and manual canonical invocation. Preserve 0.2 popup keys, colored kinds, syntax colors, checked acceptance/imports and one-step undo. Retire pending suggestions/resolution on incompatible modal transitions, and reject late replies without reopening a popup.
- [ ] T7.3: Give tree, search, argument/decision prompts, inventories, bookmarks and native Org contexts priority over the editing grammar. Compatible colon cancellation restores the original Visual state; changed buffer/view/revision retires it. Org source drafts inherit the session profile and use their existing edit/save/cancel ownership.
- [ ] T7.4: Test Escape/Meta disambiguation, protected C-g/M-x, component-owned Tab/C-i, held Return, bracketed paste and pointer focus. Popup cancellation is local before ordinary state transitions; Normal/Visual letters never schedule automatic completion. Keep literal colon/slash input ordinary inside Insert and argument prompts.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Slice 8: Terminal acceptance, resources and guidance

Acceptance: MOD01-MOD14.

- [ ] T8.1: Complete approachable canonical Markdown guidance for startup profile selection, ordinary modal editing, selections, colon commands and completion, with small visible examples. Keep Emacs as the introductory default. Update the guide order and Spore tree together, references, full sample/catalogs, baseline/model reconciliation and Unreleased through changelog-normalize.
- [ ] T8.2: Create a native Rust acceptance example and scripts/check-modal.sh using the existing owned-fixture/controlling-PTY approach. Cover Emacs startup isolation, Vim editing/operators/Visual/jumps/repeat, colon/M-x equivalence, native plus one installed rust-analyzer semantic completion path, undo and clean terminal/child restoration. No new Python harness or project dependency.
- [ ] T8.3: Measure large-file/count/block behavior, a finite input burst, executable bytes under the same normal release flags and idle work. Preserve bounded worker ownership and no modal idle polling; protect first-party safe Rust, source quality and unchanged input batching without unrelated cleanup or tuning.
- [ ] T8.4: Run focused acceptance, then hand clean integrated dev to the controller's unchanged full gate and closure. Verify all requirement/coverage rows, reports, actual docs/Spore output and preservation cases before marking delivered; no controller checkpoint edits, fake receipts or untested closure claims.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Delivery closure

- [ ] All eight configured slices integrated into dev with canonical PR evidence.
- [ ] The unchanged integrated full gate passed for the exact clean candidate.
- [ ] All eight reports are delivered and record actual validation boundaries.
- [ ] Plan, tracker, modal spec/model and pending-work index agree on completion.
- [ ] Deferred item 10 and unrelated work remain separate; no main/release/site publication inferred.
