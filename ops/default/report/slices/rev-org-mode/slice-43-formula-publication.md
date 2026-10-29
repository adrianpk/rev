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

## Contracts Added or Changed

- ORG13: explicit recalculation publishes prepared results as one undo unit.
- ORG14: failed calculations preserve existing results and formula source.
- ORG17: compiled native recalculation command.

## Files of Interest

- `src/org/table/calculate.rs`: numerical preparation and checked field plans.
- `src/session/tables.rs`: existing publication owner and natural point affinity.
- `tests/org_tables/recalculation.rs`: real canonical history/failure workflows.
- `docs/reference/org-tables.md`: explicit recalculation semantics.

## Validation

- `cargo test --test org_tables recalculation -- --nocapture`: two transaction and rejection workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed with unchanged parameter, nesting and cyclomatic limits.

## Risks and Follow-ups

Cell/column selection and contextual formula-line dispatch remain the next
recorded task in this slice.
