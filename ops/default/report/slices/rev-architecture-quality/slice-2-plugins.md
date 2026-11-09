# Slice 2: Plugin and SDK ownership

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-02-plugins`
PR: pending

## Purpose

Resolve the approved plugin payload/cache findings and clear the complete SDK,
native plugin and session-plugin ownership/source profile before integration.

## Delivered Behavior

Queued native invocations transfer their argument value into request encoding.
The pending map holds a separate completion owner with the exact captured
context, command/ID, cancellation flag, deadline and permits. It cannot retain
the invocation arguments. Completion/drop behavior still resolves caller and
receipt once, with the existing disconnection fallback.

Keymap observations take ownership of the already collected package states.
They no longer clone that vector and its package strings before checking for an
unchanged cache. Generation, enabled-state, buffer and serial observations remain.

## Implementation Notes

The internal host entry point accepts one domain invocation containing its
command, context and arguments. Admission transfers its context into pending
correlation rather than cloning a caller-owned local value. Encoding still uses
the existing typed public schema; pending context is intentionally retained for
causal callbacks, cancellation and captured-target validation. This is an
ownership correction, without a measured latency/allocation claim.

## Contracts Added or Changed

The native host's internal invocation API groups the actual request payload.
Public SDK types, rev.plugins/0.1, wire fields, state formats, deadlines and
resource ceilings are unchanged. No new dependency is introduced.

## Files of Interest

- `src/plugins/lifecycle.rs`: queued payload and pending completion owners.
- `src/plugins/lifecycle/worker.rs`: transferred encoding and pending correlation.
- `src/session/plugins/invocation.rs` and `recording.rs`: explicit invocation input.
- `src/session/plugins/keymaps.rs`: observation transfer without vector duplication.

## Validation

The existing native plugin suite passed all 86 tests with:

`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`

This includes protocol, callbacks, captured contexts, cancellation/restart,
retirement, keymap precedence, atomic publication, state and owned process/PTY
resource assertions. Runtime was 97.31 seconds. `git diff --check` passed.

The initial full Slice 2 quality selection failed on the existing selected
profile and source diagnostics. T2.2 cleanup remains in progress; this is not a
passing slice-quality claim or final clean-HEAD validation receipt.

## Risks and Follow-ups

Complete SDK error/panic documentation, selected ownership/conversion cleanup and
native decomposition remain required under T2.2. T2.3 and the final exact ordered
five-command Looper validation remain pending. No canonical PR is ready yet.
