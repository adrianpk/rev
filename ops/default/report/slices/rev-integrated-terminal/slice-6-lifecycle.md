# Slice 6: Session Lifecycle and Failure Containment

Status: reviewing
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

T6.2 resolves native source-save and bookmark decisions before one aggregate
terminal confirmation. Declining preserves live sessions and dirty sources.
Accepted closure submits existing bounded workers and keeps native input and
rendering responsive until ownership and terminal buffers are released.
C-g cancels exit while already approved cleanup continues.

T6.3 adds real-PTY flood/backpressure fairness, retained-extra admission failure
containment and incomplete-control resize/close scenarios. Existing descriptor,
queue, grid, session, relocation and owned-group regressions remain unchanged.

## Implementation Notes

The session owner exposes whether a process or cleanup receipt remains owned.
Presentation and buffer identity remain separate from process lifetime. Status
uses actual exit-code/signal observations, not text inserted into the grid.

Discard permission records exact source stamps. A later source publication,
including during terminal cleanup, cancels exit without marking sources clean.
Cleanup failure retains its session/buffer and cancels exit. Native Exit remains
observable after rendering consumes worker receipts. Live buffer-close decline
releases the prompt; repeated approved closure does not ask again.

## Contracts Added or Changed

Existing session, cell, queue, copy and cleanup bounds remain in force.
Running commands and loops have no runtime deadline. Cleanup retains the
500 ms grace and 2 second deadline, with discovery/signaling/reaping on workers.

## Files of Interest

- `src/term/session.rs`: process ownership and actual status formatting.
- `src/session/terminals/render.rs`: live and frozen status presentation.
- `src/session/terminals.rs`: immediate release of closed copy attachments.
- `src/session/terminals/shutdown.rs`: aggregate confirmation and asynchronous exit.
- `src/session/quit.rs`: source revision discard permissions.
- `src/session/events.rs`: native completion and readiness ownership.
- `tests/integrated_terminal/lifecycle.rs`: native real-PTY lifecycle fixtures.
- `tests/integrated_terminal/resources.rs`: fair service and bounded cleanup.

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

Controller receipt `PPD4GYIBWP77SFKH7JFLMK3L4K` passed at clean T6.2 head
`fbaff04c09f8d1f31a23faac5b2bdc6cbdcaed91`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example source-check -- --file src/session/terminals/shutdown.rs --file src/session/terminals/lifecycle.rs --file src/session/quit.rs --file src/session/bookmarks/quit.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::lifecycle -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::quit -- --test-threads=1`
- `git diff --check`

Four lifecycle scenarios pass in both profiles; one unresolved-save continuation
regression passes. The initial compile attempt found an ownership visibility
error; the terminal owner now exposes a pending-close query.

Controller receipt `OSU7364GZSOJZEX53J6V4ZMEZP` passed at clean T6.3 head
`1b0fb2569df031a3b1f6b47a20f530620da880b1`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal resources:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal panel::lifecycle::extras_failure -- --exact --test-threads=1`
- `git diff --check`

Eight resource scenarios and the extras-failure scenario pass. A 1 MiB output
flood completes while a blocked input queue stays bounded and another session
answers input. Actual hyperlink extras trigger the 8 MiB admission failure;
source revisions, peer process and failed-buffer capacity survive. An oversized
unterminated control survives 64 valid/rejected resize pairs and repeated close
without leaking descriptors. Existing rejected-launch FD and 500 ms grace /
2 second cleanup assertions pass unchanged.

The final PR head uses the exact six ordered commands from the approved plan
and frozen configuration. Its receipt is retained by the controller after the
PR-reference follow-up commit.

## Risks and Follow-ups

The cleanup deadline remains finite. A failed worker retains owned state for
retry and cancels editor exit; accepted cleanup can continue after C-g cancels
the exit request. Source revisions still require a fresh save/discard decision.
