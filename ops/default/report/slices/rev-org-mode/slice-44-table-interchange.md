# Slice 44: import and export Org table data

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-44-table-interchange`
PR: pending

## Purpose

Deliver native table interchange under the
[approved plan](../../../plan/rev-org-mode.md#slice-44-import-and-export-org-table-data)
and [tracker](../../../tracker/rev-org-mode.md#slice-44-import-and-export-org-table-data).

## Delivered Behavior

T44.1 adds compiled `org-table-import` and `org-table-export` commands. Explicit
CSV/TSV input becomes a table on a blank line in one canonical undo unit.
Export writes data rows through checked file publication without changing or
saving the Org source. Existing files require version-bound confirmation;
canonical open destinations and their aliases reject. The existing native
flat-list/one-column-table conversions remain available through M-x.

T44.2 retains quoted delimiters, literal quotes, empty cells, internal tabs,
Unicode and non-ASCII spacing. Malformed quoting, embedded physical/Unicode line
breaks, cell delimiters, controls, significant edge whitespace, ragged records
and exceeded native limits reject before source publication. The entire imported
Org candidate remains bounded to 8 MiB. Native model observations and conversion
workflows verify values independently of codec round trips.

T44.3 verifies real source saves, BOM/CRLF/EOF, peer positions, unsaved canonical
input through hard links, observed overwrite consent, permissions, missing and
invalid files, symlink referents, read-only authority, cancellation and stale
views. The two existing owned terminal sessions exercise M-x interchange,
independent Python CSV parsing, exact output bytes, undo/redo and failed import.

## Implementation Notes

The existing single cancellable worker owns data reads, table preparation and
file output. Canonical open input buffers supply unsaved source. Import rechecks
both input evidence and destination source/view authority before publication.
Export stages bounded bytes through the ordinary file owner, retains actual
replacement outcomes after late cancellation and never acknowledges a source save.

## Contracts Added or Changed

- ORG12: imported values publish through one checked table history transaction.
- ORG27: explicit CSV/TSV interchange retains representable values and file outcomes.

The entered filename suffix selects CSV/TSV even when canonical resolution
follows a symlink to another suffix. Input IO clamps the existing source-reader
budget to the native 8 MiB table ceiling before reading; smaller configured
limits still apply. No new worker slot or resource ceiling is introduced.

## Files of Interest

- `src/org/table/interchange.rs`: native source projection and operation owner.
- `src/org/table/interchange/data.rs`: bounded delimited-data codec.
- `src/session/tables/interchange.rs`: explicit paths, confirmation and live authority.
- `tests/org_tables/interchange.rs`: actual CSV/TSV native round trips.
- `tests/org_io/table_interchange.rs`: checked file lifecycle and identity workflows.
- `scripts/configuration_ui/org_tables.py`: existing owned RGB/NO_COLOR journeys.
- `docs/reference/org-tables.md`: supported formats and rejection rules.

## Validation

- `cargo test --test org_tables interchange -- --nocapture`: CSV/TSV native round-trip workflow passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed.

- `cargo test --test commands -- visibility_keys catalog_reference`: both compiled command/catalog checks passed, zero failures or ignored tests.
- Changed-source quality passed with the existing parameter, nesting and cyclomatic limits.

- `cargo test --test org_tables interchange -- --nocapture`: five native interchange workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed; changed-source quality passed.

- `cargo test --test org_io table_ -- --nocapture`: six native file workflows passed, zero failures or ignored tests.
- `cargo test --test org_tables interchange_readonly -- --nocapture`: canonical import/export authority passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_tables --test org_io -- -D warnings`: passed; changed-source quality passed after final source-reader and authority coverage.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tables`: both owned RGB/NO_COLOR sessions passed, zero skipped. Existing canvas, byte, cleanup and resource assertions remain in place.
- The initial new stale-view driver timed out because a split retains the selected pane. The driver now selects the actual peer during owned work; the original five-second deadline and source/history preservation assertions remain unchanged.

Final exact-candidate ordered checks are recorded through the controller before
handoff. Integration remains open until the controller verifies the canonical PR.

## Risks and Follow-ups

None recorded.
