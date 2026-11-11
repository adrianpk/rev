# Slice 9: Fixtures and developer examples

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-09-fixtures`
PR: pending

## Purpose

Keep native test and developer example ownership within the existing selected
Clippy profile and source bounds, preserving behavioral and resource acceptance.

## Delivered Behavior

Shared fixture snapshots now clone their `Arc` or `Rc` owner explicitly. Values
that have no later use move into their existing recipient, and duplicate plugin
handle JSON borrows the retained handle. Concurrent publication fixtures start
both barrier participants eagerly before joining either thread.

## Implementation Notes

The baseline covers 704 of 1,836 first-party Rust files, both workspace packages
and 70 of 73 Cargo roots. It reports 201 blocking selected Clippy diagnostics,
four selected-profile diagnostics deferred outside this scope, and 123 native
source findings.

The first ownership cohort replaces 91 implicit reference-counted clone sites
with explicit shared ownership operations or the final move. It removes unused
fixture clones in manifests, JSON, context roots, staged contents and timezone
bytes. Git snapshot identity and freshness checks keep their existing inputs.

The command-usage and context publication races use eager array mapping instead
of collecting an intermediate vector of thread handles. Both threads still
start before either join, protecting the two-party barrier, lock contention,
publication-order assertions and existing two-second retry deadline.

The cohort preserves all existing test identities and assertion-site counts in
every changed Rust file. The complete selected baseline contains 2,889 test
identities and 19,672 assertion sites. No assertions, varied Unicode/numeric
inputs, PTY behavior, deadlines or resource limits are removed or relaxed.

## Contracts Added or Changed

No runtime, package API, protocol, policy or acceptance changes are planned.
The existing five frozen commands and final controller gate remain required.

## Files of Interest

- `tests/git_integration/`: explicit ownership of retained operation snapshots.
- `tests/command_usage/store.rs` and `tests/contexts/storage.rs`: eager concurrent
  publication fixtures with unchanged barrier and publication contracts.
- `tests/plugins/declarations.rs` and `tests/plugins/events.rs`: manifest and
  subscription fixture ownership.
- `tests/org_time/zone_bounds.rs`: final ownership of corrupted TZif bytes.
- `examples/git_integration_acceptance/`: snapshot and staged-content ownership.
- `examples/integrated_terminal_acceptance/failure.rs`: explicit shared failure
  injection state; host terminal cleanup behavior is unchanged.

## Validation

Final successful clean-HEAD evidence is pending. Frozen ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros --test commands --test plugins -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

Development checks for shared fixture ownership:

- The frozen scoped quality command remains failing overall: formatting and
  all-target compilation pass; blocking selected Clippy diagnostics decrease
  from 201 to 99, with four deferred outside the selected sources. No selected
  `clone_on_ref_ptr`, `redundant_clone` or `needless_collect` findings remain.
  All 123 native source findings still require correction; this is scoped
  progress, not successful full quality adoption.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test command_usage --test contexts --test plugins --test org_time --test git_integration -- concurrent_ domain::aliases_and_overlap domain::canonical_files declarations::admission_bounds events::subscriptions zone_bounds::invalid_tzif_fields zone_bounds::tzif_limits diffs::stale_generation index::groups::hunks history::ancestry_pages commits::modes::fixup_objects --test-threads=1`: passed all 13 selected cases across command usage, contexts, Git, Org time and plugins; other cases were filtered out.
- Static comparison preserves each changed file's existing test identities and
  assertion-site count. No lint suppression or policy change was introduced.
- `git diff --check`: passed.

## Risks and Follow-ups

The remaining 99 selected Clippy diagnostics and 123 native source findings
require correction within this slice. T9.1-T9.3 remain in progress.
The canonical PR, final clean-HEAD evidence and controller integration are pending.
