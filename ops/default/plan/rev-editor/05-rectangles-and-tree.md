# rev Editor: Rectangles and tree

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 33: Kill rectangles as one transaction

Requirements: R09, R20, R21
Dependencies: Slice 4, Slice 11, Slice 19, Slice 32
Owners: multi-row prepared edits
Focused validation: `cargo test --test rectangles`

### T33.1

Implement kill-rectangle with all row changes prepared in a declared coordinate space,
retaining the rectangle and mapping every affected live view coherently.

### T33.2

Exercise partially touched tabs, wide clusters, empty rows, read-only/invalid admission,
large row counts, and atomic undo/redo without partial row publication.

### Required evidence

One rejected row leaves all text unchanged; one undo restores every changed row, mark,
and valid cursor state.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-33-rectangle-kill.md`. Update the tracker after integration.

## Slice 34: Yank blocks with coherent padding and undo

Requirements: R10, R20, R21
Dependencies: Slice 33
Owners: rectangle insertion and line creation
Focused validation: `cargo test --test rectangles`

### T34.1

Implement yank-rectangle across successive lines, deterministic insertion at target cell
boundaries, necessary short-line padding, and additional lines under the newline policy.

### T34.2

Exercise unequal source widths, wide target graphemes, partial tabs, absent rectangle,
insertion past the final line, and undo of padding plus all inserted rows.

### Required evidence

Adjacent content remains valid and no copy-time invented padding leaks into the
rectangle; one undo reverses the full block insertion.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-34-rectangle-yank.md`. Update the tracker after integration.

## Slice 35: Render and focus the left file tree

Requirements: R05, R06, R28
Dependencies: Slice 18, Slice 20, Slice 28
Owners: tree visibility, root, focus, and drawing
Focused validation: `cargo test --test tree`

### T35.1

Implement toggle-file-tree, root selection, a narrow left allocation, selected entry
indication, and predictable focus transfer retaining the last editing view.

### T35.2

Exercise show/hide with dirty buffers, M-x invocation, narrow-width auto-hide, resize
restoration, and exclusion from other-window cycling.

### Required evidence

Tree presentation never replaces an editing buffer or requires Git metadata, and hidden
tree state cannot corrupt editing focus.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-35-tree-pane.md`. Update the tracker after integration.

## Slice 36: Traverse directories lazily and safely

Requirements: R28, R29
Dependencies: Slice 35
Owners: tree entries, lazy children, refresh
Focused validation: `cargo test --test tree`

### T36.1

Implement Up/Down visible traversal, directory expansion/collapse, deterministic
ordering, lazy child reads, explicit refresh, and symlinked-directory leaves.

### T36.2

Exercise deep trees, symlink cycles, inaccessible/deleted directories, stale selections,
and refresh after external filesystem changes.

### Required evidence

Traversal performs bounded on-demand work, cannot recursively follow cycles, and leaves
a usable selection after ordinary errors.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-36-tree-navigation.md`. Update the tracker after integration.

## Slice 37: Open and create tree entries

Requirements: R11, R12, R28, R30
Dependencies: Slice 13, Slice 15, Slice 28, Slice 36
Owners: tree open/create commands and file owner
Focused validation: `cargo test --test tree`

### T37.1

Open selected files through find-file and add prompted new-file/new-directory creation
at the selected directory, validating names and conflicts before disk mutation.

### T37.2

Exercise existing visits, unsupported extension fallback, occupied/invalid names,
permission errors, canceled creation, and synchronization of tree entries with buffers.

### Required evidence

Tree opening reuses the canonical file/buffer pipeline; canceled or failed creation
never discards existing document text.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-37-tree-create.md`. Update the tracker after integration.

## Slice 38: Reconcile visited paths after tree rename

Requirements: R11, R13, R30
Dependencies: Slice 15, Slice 37
Owners: filesystem rename and association reconciliation
Focused validation: `cargo test --test tree`

### T38.1

Implement prompted rename with conflict checks, canonical path/index updates, and
descendant visited-path reconciliation for a renamed directory.

### T38.2

Exercise occupied targets, visited buffers with equal basenames, symlink entries,
permission failure, cancellation, and a later save using the successful new path.

### Required evidence

Only successful disk rename changes visiting associations; buffer labels and later saves
identify the actual destination.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-38-tree-rename.md`. Update the tracker after integration.

## Slice 39: Confirm deletion while preserving open text

Requirements: R04, R11, R29, R30
Dependencies: Slice 16, Slice 38
Owners: tree delete decisions and missing associations
Focused validation: `cargo test --test tree`

### T39.1

Implement explicit delete confirmation and a separate nonempty-directory recursive
confirmation, reconciling retained visiting buffers as missing under D07.

### T39.2

Exercise canceled/nonempty deletion, dirty visiting files, inaccessible entries, symlink
deletion without referent recursion, and refresh after partial filesystem errors.

### Required evidence

Deletion cannot silently erase an in-memory rope or recreate its removed file; actual
filesystem failures are reported without false all-or-nothing claims.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-39-tree-delete.md`. Update the tracker after integration.

## Slice 40: Bundle Rust syntax highlighting

Requirements: R05, R31, R34
Dependencies: Slice 6, Slice 12, Slice 27
Owners: syntax grammar, query captures, and palette
Focused validation: `cargo test --test syntax`

### T40.1

Integrate the pinned Rust grammar and bundled HIGHLIGHTS_QUERY with safe Tree-sitter
APIs, a small capture palette, and extension selection with plain-text fallback.

### T40.2

Exercise Rust keywords/strings/comments plus grammar-supported
types/functions/numbers/operators, unsupported files, incomplete source, and
missing/failed query fallback.

### Required evidence

Highlighting changes only styles; the offline executable requires no grammar
installation and advertises Rust or Text truthfully.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-40-rust-highlights.md`. Update the tracker after integration.
