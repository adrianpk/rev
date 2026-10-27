# Slice 26: Publish exhaustive configuration reference

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `docs/rev-config-ui-26-configuration-reference`
PR: [#75](https://forge.adrianpk.com/adrianpk/rev/pulls/75)

## Purpose

Provide the complete supported startup configuration sample and an exact
reference whose schema, defaults, bindings and examples are checked for drift.

## Delivered Behavior

- `docs/reference/config.toml` lists all active global defaults, all 25 semantic
  RGB assignments and complete editor/minibuffer/tree maps: 66, 22 and 21 entries.
  English comments distinguish active defaults from optional include, language
  and association examples and explain how to enable the optional blocks.
- The existing defaults reference documents all 59 schema/catalog entries,
  including source metadata, maps, language leaves, types, enum choices, inclusive
  bounds, units, defaults, scope and inheritance. It links the operational sample.
- Source composition documents ordered depth-first layering, leaf/map behavior,
  source/graph limits, cycle handling, complete invalid-candidate fallback and
  startup-only lifetime. No implicit project discovery or settings creation is
  implied. Existing file formats, dirty buffers and canonical owners remain
  authoritative.
- Command matching/history, shared panel geometry and selected/typed find-file
  flows are documented together with their detailed owner references. Terminal
  color/cursor and physical key limitations remain explicit.
- Binding documentation explains partial overrides versus full explicit maps
  and native stage distinctions. Syntax recognition uses visiting basenames and
  configured language/grammar rules. The reference index, concise README and
  existing Unreleased capability entry link the complete sample.
- The deterministic reference checker compares sample tables against declared
  Rust fields, exact default binding catalogs, metadata types/defaults and all
  enum choices. It rejects missing leaves that runtime defaulting could hide,
  validates optional example coverage and checks local documentation links.
- Eight native tests use the actual parser/loader/resolver for typed default
  equality, complete declared/effective maps, uncommented optional examples and
  every published numeric/enum domain. Seven named in-memory drift probes verify
  the checker rejects missing/new leaves, stale command targets, omitted palette
  documentation, incorrect defaults/types and missing enum alternatives.

## Implementation Notes

- T26.1: `f71b106df0511709e9bbfe097fc55257f59b13a1` introduces the complete
  English sample with all accepted active defaults and commented optional blocks.
- T26.2: `030d2f2ed3d9231293a1a61e6a533944e14184f5` expands existing reference
  pages, preserves the README product/build style and updates reader-facing links.
- T26.3: `b965ba71ecc3223a70be78a5ae3412a53c33e0d2` adds schema/catalog checks,
  typed/default/map/example/parser-bound tests, drift probes and this report.
- The checker is a read-only test of the current model declarations and public
  sample. Rust remains the owner of type admission, resolution and compiled
  defaults. No runtime schema API, dependency or configuration behavior changes.
- Native tests strip only declared key overrides before comparing complete typed
  settings with `Settings::default()`. Binding maps are checked independently for
  every declared spelling and normalized effective target; inherited lookup alone
  cannot conceal a missing distributed entry.
- Optional examples are extracted from the distributed comments and prepared in
  owned fixtures. Parser-bound tests derive cases from the published numeric and
  enum rows; endpoint neighbors and nonfinite values must reject appropriately.
- T26.3 follow-up `6cfd35d479d313662cbbdbaa514c871d4073aba5` updates the existing
  isolated startup-consumer assertion to locate the Echo query above candidates
  and verify its cursor; startup/palette/binding/tree checks remain intact.
- Added or changed Rust functions use at most five parameters, cyclomatic
  complexity six and nesting two. Existing source-gate limits remain unchanged.

## Contracts Added or Changed

- CFG05-CFG06: exhaustive sample/reference, active palette defaults, complete
  context bindings, and explicit commented-example inheritance behavior.
- CFG20: published controls correspond only to supported owners and scopes.
- CFG22: deterministic schema/default/reference agreement is separate from
  rendered effects and actual terminal acceptance.
- UI01-UI04: documentation covers vertical command candidates, shared composition
  and bounded file completion/acceptance using the delivered owner contracts.

## Files of Interest

- `docs/reference/config.toml`: complete supported default configuration.
- `docs/reference/defaults.md`: 59-entry metadata catalog and source/UI semantics.
- `docs/reference/index.md`, `bindings.md`, `syntax.md`: reference discovery and contracts.
- `README.md`, `CHANGELOG.md`: concise reader-facing reference links.
- `scripts/check-config-reference.py`, `test_config_reference.py`: completeness and drift checks.
- `tests/configuration/reference.rs`, `reference_bounds.rs`: eight typed integration tests.
- `tests/configuration.rs`: register the focused test modules.
- `tests/configuration/startup.rs`: startup consumer query geometry and cursor.

## Validation

Successful development checks:

- Python TOML parsing of the sample passed with 25 active RGB roles and 109 bindings.
- A local documentation-link/anchor check passed for 48 references and 59 unique catalog entries.
- `cargo test --test configuration reference`: eight native reference tests passed, zero skips.
- `cargo clippy --test configuration -- -D warnings`: passed.
- `cargo test --test configuration startup::discovered_consumers`: isolated startup consumers and query cursor passed.
- `cargo run --quiet --example source-check -- --source tests/configuration/startup.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/configuration/reference.rs --source tests/configuration/reference_bounds.rs --root tests/configuration.rs`: passed; changed functions have nesting at most two.
- `python3 scripts/check-config-reference.py`: complete schema/sample/catalog/maps/enums/examples and documentation links agreed.
- `python3 scripts/test_config_reference.py`: all seven named drift subcases passed.
- Python AST parsing of both reference scripts passed.

Final readiness uses controller evidence for the clean PR head after the
PR-reference follow-up commit, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-source.py`
- `python3 scripts/test_config_reference.py`
- `git diff --check`

## Risks and Follow-ups

Schema/default consistency does not establish terminal rendering, physical input
reports or combined UI runtime acceptance. Those retain separate native/terminal
evidence and the approved subsequent interaction journeys. The current checker
fails on unrecognized model declaration shapes rather than silently omitting them.
