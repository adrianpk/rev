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

`bookmark-next` and `bookmark-previous` traverse only current revision-qualified
resolved anchors in the current canonical file. Ordering is by scalar position,
then lexical name, with wrap in either direction. First use includes targets at
point. Repetition retains the last stable ID, point/revision, pane and collection
generation, so same-position names are independent steps. Movement, edits,
collection mutation or pane selection resets that observation. There are no new
factory shortcuts, source visits for ordering, or source writes. Unresolved and
stale records remain available for a checked named jump.

## Implementation Notes

One collection generation invalidates retained row and iteration observations
when records, anchors, context or canonical paths change. The durable format
remains unchanged. Ordered queries copy only bounded location/name metadata and
sort once per command; frame projection formats visible list rows only.

## Validation

Development check `cargo test --locked --test bookmarks list:: -- --nocapture`
passed seven native list tests, including 1024 unresolved targets, local key
remapping, cancellation, real source lifetimes, cross-pane collection changes
and bounded visit cancellation. The documentation checker passed 316 commands,
186 configuration entries, 58 pages and 331 links/anchors at the list task.
Final exact-HEAD controller receipts follow the canonical PR-reference commits.

Development checks also passed eight list journeys (including deferred visit
completion) and four ordered-navigation journeys. The final catalog contains
318 implemented commands and 186 configuration entries.

After PR-reference follow-up commits, the controller records the exact clean
HEAD against this ordered required list:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test commands --test configuration --test windows --test display --test panels`
3. `git diff --check`

A separate controller batch checks `cargo test --locked --lib bookmark`,
`cargo clippy --locked --all-targets -- -D warnings` and
`cargo run --locked --quiet --example documentation-check` on that same final
candidate before the required batch. Handoff requires both batches to pass.
The controller owns the combined gate after all recorded slice integrations.

## Task Commits

T4.1 introduces native listing, the local map, stable generation-checked rows and
source-safe cancellation/return. T4.2 adds ordered traversal and repetition
invalidation, with native eligibility and tie coverage. Their exact references
are recorded in the tracker.

## Risks and Follow-ups

Unresolved, stale or unavailable records have no eligible current-file order;
the user can explicitly check them through a named jump or list visit. A changed
list snapshot clears pending marks and requires a retry. Marker projection and
joint acceptance remain assigned to the recorded successor slices.
