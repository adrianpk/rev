# rev Editor Tracker

Status: approved
Approved: product owner, 2026-10-26; complete baseline implementation and Looper startup requested.
Delivery set: rev-editor
Active slice: Slice 47 - Audit source quality and product reference
Plan: [rev Editor Delivery](../plan/rev-editor.md)
Spec: [rev Editor](../spec/rev-editor.md)
Model: [rev Editor Model](../spec/rev-editor-model.md)

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

## Task state

Task scope and focused commands are defined in the linked plan parts. Record
actual task commit hashes, successful checks, PR numbers, and merge evidence
under each section as work completes. Unchecked items are not delivered.

### Slice 1: Own terminal lifetime

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R01, R02, R03, R34, R35, R38.

- [x] T1.1: Create the Rust 2024 executable/library package, pin the approved dependency set, forbid unsafe at every first-party crate root, and enter/restore the terminal through one RAII owner. Implement the real exit sequence without placeholder editor commands.
- [x] T1.2: Provide executable source-quality and per-slice/full-gate entry points. Add a minimal PTY lifecycle probe for startup, ordinary exit, and a failure after terminal acquisition; later suites must be required by the final gate rather than silently skipped.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T1.1 and T1.2 delivered through PR #1.
Task commits: T1.1 `598eae082ff24fb65d9a0ae321394fa3d7c3803a`;
T1.2 `e54df2cc72c8916b9fbe24dafa6ed8552b31f727`.
Validation: required Cargo/source commands passed; the terminal PTY suite passed
six cases with zero skips. Final PR-head evidence is recorded through Looper.
PR: [#1](https://forge.adrianpk.com/adrianpk/rev/pulls/1).
Integration: Rebase + Fast-forward into dev at
`a37fb53316149966ed3d7457fb6cd47a94561035`; all six ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 1](../report/slices/rev-editor/slice-1-terminal.md).

### Slice 2: Introduce canonical rope buffers

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R08, R09, R11, R33.

- [x] T2.1: Add stable buffer identity, one canonical Ropey store, initial empty state, checked scalar ranges, and bounded chunk/line reads. Keep file identity and view identity separate.
- [x] T2.2: Exercise insertion-range admission, empty/multiline stores, foreign identities, stale revisions, UTF-8 boundaries, and two references observing one text owner without a second mutable copy.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T2.1 and T2.2 delivered through PR #2.
Task commits: T2.1 `f3fc075194321f8603140bdb717597dac12a1688`;
T2.2 `9487765a9240a3bb49de35dc3a117d8523432ba8`.
Validation: required Cargo/source commands passed; `cargo test --test core`
passed six tests with zero skips. Final PR-head evidence is recorded through Looper.
PR: [#2](https://forge.adrianpk.com/adrianpk/rev/pulls/2).
Integration: Rebase + Fast-forward into dev at
`87d08134d244d2670f4e8f8d5b23a78738e5aa47`; all six ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 2](../report/slices/rev-editor/slice-2-buffers.md).

### Slice 3: Resolve Unicode and display coordinates

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R07, R09, R10, R16.

- [x] T3.1: Implement explicit scalar/byte/line-byte/display conversion and grapheme navigation over rope chunks. Use Unicode segmentation context across seams; retain CRLF as one logical separator.
- [x] T3.2: Add behavior cases for emoji sequences, combining marks, CJK width, tab stops, empty text, and a grapheme straddling chunks. Define safe display of controls without emitting terminal escape bytes.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T3.1 and T3.2 delivered through PR #3.
Task commits: T3.1 `b21fb6f837c6d959ee37716a88e96ac5da2ff400`;
T3.2 `7a04433a8c7134b8c060379b1f4d1ca6b0a528b4`.
Validation: required Cargo/source commands passed; `cargo test --test coordinates`
passed eight tests with zero skips. Final PR-head evidence is recorded through Looper.
PR: [#3](https://forge.adrianpk.com/adrianpk/rev/pulls/3).
Integration: Rebase + Fast-forward into dev at
`22184eecdda62ac5f5602b74afa198b85de494f3`; all six ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 3](../report/slices/rev-editor/slice-3-coordinates.md).

### Slice 4: Publish coherent text edits

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R08, R09, R17, R21, R33.

- [x] T4.1: Introduce one checked edit path with read-only admission, a prepared replacement, reversible history data, and exact position mapping. Multi-range edits state their coordinate space before publication.
- [x] T4.2: Reject stale/read-only/invalid-range edits without partial publication. Establish atomic multi-range rollback and mapped point/mark behavior so later commands cannot bypass the owner.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T4.1 and T4.2 delivered through PR #4.
Task commits: T4.1 `2fed813e6ac8c1a5761e1a2733ccda7b9998d9fb`;
T4.2 `ba6a69da8fad140524c0abaa4a7dc000624e31ff`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed ten tests with zero skips. Final PR-head evidence is recorded through Looper.
PR: [#4](https://forge.adrianpk.com/adrianpk/rev/pulls/4).
Integration: Rebase + Fast-forward into dev at
`709ec2682b739a896c012bc5451766ab5d7dff4d`; all six ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 4](../report/slices/rev-editor/slice-4-edits.md).

### Slice 5: Decode prefixes and focused prompts

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R22, R23, R24, R25, R33.

- [x] T5.1: Normalize keyboard events, route one focused context, decode nested chord prefixes, and dispatch only implemented command variants. Provide a small editable argument/decision prompt used by subsequent file and safety flows.
- [x] T5.2: Cover invalid prefixes, C-g cancellation, prompt/document isolation, press versus release, and exactly-once dispatch. Advertise no unavailable command or functional-looking stub.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T5.1 and T5.2 delivered through PR #5.
Task commits: T5.1 `a7a927c5b09b80ba7f343346d05f341e2f2eca56`;
T5.2 `aa5dc2231a84db61fafb0b6b027fb1d72aeb5472`.
Validation: required Cargo/source commands passed; `cargo test --test commands`
passed eleven tests and the terminal PTY suite passed seven cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#5](https://forge.adrianpk.com/adrianpk/rev/pulls/5).
Integration: Rebase + Fast-forward into dev at
`2332faed5849d40c33930c903f946b847a3da55a`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 5](../report/slices/rev-editor/slice-5-input.md).

### Slice 6: Render one rope-backed viewport

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R05, R06, R07, R08.

