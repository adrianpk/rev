# Slice 40: edit Org table rows columns and references

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-40-table-structure`
PR: pending

## Purpose

Deliver structural table editing under the
[approved plan](../../../plan/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references)
and [tracker](../../../tracker/rev-org-mode.md#slice-40-edit-org-table-rows-columns-and-references).

## Delivered Behavior

T40.1 delivers row/column insertion, removal and movement, stable alphabetical
and numeric sorting, and checked flat-list/one-column-table conversion. Meta
arrows dispatch to native table operations in table context. Sorting honors the
active region's endpoint rows and mark column, while retaining separator groups.
All source changes use one ordinary undo unit.

## Implementation Notes

Structural plans use the existing captured canonical rope, native scanner,
layout and single cancellable job. The buffer transaction owner verifies retained
source stamps, equal candidate bytes, disjoint source/output ranges and containment
inside the actual changed source/candidate pair. Shared points and marks follow
those retained cell values. Other positions use ordinary edit affinity.
History replays the same map in both directions; sparse lookup uses binary search.
No fragment receives an invented buffer identity or separate persistence owner.

The existing table bounds remain: 8 MiB source/output, 10000 rows, 256 columns,
100000 rendered data cells and 4096 displayed columns per fragment. Canonical
retained maps admit at most 200000 runs. Sorting is bounded to selected rows or
one separator group and admits a/A or n/N; non-finite/nonnumeric numeric keys
reject. Uppercase selects reverse order without reversing equal-key groups.
Conversion admits contiguous flat single-line lists and one-column tables
without separators; nested/continued items, literal table delimiters and
ambiguous table shapes reject before publication.

## Contracts Added or Changed

- ORG12: real structural table operations and retained cell/source coordinates.
- ORG14: unsupported formula-bearing transformations remain diagnosed before publication; checked rewrites remain assigned to T40.2.

## Files of Interest

- `src/org/table/edit/`: structural plans, sorting and conversion.
- `src/buffer/edit/retained.rs`: canonical retained-source admission.
- `src/session/tables.rs`: current-source publication and owned sort prompts.
- `tests/org_tables/structure.rs`: actual editing/history/shared-view workflows.
- `docs/reference/org-tables.md`: supported commands and boundaries.

## Validation

- `cargo test --test org_tables -- --nocapture`: 23 workflows pass, zero failed or ignored, including six structural workflows for row/column edits, separator boundaries, group/region sorting, conversion and shared positions/marks with undo/redo.
- Changed-source quality passes with unchanged parameter, nesting and cyclomatic limits.
- The first row-up fixture expected a row to jump over two lines; its exact expectation now matches a single adjacent-row move, including movement across a separator. The implementation's row move did not change for this fixture correction.

- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed.
- `cargo test --lib buffer::edit::retained -- --nocapture`: canonical retained-source admission passed, including mismatched bytes, stale/foreign identities, overlapping/out-of-range claims, claims outside actual changes and read-only rejection.
- `cargo test --test commands command_sequences -- --nocapture`: passed.
- `cargo test --test commands catalog_reference -- --nocapture`: passed after removing one duplicate documentation entry for the existing row-insert command; the new binding remains documented.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog entries, 54 RGB roles and 249 bindings agree; the 19431-byte packaged template is exact.

## Risks and Follow-ups

T40.2 formula-reference rewrites and T40.3 extended structural validation remain
pending. Final exact-HEAD controller checks and the canonical PR remain pending.
The pre-existing outline timer assertion diagnostic remains in the Slice 39
report; no timer behavior or assertion is changed here.
