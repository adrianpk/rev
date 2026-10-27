# Slice 1: Model typed configuration

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-01-config-types`
PR: pending

## Purpose

Establish the typed configuration owner and complete compiled defaults for the
accepted catalog, retaining source presence for subsequent validation and
resolution.

## Delivered Behavior

- Provides complete global defaults for editing, scrolling, windows,
  minibuffer, history, kill ring, tree, syntax, new-file policy and all 25
  accepted semantic RGB roles.
- Deserializes concern tables and language overrides into partial records.
  Missing leaves remain absent; explicit false, zero, empty arrays and raw
  empty strings remain available for validation rather than becoming defaults.
- Retains schema version, ordered include literals, file associations and
  the three context-specific user key maps as source data.
- Pins TOML 1.1 parsing and Serde derives and adds five integration tests for
  source presence, language inheritance inputs and accepted default values.

## Implementation Notes

- T1.1: `64038cccf6332c17410ad4fe85e589adfcccd5d9` adds the configuration
  records, compiled defaults, fixed palette and numeric semantics.
- T1.2: `21bd37e90d8cd48d93e37e4284cf511f4485a524` pins dependencies and
  adds partial deserialization and source-presence tests.
- T1.3 adds catalog invariants and this preparatory report. Startup activation
  remains assigned to Slice 20; these types do not advertise operational controls.
- `Document` contains partial concern records instead of defaulted `Settings`.
  Signed raw integers and color literals retain invalid input for later
  source-aware diagnostics. Deserialization is not candidate admission.
- User association and key maps start empty; existing built-in file recognition
  and input dispatch retain ownership of their defaults. No language override
  silently installs a Go indentation convention.
- TOML enables only `parse`, `serde` and `std`; display/serialization features
  are disabled. Serde enables derives. Rust 1.98, GPL-3.0-only and first-party
  safe Rust remain the package contract.

## Contracts Added or Changed

- `Settings::default()` supplies the accepted global catalog without disk input.
- `Document` and `Language` preserve per-leaf absence. Include arrays retain
  declaration order and distinguish omission from an explicit empty list.
- Fixed enum types encode indentation, command matching, tree root, newline
  and bundled grammar choices. Partial tables reject unrecognized field names.
- Widths, counts, UTF-8 bytes and Unicode scalars have explicit units and
  accepted bounds in source documentation. Zero syntax bytes means no configured
  cutoff; the history byte target preserves the newest coherent group.
- RGB roles are terminal-independent channel values; raw color validation and
  application remain separate from these compiled defaults.

## Files of Interest

- `src/config.rs`: complete and partial configuration records and numeric units.
- `src/config/palette.rs`: all accepted palette roles and compiled RGB values.
- `Cargo.toml`, `Cargo.lock`: exact parser/derive dependencies and resolved graph.
- `tests/configuration.rs`: partial source and language-presence contracts.
- `tests/configuration/catalog.rs`: scalar and 25-role default invariants.

## Validation

Development checks:

- `cargo check --lib`: passed.
- `cargo check --tests`: passed.
- `cargo test --test configuration -- --skip catalog::`: passed; three tests.
- `cargo test --test configuration catalog::`: passed; two catalog tests.

The controller stores final results for the exact clean PR head after its
PR-reference follow-up commit. Required commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Candidate diagnostics, include traversal and precedence remain assigned to
Slices 2-4. Startup configuration and consumer effects are not established by
this preparatory layer's tests.