- [x] T6.1: Display the initial rope-backed view, visible fragments, cursor, mode line, and echo area with clipped cell geometry and deliberate flushing. Track point and horizontal/vertical viewport state independently of text.
- [x] T6.2: Validate mixed-width clipping, tabs, zero dimensions, status dirty/read-only/mode indicators, and readable error messages. Keep projection caches derived and correctly invalidated.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T6.1 and T6.2 delivered through PR #6.
Task commits: T6.1 `9b1d62b17dc98a039243e96693e4ceed2275e94d`;
T6.2 `5ed738ed6e12f4f4c739aca025f279cd30d4a423`.
Validation: required Cargo/source commands passed; `cargo test --test display`
passed nine tests and the terminal PTY suite passed eight cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#6](https://forge.adrianpk.com/adrianpk/rev/pulls/6).
Integration: Rebase + Fast-forward into dev at
`f87c76b69ab2794f0f4a117a957bef143064f4b8`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 6](../report/slices/rev-editor/slice-6-display.md).

### Slice 7: Edit printable text and graphemes

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R02, R04, R08, R17, R25.

- [x] T7.1: Connect printable input, Enter, Tab, Backspace, and delete-char to the common edit owner and visible cursor updates. Preserve the chosen newline policy and coherent CRLF deletion.
- [x] T7.2: Add boundary, Unicode, and read-only behavior checks through command input. Guard modified text immediately: until saving exists, refuse an unsafe quit rather than offer a fake save action.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T7.1 and T7.2 delivered through PR #7.
Task commits: T7.1 `c87c86662528731c53845685640d2f5165256c12`;
T7.2 `1dfd5d17b148f0b5ddc7f5440231dff3ebeea8ed`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed twenty-one tests and the terminal PTY suite passed nine cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#7](https://forge.adrianpk.com/adrianpk/rev/pulls/7).
Integration: Rebase + Fast-forward into dev at
`79dad958ec299b222b303d03529b0181a1eaa8eb`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 7](../report/slices/rev-editor/slice-7-insertion.md).

### Slice 8: Implement character and word motion

Inputs: [Part 1](../plan/rev-editor/01-core-and-terminal.md); requirements R07, R09, R16, R17.

- [x] T8.1: Implement forward/backward character, beginning/end of line and buffer, and Unicode-consistent word navigation using checked source positions.
- [x] T8.2: Exercise empty text, punctuation, mixed scripts, CRLF, emoji clusters, and scroll reveal through the real command registry. Arrow/Home/End aliases may mirror supported commands without replacing Emacs bindings.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T8.1 and T8.2 delivered through PR #8.
Task commits: T8.1 `61360a264a44ebbdbc9139d9cb4ce63d352c9a50`;
T8.2 `0b9fb1b4931394fa419a1b2209d0a8720dd1adb8`.
Validation: required Cargo/source commands passed; `cargo test --test coordinates`
passed nineteen tests and the terminal PTY suite passed ten cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#8](https://forge.adrianpk.com/adrianpk/rev/pulls/8).
Integration: Rebase + Fast-forward into dev at
`def2f4445d1b5adf4fae08226a7accb95cbc11f8`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 8](../report/slices/rev-editor/slice-8-motion.md).

### Slice 9: Preserve vertical motion and scrolling

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R07, R16.

- [x] T9.1: Implement next/previous logical line with retained desired display column, page motion, horizontal reveal, and recenter-top-bottom behavior.
- [x] T9.2: Exercise short lines between long lines, tabs/wide clusters, tiny viewports, and repeated page/recenter operations without losing intended column.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T9.1 and T9.2 delivered through PR #9.
Task commits: T9.1 `60160f6f034d6554dc36315d706f4c221c30f544`;
T9.2 `bc16af2c0d9744525c638aff98ae336ed56bf43f` and fixture synchronization
`2c4364566b343db6b51bc5080f27e032dcbe11b3`.
Validation: required Cargo/source commands passed; `cargo test --test display`
passed eighteen tests and the terminal PTY suite passed eleven cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#9](https://forge.adrianpk.com/adrianpk/rev/pulls/9).
Integration: Rebase + Fast-forward into dev at
`151b63415c224c3e46fb8bf6b25e7424b1c71366`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 9](../report/slices/rev-editor/slice-9-scrolling.md).

### Slice 10: Restore per-buffer undo and redo

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R11, R21.

- [x] T10.1: Implement conventional per-buffer undo/redo, fresh publication revisions, logical saved-state identity, initiating-view restoration, and redo invalidation after a new edit.
- [x] T10.2: Exercise repeated navigation, unavailable history, a retained clean save point, independent buffers, read-only restrictions, and coherent mapping of non-initiating positions.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T10.1 and T10.2 delivered through PR #10.
Task commits: T10.1 `ca57d28a6027aca29186077a8932bcc24aaf5e69` with admission
refinement `20923bfd678a19798f86212c9cd61592819ce0c6`;
T10.2 `fdf16328e74c9c90571ee494d5cc8413013ce155`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed thirty-two tests and the terminal PTY suite passed twelve cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#10](https://forge.adrianpk.com/adrianpk/rev/pulls/10).
Integration: Rebase + Fast-forward into dev at
`2aaf75921e52b2666500c3c2470af1166f70d669`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 10](../report/slices/rev-editor/slice-10-history.md).

### Slice 11: Group typing and compound history

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R19, R20, R21, R25.

- [x] T11.1: Group adjacent ordinary insertion with explicit continuity breaks, preserve whole compound records, and add the agreed simple retention policy without branching-history machinery.
- [x] T11.2: Exercise paste-sized transactions, replacement groups, navigation breaks, new edits after undo, and pruning complete old groups without losing the newest reversible operation.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T11.1 and T11.2 delivered through PR #11.
Task commits: T11.1 `daf03ad40d2393c7731448a9a92d6a422180de5b`;
T11.2 `3846a2971f434b88ec40bee545b24bba938b6e38` with terminal synchronization
`febba5045e033c8409c59c254dae486fa9b82cea`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed forty-four tests and the terminal PTY suite passed thirteen cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#11](https://forge.adrianpk.com/adrianpk/rev/pulls/11).
Integration: Rebase + Fast-forward into dev at
`b81d43b0f43dbaca5718f4b21035890c7e9fbf37`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 11](../report/slices/rev-editor/slice-11-grouping.md).

### Slice 12: Load exact UTF-8 file content

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R02, R09, R10, R11, R37.

