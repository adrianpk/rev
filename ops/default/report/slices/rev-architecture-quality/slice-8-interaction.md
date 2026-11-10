# Slice 8: Session interaction and completion owner

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-08-interaction`
PR: pending

## Purpose

Own completion admission and retirement in one session concern, and correct the
selected command, input, macro and session quality findings.

## Delivered Behavior

Implementation pending. Public completion observations, native editing/context
semantics, both interaction profiles and macro history must remain compatible.

## Implementation Notes

The approved scope covers F6 and the remaining selected native source and Clippy
findings. Completion retains its existing worker, resolver, source observations,
policy and bounded recent-buffer tracking. No generic job framework, dependency,
public API reshaping or resource-limit change is planned.

## Contracts Added or Changed

None.

## Files of Interest

- `src/session.rs` and `src/session/completion/`: completion ownership and
  invocation admission, cancellation, resolution and retirement.
- `src/command/`, `src/input/`, `src/macros/` and `src/session/`: selected native
  dispatch, input and macro ownership/complexity findings.

## Validation

Pending. Frozen ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/command.rs --scope src/command --scope src/input.rs --scope src/input --scope src/macros.rs --scope src/macros --scope src/session.rs --scope src/session`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test modal --test completion --test macros -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

Preserve native input/view context, stale completion rejection, transient
ownership, semantic macro replay/history and the retained macro body identity.
Implementation and exact clean-HEAD evidence remain pending.
