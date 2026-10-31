# Slice 4: Org projection and bounded wrap lifetimes

Status: drafting
Delivery set: rev-wrap-navigation
Plan: [Wrap and Navigation](../../../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../../../tracker/rev-wrap-navigation.md)
Branch: `feat/rev-wrap-navigation-04-org-wrap`
PR: pending

## Purpose

Complete WRP04 and WRP05 with shared Org projection geometry, bounded layout
lifetimes and measured long-line control. T4.1 is implemented; T4.2 is active.

## Delivered Behavior

- Visible Org headings, table fields and source-block lines use the selected
  buffer wrap policy. Native fold requests retain hidden source rows across
  resize and wrap changes without inserting display newlines into source.
- Table-special navigation retains logical field identity, including long
  fields, tabs, combining/wide graphemes and CRLF. Alignment remains an explicit
  ordinary source edit with exact undo.
- Native source drafts and their parent both wrap. Exiting a draft publishes
  one canonical parent edit while preserving delimiters and unrelated text.
- Image rectangles begin at the actual first visible source fragment after
  preceding wrapped text. Reflow changes placement; folded or scrolled-past
  anchors receive no rectangle. Empty preview sets require no wrap traversal.

## Implementation Notes

T4.1 routes optional preview placement through the same current fragments used
by text, hit testing and gutter labels. Existing preview resource admission,
source stamps, clipping, byte/pixel limits and explicit IO ownership remain in
place. Fold, field and draft commands retain their canonical source owners.

The existing folded-Unicode hit fixture now explicitly covers both factory wrap
and the retained horizontal policy. It asserts complete grapheme destinations
and rejects the appropriate padding/partial-glyph cells in each mode.

## Contracts Added or Changed

Preview anchor rows follow the current visual projection. Pictures neither
create source rows nor acquire another anchor on a continuation. The accepted
buffer policy applies to Org source and tables without an implicit exception.

The bounded layout/cache contract is not yet implemented by this partial slice.

## Files of Interest

- `src/display/preview.rs`: visual-fragment image placement and empty-set fast path.
- `tests/wrapping/org.rs`: native folds, logical fields, source drafts, preview
  reflow and source/history preservation.
- `tests/org_editing/fold_edges.rs`: paired wrap/no-wrap folded Unicode hits.

## Validation

- `cargo test --locked --test wrapping org::`: passed, four native Org wrapping cases.
- `cargo test --locked --test org_io previews::`: passed, three existing preview regressions.
- `cargo test --locked --no-fail-fast --test org_editing --test org_tables --test org_source`:
  table/source suites passed (73 and 81 tests). One historical horizontal-only
  folded-hit expectation failed among 226 Org editing tests; its paired
  correction is checked with the focused command below.
- `cargo test --locked --test org_editing fold_edges::unicode_hits`: passed after
  the fixture correction. Final configured checks wait for completed T4.2 and
  PR-reference publication.

No full delivery gate or complete wrap acceptance is claimed by this draft.

## Risks and Follow-ups

T4.2 must replace repeated line-prefix scans with bounded resumable work and
retained derived state capped at 8192 fragments per view. Current expensive
paths include `View::cell`/`Coordinates::to_cell`, `Line::new`, reveal and neighbor
lookup, and independent frame consumers. Source edits, folds, width/tab/wrap
policy, split/rebind/close and Unicode boundary context all need coherent
invalidation. A fragment cache alone does not bound coordinate-prefix scans.

Complete lifecycle and cache-eviction acceptance, native input/resize/reversal
and idle measurements, exact successful final validation, and the approved
long-line ticket transition before preparing the canonical PR.
