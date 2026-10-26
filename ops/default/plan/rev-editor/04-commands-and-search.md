# rev Editor: Commands and search

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 25: Resolve Meta Escape and undo aliases

Requirements: R21, R24, R25
Dependencies: Slice 5, Slice 10, Slice 20
Owners: terminal key normalization and finite prefix deadline
Focused validation: `cargo test --test commands`

### T25.1

Complete Alt/Meta and compatible Escape-prefix input using a finite nonblocking
deadline, standalone Escape, C-SPC aliases, and reportable undo alternatives.

### T25.2

Exercise C-/, C-_, C-x u, M-<, M->, nested rectangle chords, key repeat/release, unknown
escape input, and cancellation during a pending Meta prefix.

### Required evidence

The documented target-terminal policy distinguishes supported chords without globally
blocking resize, quit, or guard cleanup.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-25-terminal-keys.md`. Update the tracker after integration.

## Slice 26: Insert bracketed Unicode paste atomically

Requirements: R10, R17, R21, R25
Dependencies: Slice 11, Slice 19, Slice 25
Owners: paste event handling and command bypass
Focused validation: `cargo test --test commands`

### T26.1

Enable supported bracketed paste, insert literal Unicode/newline text through one
transaction, and avoid feeding pasted key-looking content to command decoding.

### T26.2

Exercise large multiline paste, CRLF policy, control-looking text, terminal
release/repeat noise, cancellation boundaries, and one-step undo across shared views.

### Required evidence

Pasted C-x-looking text cannot execute an editor command, and the full paste is a
coherent undo unit with correct cursor mapping.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-26-paste.md`. Update the tracker after integration.

## Slice 27: Run discoverable commands through M-x

Requirements: R22, R23
Dependencies: Slice 5, Slice 16, Slice 18, Slice 24, Slice 25
Owners: command catalog and command-name prompt
Focused validation: `cargo test --test commands`

### T27.1

Expose implemented fixed commands through M-x with lowercase hyphenated names,
deterministic prefix/substring filtering, TAB selection/completion, RET invocation, and
session history.

### T27.2

Exercise unknown names, empty candidates, cancel, command history, argument
continuation, and binding/name parity for file, window, editing, and history operations.

### Required evidence

Every advertised command executes its real shared handler; command matching never
becomes language completion or an interpreted command language.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-27-command-palette.md`. Update the tracker after integration.

## Slice 28: Complete focused argument and cancel flows

Requirements: R04, R12, R23, R24
Dependencies: Slice 15, Slice 16, Slice 27
Owners: minibuffer argument and decision state machines
Focused validation: `cargo test --test commands`

### T28.1

Complete predictable multi-stage command arguments, path/buffer prompts, decision
prompts, in-prompt editing, and focus restoration after accept/error/cancel.

### T28.2

Exercise cancellation at every stage of find/write/kill, prompt editing with Emacs
movement, resize while prompting, and no document mutation from transient input.

### Required evidence

A canceled command performs no pending side effect and leaves the correct editor/tree
context and selected buffer usable.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-28-prompt-flows.md`. Update the tracker after integration.

## Slice 29: Search forward across rope chunks

Requirements: R07, R09, R26
Dependencies: Slice 9, Slice 25, Slice 28
Owners: incremental search state and query projection
Focused validation: `cargo test --test search`

### T29.1

Implement isearch-forward with visible minibuffer query, chunk-crossing literal
matching, next-match continuation, source-qualified results, and match
reveal/highlighting.

### T29.2

Exercise Unicode chunk seams, multiline text, incremental query changes, no-match
behavior, and searches on representative larger ropes without per-key whole-file copies.

### Required evidence

Search changes only view/search state, never the rope; missing matches leave valid
positions and a visible failure indication.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-29-forward-search.md`. Update the tracker after integration.

## Slice 30: Search backward with smart case and cancel

Requirements: R24, R26
Dependencies: Slice 29
Owners: search direction, original point, literal smart-case
Focused validation: `cargo test --test search`

### T30.1

Implement isearch-backward, direction repetition, smart-case query matching, RET
acceptance, and C-g restoration of the exact starting view/point.

### T30.2

Exercise mixed Unicode case, backward chunk seams, query edits, match boundaries,
direction changes, failed search, and cancellation after scrolling.

### Required evidence

Case-insensitive matching maps to original source ranges rather than folded-text
indices; cancel restores the original location without editing.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-30-search-control.md`. Update the tracker after integration.

## Slice 31: Confirm literal replacements interactively

Requirements: R21, R23, R27
Dependencies: Slice 10, Slice 28, Slice 30
Owners: query-replace command and confirmed edit owner
Focused validation: `cargo test --test search`

### T31.1

Implement M-% and query-replace prompts followed by yes/no/quit decisions, applying each
accepted occurrence through the canonical checked transaction.

### T31.2

Exercise replacement lengths, Unicode/CRLF text, skipped matches, no matches,
cancellation, read-only buffers, and undo of accepted replacements.

### Required evidence

Only confirmed occurrences change and each accepted replacement remains meaningfully
reversible without changing skipped text.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-31-query-replace.md`. Update the tracker after integration.

## Slice 32: Select and copy display-column rectangles

Requirements: R07, R09, R20
Dependencies: Slice 3, Slice 21, Slice 25
Owners: rectangle geometry and separate block store
Focused validation: `cargo test --test rectangles`

### T32.1

Implement rectangle-mark-mode, cell-based row extraction, whole-cluster edge inclusion,
partial-tab representation, and copy-rectangle-as-kill in separate rectangle storage.

### T32.2

Exercise reversed endpoints, CJK/emoji/combining clusters, tab boundaries, short/empty
lines, scrolling, and copy without source padding or mutation.

### Required evidence

Row extraction follows D04 and cannot divide Unicode text; display-cell width is not
confused with bytes or scalar indices.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-32-rectangle-copy.md`. Update the tracker after integration.
