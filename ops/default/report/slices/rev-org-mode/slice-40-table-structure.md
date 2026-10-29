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
T40.2 rewrites absolute field/column/row references across structural edits,
including assignment targets and supported rectangular ranges. Unsupported or
ambiguous transformations reject the complete operation before publication.
All table and formula changes use one ordinary undo unit.

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

Formula rewriting tracks original logical axis identities independently of
retained-byte provenance. It validates every interior range coordinate rather
than treating endpoint provenance as proof of unchanged semantics. Only changed
coordinate digits receive actual stamped replacements; all formula lines join
the same canonical transaction. Identical cell bytes still retain real movement
identity and shared positions. The rewrite scanner admits compatible arithmetic,
finite literals and the documented function list, bounded to 100000 tokens and
64 parenthesis levels; it does not evaluate or recalculate formulas.

## Contracts Added or Changed

- ORG12: real structural table operations and retained cell/source coordinates.
- ORG14: checked absolute-reference rewrites preserve logical targets; unsupported syntax, removed targets and noncontiguous/reordered ranges reject atomically.

## Files of Interest

- `src/org/table/edit/`: structural plans, sorting and conversion.
- `src/buffer/edit/retained.rs`: canonical retained-source admission.
- `src/session/tables.rs`: current-source publication and owned sort prompts.
- `tests/org_tables/structure.rs`: actual editing/history/shared-view workflows.
- `src/org/table/references/`: bounded reference scanning and explicit logical-axis/range validation.
- `tests/org_tables/references.rs`: actual structural reference transactions and atomic rejection.
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

- `cargo test --test org_tables references -- --nocapture`: seven reference workflows passed, covering multiple CRLF formula lines, insert/delete/move, rectangular shifts, complete range-target proof, unsupported forms, identical cells and undo/redo.

- `cargo test --test org_tables metadata -- --nocapture`: five existing metadata, unknown-formula preservation and table-ownership workflows passed.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed for the reference implementation; changed-source quality also passed.

## Risks and Follow-ups

T40.3 extended structural validation remains pending. Final exact-HEAD controller checks and the canonical PR remain pending.
The pre-existing outline timer assertion diagnostic remains in the Slice 39
report; no timer behavior or assertion is changed here.