- [x] T12.1: Load existing files into canonical ropes with strict UTF-8, conservative NUL binary rejection, exact LF/CRLF and final-newline retention, recognized BOM metadata, and prevailing newline preference.
- [x] T12.2: Exercise invalid encoding, empty files, mixed line endings, BOM round trips, missing paths, permission failures, and preservation of the previous active buffer on rejected ingress.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T12.1 and T12.2 delivered through PR #12.
Task commits: T12.1 `5f874847bc942eb55481037004383fb01d46699f` with startup
ownership `69ad76b8616b1139b27aa9b6cdc8f057ac4be078`;
T12.2 `f012fa51de4fc8d29f1afcfc18f37e5d2fb9053f` with permission fixture
`c744436be30be980975a269f3ff228da2d94c697`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed eleven tests and the terminal PTY suite passed twenty cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#12](https://forge.adrianpk.com/adrianpk/rev/pulls/12).
Integration: Rebase + Fast-forward into dev at
`9bf25810d05f17fb04d3b535899a1d1ebf955caa`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 12](../report/slices/rev-editor/slice-12-file-ingress.md).

### Slice 13: Reuse canonical visited files

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R02, R11, R12, R23.

- [x] T13.1: Implement find-file argument flow, normalized intended paths for absent files, canonical existing referents, unique buffer labels, and existing-visit activation shared by all callers.
- [x] T13.2: Exercise symlink aliases, duplicate basenames, nonexistent parents, canceled prompts, directory paths, and collision with an already visited canonical target.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T13.1 and T13.2 delivered through PR #13.
Task commits: T13.1 `1b6ab4d133d1cb54538fc9a63595a47b971c95ca`, completed
through `38471f583db48ceaa8602febe5183e783dd88d1f`;
T13.2 `d5ff90a3cde1cb54309309ebf7165c558bad3df4` with viewport synchronization
`bce68fa55cb13316148deb9992d0712cc902e2c1`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed twenty-two tests and the terminal PTY suite passed twenty-two cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#13](https://forge.adrianpk.com/adrianpk/rev/pulls/13).
Integration: Rebase + Fast-forward into dev at
`76aabe1a57529f9ec703770a184f992ce5cd09e5`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 13](../report/slices/rev-editor/slice-13-file-visits.md).

### Slice 14: Publish safe same-directory saves

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R10, R13, R21.

- [x] T14.1: Implement save-buffer using captured rope text, same-directory temporary output, checked writes, permission preservation where practical, replacement, and explicit completion/publication outcomes.
- [x] T14.2: Exercise injected write/flush/replace/post-replace failures and cleanup. Reconcile actual publication, saved logical state, dirty flags, and a first successful file association truthfully.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T14.1 and T14.2 delivered through PR #14.
Task commits: T14.1 `c9d8c39b05901db5b80c2fba1bfd60d8af5493fd`, completed
through `f16523df10507278497ad7d8cc6454ee096119bc`;
T14.2 `7bdc7fa8bed7bd3db00340f9b1cc877b56b36885`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed thirty-seven tests and the terminal PTY suite passed twenty-four cases with zero skips.
Integrated into `dev` at `50fd66b75b57f4a1ad54ebd4be15f47e7bea2ea2`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
PR: [#14](https://forge.adrianpk.com/adrianpk/rev/pulls/14).
Report: [Slice 14](../report/slices/rev-editor/slice-14-saving.md).

### Slice 15: Confirm overwrite and external changes

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R12, R13, R23.

- [x] T15.1: Implement write-file, unnamed save prompts, occupied-path confirmation, metadata-based external-change warnings, canonical symlink referent saving, and live-buffer target collision refusal.
- [x] T15.2: Exercise cancellation, denied/disk-full writes, removed paths, save-as before/after publication failures, and clean/dirty history state after successful save and undo.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T15.1 and T15.2 delivered through PR #15.
Task commits: T15.1 `06f8f340815ab3cd760d7fb38784014cf742e25c`;
T15.2 `ad26ededc8a9033df27eddb7242d607147320f3b`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed forty-eight tests and the terminal PTY suite passed twenty-six cases with zero skips.
Integrated into `dev` at `127426edc383900d2111847742591bcf575f2339`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
PR: [#15](https://forge.adrianpk.com/adrianpk/rev/pulls/15).
Report: [Slice 15](../report/slices/rev-editor/slice-15-save-decisions.md).

### Slice 16: Switch list and close buffers

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R05, R11, R12, R23.

- [x] T16.1: Implement switch-to-buffer completion, list-buffers selection, kill-buffer decisions, deterministic replacement, and unique labels independent of window lifetime.
- [x] T16.2: Exercise several modified/unnamed/file-backed buffers, canceled close, failed required save, final-buffer replacement, and index cleanup. Preserve open text when a path disappears.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T16.1 and T16.2 delivered through PR #16.
Task commits: T16.1 `6788da0e8274452b56409596f5c3fdc09e2444e6`;
T16.2 `1e528069250b38fec414c0305a485e161eef70de`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed sixty tests and the terminal PTY suite passed twenty-eight cases with zero skips.
Integrated into `dev` at `3e2e4703b0880fdf2087f2b7a2be2fbf0a92cd73`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
PR: [#16](https://forge.adrianpk.com/adrianpk/rev/pulls/16).
Report: [Slice 16](../report/slices/rev-editor/slice-16-buffer-commands.md).

### Slice 17: Create nested editing splits

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R06, R07, R14, R33.

- [x] T17.1: Implement split-window-below and split-window-right with nested layout nodes, geometry admission, initial same-buffer view state, and retained original focus.
- [x] T17.2: Exercise mixed split directions, minimum dimensions, allocation of distinct view IDs, and independent point/scroll after split. Refuse non-fitting changes before mutating layout.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T17.1 and T17.2 delivered through PR #17.
Task commits: T17.1 `d8aa6ff34885e2499e81ab5faa57c544cff9b68b`;
T17.2 `e6a2e82e6566d590564de3d3ac6028160f953028`.
Validation: required Cargo/source commands passed; `cargo test --test windows`
passed ten tests and the terminal PTY suite passed thirty cases with zero skips.
Integrated into `dev` at `db4f7813fdf3ee905b4447a083a6d2ff3901dfbe`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
PR: [#17](https://forge.adrianpk.com/adrianpk/rev/pulls/17).
Report: [Slice 17](../report/slices/rev-editor/slice-17-splits.md).

### Slice 18: Delete and cycle editing windows

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R14, R15.

- [x] T18.1: Implement delete-window, delete-other-windows, and deterministic other-window traversal with wrapping and a separate tree-focus path.
- [x] T18.2: Exercise repeated split/delete, deleting the sole view, selected-view replacement, and preservation of unsaved buffers after their last visible window closes.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T18.1 and T18.2 delivered through PR #18.
Task commits: T18.1 `56a933e7857aed9896435025fae3d74d0ea9074d`;
T18.2 `fa7726410cfd25766adbd81a8ed23c3ce2c4ef69`.
Validation: required Cargo/source commands passed; `cargo test --test windows`
passed twenty tests and the terminal PTY suite passed thirty-three cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#18](https://forge.adrianpk.com/adrianpk/rev/pulls/18).
Integration: Rebase + Fast-forward into dev at
`9273fafb0d6bb86a563e8fb512d5aa61de406379`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 18](../report/slices/rev-editor/slice-18-window-commands.md).

### Slice 19: Map shared-buffer edits across views

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R08, R09, R11, R14, R17, R21.

- [x] T19.1: Complete point/top/mark mapping for insertion, deletion, replacement, undo, and redo across every view of the same buffer using the accepted affinities.
- [x] T19.2: Exercise edits before and across another cursor, deletion of visible lines, killing a multiply visible buffer, and history initiated from a view that later closes.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T19.1 and T19.2 delivered through PR #19.
Task commits: T19.1 `b0f19612089f873c8dd2cd7592479ff2f0020739`;
T19.2 `9849102407d640f212ea27808b29b3c9e63183db` with invalid-range fixture
`e9b129320fe6a00225d7be3a6323a195940a5534`.
Validation: required Cargo/source commands passed; `cargo test --test windows`
passed twenty-nine tests and the terminal PTY suite passed thirty-six cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#19](https://forge.adrianpk.com/adrianpk/rev/pulls/19).
Integration: Rebase + Fast-forward into dev at
`8295f8b32759eed0c88a811019eceb3fe6c107c8`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 19](../report/slices/rev-editor/slice-19-shared-views.md).

### Slice 20: Handle tiny and changing terminal geometry

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R03, R05, R06, R07, R14.

- [x] T20.1: Recompute nested geometry on resize, preserve the logical layout under suspended areas, and apply minimum pane/status/minibuffer dimensions without underflow.
- [x] T20.2: Exercise zero dimensions, extremely narrow/tall/wide reports, repeated resize bursts, rejected splits, and restoration of usable viewports after expansion.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T20.1 and T20.2 delivered through PR #20.
Task commits: T20.1 `18647c936a7172319c5c96b2c7af598cf8cb2a44`;
T20.2 `555985b025019931a37c6d5903d5f07ce1f3ec0c`.
Validation: required Cargo/source commands passed; `cargo test --test windows`
passed thirty-four tests and the terminal PTY suite passed thirty-nine cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#20](https://forge.adrianpk.com/adrianpk/rev/pulls/20).
Integration: Rebase + Fast-forward into dev at
`95cca8e1ffe5c510cf030ae2d415d21898d5bae0`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 20](../report/slices/rev-editor/slice-20-resize.md).

### Slice 21: Activate and render linear regions

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R07, R18, R24.

- [x] T21.1: Implement set-mark-command, exchange-point-and-mark, movement extension, region deactivation, and focused multi-line selection rendering.
- [x] T21.2: Exercise reversed/empty ranges, region updates through shared edits, focus changes, viewport clipping, and C-g cancellation without losing stored mark unnecessarily.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T21.1 and T21.2 delivered through PR #21.
Task commits: T21.1 `1e91e88dc3e8f837d80b4ae987d90a3a02f95f64`;
T21.2 `25bca45422f59fbd9548ab8017fc0e428c42e4c2`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed fifty-one tests and the terminal PTY suite passed forty-two cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#21](https://forge.adrianpk.com/adrianpk/rev/pulls/21).
Integration: Rebase + Fast-forward into dev at
`29d62389f3d0a1ebd3a381acbe4b920498fb16b7`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 21](../report/slices/rev-editor/slice-21-regions.md).

