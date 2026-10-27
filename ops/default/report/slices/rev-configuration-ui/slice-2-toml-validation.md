# Slice 2: Validate TOML and diagnostics

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-02-toml-validation`
PR: pending

## Purpose

Admit one typed configuration source only after its encoding, schema, bounds,
colors and scoped values validate, with actionable source diagnostics.

## Delivered Behavior

- Parses UTF-8 TOML 1.1, accepting omitted schema version as 1 and preserving
  absent leaves, explicit false, zero and empty include arrays.
- Rejects unsupported versions, unknown fields, wrong types and enum values,
  invalid numeric bounds, nonfinite geometry values and malformed RGB colors.
- Validates language IDs and permitted partial overrides, literal basename or
  suffix association patterns and existing command-name binding values.
- Reports the supplied source path, decoded key components and available
  original byte span, with one-based line and Unicode-scalar column positions.
- Exercises every admitted numeric boundary, all 25 palette roles, malformed
  and duplicate TOML, encoding failures and dynamic key/value locations.

## Implementation Notes

- T2.1: `2b4ff0093157ca30bc540c935f12fd6f18b5ec87` introduces the parser,
  typed source diagnostics, numeric admission and palette validation.
- T2.2: `4c1512f370a0731c82e97ba30d250c1ab102057d` validates language and
  association scopes and command values, distinguishing key and value spans.
- T2.3 adds boundary coverage and this report. A regression fixture reproduced
  signed numeric fragments being admitted as RGB channels; requiring six ASCII
  hexadecimal digits rejects them before channel conversion.
- The TOML parser builds one spanned tree. A shallow location index retains
  the accepted schema's three-component paths before typed deserialization
  consumes that tree. Arbitrary rejected subtrees do not cause an unbounded
  diagnostic walk.
- Raw byte spans and scalar columns remain distinct. Diagnostics omit source
  snippets and unrelated environment values. Syntax failures retain available
  locations without inventing a semantic key when no parsed tree exists.
- Parsing accepts bytes supplied by its caller and has no file, terminal or
  editor-state mutation. Small history byte targets are admitted as soft
  preferences; validation does not allocate history or clamp layout geometry.

## Contracts Added or Changed

- `config::parse(&Path, &[u8])` returns a validated partial `Document` or one
  `Diagnostic`; failure never returns a partly accepted document.
- `Diagnostic` carries source, decoded key, optional byte span and optional
  `Position`. Both position components are one-based; columns count scalars.
- `Rgb::parse` admits exactly `#RRGGBB`, with either hexadecimal letter case.
- Language tables admit only indentation style/width, tab width and bundled
  grammar selection. Association values may name valid IDs without installing
  a grammar or declaring a language table.
- Binding values must name existing commands; non-string values, unknown names
  and attempted unbinding fail. Complete context and chord checks remain owned
  by the later keymap validation layer.

## Files of Interest

- `src/config/parse.rs`: one-source parsing and shallow source-location index.
- `src/config/diagnostic.rs`: structured diagnostics and scalar positions.
- `src/config/validate.rs`: numeric, palette, language and mapping admission.
- `src/config/palette.rs`: strict RGB channel decoding.
- `tests/configuration/parsing.rs`, `scopes.rs`, `boundaries.rs`: source,
  diagnostic, scope and catalog-boundary coverage.

## Validation

Development checks:

- `cargo clippy --lib -- -D warnings`: passed.
- `cargo test --test configuration scopes::`: passed; three scope tests.
- `cargo test --test configuration boundaries::`: passed; eight boundary tests
  after reproducing and correcting signed-channel RGB admission.
- `cargo test --test configuration -- --skip catalog:: --skip partial_presence --skip language_presence --skip source_records`: passed; 15 source-validation tests.

The controller stores final results for the exact clean PR head after its
PR-reference follow-up commit. Required commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Include traversal and layered candidate resolution remain assigned to Slices
3-4. Context-specific chord admission and prefix conflicts remain assigned to
Slice 17; runtime configuration activation remains assigned to Slice 20.
