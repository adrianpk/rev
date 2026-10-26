# rev Editor: Files and history

Plan: [rev Editor](../rev-editor.md)
Specification: [rev Editor](../../spec/rev-editor.md)
Model: [rev Editor Model](../../spec/rev-editor-model.md)

Read the master plan, its pinned decisions, and the current section before work.
This file supplies detailed tasks; the master plan/tracker own slice identities.
Every task includes appropriate behavior tests. Each slice also requires the
common quality gate, one canonical report, one PR, and controller-owned merge.

## Slice 9: Preserve vertical motion and scrolling

Requirements: R07, R16
Dependencies: Slice 6, Slice 8
Owners: view goal column and viewport navigation
Focused validation: `cargo test --test display`

### T9.1

Implement next/previous logical line with retained desired display column, page motion,
horizontal reveal, and recenter-top-bottom behavior.

### T9.2

Exercise short lines between long lines, tabs/wide clusters, tiny viewports, and
repeated page/recenter operations without losing intended column.

### Required evidence

Moving through a short line restores the intended column on a later long line; scroll
state belongs to the view and remains in bounds.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-9-scrolling.md`. Update the tracker after integration.

## Slice 10: Restore per-buffer undo and redo

Requirements: R04, R11, R21
Dependencies: Slice 4, Slice 7, Slice 8
Owners: history and saved-state identity
Focused validation: `cargo test --test editing`

### T10.1

Implement conventional per-buffer undo/redo, fresh publication revisions, logical
saved-state identity, initiating-view restoration, and redo invalidation after a new
edit.

### T10.2

Exercise repeated navigation, unavailable history, a retained clean save point,
independent buffers, read-only restrictions, and coherent mapping of non-initiating
positions.

### Required evidence

Undo restores content and clean status when returning to the saved logical state;
revisions do not masquerade as history state IDs.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-10-history.md`. Update the tracker after integration.

## Slice 11: Group typing and compound history

Requirements: R19, R20, R21, R25
Dependencies: Slice 10
Owners: history transaction grouping and retention
Focused validation: `cargo test --test editing`

### T11.1

Group adjacent ordinary insertion with explicit continuity breaks, preserve whole
compound records, and add the agreed simple retention policy without branching-history
machinery.

### T11.2

Exercise paste-sized transactions, replacement groups, navigation breaks, new edits
after undo, and pruning complete old groups without losing the newest reversible
operation.

### Required evidence

One coherent compound command is one undo unit; saving does not discard history and a
large recent group is not split to meet a soft retention target.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-11-grouping.md`. Update the tracker after integration.

## Slice 12: Load exact UTF-8 file content

Requirements: R02, R09, R10, R11, R37
Dependencies: Slice 2, Slice 3, Slice 10
Owners: file ingress and format metadata
Focused validation: `cargo test --test files`

### T12.1

Load existing files into canonical ropes with strict UTF-8, conservative NUL binary
rejection, exact LF/CRLF and final-newline retention, recognized BOM metadata, and
prevailing newline preference.

### T12.2

Exercise invalid encoding, empty files, mixed line endings, BOM round trips, missing
paths, permission failures, and preservation of the previous active buffer on rejected
ingress.

### Required evidence

An unchanged file round trip can preserve its bytes; rejected data is never installed as
a lossy editable copy.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-12-file-ingress.md`. Update the tracker after integration.

## Slice 13: Reuse canonical visited files

Requirements: R02, R11, R12, R23
Dependencies: Slice 5, Slice 12
Owners: path normalization, association index, find-file
Focused validation: `cargo test --test files`

### T13.1

Implement find-file argument flow, normalized intended paths for absent files, canonical
existing referents, unique buffer labels, and existing-visit activation shared by all
callers.

### T13.2

Exercise symlink aliases, duplicate basenames, nonexistent parents, canceled prompts,
directory paths, and collision with an already visited canonical target.

### Required evidence

Reopening a canonical file selects its existing buffer; a failed open leaves the current
document and association intact.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-13-file-visits.md`. Update the tracker after integration.

## Slice 14: Publish safe same-directory saves

Requirements: R04, R10, R13, R21
Dependencies: Slice 11, Slice 12, Slice 13
Owners: file output and save result
Focused validation: `cargo test --test files`

### T14.1

Implement save-buffer using captured rope text, same-directory temporary output, checked
writes, permission preservation where practical, replacement, and explicit
completion/publication outcomes.

### T14.2

Exercise injected write/flush/replace/post-replace failures and cleanup. Reconcile
actual publication, saved logical state, dirty flags, and a first successful file
association truthfully.

### Required evidence

Failure before replacement preserves the original destination; failure afterward reports
that publication occurred and cannot falsely satisfy a required save.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-14-saving.md`. Update the tracker after integration.

## Slice 15: Confirm overwrite and external changes

Requirements: R04, R12, R13, R23
Dependencies: Slice 14
Owners: write-file and overwrite decisions
Focused validation: `cargo test --test files`

### T15.1

Implement write-file, unnamed save prompts, occupied-path confirmation, metadata-based
external-change warnings, canonical symlink referent saving, and live-buffer target
collision refusal.

### T15.2

Exercise cancellation, denied/disk-full writes, removed paths, save-as before/after
publication failures, and clean/dirty history state after successful save and undo.

### Required evidence

No unconfirmed target is overwritten; a failed unpublished save-as does not relocate the
visited path, while an actual publication remains visible.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-15-save-decisions.md`. Update the tracker after integration.

## Slice 16: Switch list and close buffers

Requirements: R04, R05, R11, R12, R23
Dependencies: Slice 13, Slice 14, Slice 15
Owners: buffer commands and selectable inventory
Focused validation: `cargo test --test files`

### T16.1

Implement switch-to-buffer completion, list-buffers selection, kill-buffer decisions,
deterministic replacement, and unique labels independent of window lifetime.

### T16.2

Exercise several modified/unnamed/file-backed buffers, canceled close, failed required
save, final-buffer replacement, and index cleanup. Preserve open text when a path
disappears.

### Required evidence

Closing a modified buffer requires a real decision; cancel leaves it usable and killing
the final buffer still leaves a valid editing session.

Record both task commits, actual successful checks, and the canonical PR in
`ops/default/report/slices/rev-editor/slice-16-buffer-commands.md`. Update the tracker after integration.
