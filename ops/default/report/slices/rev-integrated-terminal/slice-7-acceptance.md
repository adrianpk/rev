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

The T7.2 driver additionally covers native source exclusions, nested tool and
input cases, and one injected host write failure. This work is incomplete until
the full nested suite passes in both profiles.

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

The real PTY run reproduced a live session incorrectly failing when an owned
host wake interrupted the zero-time pidfd observation. The process owner now
defers that inconclusive observation to its next service turn. It performs no
retry loop, retains the child and changes no resource limit, dependency or gate.

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

T7.2 remains open. Focused receipt `CIF4Y234KOKP3DIRTDF5I4H2CB` at
`a2ac4b1748c0bfc6067aee538f17f0f95675dddd` fails the full nested suite while
waiting for the live-buffer closure prompt. Before failure, the non-modal tool
journey passed relocation, PID/shell-state retention, actual exit and host
restoration within 812850 captured bytes. The raw-input case passed its exact
literal/mouse/bracketed-paste bytes and a 350 ms idle observation at zero CPU
ticks and 17792 kB peak editor RSS. These partial observations do not establish
a passing nested suite or the modal cases.

Traditional Escape-prefixed Meta entry also missed prompt observations in prior
nested runs. The current CSI-u Alt-x probe retains the actual M-x prompt and
placement assertions; it does not establish equivalence of the two transports.
The live-buffer close path and these input observations require continued
diagnosis. The eight-second / 1 MiB case limits and exact assertions remain.
T7.3 integrated resource measurements and focused production source coverage
remain pending. No canonical Slice 7 PR has been opened.
