# Slice 4: List and ordered navigation

Status: implementing
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-04-navigation`
PR: pending

## Purpose

Complete BMK04 through native read-only listing, stable row actions and
current-file ordered navigation.

## Delivered Behavior

`C-x r l` opens a native read-only `*Bookmarks*` workspace in the selected pane.
Names appear lexically with observed status and file paths. Row navigation and
painting use bounded retained metadata and do not visit every referenced file.
Return uses the existing checked navigation owner; failed or canceled visits
retain the list and source. Only visible rows are formatted.

The list's local configurable keys move rows, mark/unmark and advance, execute
checked atomic deletion, or return and discard unexecuted marks. A collection
generation and stable record/target identities protect actions after edits,
reload, deletion/recreation or mutations from another pane. A changed snapshot
refreshes, clears staged marks and requires a retry before publication.

The workspace borrows the native pane projection used by other read-only
result views. It retains the original source and checked return point, with no
second document rope or editing history. Source input is blocked in its pane.
Other splits retain their independent input and source views. A changed or
closed origin cannot receive a stale return point; the current valid view is
retained with a diagnostic. Closing a pane or quitting drops its staged marks.

## Validation

Development check `cargo test --locked --test bookmarks list:: -- --nocapture`
passed seven native list tests, including 1024 unresolved targets, local key
remapping, cancellation, real source lifetimes, cross-pane collection changes
and bounded visit cancellation. The documentation checker passed 316 commands,
186 configuration entries, 58 pages and 331 links/anchors at the list task.
Final exact-HEAD controller receipts follow the canonical PR-reference commits.
