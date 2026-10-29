# Slice 42: evaluate basic Org table formulas safely

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-42-formula-evaluation`
PR: pending

## Purpose

Deliver private native formula evaluation under the
[approved plan](../../../plan/rev-org-mode.md#slice-42-evaluate-basic-org-table-formulas-safely)
and [tracker](../../../tracker/rev-org-mode.md#slice-42-evaluate-basic-org-table-formulas-safely).

## Delivered Behavior

T42.1 evaluates the selected immutable formula program against a bounded data
grid. Iterative dependency ordering supports forward references, field/column
precedence and inclusive rectangular aggregates. Arithmetic uses finite
binary64 values, explicit half-away rounding and bounded significant/fixed
decimal formatting. Results are private row-major values and text.

## Implementation Notes

The numerical layer consumes the existing typed syntax arena. It neither
interprets external expressions nor acquires source publication authority.
Header rows are excluded from column assignments; explicit fields override
columns and repeated definitions use the last definition of the same target.
Relative coordinates resolve independently at each output cell. Range endpoints
define an inclusive rectangle. Empty cells contribute zero; aggregates omit
text, while scalar arithmetic rejects it. Dependency values retain binary64
precision independently of final text formatting. Explicit `round` changes
numeric values and supports signed decimal precision.

Defaults are 100000 grid cells, 1000000 work steps, 12 significant digits and 12
decimal places. Public ceilings follow the frozen model. Grid admission retains
10000 rows, 256 columns and 32 MiB input bytes. No recursive dependency walk is
used. Mean accumulation avoids overflowing a finite same-sign average through
an unnecessary sum. Every result is prepared privately before returning success.

## Contracts Added or Changed

- ORG13: finite binary64 evaluation, aggregates, rounding and formatting.
- ORG14: whole-result rejection and bounded numerical work.

## Files of Interest

- `src/org/formula/eval.rs`: immutable numerical API and resource policy.
- `src/org/formula/eval/`: target/dependency resolution, arithmetic and formatting.
- `tests/org_tables/evaluation.rs`: independent expected numerical results.
- `docs/reference/org-tables.md`: native numeric and coordinate policy.

## Validation

- `cargo test --test org_tables evaluation`: four workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed after extracting range aggregation to preserve the unchanged nesting limit.

## Risks and Follow-ups

Source publication and interactive recalculation belong to the recorded Slice
43. Numeric output follows binary64 and the frozen native subset, without
claiming arbitrary-precision Calc compatibility.
