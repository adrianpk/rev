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

Implementation pending. Slice 8 is canonically integrated through PR #22.

## Implementation Notes

Measure selected findings before changing fixtures. Preserve case identities,
assertions, varied Unicode/numeric inputs, PTY behavior, deadlines and resource
limits. Extract only cohesive setup or checking concerns; keep branches visible.

## Contracts Added or Changed

No runtime, package API, protocol, policy or acceptance changes are planned.
The existing five frozen commands and final controller gate remain required.

## Files of Interest

- `tests/`: native integration cases and owned fixtures.
- `examples/`: developer and publication helpers.
- `crates/rev-plugin-sdk/tests/`: SDK acceptance fixtures.

## Validation

Pending. Frozen ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros --test commands --test plugins -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

Selected diagnostics and source findings remain to be measured and corrected.
The canonical PR, final clean-HEAD evidence and controller integration are pending.
