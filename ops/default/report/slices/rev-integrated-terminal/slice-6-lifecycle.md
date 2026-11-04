# Slice 6: Session Lifecycle and Failure Containment

Status: drafting
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-06-lifecycle`
PR: pending

## Purpose

Preserve owned terminal sessions across presentation changes and complete
confirmed shutdown without losing source buffers or unrelated work.

## Delivered Behavior

T6.1 keeps final output and actual exit status inspectable after the last view
is detached. Opening after default exit creates a new default while retaining
the old terminal buffer and its capacity reservation until explicit closure.
Native live/copy headers report readable actual status outside child output.
Completed buffer closure releases its copy projections immediately.

## Implementation Notes

The session owner exposes whether a process or cleanup receipt remains owned.
Presentation and buffer identity remain separate from process lifetime. Status
uses actual exit-code/signal observations, not text inserted into the grid.

## Contracts Added or Changed

Existing session, cell, queue, copy and cleanup bounds remain in force.
Running commands and loops have no runtime deadline. Cleanup retains the
500 ms grace and 2 second deadline, with discovery/signaling/reaping on workers.

## Files of Interest

- `src/term/session.rs`: process ownership and actual status formatting.
- `src/session/terminals/render.rs`: live and frozen status presentation.
- `src/session/terminals.rs`: immediate release of closed copy attachments.
- `tests/integrated_terminal/lifecycle.rs`: native real-PTY lifecycle fixtures.

## Validation

Controller receipt `4YZKUDYB6PWBIR4MIKAPNVDJTR` passed at clean T6.1 head
`ea41ea80cf393434cf64b00233317eea91d04335` with this exact ordered list:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/term/session.rs --file src/session/terminals.rs --file src/session/terminals/render.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::lifecycle::retained_exit -- --exact --test-threads=1`
- `git diff --check`

All three source files passed the repository's safe-Rust, parameter, nesting and
complexity checks. The real-PTY scenario passed both profiles, actual status and
final-output retention, default replacement, retained capacity, explicit closure
and renewed capacity admission.
Required Slice 6 validation follows completion of T6.2 and T6.3.

## Risks and Follow-ups

T6.2 editor-wide confirmed asynchronous shutdown and failure handling are pending.
T6.3 bounded admission/flood/close/error regressions are pending.
