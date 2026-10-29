# Slice 41: parse the bounded Org formula subset

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-41-formula-syntax`
PR: pending

## Purpose

Deliver data-only formula syntax under the
[approved plan](../../../plan/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset)
and [tracker](../../../tracker/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset).

## Delivered Behavior

T41.1 introduces a native typed formula arena for compatible field/column
assignments, arithmetic, unary signs, parentheses, absolute/current/relative
references, inclusive ranges, aggregates, round and abs. Multiple assignments
retain their order and source spans. Fixed `fN` formatting is represented as
data. Parsing neither evaluates expressions nor changes canonical source.

## Implementation Notes

`org::formula::Program` owns finite immutable syntax observations. Arena indices
refer only to earlier child nodes, so dropping deeply nested or long syntax does
not recursively drop a boxed tree. Byte spans are relative to the supplied
formula value; they cannot grant buffer identity, revision or edit authority.
No dependency, process, callback, expression interpreter or M-x placeholder is
introduced. Evaluation/publication remain the approved later layers.

## Contracts Added or Changed

- ORG13: typed native formula syntax and compatible assignment/reference forms.
- ORG14: explicit unsupported/syntax/limit/cancellation diagnostics retain source.

## Files of Interest

- `src/org/formula.rs`: public immutable syntax and diagnostic contracts.
- `src/org/formula/`: bounded lexer, references and expression parser.
- `tests/org_tables/syntax.rs`: typed public syntax/assignment workflows.

## Validation

- `cargo check --lib`: passed.
- `cargo test --test org_tables syntax -- --nocapture`: three named workflows passed for assignments, coordinates/ranges and all declared function identities; zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed with the unchanged parameter, nesting and cyclomatic limits.

## Risks and Follow-ups

T41.2 precedence/boundary validation and shared structural-parser integration,
T41.3 preservation/diagnostic verification, final exact-HEAD checks and the
canonical PR remain pending. No result evaluation or recalculation is claimed.