### Slice 22: Kill lines and words directionally

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R17, R19, R21.

- [x] T22.1: Implement kill-line, kill-word, and backward-kill-word through checked transactions, including coherent CRLF removal at logical line end.
- [x] T22.2: Exercise boundaries, Unicode punctuation/words, directional coalescing, read-only rejection, and one-step undo of each kill without unrelated changes.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T22.1 and T22.2 delivered through PR #22.
Task commits: T22.1 `142154d0ed100c595ce03e21096a3dca6bcc03bb`;
T22.2 `a956cbee3c9880cbf15e2664f9b1d0959552ff22`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed sixty-three tests and the terminal PTY suite passed forty-four cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#22](https://forge.adrianpk.com/adrianpk/rev/pulls/22).
Integration: Rebase + Fast-forward into dev at
`e1a83d8fd532dff7aff461269668301a27af9321`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 22](../report/slices/rev-editor/slice-22-kills.md).

### Slice 23: Copy kill and yank linear text

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R18, R19, R21.

- [x] T23.1: Implement kill-region, kill-ring-save, yank, a 32-entry session ring, and insertion transaction grouping while keeping rectangle storage separate.
- [x] T23.2: Exercise copying without mutation, reversed regions, empty ring, multiline Unicode/CRLF text, ring eviction, and undo after yanking.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T23.1 and T23.2 delivered through PR #23.
Task commits: T23.1 `e167d8c7cf8d8ee907a09996c146f47655cab0f5`;
T23.2 `4ee8ffb17378608dd1cf6b391d9c33b995fc0601`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed seventy-seven tests and the terminal PTY suite passed forty-six cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#23](https://forge.adrianpk.com/adrianpk/rev/pulls/23).
Integration: Rebase + Fast-forward into dev at
`c2f20bec35a8f3d2d5e5815b780f027951ea58d4`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 23](../report/slices/rev-editor/slice-23-kill-ring.md).

### Slice 24: Rotate only the current yank range

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R19, R21.

- [x] T24.1: Implement yank-pop with buffer/view/revision-qualified inserted range and rotation; replace exactly the immediately preceding yank sequence.
- [x] T24.2: Exercise unequal-length entries, repeated rotations, unrelated commands, edits in another view, undo, and attempted pop before any yank.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T24.1 and T24.2 delivered through PR #24.
Task commits: T24.1 `a538935b48083ba5806434472300f0d7b8d4ab07`;
T24.2 `4c2675d43c564170a1c2a433b1f01e110ee5c3ba`.
Validation: required Cargo/source commands passed; `cargo test --test editing`
passed ninety-one tests and the terminal PTY suite passed forty-eight cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#24](https://forge.adrianpk.com/adrianpk/rev/pulls/24).
Integration: Rebase + Fast-forward into dev at
`50ccd9554653dea4e5356d77c22a5dcde8464922`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 24](../report/slices/rev-editor/slice-24-yank-pop.md).

### Slice 25: Resolve Meta Escape and undo aliases

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R21, R24, R25.

