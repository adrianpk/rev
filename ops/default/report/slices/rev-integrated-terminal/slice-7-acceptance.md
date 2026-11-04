# Slice 7: Native and PTY Acceptance

Status: drafting
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-07-acceptance`
PR: pending

## Purpose

Verify real shells and installed lazygit under the native session and nested
host PTY boundaries, with finite fixtures and measured resource ownership.

## Delivered Behavior

The initial driver creates isolated repositories and runs interactive Bash and
actual lazygit in both profiles. It checks cwd, alternate-screen help interaction
and stable process/buffer identity across panel and main-buffer relocation.

## Implementation Notes

The existing Rust acceptance layout and source checker are reused. Git and
lazygit use disposable stores, explicit configuration and no network operations.
Fixture deadlines constrain validation only; live commands have no runtime cap.
The host PTY uses compact 80x24 geometry with an admitted 15-row tool pane and four cohesive cases across both
profiles: tool relocation and input/copy each own a fresh eight-second, 1 MiB
stream. A fifth case checks one injected output fault; the aggregate driver
allows 45 seconds for these five bounded cases. Larger combined journeys
exceeded the unchanged stream cap. The shared fixture recognizes a hidden
cursor only after the owned geometry's final row is fully painted.

## Contracts Added or Changed

No production resource limit, dependency or gate changes.

## Files of Interest

- `examples/integrated-terminal-acceptance.rs`: acceptance entry point.
- `examples/integrated_terminal_acceptance/`: native journey and shared setup.
- `scripts/check-integrated-terminal.sh`: release build and isolated execution.

## Validation

T7.1 receipt `PGKUQXPYN44HYS2BXYNCZKH7VC` passed at `8f0a16c510ce54cd90197c639f72bcb2efd6bb30`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh --native`
- `git diff --check`

Both profiles passed. The cold release build took 7 minutes 40 seconds. The
initial source-quality failure was resolved by splitting the journey by owner;
a screen-restoration/reap race now waits for actual process disappearance
within two seconds, preserving the final no-process assertion.

 The final candidate uses the six exact
ordered commands from the approved plan and frozen controller configuration.

## Risks and Follow-ups

Nested PTY journeys, source exclusions and resource measurements remain under
T7.2 and T7.3.
