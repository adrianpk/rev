# Slice 2: Checked location lifecycle

Status: delivered
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-02-locations`
PR: [#186](https://forge.adrianpk.com/adrianpk/rev/pulls/186)

## Purpose

Deliver BMK02 and BMK06 through canonical source transactions, checked context
resolution and native file lifetimes.

## Delivered Behavior

Bookmarks follow ordinary and compound edits, retained structural moves and
source undo/redo across shared views. Insertion at a point uses left affinity.
Removing its original position or joining away its grapheme boundary makes a
record stale; undo can restore the earlier valid outcome. Deleted names cannot
reappear through source history, and explicitly replaced or later-created
records do not inherit another target's history observations.

Closing a buffer retains bounded context and drops its live binding. Reopening
checks the saved scalar boundary, then searches for one unique context match.
Ambiguous, absent, missing or changed targets preserve the origin. Already open
canonical buffers, including unsaved text and externally removed live files,
take precedence. Discarded source text is never recreated implicitly.

Canonical file/directory rename updates associated records, including closed
records. Save-as follows the canonical destination. Navigation reveals folded
ancestors and wrapped fragments in the selected view while other splits retain
their view state.

## Implementation Notes

One collection is shared by the buffer registry and each native buffer's
publication owner. Prepared edits capture stable bookmark IDs, explicit target
generations and scalar outcomes before publication. Admission rechecks those
observations. Source history retains compact reversible outcomes, counts their
payload in its existing retention budget and holds no additional whole-source
snapshot or context history. Grouped typing preserves the first/last outcomes.
Retained mappings distinguish unchanged source pieces from rewritten counters.

Context captures at most 64 scalars per side with explicit BOF/EOF evidence.
A streaming KMP search admits at most 262144 source scalars and consumes at most
16384 per pass. Original-boundary checking, search and bounded native reveal use
separate phases. A match waits for the complete search before admission; a later
second match cannot be ignored. Input changes or cancellation drop pending work;
key releases and resize retain it. Source revision, record identity/generation,
origin and reopened-file observation are checked again before publication.
There is no directory scan, render-time IO, idle polling or new dependency.

Close prepares context before removing the live owner. Rejected file publication
leaves bookmark state intact. Published source associations follow the existing
file owner's outcome and retain its unresolved-publication warning when present;
context refresh acknowledges only completed saves. Context matching supplies
location evidence, not proof of historical identity after duplicated edits.

## Contracts Added or Changed

The public collection observation stays read-only. Resolution distinguishes live
resolved, stale, closed unresolved and unavailable targets. Existing native
commands, factory chords and startup settings remain the public controls for
this increment; command guidance and the existing Unreleased entry now describe
the delivered location lifecycle.

## Files of Interest

- `src/bookmark.rs`, `src/bookmark/context.rs`: stable outcomes and bounded evidence.
- `src/buffer/edit/bookmarks.rs`, `history.rs`, `grouping.rs`, `reorder.rs`:
  prepared mappings, checked replay, retention and retained source identity.
- `src/session/bookmarks/navigation.rs`: cancellable checked destination admission.
- `src/buffer/lifecycle.rs`, `rename.rs`, `save.rs`, `visits.rs`: file lifetimes.
- `tests/bookmarks/locations.rs`, `reopening.rs`: native edit/reopen/view journeys.
- `src/bookmark/lifecycle_tests.rs`: real rename and rejected-publication evidence.

## Validation

Development checks passed:

- `cargo test --locked --test bookmarks`: 24 native workflow cases.
- `cargo test --locked --lib bookmark`: five focused native-owner cases.

Coverage includes insertion/deletion boundaries, grapheme joins, grouped typing,
compound replacements, preparation invalidation, retained subtree movement,
undo/redo, history eviction, explicit generation changes, split views, Unicode
reopening, context ambiguity and limits, BOF/EOF/empty targets, unsaved live
preference, discard, save-as, file/directory rename, real staging rejection,
pending cancellation, key release/resize, external file races and fold/wrap reveal.

The old missing-target expectation now checks the delivered unavailable result
instead of Slice 1's provisional stale result. Source/view/record preservation
assertions remain active. The resolver separates original-position checking and
search/reveal passes to preserve its scalar budget.

T2.1 is the native source-transaction mapping commit. T2.2 is the checked reopen
and file-lifetime commit. Their full references are recorded in the tracker.
The controller stores final clean-HEAD receipts outside the repository. The
canonical PR uses this exact required ordered list:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test core --test editing --test rectangles --test files --test windows --test wrapping --test org_editing`
3. `git diff --check`

A separate controller batch checks `cargo test --locked --lib bookmark`,
`cargo clippy --locked --lib -- -D warnings` and
`cargo run --locked --quiet --example documentation-check` on the same final
candidate before the required batch. Handoff requires both batches to pass.
The completed combined delivery-set gate is recorded below.

Canonical PR #186 merged through Rebase + Fast-forward into `dev` at
`430441c4c3708bf70b416432767da4483dd1826c`. Required receipt
`4X5FHJFBNWQNSJWC6TJFMCMRAT` passed the exact three-command list above and
535 native tests. Focused receipt `UX6F7DXZRLYYOJMSRMQENMH42J` passed five native
owner tests, library clippy and documentation on that same clean candidate.
The controller verified and recorded the merge in Acta. Slice 2 of 6 completed.

## Risks and Follow-ups

Repeated or unavailable context can prevent relocation; the origin and record
remain available for an explicit retry or replacement. Collection persistence
and configurable storage/limits were completed in Slice 3.

## Delivery-set closure

Completed: 2026-10-31
Validated candidate: `41ba5f89465b2be02adc613cd796271169c245e0`
Gate receipt: `7WOA4YIWYISUYIDTDT3ZC5SI5A`

The controller passed the unchanged Full gate recorded in the
[tracker](../../../tracker/rev-bookmarks.md#final-gate-and-closure) after all
six slice PRs and three authorized repairs integrated. The delivery set is
completed; the slice receipts above retain their original candidate identities.