- [x] T25.1: Complete Alt/Meta and compatible Escape-prefix input using a finite nonblocking deadline, standalone Escape, C-SPC aliases, and reportable undo alternatives.
- [x] T25.2: Exercise C-/, C-_, C-x u, M-<, M->, nested rectangle chords, key repeat/release, unknown escape input, and cancellation during a pending Meta prefix.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T25.1 and T25.2 delivered through PR #25.
Task commits: T25.1 `3c1f54f394d1c445ca78491a8636d8925347561c`;
T25.2 `c3be5ef27235601b0752d524e6a9a3ef64c447fc`.
Validation: required Cargo/source commands passed; `cargo test --test commands`
passed twenty-six tests and the terminal PTY suite passed fifty cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#25](https://forge.adrianpk.com/adrianpk/rev/pulls/25).
Integration: Rebase + Fast-forward into dev at
`68c55eb542060a0b99ef2ba90118e0a975da7105`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 25](../report/slices/rev-editor/slice-25-terminal-keys.md).

### Slice 26: Insert bracketed Unicode paste atomically

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R10, R17, R21, R25.

- [x] T26.1: Enable supported bracketed paste, insert literal Unicode/newline text through one transaction, and avoid feeding pasted key-looking content to command decoding.
- [x] T26.2: Exercise large multiline paste, CRLF policy, control-looking text, terminal release/repeat noise, cancellation boundaries, and one-step undo across shared views.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T26.1 and T26.2 delivered through PR #26.
Task commits: T26.1 `c944a880f3ef90d640df88c51daa3354261d07d6`;
T26.2 `fd88fccd2936c41dd2c88c3fddef5dce8c9ee953`.
Validation: required Cargo/source commands passed; `cargo test --test commands`
passed thirty-eight tests and the terminal PTY suite passed fifty-four cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#26](https://forge.adrianpk.com/adrianpk/rev/pulls/26).
Integration: Rebase + Fast-forward into dev at
`2dbaa9b2cc86e7ee9d3c253f55aaa6396fcd6d5d`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 26](../report/slices/rev-editor/slice-26-paste.md).

### Slice 27: Run discoverable commands through M-x

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R22, R23.

- [x] T27.1: Expose implemented fixed commands through M-x with lowercase hyphenated names, deterministic prefix/substring filtering, TAB selection/completion, RET invocation, and session history.
- [x] T27.2: Exercise unknown names, empty candidates, cancel, command history, argument continuation, and binding/name parity for file, window, editing, and history operations.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T27.1 and T27.2 delivered through PR #27.
Task commits: T27.1 `57f42e8a1fdc3ad80d2d89c725d75e9f32f3d22b`;
T27.2 `3e4e30cbbd9c1bee8d8a5c9135ad13ad0b194c11`.
Validation: required Cargo/source commands passed; `cargo test --test commands`
passed fifty-one tests and the terminal PTY suite passed fifty-six cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#27](https://forge.adrianpk.com/adrianpk/rev/pulls/27).
Integration: Rebase + Fast-forward into dev at
`b378080307fba5ede92970d3011c61037485d132`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 27](../report/slices/rev-editor/slice-27-command-palette.md).

### Slice 28: Complete focused argument and cancel flows

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R04, R12, R23, R24.

- [x] T28.1: Complete predictable multi-stage command arguments, path/buffer prompts, decision prompts, in-prompt editing, and focus restoration after accept/error/cancel.
- [x] T28.2: Exercise cancellation at every stage of find/write/kill, prompt editing with Emacs movement, resize while prompting, and no document mutation from transient input.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T28.1 and T28.2 delivered through PR #28.
Task commits: T28.1 `6f6cb8a889335c8e14596e81717910f557523c85`;
T28.2 `716a3022b6edb898e41fe0c6bf6cef5f940d2d42`.
Validation: required Cargo/source commands passed; `cargo test --test commands`
passed sixty-two tests and the terminal PTY suite passed fifty-eight cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#28](https://forge.adrianpk.com/adrianpk/rev/pulls/28).
Integration: Rebase + Fast-forward into dev at
`543b5df1281504aae93d5220962583f6a8a882f2`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 28](../report/slices/rev-editor/slice-28-prompt-flows.md).

### Slice 29: Search forward across rope chunks

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R07, R09, R26.

- [x] T29.1: Implement isearch-forward with visible minibuffer query, chunk-crossing literal matching, next-match continuation, source-qualified results, and match reveal/highlighting.
- [x] T29.2: Exercise Unicode chunk seams, multiline text, incremental query changes, no-match behavior, and searches on representative larger ropes without per-key whole-file copies.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T29.1 and T29.2 delivered through PR #29.
Task commits: T29.1 `65df69161fa93616a191f50f2c70bfdcad22cf98`;
T29.2 `3ba1af483b66be0caf17f6d676d0c9ca8157a9bc`.
Validation: required Cargo/source commands passed; `cargo test --test search`
passed eleven tests and the terminal PTY suite passed sixty cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#29](https://forge.adrianpk.com/adrianpk/rev/pulls/29).
Integration: Rebase + Fast-forward into dev at
`9e7decac0237df6877707fee780801b858487fa2`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 29](../report/slices/rev-editor/slice-29-forward-search.md).

### Slice 30: Search backward with smart case and cancel

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R24, R26.

- [x] T30.1: Implement isearch-backward, direction repetition, smart-case query matching, RET acceptance, and C-g restoration of the exact starting view/point.
- [x] T30.2: Exercise mixed Unicode case, backward chunk seams, query edits, match boundaries, direction changes, failed search, and cancellation after scrolling.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T30.1 and T30.2 delivered through PR #30.
Task commits: T30.1 `c41066a63d82a1eb10961642046b60cbaa42a530`;
T30.2 `d03adaff8e5475a8aca97f23cbc13bf3236da4d5`.
Validation: required Cargo/source commands passed; `cargo test --test search`
passed nineteen tests and the terminal PTY suite passed sixty-two cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#30](https://forge.adrianpk.com/adrianpk/rev/pulls/30).
Integration: Rebase + Fast-forward into dev at
`144155023f101134aa4fb50320049d9fe96d8bf3`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 30](../report/slices/rev-editor/slice-30-search-control.md).

### Slice 31: Confirm literal replacements interactively

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R21, R23, R27.

