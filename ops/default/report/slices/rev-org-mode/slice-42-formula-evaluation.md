# Slice 42: evaluate basic Org table formulas safely

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-42-formula-evaluation`
PR: [#143](https://forge.adrianpk.com/adrianpk/rev/pulls/143)

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

Task commits: T42.1 `9b64692c22d779254de9e0bc673a835ed37ccf4d`, T42.2 `a82d0bafe9b0c99def549d265c6a87a2bc401712`, T42.3 `858a37838e90fee2cc6d83cc6da767c542a19ac4`.

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

T42.2 rejects cyclic graphs, invalid coordinates, text/range scalar misuse,
zero division, non-finite inputs and operations, and empty non-sum aggregates.
Cancellation/deadline is checked before allocating engine arrays. Input parsing
charges original bytes before trimming, so whitespace cannot evade work limits.
Failures after earlier successful computations or during final formatting still
return no result set. Diagnostics retain the failing reference/expression span
and output cell; an invalid assignment target does not claim an existing cell.

T42.3 verifies literal expected arithmetic, both signs of halfway rounding,
forward/backward relative axes, current/omitted coordinates, reversed rectangular
endpoints and dependencies on interior range cells. A 10000-row dependency chain
completes iteratively. Ragged inputs, field precedence, duplicate definitions,
header exclusion, finite large means and significant-digit carry are covered.
Actual canonical table-cell and formula observations are evaluated from a
read-only BOM/CRLF/Unicode document; both success and late division failure leave
the original bytes, stamp, state, modification flag and history unchanged.

## Contracts Added or Changed

- ORG13: finite binary64 evaluation, aggregates, rounding and formatting.
- ORG14: whole-result rejection and bounded numerical work.

## Files of Interest

- `src/org/formula/eval.rs`: immutable numerical API and resource policy.
- `src/org/formula/eval/`: target/dependency resolution, arithmetic and formatting.
- `tests/org_tables/evaluation.rs`: independent expected numerical results.
- `docs/reference/org-tables.md`: native numeric and coordinate policy.
- `tests/org_tables/evaluation_errors.rs` and `evaluation_workflows.rs`: rejection, limits and coordinate/dependency expectations.
- `tests/org_source/formulas.rs`: canonical numerical source fidelity.

## Validation

- `cargo test --test org_tables evaluation`: four workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed after extracting range aggregation to preserve the unchanged nesting limit.
- `cargo test --test org_tables evaluation -- --nocapture`: eight evaluation workflows passed, including rejection, resource, non-finite and diagnostic cases; zero failed or ignored.
- `cargo test --lib evaluation_deadline`: the private worker deadline test passed.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed after failure handling changes; changed-source quality passed.
- `cargo test --test org_tables --test org_source evaluation -- --nocapture`: 15 focused workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables --test org_source -- -D warnings`: passed; changed-source quality passed after the final expected-result cases.

- Final required receipt `LRKE3EYHBXLIIWBJDLGJGLO4NZ` at `04e2f4d8fe5b932eecf9f767c1a2b9035a0947e0` passed the five ordered checks and 138 tests.
- Final focused receipt `ZCW5ICYYNSCDKALWG77QG5INED` at the same head passed 111 library/source tests and two owned terminal sessions. Final receipts have zero failures, ignored tests or skipped sessions.
- Canonical PR #143 merged into dev at `04e2f4d8fe5b932eecf9f767c1a2b9035a0947e0`, verified against Forgejo and origin/dev.

## Risks and Follow-ups

Source publication and interactive recalculation belong to the recorded Slice
43. Numeric output follows binary64 and the frozen native subset, without
claiming arbitrary-precision Calc compatibility.
