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

## Contracts Added or Changed

No production resource limit, dependency or gate changes.

## Files of Interest

- `examples/integrated-terminal-acceptance.rs`: acceptance entry point.
- `examples/integrated_terminal_acceptance/`: native journey and shared setup.
- `scripts/check-integrated-terminal.sh`: release build and isolated execution.

## Validation

T7.1 focused validation is pending. The final candidate uses the six exact
ordered commands from the approved plan and frozen controller configuration.

## Risks and Follow-ups

Nested PTY journeys, source exclusions and resource measurements remain under
T7.2 and T7.3.