- [x] T31.1: Implement M-% and query-replace prompts followed by yes/no/quit decisions, applying each accepted occurrence through the canonical checked transaction.
- [x] T31.2: Exercise replacement lengths, Unicode/CRLF text, skipped matches, no matches, cancellation, read-only buffers, and undo of accepted replacements.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T31.1 and T31.2 delivered through PR #31.
Task commits: T31.1 `802e14981c2f83f1211786ca16cba39aadf06d45`;
T31.2 `2231f92989703ee543c68036c7c64ac96fde60f5`.
Validation: required Cargo/source commands passed; `cargo test --test search`
passed thirty tests and the terminal PTY suite passed sixty-four cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#31](https://forge.adrianpk.com/adrianpk/rev/pulls/31).
Integration: Rebase + Fast-forward into dev at
`64b274d8c1b2b617bfb263dd362fe6a515932a1a`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 31](../report/slices/rev-editor/slice-31-query-replace.md).

### Slice 32: Select and copy display-column rectangles

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R07, R09, R20.

- [x] T32.1: Implement rectangle-mark-mode, cell-based row extraction, whole-cluster edge inclusion, partial-tab representation, and copy-rectangle-as-kill in separate rectangle storage.
- [x] T32.2: Exercise reversed endpoints, CJK/emoji/combining clusters, tab boundaries, short/empty lines, scrolling, and copy without source padding or mutation.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T32.1 and T32.2 delivered through PR #32.
Task commits: T32.1 `c68bd85ba5702d8c6a7f46d2be23557b1c010655`,
qualified at `d7b17247f5265a2678196aeed944e88a49e435c3`;
T32.2 `3b0d40c28bfd16ed6bf468024067410cfa08fe29`.
Validation: required Cargo/source commands passed; `cargo test --test rectangles`
passed twelve tests and the terminal PTY suite passed sixty-six cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#32](https://forge.adrianpk.com/adrianpk/rev/pulls/32).
Integration: Rebase + Fast-forward into dev at
`42b879a78263a3cb5dd6dcc24cd920a9173a14f9`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 32](../report/slices/rev-editor/slice-32-rectangle-copy.md).

### Slice 33: Kill rectangles as one transaction

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R09, R20, R21.

- [x] T33.1: Implement kill-rectangle with all row changes prepared in a declared coordinate space, retaining the rectangle and mapping every affected live view coherently.
- [x] T33.2: Exercise partially touched tabs, wide clusters, empty rows, read-only/invalid admission, large row counts, and atomic undo/redo without partial row publication.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T33.1 and T33.2 delivered through PR #33.
Task commits: T33.1 `cc6c158a59fd71ab025eb01b96264ff21de04fc1`;
T33.2 `e99ace6bb3d6841e9e5cdd2196ba1a091091cedc`.
Validation: required Cargo/source commands passed; `cargo test --test rectangles`
passed 23 tests and the terminal PTY suite passed 68 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#33](https://forge.adrianpk.com/adrianpk/rev/pulls/33).
Integration: Rebase + Fast-forward into dev at
`80ae94eb8b3b71999e79a8d00cfa23873be4f5c9`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 33](../report/slices/rev-editor/slice-33-rectangle-kill.md).

### Slice 34: Yank blocks with coherent padding and undo

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R10, R20, R21.

- [x] T34.1: Implement yank-rectangle across successive lines, deterministic insertion at target cell boundaries, necessary short-line padding, and additional lines under the newline policy.
- [x] T34.2: Exercise unequal source widths, wide target graphemes, partial tabs, absent rectangle, insertion past the final line, and undo of padding plus all inserted rows.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T34.1 and T34.2 delivered through PR #34.
Task commits: T34.1 `ec1f99efc1f9d8645c7816819776e662a806774e`;
T34.2 `e061feba7e4b35e844ca905ad164be904e3e2a09`.
Validation: required Cargo/source commands passed; `cargo test --test rectangles`
passed 34 tests and the terminal PTY suite passed 70 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#34](https://forge.adrianpk.com/adrianpk/rev/pulls/34).
Integration: Rebase + Fast-forward into dev at
`014661dbb26aaa1941460f675c2f519c49317830`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 34](../report/slices/rev-editor/slice-34-rectangle-yank.md).

### Slice 35: Render and focus the left file tree

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R05, R06, R28.

- [x] T35.1: Implement toggle-file-tree, root selection, a narrow left allocation, selected entry indication, and predictable focus transfer retaining the last editing view.
- [x] T35.2: Exercise show/hide with dirty buffers, M-x invocation, narrow-width auto-hide, resize restoration, and exclusion from other-window cycling.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.


Implementation: T35.1 and T35.2 delivered through PR #35.
Task commits: T35.1 `7662b67f95c256a78986ed3a7a923ea587e9da7f`;
T35.2 `929c8c881156181e20bd6a06c7fd965d7989259c`.
Validation: required Cargo/source commands passed; `cargo test --test tree`
passed ten tests and the terminal PTY suite passed 72 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#35](https://forge.adrianpk.com/adrianpk/rev/pulls/35).
Integration: Rebase + Fast-forward into dev at
`242b1f8c155d6a8af9d750075a50dac87eb2180b`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 35](../report/slices/rev-editor/slice-35-tree-pane.md).

### Slice 36: Traverse directories lazily and safely

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R28, R29.

- [x] T36.1: Implement Up/Down visible traversal, directory expansion/collapse, deterministic ordering, lazy child reads, explicit refresh, and symlinked-directory leaves.
- [x] T36.2: Exercise deep trees, symlink cycles, inaccessible/deleted directories, stale selections, and refresh after external filesystem changes.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.


Implementation: T36.1 and T36.2 delivered through PR #36.
Task commits: T36.1 `62490595d7d8803365e083d8168f26cf08e261b7`;
T36.2 `391dfd769d054c173b1794e26ef18ed07e63b5f5`, corrected at
`b2d2144e6eaa66c1add562361b6c9f4ff32fb777`.
Validation: required Cargo/source commands passed; `cargo test --test tree`
passed 22 tests and the terminal PTY suite passed 74 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#36](https://forge.adrianpk.com/adrianpk/rev/pulls/36).
Integration: Rebase + Fast-forward into dev at
`50ec7694c578e327b272b481b9da7c707a8321cf`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 36](../report/slices/rev-editor/slice-36-tree-navigation.md).

### Slice 37: Open and create tree entries

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R11, R12, R28, R30.

- [x] T37.1: Open selected files through find-file and add prompted new-file/new-directory creation at the selected directory, validating names and conflicts before disk mutation.
- [x] T37.2: Exercise existing visits, unsupported extension fallback, occupied/invalid names, permission errors, canceled creation, and synchronization of tree entries with buffers.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.


