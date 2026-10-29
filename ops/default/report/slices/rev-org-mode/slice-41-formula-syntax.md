# Slice 41: parse the bounded Org formula subset

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-41-formula-syntax`
PR: [#142](https://forge.adrianpk.com/adrianpk/rev/pulls/142)

## Purpose

Deliver data-only formula syntax under the
[approved plan](../../../plan/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset)
and [tracker](../../../tracker/rev-org-mode.md#slice-41-parse-the-bounded-org-formula-subset).

## Delivered Behavior

T41.1 introduces a native typed formula arena for compatible field/column
assignments, arithmetic, unary signs, parentheses, absolute/current/relative
references, inclusive ranges, aggregates, round and abs. Multiple assignments
retain their order and source spans. Fixed `fN` and `%.Nf` formatting are represented as data. Parsing neither evaluates expressions nor changes canonical source.

## Implementation Notes

Task commits: T41.1 `9f599f4cd663572378ee0820c8e242204642bd35`,
T41.2 `cab7833a5be8c819c9b17548504b09ca19805bbe`,
T41.3 `3872b96e962cd03dfd3700f44f763e5528492065`.

`org::formula::Program` owns finite immutable syntax observations. Arena indices
refer only to earlier child nodes, so dropping deeply nested or long syntax does
not recursively drop a boxed tree. Byte spans are relative to the supplied
formula value; they cannot grant buffer identity, revision or edit authority.
No dependency, process, callback, expression interpreter or M-x placeholder is
introduced. Evaluation/publication remain the approved later layers.

T41.2 makes the syntax arena the shared structural rewrite grammar and removes
the earlier lexical scanner. Absolute rewrites still validate every original
range coordinate and reject relative/current transformation ambiguity before
publication. Grammar admission does not confer target or mutation authority.

Powers associate right and bind above unary signs. Org's multiplication-above-
division rule is retained; subtraction/division associate left. No computed
numeric result is used as proof of parser precedence. Independent expected trees
establish grouping. Defaults admit 8 MiB input, 100000 tokens/nodes, 1000000 work
steps, depth 64 and 12 decimal places; public limit ceilings follow the frozen
model. Iterative token/node arenas and guarded expression descent bound memory,
work and stack depth. Cancellation and the existing worker deadline propagate
through lexing and parsing. Formatting is bounded data, never a printf executor.

T41.3 checks exact UTF-8-relative error spans, EOF diagnostics, unsupported
Calc/Lisp/named/hline forms, malformed numeric coordinates and unsigned decimal
precision. The malformed exponent regression first reported EOF alone; numeric
errors now cover the complete bad literal, and coordinate overflow covers its
complete numeric spelling. Missing coordinates are syntax errors, while known
advanced coordinate forms remain unsupported. Canonical read-only observations
serialize byte-identically after successful and failed parsing, including
BOM/CRLF/Unicode/EOF. A real formatted-reference edit changes only admitted
coordinate digits and retains ordinary one-operation undo/redo.

## Contracts Added or Changed

- ORG13: typed native formula syntax and compatible assignment/reference forms.
- ORG14: explicit unsupported/syntax/limit/cancellation diagnostics retain source.

## Files of Interest

- `src/org/formula.rs`: public immutable syntax and diagnostic contracts.
- `src/org/formula/`: bounded lexer, references and expression parser.
- `tests/org_tables/syntax.rs`: typed public syntax/assignment workflows.
- `tests/org_tables/diagnostics.rs`: precise errors, public limit ceilings and real formatted-reference publication.
- `tests/org_source/formulas.rs`: canonical formula observation and file-format preservation.

## Validation

- `cargo check --lib`: passed.
- `cargo test --test org_tables syntax -- --nocapture`: three named workflows passed for assignments, coordinates/ranges and all declared function identities; zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed with the unchanged parameter, nesting and cyclomatic limits.

- `cargo test --test org_tables -- syntax references lifecycle::structural_rejections`: 14 syntax/reference/boundary workflows passed, including independently expected precedence trees, fixed formats, 64-level boundary, 100000-token exhaustion and existing atomic range rejection.
- `cargo test --lib formula_deadline -- --nocapture`: the private worker deadline token rejects expired parsing.

- `cargo test --test org_tables --test org_source -- diagnostics formulas`: four workflows passed for exact diagnostics, limit admission through depth 256, formatted reference editing/undo and canonical read-only source fidelity.
- `cargo clippy --lib --test org_tables --test org_source -- -D warnings`: passed; changed-source quality passed with unchanged limits.

## Risks and Follow-ups

Exact-HEAD required/focused validation is recorded through the controller for
the PR-reference commit. Canonical PR #142 records the successful required
command list. Verified dev integration remains pending. The approved evaluation/publication layers remain separate. The
pre-existing outline timer fixture diagnostic retained in Slice 39 is unchanged.
