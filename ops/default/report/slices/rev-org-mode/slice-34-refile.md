# Slice 34: refile Org subtrees across canonical documents

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-34-refile`
PR: pending

## Purpose

Deliver checked complete-subtree refile/copy under the
[approved plan](../../../plan/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents)
and [tracker](../../../tracker/rev-org-mode.md#slice-34-refile-org-subtrees-across-canonical-documents).
The [Org reference](https://orgmode.org/manual/Refile-and-Copy.html) supplies
complete-subtree destination selection and child placement semantics.

## Delivered Behavior

T34.1 prepares revision-qualified complete sibling subtrees and bounded target
headings. Plans preserve relative levels, metadata and literal body source;
new separators follow the destination format. Self/descendant targets and stale
same-source observations reject before edit qualification. Same-buffer boundary
moves use one replacement; cross-buffer move/copy keeps each actual owner.

## Implementation Notes

The source uses the existing complete structural model and native checked spans.
Target observations use the existing bounded agenda/source scan; BOM evidence
stays with its immutable source snapshot. Shared structural normalization serves
capture and refile. Plans qualify ordinary replacements without publishing or
writing files. No synthetic canonical buffer identity represents a fragment.

## Contracts Added or Changed

- ORG11: checked complete subtree and destination plans.
- ORG02: canonical revision, ordinary history and source-format boundaries.

## Files of Interest

- `src/org/refile/`: bounded target observations and checked move/copy plans.
- `src/org/visibility/subtree.rs`: complete native subtree selection.
- `src/org/structure/entry.rs`: shared heading-level normalization.
- `src/org/agenda/sources/scan.rs`: retained source BOM evidence.

## Validation

- `cargo test --lib refile -- --nocapture`: two owned-file tests pass,
  including cross-buffer move/copy, literal source, BOM/CRLF, same-buffer
  boundary placement, descendant rejection and stale observations.
- `cargo clippy --lib -- -D warnings`: passed.
- Changed-source quality passes with unchanged parameter, nesting and complexity limits.

The literal block fixture uses the native comma-quoted heading syntax, retaining
its expected bytes during movement. Source and destination disk bytes remain
unchanged while planning and publishing the tested in-memory edits.

## Risks and Follow-ups

T34.2 native workflow/coherent publication and T34.3 failure evidence remain in
progress. The delivery-set gate remains controller-owned.
