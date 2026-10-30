# Slice 2: Independent Org Documentation

Status: reviewing
Delivery set: rev-documentation
Plan: `ops/default/plan/rev-documentation.md`
Tracker: `ops/default/tracker/rev-documentation.md`
Branch: `docs/rev-documentation-02-org`
PR: `#169`

## Purpose

Give Org readers complete reference and task entry points while keeping everyday
editor commands, bindings and settings first in the core documentation.

## Delivered Behavior

- Org has its own command, binding and configuration catalogs, reference index
  and task index, connected to its independent learning path and explanation.
- Core catalogs start with ordinary editing. Globally available Org entry keys
  remain documented, and focus-specific defaults retain their actual owners.
- Existing feature pages and chapter URLs remain. Eleven extracted section
  anchors retain concise onward links, with direct links from affected pages.
- The two command inventories each cover all 304 compiled names exactly once.
  The combined configuration catalogs cover all 178 accepted schema entries.
- The complete configuration sample and packaged defaults remain unchanged.

## Implementation Notes

T2.1 is `8a8dfd94b1fe6c2b740dc9fb35f8870784dfd3bf`; T2.2 is
`9c7d8d0deb7f2d1496027d97772dfcacca0d4b04`.

Catalog extraction retains literal rows, default values and detailed feature
behavior. Shared key syntax, input conventions and startup loading stay in the
core references. Org context sections own agenda, calendar, source and draft
behavior; the capability map connects their entry commands to feature details.

The native checker uses the compiled registry and existing Rust grammar to
enumerate configuration model fields. It compares both catalog unions against
those owners and the packaged sample, rejecting missing or duplicate entries
and changed defaults. Regression cases mutate real catalog data in memory,
including an added schema field that defaulting could otherwise conceal.

Existing Python guards read the same two configuration catalogs together;
their assertions remain intact. They are historical compatibility checks, not
the Rust/Bash validation commands for this delivery.

## Contracts Added or Changed

Each command has one entry per combined inventory. Each accepted configuration
leaf or map has one catalog row. Context tests read their own feature page and
retain inherited/global target assertions. Integer/float endpoints, enum
admission and packaged sample checks still use the actual native parser.

The checker covers all 58 documentation pages, README, CHANGELOG, local targets,
heading anchors, reachability, both learning paths and eleven legacy anchors.

## Files of Interest

- `docs/reference/org/`: Org command, binding, configuration and feature entries.
- `docs/how-to/org/index.md`: focused Org task grouping.
- `docs/reference/commands.md`, `bindings.md`, `defaults.md`: core catalogs and
  retained destinations.
- `examples/documentation_check/catalogs.rs`: live registry/schema comparisons
  and drift regressions, called by the native documentation checker.
- `tests/commands/palette.rs`, `org_catalog.rs` and
  `tests/configuration/reference_bounds.rs`: complete split reference inputs.

## Validation

Implementation checks passed: 304 commands in each combined inventory, 178
configuration entries, 58 reachable pages and 320 local links/anchors. Seven
checker tests, ten native command-reference tests and eleven native
configuration-reference tests passed during implementation. The focused native
source-quality check passed for all changed Rust files.

The controller records these commands in order for the final clean HEAD before
handoff; PR-reference follow-up changes require a new receipt:

- `scripts/check-documentation.sh`
- `cargo test --example documentation-check`
- `cargo run --quiet --example source-check -- --root examples/documentation-check.rs --source examples/documentation_check/catalogs.rs --source tests/commands/palette.rs --source tests/commands/org_catalog.rs --source tests/configuration/reference_bounds.rs`
- `rustfmt --check --edition 2024 examples/documentation-check.rs tests/commands/palette.rs tests/commands/org_catalog.rs tests/configuration/reference_bounds.rs`
- `git diff --check`

The required Bash driver runs the documentation checker, native command
reference tests and native configuration reference tests in the approved order.
External HTTP destinations and whole-editor runtime journeys are outside these
documentation checks; executable examples are unchanged.

## Risks and Follow-ups

Slice 3 reconciles v0.1.0 documentation and the changelog after this slice
integrates. PR integration remains controller-owned.