Implementation: T37.1 and T37.2 delivered through PR #37.
Task commits: T37.1 `215ef188ed4494036f1d193a61ef85532bb816f2`, completed at
`6f72e3bd1b93883aec5b68529f009dcb0cd770ec`;
T37.2 `0abd8a723fdc6fe47e440f6c07b549353580e3a8`.
Validation: required Cargo/source commands passed; `cargo test --test tree`
passed 33 tests and the terminal PTY suite passed 76 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#37](https://forge.adrianpk.com/adrianpk/rev/pulls/37).
Integration: Rebase + Fast-forward into dev at
`b7c647290236a471f17e4c3a2c5fe8f0a54688be`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 37](../report/slices/rev-editor/slice-37-tree-create.md).

### Slice 38: Reconcile visited paths after tree rename

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R11, R13, R30.

- [x] T38.1: Implement prompted rename with conflict checks, canonical path/index updates, and descendant visited-path reconciliation for a renamed directory.
- [x] T38.2: Exercise occupied targets, visited buffers with equal basenames, symlink entries, permission failure, cancellation, and a later save using the successful new path.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.


Implementation: delivered through PR #38.
Task commits: T38.1 `4960ae4b778a5da24a6b4f9426d765a74cd507ec`;
T38.2 `063ffc65aaa8a553e8eeb7d5260a4661ccd55f9d`.
Validation: required Cargo/source commands passed; `cargo test --test tree`
passed 46 tests and the terminal PTY suite passed 78 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#38](https://forge.adrianpk.com/adrianpk/rev/pulls/38).
Integration: `e227443be27477c787d0451ee1f1c50c052b1140`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 38](../report/slices/rev-editor/slice-38-tree-rename.md).

### Slice 39: Confirm deletion while preserving open text

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R04, R11, R29, R30.

- [x] T39.1: Implement explicit delete confirmation and a separate nonempty-directory recursive confirmation, reconciling retained visiting buffers as missing under D07.
- [x] T39.2: Exercise canceled/nonempty deletion, dirty visiting files, inaccessible entries, symlink deletion without referent recursion, and refresh after partial filesystem errors.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: delivered through PR #39.
Task commits: T39.1 `e83b50ec10f31e6129f48b9ad6e6cb5bf1252d28`;
T39.2 `fa5d92ce59fc2608d028bb01027a2429c731cc11`.
Validation: required Cargo/source commands passed; `cargo test --test tree`
passed 56 tests and the terminal PTY suite passed 80 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#39](https://forge.adrianpk.com/adrianpk/rev/pulls/39).
Integration: `63db77602379fb04b39ec2f592bed51552aee5f5`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 39](../report/slices/rev-editor/slice-39-tree-delete.md).

### Slice 40: Bundle Rust syntax highlighting

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R05, R31, R34.

