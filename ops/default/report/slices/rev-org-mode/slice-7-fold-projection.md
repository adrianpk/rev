# Slice 7: Map folded Org views to canonical source

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-07-fold-projection`
PR: [#108](https://forge.adrianpk.com/adrianpk/rev/pulls/108)

## Purpose

Provide view-local hidden-source projection through existing canonical coordinates.

## Delivered Behavior

Each native Org view admits revision-qualified complete logical-line source
ranges. Overlapping ranges normalize into one hidden interval. Visible screen
rows map bijectively to original logical source rows; hidden source has no screen
position. EOF, foreign/stale authority, partial lines and current-point hiding
have explicit admission outcomes. Source, history and shared peer views survive.

Display text, semantic captures, cursor placement, gutter labels, selections,
search overlays and mouse hit testing resolve the same projected source rows.
Vertical/page/viewport scrolling and pointer autoscroll advance visible rows.
Status and relative/absolute numbering continue to identify original source.
Canonical regions include their hidden source instead of substituting visible
text. Targeted navigation/search reveals its containing hidden interval.

Task commits: T7.1 `24018c71cda02a70db9e371bcdbf56806c566f8a`, T7.2
`b8a12209f31c2d3209c77cec5d7048bdff1b9feb`, T7.3
`8ef64c23b0dd982bf0ead9a9e523fc692a976005`.

## Implementation Notes

An immutable shared interval table keeps view cloning cheap. Admission allows
at most 4096 ranges, sorts/merges their row intervals, and stores cumulative
hidden counts. Forward/reverse queries use binary search without constructing a
whole-document row array. `View.top` remains a canonical logical row.

Entering a hidden row through mutable reveal removes that containing interval
only in the requesting view. Stale or inactive-mode maps project plain source;
reveal releases stale offsets. New buffer bindings reset visibility.

Native regressions verify Unicode/wide/ZWJ and tab-edge hits, tiny geometry,
maximum-range round trips, stale view/range rejection, edit/undo/redo reveal,
read-only failures and independent split search/cancel. Actual mouse events
exercise region endpoints across a hidden gap, wheel movement and autoscroll
termination at a hidden source tail. Left absolute and right relative gutters
retain source numbering and current-line color across gaps.

## Contracts Added or Changed

- ORG03: independent revision-qualified view visibility and non-aliasing row maps.
- ORG18: finite admission, explicit rejection and unchanged source authority.

## Files of Interest

- `src/view/fold.rs`, `src/view/fold/build.rs`: visibility admission and row maps.
- `src/view.rs`: native per-view ownership and checked reveal.
- `src/display.rs`, `src/display/gutter.rs`, `src/display/search.rs`: shared row projection.
- `src/view/hit.rs`, `src/view/scroll.rs`, `src/session/pointer/autoscroll.rs`:
  native hit, motion and viewport mappings.
- `src/window/points.rs`: release obsolete fold offsets on canonical publication.
- `tests/org_editing/folds.rs`: independent maps, admission and EOF contracts.
- `tests/org_editing/projection.rs`: display, regions, search, hits and motion.
- `tests/org_editing/fold_edges.rs`, `tests/pointer/folds.rs`, `tests/gutter/folds.rs`:
  native edge, lifetime, gesture, timer-cleanup and source-numbering contracts.
- `docs/reference/syntax.md`, `docs/reference/gutter.md`: library projection rules.

## Validation

Implementation checks:

- `cargo check --lib`: passed.
- `cargo test --test org_editing folds::`: three passed.
- `cargo test --test org_editing projection::`: three passed.
- `cargo test --test org_editing fold_edges::`: six passed.
- `cargo test --test pointer folds::`: two passed.
- `cargo test --test gutter folds::`: one passed.
- Focused source-quality checks passed for changed native owners and fixtures.

After the pushed PR-reference follow-up, the controller records the final clean
candidate and outcomes for this exact ordered batch in its validation ledger:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test display --test gutter --test pointer`
- `python3 scripts/check-source.py`
- `git diff --check`

Final receipts are recorded in Acta; the integration follow-up retains their
exact candidate and outcome here.

## Risks and Follow-ups

Canonical edits reveal all affected views rather than requalifying obsolete fold
offsets. Native visibility commands and hierarchy cycles belong to recorded
Slice 8; this slice supplies their checked view projection.
