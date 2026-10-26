# rev Editor Tracker

Status: approved
Approved: product owner, 2026-10-26; complete baseline implementation and Looper startup requested.
Delivery set: rev-editor
Active slice: Slice 12 - Load exact UTF-8 file content
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

- [ ] T12.1: Load existing files into canonical ropes with strict UTF-8, conservative NUL binary rejection, exact LF/CRLF and final-newline retention, recognized BOM metadata, and prevailing newline preference.
- [ ] T12.2: Exercise invalid encoding, empty files, mixed line endings, BOM round trips, missing paths, permission failures, and preservation of the previous active buffer on rejected ingress.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

Implementation: T12.1 and T12.2 implemented through PR #12.
Task commits: T12.1 `5f874847bc942eb55481037004383fb01d46699f` with startup
ownership `69ad76b8616b1139b27aa9b6cdc8f057ac4be078`;
T12.2 `f012fa51de4fc8d29f1afcfc18f37e5d2fb9053f` with permission fixture
`c744436be30be980975a269f3ff228da2d94c697`.
Validation: required Cargo/source commands passed; `cargo test --test files`
passed eleven tests and the terminal PTY suite passed twenty cases with zero skips.
Final PR-head evidence is recorded through Looper after PR references are committed.
PR: [#12](https://forge.adrianpk.com/adrianpk/rev/pulls/12).
Report: [Slice 12](../report/slices/rev-editor/slice-12-file-ingress.md).

### Slice 13: Reuse canonical visited files

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R02, R11, R12, R23.

- [ ] T13.1: Implement find-file argument flow, normalized intended paths for absent files, canonical existing referents, unique buffer labels, and existing-visit activation shared by all callers.
- [ ] T13.2: Exercise symlink aliases, duplicate basenames, nonexistent parents, canceled prompts, directory paths, and collision with an already visited canonical target.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 14: Publish safe same-directory saves

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R10, R13, R21.

- [ ] T14.1: Implement save-buffer using captured rope text, same-directory temporary output, checked writes, permission preservation where practical, replacement, and explicit completion/publication outcomes.
- [ ] T14.2: Exercise injected write/flush/replace/post-replace failures and cleanup. Reconcile actual publication, saved logical state, dirty flags, and a first successful file association truthfully.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 15: Confirm overwrite and external changes

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R12, R13, R23.

- [ ] T15.1: Implement write-file, unnamed save prompts, occupied-path confirmation, metadata-based external-change warnings, canonical symlink referent saving, and live-buffer target collision refusal.
- [ ] T15.2: Exercise cancellation, denied/disk-full writes, removed paths, save-as before/after publication failures, and clean/dirty history state after successful save and undo.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 16: Switch list and close buffers

Inputs: [Part 2](../plan/rev-editor/02-files-and-history.md); requirements R04, R05, R11, R12, R23.

- [ ] T16.1: Implement switch-to-buffer completion, list-buffers selection, kill-buffer decisions, deterministic replacement, and unique labels independent of window lifetime.
- [ ] T16.2: Exercise several modified/unnamed/file-backed buffers, canceled close, failed required save, final-buffer replacement, and index cleanup. Preserve open text when a path disappears.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 17: Create nested editing splits

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R06, R07, R14, R33.

- [ ] T17.1: Implement split-window-below and split-window-right with nested layout nodes, geometry admission, initial same-buffer view state, and retained original focus.
- [ ] T17.2: Exercise mixed split directions, minimum dimensions, allocation of distinct view IDs, and independent point/scroll after split. Refuse non-fitting changes before mutating layout.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 18: Delete and cycle editing windows

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R14, R15.

- [ ] T18.1: Implement delete-window, delete-other-windows, and deterministic other-window traversal with wrapping and a separate tree-focus path.
- [ ] T18.2: Exercise repeated split/delete, deleting the sole view, selected-view replacement, and preservation of unsaved buffers after their last visible window closes.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 19: Map shared-buffer edits across views

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R08, R09, R11, R14, R17, R21.

- [ ] T19.1: Complete point/top/mark mapping for insertion, deletion, replacement, undo, and redo across every view of the same buffer using the accepted affinities.
- [ ] T19.2: Exercise edits before and across another cursor, deletion of visible lines, killing a multiply visible buffer, and history initiated from a view that later closes.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 20: Handle tiny and changing terminal geometry

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R03, R05, R06, R07, R14.

- [ ] T20.1: Recompute nested geometry on resize, preserve the logical layout under suspended areas, and apply minimum pane/status/minibuffer dimensions without underflow.
- [ ] T20.2: Exercise zero dimensions, extremely narrow/tall/wide reports, repeated resize bursts, rejected splits, and restoration of usable viewports after expansion.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 21: Activate and render linear regions

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R07, R18, R24.

- [ ] T21.1: Implement set-mark-command, exchange-point-and-mark, movement extension, region deactivation, and focused multi-line selection rendering.
- [ ] T21.2: Exercise reversed/empty ranges, region updates through shared edits, focus changes, viewport clipping, and C-g cancellation without losing stored mark unnecessarily.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 22: Kill lines and words directionally

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R17, R19, R21.

- [ ] T22.1: Implement kill-line, kill-word, and backward-kill-word through checked transactions, including coherent CRLF removal at logical line end.
- [ ] T22.2: Exercise boundaries, Unicode punctuation/words, directional coalescing, read-only rejection, and one-step undo of each kill without unrelated changes.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 23: Copy kill and yank linear text

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R18, R19, R21.

- [ ] T23.1: Implement kill-region, kill-ring-save, yank, a 32-entry session ring, and insertion transaction grouping while keeping rectangle storage separate.
- [ ] T23.2: Exercise copying without mutation, reversed regions, empty ring, multiline Unicode/CRLF text, ring eviction, and undo after yanking.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 24: Rotate only the current yank range

Inputs: [Part 3](../plan/rev-editor/03-windows-and-regions.md); requirements R19, R21.

- [ ] T24.1: Implement yank-pop with buffer/view/revision-qualified inserted range and rotation; replace exactly the immediately preceding yank sequence.
- [ ] T24.2: Exercise unequal-length entries, repeated rotations, unrelated commands, edits in another view, undo, and attempted pop before any yank.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 25: Resolve Meta Escape and undo aliases

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R21, R24, R25.

- [ ] T25.1: Complete Alt/Meta and compatible Escape-prefix input using a finite nonblocking deadline, standalone Escape, C-SPC aliases, and reportable undo alternatives.
- [ ] T25.2: Exercise C-/, C-_, C-x u, M-<, M->, nested rectangle chords, key repeat/release, unknown escape input, and cancellation during a pending Meta prefix.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 26: Insert bracketed Unicode paste atomically

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R10, R17, R21, R25.

- [ ] T26.1: Enable supported bracketed paste, insert literal Unicode/newline text through one transaction, and avoid feeding pasted key-looking content to command decoding.
- [ ] T26.2: Exercise large multiline paste, CRLF policy, control-looking text, terminal release/repeat noise, cancellation boundaries, and one-step undo across shared views.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 27: Run discoverable commands through M-x

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R22, R23.

- [ ] T27.1: Expose implemented fixed commands through M-x with lowercase hyphenated names, deterministic prefix/substring filtering, TAB selection/completion, RET invocation, and session history.
- [ ] T27.2: Exercise unknown names, empty candidates, cancel, command history, argument continuation, and binding/name parity for file, window, editing, and history operations.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 28: Complete focused argument and cancel flows

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R04, R12, R23, R24.

- [ ] T28.1: Complete predictable multi-stage command arguments, path/buffer prompts, decision prompts, in-prompt editing, and focus restoration after accept/error/cancel.
- [ ] T28.2: Exercise cancellation at every stage of find/write/kill, prompt editing with Emacs movement, resize while prompting, and no document mutation from transient input.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 29: Search forward across rope chunks

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R07, R09, R26.

- [ ] T29.1: Implement isearch-forward with visible minibuffer query, chunk-crossing literal matching, next-match continuation, source-qualified results, and match reveal/highlighting.
- [ ] T29.2: Exercise Unicode chunk seams, multiline text, incremental query changes, no-match behavior, and searches on representative larger ropes without per-key whole-file copies.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 30: Search backward with smart case and cancel

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R24, R26.

- [ ] T30.1: Implement isearch-backward, direction repetition, smart-case query matching, RET acceptance, and C-g restoration of the exact starting view/point.
- [ ] T30.2: Exercise mixed Unicode case, backward chunk seams, query edits, match boundaries, direction changes, failed search, and cancellation after scrolling.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 31: Confirm literal replacements interactively

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R21, R23, R27.

- [ ] T31.1: Implement M-% and query-replace prompts followed by yes/no/quit decisions, applying each accepted occurrence through the canonical checked transaction.
- [ ] T31.2: Exercise replacement lengths, Unicode/CRLF text, skipped matches, no matches, cancellation, read-only buffers, and undo of accepted replacements.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 32: Select and copy display-column rectangles

Inputs: [Part 4](../plan/rev-editor/04-commands-and-search.md); requirements R07, R09, R20.

- [ ] T32.1: Implement rectangle-mark-mode, cell-based row extraction, whole-cluster edge inclusion, partial-tab representation, and copy-rectangle-as-kill in separate rectangle storage.
- [ ] T32.2: Exercise reversed endpoints, CJK/emoji/combining clusters, tab boundaries, short/empty lines, scrolling, and copy without source padding or mutation.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 33: Kill rectangles as one transaction

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R09, R20, R21.

- [ ] T33.1: Implement kill-rectangle with all row changes prepared in a declared coordinate space, retaining the rectangle and mapping every affected live view coherently.
- [ ] T33.2: Exercise partially touched tabs, wide clusters, empty rows, read-only/invalid admission, large row counts, and atomic undo/redo without partial row publication.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 34: Yank blocks with coherent padding and undo

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R10, R20, R21.

- [ ] T34.1: Implement yank-rectangle across successive lines, deterministic insertion at target cell boundaries, necessary short-line padding, and additional lines under the newline policy.
- [ ] T34.2: Exercise unequal source widths, wide target graphemes, partial tabs, absent rectangle, insertion past the final line, and undo of padding plus all inserted rows.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 35: Render and focus the left file tree

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R05, R06, R28.

- [ ] T35.1: Implement toggle-file-tree, root selection, a narrow left allocation, selected entry indication, and predictable focus transfer retaining the last editing view.
- [ ] T35.2: Exercise show/hide with dirty buffers, M-x invocation, narrow-width auto-hide, resize restoration, and exclusion from other-window cycling.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 36: Traverse directories lazily and safely

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R28, R29.

- [ ] T36.1: Implement Up/Down visible traversal, directory expansion/collapse, deterministic ordering, lazy child reads, explicit refresh, and symlinked-directory leaves.
- [ ] T36.2: Exercise deep trees, symlink cycles, inaccessible/deleted directories, stale selections, and refresh after external filesystem changes.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 37: Open and create tree entries

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R11, R12, R28, R30.

- [ ] T37.1: Open selected files through find-file and add prompted new-file/new-directory creation at the selected directory, validating names and conflicts before disk mutation.
- [ ] T37.2: Exercise existing visits, unsupported extension fallback, occupied/invalid names, permission errors, canceled creation, and synchronization of tree entries with buffers.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 38: Reconcile visited paths after tree rename

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R11, R13, R30.

- [ ] T38.1: Implement prompted rename with conflict checks, canonical path/index updates, and descendant visited-path reconciliation for a renamed directory.
- [ ] T38.2: Exercise occupied targets, visited buffers with equal basenames, symlink entries, permission failure, cancellation, and a later save using the successful new path.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 39: Confirm deletion while preserving open text

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R04, R11, R29, R30.

- [ ] T39.1: Implement explicit delete confirmation and a separate nonempty-directory recursive confirmation, reconciling retained visiting buffers as missing under D07.
- [ ] T39.2: Exercise canceled/nonempty deletion, dirty visiting files, inaccessible entries, symlink deletion without referent recursion, and refresh after partial filesystem errors.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 40: Bundle Rust syntax highlighting

Inputs: [Part 5](../plan/rev-editor/05-rectangles-and-tree.md); requirements R05, R31, R34.

- [ ] T40.1: Integrate the pinned Rust grammar and bundled HIGHLIGHTS_QUERY with safe Tree-sitter APIs, a small capture palette, and extension selection with plain-text fallback.
- [ ] T40.2: Exercise Rust keywords/strings/comments plus grammar-supported types/functions/numbers/operators, unsupported files, incomplete source, and missing/failed query fallback.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 41: Update syntax from exact rope edits

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R09, R32, R40.

- [ ] T41.1: Translate every accepted edit/undo/redo/rectangle/paste to correct Tree-sitter byte and row/byte-column edits, retaining parser/tree state and bounding visible capture work.
- [ ] T41.2: Exercise multibyte edits before captures, shared windows, parse/query failures, selection/search/cursor layering, and stale syntax invalidation without whole-file rebuild per key.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 42: Complete guarded multi-buffer session exit

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R03, R04, R11, R13, R21, R24.

- [ ] T42.1: Complete save-buffers-kill-terminal across all modified named and unnamed buffers, including sequential save decisions, canceled path prompts, read-only and uncertain-publication outcomes.
- [ ] T42.2: Exercise cancellation after earlier saves, failed required saves, several windows of one buffer, a removed destination, and final terminal restoration after a genuinely accepted quit.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 43: Harden ordinary failure boundaries

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R03, R04, R06, R10, R13, R20, R25, R30, R32, R37, R39.

- [ ] T43.1: Extend real boundary fault injection for recoverable ingress, save stages, tree operations, parser/query execution, and post-acquisition terminal errors, retaining truthful outcomes.
- [ ] T43.2: Exercise combined failures and cleanup ordering, canceled transient interactions, readonly rejection, and state equality after rejected compound edits. Keep fixes within already approved behavior.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 44: Drive user journeys through a PTY

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R01, R03, R04, R12, R14, R15, R18, R19, R20, R22, R25, R26, R27, R28, R31, R39.

- [ ] T44.1: Extend the existing standard-library Python PTY harness to drive launch/edit/save/reopen, bindings and M-x, buffer/window operations, selection/rings/rectangles, search, tree actions, and quit.
- [ ] T44.2: Use bounded condition waits, owned fixtures, real filesystem byte comparisons, resize events, and termios/screen restoration checks. Separate pure-state assertions from terminal encoding variability.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 45: Exercise cross-feature Unicode and file fidelity

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R09, R10, R19, R20, R21, R26, R30, R32, R39.

- [ ] T45.1: Add distinct adversarial regressions across editing/history/search/syntax and file save/reopen using emoji, combining text, CJK, tabs, CRLF, BOM, absent final newline, long lines, and shared views.
- [ ] T45.2: Exercise repeated split/close/resize, invalid yank-pop, rectangle seams, symlink loops, inaccessible paths, and canceled destructive actions. Reuse subsystem suites rather than duplicate every journey.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

### Slice 46: Measure representative editing workloads

Inputs: [Part 6](../plan/rev-editor/06-syntax-and-acceptance.md); requirements R07, R08, R26, R32, R40.

- [ ] T46.1: Add a deterministic mixed-width UTF-8 fixture generator with tens of thousands of lines and measure startup, insert/delete, scroll, split redraw, and highlighting on recorded hardware context.
- [ ] T46.2: Inspect/measure work for pathological whole-file copying/reparse and idle repaint loops, correcting only demonstrated in-scope causes. Store concise fixture/measurement evidence without hostnames or invented guarantees.
- [ ] Record passing required/focused validation, canonical PR integration, and delivered report.

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