- [x] T40.1: Integrate the pinned Rust grammar and bundled HIGHLIGHTS_QUERY with safe Tree-sitter APIs, a small capture palette, and extension selection with plain-text fallback.
- [x] T40.2: Exercise Rust keywords/strings/comments plus grammar-supported types/functions/numbers/operators, unsupported files, incomplete source, and missing/failed query fallback.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: delivered through PR #40.
Task commits: T40.1 `ad71197b9defa146a2ac02b1a824979465e48c2f`;
Clippy assertion correction `07faa9a631bae8aea364cefc939a8ca6044ca74b`;
T40.2 `a916ffa8507896869e172e3679188bbb3ce868ba`.
Validation: required Cargo/source commands passed; `cargo test --test syntax`
passed 10 tests and the terminal PTY suite passed 82 cases with zero skips.
Final PR-head evidence is recorded through Looper.
PR: [#40](https://forge.adrianpk.com/adrianpk/rev/pulls/40).
Integration: `c2c5e08dad8669ac68b7c5b7e0fae868b376c939`; all seven ordered checks passed
on that exact head, with merge activity recorded by the controller.
Report: [Slice 40](../report/slices/rev-editor/slice-40-rust-highlights.md).

### Slice 41: Update syntax from exact rope edits

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R09, R32, R40.

- [x] T41.1: Translate every accepted edit/undo/redo/rectangle/paste to correct Tree-sitter byte and row/byte-column edits, retaining parser/tree state and bounding visible capture work.
- [x] T41.2: Exercise multibyte edits before captures, shared windows, parse/query failures, selection/search/cursor layering, and stale syntax invalidation without whole-file rebuild per key.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: delivered through PR #41.
Task commits: T41.1 `41d9253f826832cdcabce9a5184922924acfb421`;
T41.2 `6bd57189b9565dd936af46c9104d207419929bfb`.
Validation: required Cargo/source commands passed; `cargo test --test syntax`
passed 20 tests and the terminal PTY suite passed 84 cases with zero skips.
Integration: `09840bfebb3a4fdeb041a458d7e502790e8e5142`; all seven ordered checks passed at this exact head through Looper (`6QZKA7LNZQYXFCVG432M3RKBAJ`).
PR: [#41](https://forge.adrianpk.com/adrianpk/rev/pulls/41).
Report: [Slice 41](../report/slices/rev-editor/slice-41-incremental-syntax.md).

### Slice 42: Complete guarded multi-buffer session exit

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R03, R04, R11, R13, R21, R24.

- [x] T42.1: Complete save-buffers-kill-terminal across all modified named and unnamed buffers, including sequential save decisions, canceled path prompts, read-only and uncertain-publication outcomes.
- [x] T42.2: Exercise cancellation after earlier saves, failed required saves, several windows of one buffer, a removed destination, and final terminal restoration after a genuinely accepted quit.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: delivered through PR #42.
Task commits: T42.1 `28058fec1a0122eaba717c25089908cc793d12ff`;
T42.2 `d0674f73fcb8b185ceb9ed7e7d5bfd572f2deeb8`. Retained journey adaptation:
`577e8c2c0b21e93ac8edd514ac11741b2879930f`.
Validation: required Cargo/source commands passed; `cargo test --test recovery`
passed nine tests and the terminal PTY suite passed 86 cases with zero skips.
Integration: `0449afb084d68186400077b00c907ea820a4ae9b`; all seven ordered checks passed at this exact head through Looper (`PGK434SPJFCKEX3CH3PX6Z2RWG`).
PR: [#42](https://forge.adrianpk.com/adrianpk/rev/pulls/42).
Report: [Slice 42](../report/slices/rev-editor/slice-42-session-quit.md).

### Slice 43: Harden ordinary failure boundaries

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R03, R04, R06, R10, R13, R20, R25, R30, R32, R37, R39.

- [x] T43.1: Extend real boundary fault injection for recoverable ingress, save stages, tree operations, parser/query execution, and post-acquisition terminal errors, retaining truthful outcomes.
- [x] T43.2: Exercise combined failures and cleanup ordering, canceled transient interactions, readonly rejection, and state equality after rejected compound edits. Keep fixes within already approved behavior.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: delivered through PR #43.
Task commits: T43.1 `2e45d275864e289f10c83776166569fe197fce3b`;
T43.2 `21cde797e6cc8289178dac0d7d118ce0dd9b437c`.
Validation: required Cargo/source commands passed; `cargo test --test recovery`
passed 16 tests and the terminal PTY suite passed 90 cases with zero skips.
Integration: `99c13fa2b99cecb5553bf2c5074edeeb6ba6c8ae`; all seven ordered checks passed at this exact head through Looper (`YAINMMT7QKOFJDQQCAPVMFA422`).
PR: [#43](https://forge.adrianpk.com/adrianpk/rev/pulls/43).
Report: [Slice 43](../report/slices/rev-editor/slice-43-failure-recovery.md).

### Slice 44: Drive user journeys through a PTY

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R01, R03, R04, R12, R14, R15, R18, R19, R20, R22, R25, R26, R27, R28, R31, R39.

- [x] T44.1: Extend the existing standard-library Python PTY harness to drive launch/edit/save/reopen, bindings and M-x, buffer/window operations, selection/rings/rectangles, search, tree actions, and quit.
- [x] T44.2: Use bounded condition waits, owned fixtures, real filesystem byte comparisons, resize events, and termios/screen restoration checks. Separate pure-state assertions from terminal encoding variability.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T44.1 and T44.2 delivered through PR #44.
Task commits: T44.1 `c4b87864c2fe712ad819f0884df2744294c55664`;
T44.2 `2de738248d6db0dac5a2a83e65ea43f32b8e79cd`.
Validation: required Cargo/source commands passed; the acceptance PTY suite
passed four connected journeys and the terminal PTY suite passed 90 cases,
with zero skips. Final PR-head evidence is recorded through Looper.
Integration: `0e341d4a277e3b7b6159d1fa9c7978defe3d16b8`; all seven checks passed
under Looper validation `VJ6DG2YGPMBNU7JBLOS5HGAHYM`.
PR: [#44](https://forge.adrianpk.com/adrianpk/rev/pulls/44).
Report: [Slice 44](../report/slices/rev-editor/slice-44-terminal-journeys.md).

### Slice 45: Exercise cross-feature Unicode and file fidelity

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R09, R10, R19, R20, R21, R26, R30, R32, R39.

- [x] T45.1: Add distinct adversarial regressions across editing/history/search/syntax and file save/reopen using emoji, combining text, CJK, tabs, CRLF, BOM, absent final newline, long lines, and shared views.
- [x] T45.2: Exercise repeated split/close/resize, invalid yank-pop, rectangle seams, symlink loops, inaccessible paths, and canceled destructive actions. Reuse subsystem suites rather than duplicate every journey.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T45.1 and T45.2 delivered through PR #45.
Task commits: T45.1 `84c23e4f3c1b546574cafbc834bce7adaf2aeb31`;
T45.2 `24ca6751d3806673884cb0b20d594e3a8c7fa7dc`; fixture correction `2926c64622fb85451d772a961cf1393b7ea9bd4e`.
Validation: all five ordered required/focused checks passed through Looper,
including 454 native tests with zero ignored. Final PR-head evidence follows
the report and PR-reference commits. The shared focused/full Cargo command runs
once in the ordered list.
Integration: `b5c73f7cc61f95e51ea4956b6e8fc86335257530`; all five ordered checks passed
under Looper validation `ZJBTVBRCCNMOUJSGF627SKHLI2`.
PR: [#45](https://forge.adrianpk.com/adrianpk/rev/pulls/45).
Report: [Slice 45](../report/slices/rev-editor/slice-45-adversarial-text.md).

### Slice 46: Measure representative editing workloads

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R08, R26, R32, R40.

- [x] T46.1: Add a deterministic mixed-width UTF-8 fixture generator with tens of thousands of lines and measure startup, insert/delete, scroll, split redraw, and highlighting on recorded hardware context.
- [x] T46.2: Inspect/measure work for pathological whole-file copying/reparse and idle repaint loops, correcting only demonstrated in-scope causes. Store concise fixture/measurement evidence without hostnames or invented guarantees.
- [x] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T46.1 and T46.2 delivered through PR #46.
Task commits: T46.1 `c5c2f3ea49e0cdb1a0dd1cf9ac476b3d3ee17b4e`;
T46.2 `b67b5b3d5aabd03c396f407fcc44a4e3f9f91d01`.
Validation: all six ordered checks passed through Looper, including 454 native
tests and the performance command's two large fixtures, eight native workloads,
parser-work comparison and real PTY startup/idle/restoration. Final PR-head
evidence follows the report and PR-reference commits.
Integration: `299b78a06ebd413d4064ef90093fecbccbe820ed`; all six ordered checks passed
under Looper validation `GIX4W5KHFHQ5ZID5ACWNTE5TCZ`.
PR: [#46](https://forge.adrianpk.com/adrianpk/rev/pulls/46).
Report: [Slice 46](../report/slices/rev-editor/slice-46-performance.md).

### Slice 47: Audit source quality and product reference

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R01, R22, R25, R33, R34, R35, R36, R37, R38.

- [ ] T47.1: Review actual production functions for cyclomatic paths, at most five parameters including self, shallow nesting, concise names, restrained comments, ownership, dependency necessity, and complete real command catalog.
- [ ] T47.2: Finish README usage plus concise Diataxis reference/how-to material for bindings, scope, terminal limitations, file/undo defaults, and recovery. Keep Unreleased changelog outcomes accurate and complete.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 48: Verify the complete integrated editor contract

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R22, R23, R24, R25, R26, R27, R28, R29, R30, R31, R32, R33, R34, R35, R36, R37, R38, R39, R40.

- [ ] T48.1: Complete a reviewed A01-A20 evidence map linking the real regression/PTY/performance checks, close any remaining within-scope integration defects, and make scripts/check-all.sh invoke the exact mandatory gate without skips.
- [ ] T48.2: Exercise the assembled user journeys through the built executable and verify all gate prerequisites, reports, required commands, and unchanged identities. Leave primary dev clean and pushed for controller-owned final validation.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

## Integrated closure

- [ ] All 48 canonical slice PRs are merged into dev and every task outcome is recorded.
- [ ] R01-R40 and A01-A20 have real, current acceptance evidence with no required skips.
- [ ] The controller full gate passes on the exact clean integrated candidate.
- [ ] All 48 reports are delivered and dev is clean and synchronized with origin/dev.
- [ ] Plan/tracker are completed and post-gate changes stay within registered closure paths.
