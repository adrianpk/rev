# rev Editor: Windows and regions

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 17: Create nested editing splits

Requirements: R06, R07, R14, R33
Dependencies: Slice 6, Slice 9, Slice 16
Owners: view registry and recursive layout
Focused validation: `cargo test --test windows`

### T17.1

Implement split-window-below and split-window-right with nested layout nodes, geometry
admission, initial same-buffer view state, and retained original focus.

### T17.2

Exercise mixed split directions, minimum dimensions, allocation of distinct view IDs,
and independent point/scroll after split. Refuse non-fitting changes before mutating
layout.

### Required evidence

Two views share one rope while independently navigating; a rejected split leaves the
existing tree and focus unchanged.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-17-splits.md`. Update the tracker after integration.

## Slice 18: Delete and cycle editing windows

Requirements: R14, R15
Dependencies: Slice 17
Owners: layout deletion and focus traversal
Focused validation: `cargo test --test windows`

### T18.1

Implement delete-window, delete-other-windows, and deterministic other-window traversal
with wrapping and a separate tree-focus path.

### T18.2

Exercise repeated split/delete, deleting the sole view, selected-view replacement, and
preservation of unsaved buffers after their last visible window closes.

### Required evidence

Deleting a view never kills its buffer; every selected leaf and buffer reference remains
live and traversal order is deterministic.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-18-window-commands.md`. Update the tracker after integration.

## Slice 19: Map shared-buffer edits across views

Requirements: R08, R09, R11, R14, R17, R21
Dependencies: Slice 10, Slice 17, Slice 18
Owners: edit position mapping and live view reconciliation
Focused validation: `cargo test --test windows`

### T19.1

Complete point/top/mark mapping for insertion, deletion, replacement, undo, and redo
across every view of the same buffer using the accepted affinities.

### T19.2

Exercise edits before and across another cursor, deletion of visible lines, killing a
multiply visible buffer, and history initiated from a view that later closes.

### Required evidence

All views show one published text state, independently valid points and scroll; no
deleted view is resurrected by history.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-19-shared-views.md`. Update the tracker after integration.

## Slice 20: Handle tiny and changing terminal geometry

Requirements: R03, R05, R06, R07, R14
Dependencies: Slice 17, Slice 18, Slice 19
Owners: layout geometry and presentation suspension
Focused validation: `cargo test --test windows`

### T20.1

Recompute nested geometry on resize, preserve the logical layout under suspended areas,
and apply minimum pane/status/minibuffer dimensions without underflow.

### T20.2

Exercise zero dimensions, extremely narrow/tall/wide reports, repeated resize bursts,
rejected splits, and restoration of usable viewports after expansion.

### Required evidence

Presentation never writes beyond assigned geometry and no resize deletes text, history,
or valid independent view state.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-20-resize.md`. Update the tracker after integration.

## Slice 21: Activate and render linear regions

Requirements: R07, R18, R24
Dependencies: Slice 5, Slice 8, Slice 19
Owners: buffer mark and selected-view region
Focused validation: `cargo test --test editing`

### T21.1

Implement set-mark-command, exchange-point-and-mark, movement extension, region
deactivation, and focused multi-line selection rendering.

### T21.2

Exercise reversed/empty ranges, region updates through shared edits, focus changes,
viewport clipping, and C-g cancellation without losing stored mark unnecessarily.

### Required evidence

Mark belongs to the buffer while point belongs to the selected view; selected text and
rendering agree across rows and scrolling.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-21-regions.md`. Update the tracker after integration.

## Slice 22: Kill lines and words directionally

Requirements: R17, R19, R21
Dependencies: Slice 8, Slice 11, Slice 21
Owners: kill-line and directional word commands
Focused validation: `cargo test --test editing`

### T22.1

Implement kill-line, kill-word, and backward-kill-word through checked transactions,
including coherent CRLF removal at logical line end.

### T22.2

Exercise boundaries, Unicode punctuation/words, directional coalescing, read-only
rejection, and one-step undo of each kill without unrelated changes.

### Required evidence

Consecutive directional kills compose retained text in the correct order and an
intervening non-kill ends continuity.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-22-kills.md`. Update the tracker after integration.

## Slice 23: Copy kill and yank linear text

Requirements: R18, R19, R21
Dependencies: Slice 21, Slice 22
Owners: bounded ring and linear yank
Focused validation: `cargo test --test editing`

### T23.1

Implement kill-region, kill-ring-save, yank, a 32-entry session ring, and insertion
transaction grouping while keeping rectangle storage separate.

### T23.2

Exercise copying without mutation, reversed regions, empty ring, multiline Unicode/CRLF
text, ring eviction, and undo after yanking.

### Required evidence

Copied and killed bytes remain accurate, ring capacity is bounded, and yanking uses the
canonical edit path exactly once.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-23-kill-ring.md`. Update the tracker after integration.

## Slice 24: Rotate only the current yank range

Requirements: R19, R21
Dependencies: Slice 23
Owners: yank continuity capture
Focused validation: `cargo test --test editing`

### T24.1

Implement yank-pop with buffer/view/revision-qualified inserted range and rotation;
replace exactly the immediately preceding yank sequence.

### T24.2

Exercise unequal-length entries, repeated rotations, unrelated commands, edits in
another view, undo, and attempted pop before any yank.

### Required evidence

An invalidated capture cannot replace unrelated text; successful rotation updates its
own range and coherent undo state.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-24-yank-pop.md`. Update the tracker after integration.
