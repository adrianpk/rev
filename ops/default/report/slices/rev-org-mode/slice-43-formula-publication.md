# Slice 43: publish Org recalculation as one transaction

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-43-formula-publication`
PR: pending

## Purpose

Deliver explicit checked recalculation under the
[approved plan](../../../plan/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction)
and [tracker](../../../tracker/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction).

## Delivered Behavior

T43.1 publishes explicit whole-table formula results through one canonical
history transaction. All numerical values and formatted results are prepared
before source changes. Only changed result fields are replaced; formula text,
unrelated values, separators and padding remain intact. Identical results add
no history. Ordinary undo/redo restores the complete selected operation.

T43.2 exposes table, cell and column recalculation through the compiled M-x
catalog. C-c * runs the native whole-table operation. C-c C-c on a TBLFM line
applies that program; table-body C-c C-c remains alignment. Ordinary table
recalculation uses the first formula line. Cell/column selection evaluates only
its transitive assigned dependencies and publishes only the selected outputs.
Unselected dependency result fields and unrelated failing formulas remain intact.

## Implementation Notes

The existing table snapshot captures the canonical stamp and immutable source.
Its owned cancellable worker parses the selected program, projects actual
table values, computes private numerical results and validates every output
destination. Publication rechecks the source/view request and delegates source,
read-only, point/mark affinity and history authority to the existing buffer
transaction. No numerical observation supplies a new buffer identity.

Recalculation uses existing source fields only. Ragged missing input is zero,
but a missing output field rejects before publication. Proposed source/output
retains the native 8 MiB ceiling. Header detection uses the first separator
with data above and below. A later numeric failure or unsupported program
cannot publish earlier successful calculations. No idle recalculation runs.

The source owner captures validated numeric preferences with the initiating
configuration. Significant digits, grid cells, syntax/numeric work steps,
syntax depth and decimal precision have typed defaults, bounded parser
admission, per-leaf overlay/provenance and live operation consumers. Empty input
and half-away rounding retain the fixed native profile. The complete sample,
binding catalog and configuration reference describe the actual accepted leaves.

## Contracts Added or Changed

- ORG13: explicit recalculation publishes prepared results as one undo unit.
- ORG14: failed calculations preserve existing results and formula source.
- ORG17: compiled native recalculation command.

## Files of Interest

- `src/org/table/calculate.rs`: numerical preparation and checked field plans.
- `src/session/tables.rs`: existing publication owner and natural point affinity.
- `tests/org_tables/recalculation.rs`: real canonical history/failure workflows.
- `docs/reference/org-tables.md`: explicit recalculation semantics.
- `src/org/formula/eval/graph.rs`: selected dependency closure without recursive traversal.
- `src/config.rs` and `src/config/`: typed numerical settings and source-owned projection.

## Validation

- `cargo test --test org_tables recalculation -- --nocapture`: two transaction and rejection workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed with unchanged parameter, nesting and cyclomatic limits.
- `cargo test --test org_tables --test commands -- recalculation visibility_keys`: seven native workflows passed for publication, selection, formula context, settings and compiled bindings; zero failed or ignored.
- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed; changed-source quality passed after moving formula-selection logic into its owner.
- `python3 scripts/check-config-reference.py`: passed after updating the documented Org map to 71 entries; 160 schema/catalog entries and 250 default bindings agree with the packaged reference.

## Risks and Follow-ups

The owned terminal journey and source/worker lifecycle coverage remain the
next recorded task in this slice.
