# Slice 45: display Org property columns and native reports

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-45-columns-dynamic-blocks`
PR: pending

## Purpose

Deliver property columns and explicit native reports under the
[approved plan](../../../plan/rev-org-mode.md#slice-45-display-org-property-columns-and-native-reports)
and [tracker](../../../tracker/rev-org-mode.md#slice-45-display-org-property-columns-and-native-reports).

## Delivered Behavior

T45.1 adds native source property columns and `columnview` dynamic reports.
Queries retain canonical heading/revision provenance. Numeric/duration summaries
compute parents from direct children; recorded clock totals include each
heading's own intervals and descendants. Opening and refreshing columns do not
write source or property drawers. Open, unresolved and overlapping intervals
remain visible. Native clock-table commands retain their existing behavior.

T45.2 keeps report definitions and computed values separate from source
properties. Unknown writers, malformed/unsupported options, invalid late aggregate
values and unrepresentable cells preserve the complete existing body. Actual
source edits, selected-pane changes, cancellation and read-only authority cannot
publish a retained result. Generic agenda presentation commands cannot discard
the source-owned COLUMNS semantics; the view refreshes through its native owner.

T45.3 verifies deep direct-child summaries, inherited COLUMNS, local scope and
maxlevel, configured duration units, literal declaration exclusion, exact source
provenance, physical C-c C-c, persisted BOM/CRLF/EOF, peer positions and inactive
marks. Native insertion, property changes and saves require explicit report
refresh; identical updates add no history. The existing two editing terminal
journeys now exercise source columns, read-only input, refresh and close.

## Implementation Notes

The existing source, query, clock-scan, report-worker and checked transaction
owners prepare the projection. File COLUMNS declarations come from actual
structural keyword observations; inherited COLUMNS properties use the existing
property resolver. Only explicit recognized report bodies can become edits.
No additional task database, worker slot, runtime dependency or evaluator is added.

## Contracts Added or Changed

- ORG21: effort and interval totals derive from actual canonical source.
- ORG23: property projections retain checked source provenance.
- ORG27: explicit native columnview bodies update coherently.

## Files of Interest

- `src/org/columns.rs`: bounded source projection/report work.
- `src/org/columns/summary.rs`: hierarchical property and interval totals.
- `src/org/visibility/reports/columns.rs`: structural declarations and exact body scope.
- `src/session/outline/reports/columns.rs`: existing report publication owner.
- `tests/org_agenda/columns.rs`: native projection, source visits and update history.
- `docs/reference/org-columns.md`: admitted source formats and operation bounds.

## Validation

- `cargo test --test org_agenda column_ -- --nocapture`: both native projection/report workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.

- `cargo test --test org_agenda --test commands -- column_ visibility_keys catalog_reference`: four projection, report and compiled-catalog workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda --test commands -- -D warnings`: passed; changed-source quality passed with unchanged parameter, nesting and cyclomatic limits.
- `git diff --check`: passed.

- `cargo test --test org_agenda column_ -- --nocapture`: six native projection, report and rejection workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed; changed-source quality passed after publication guard coverage.

- `cargo test --test org_agenda column_ -- --nocapture`: 11 native column workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed; changed-source quality passed after lifecycle coverage.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-agenda`: all six existing owned RGB/NO_COLOR editing, read-only and cancellation/stale sessions passed, zero skipped. The two editing sessions include source columns; existing canvas, persisted-byte, cleanup and resource assertions remain unchanged.

The file workflow retains the entered `30min` EFFORT spelling in canonical
source and independently expects `0:30` in the derived report. Saving retains
the previous report body until its explicit update.

Final ordered validation is recorded through the controller for the clean PR
candidate before handoff. Integration remains open for controller verification.

## Risks and Follow-ups

None recorded.
