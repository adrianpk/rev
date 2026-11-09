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

SDK initialization, callback/notification admission and method-specific host
validation now have separate cohesive operations. Replacement ordering/byte
measurement and argument-type admission retain their original validation order
and results. Public error and deliberate invariant-panic contracts are documented
and checked with SDK-local Clippy documentation lints. Public constants keep
their types and values while the wire profile uses its native u32 constants.

## Implementation Notes

The internal host entry point accepts one domain invocation containing its
command, context and arguments. Admission transfers its context into pending
correlation rather than cloning a caller-owned local value. Encoding still uses
the existing typed public schema; pending context is intentionally retained for
causal callbacks, cancellation and captured-target validation. This is an
ownership correction, without a measured latency/allocation claim.

The SDK's private queue and test framing helper borrow messages. Existing
fragmentation, Unicode, bounds, correlation and callback cases keep all their
assertions; unnecessary fixture clones are removed. The actual integration-test
crate root now declares the existing unsafe prohibition explicitly.

## Contracts Added or Changed

The native host's internal invocation API groups the actual request payload.
Public SDK types, rev.plugins/0.1, wire fields, state formats, deadlines and
resource ceilings are unchanged. No new dependency is introduced.

## Files of Interest

- `src/plugins/lifecycle.rs`: queued payload and pending completion owners.
- `src/plugins/lifecycle/worker.rs`: transferred encoding and pending correlation.
- `src/session/plugins/invocation.rs` and `recording.rs`: explicit invocation input.
- `src/session/plugins/keymaps.rs`: observation transfer without vector duplication.
- `crates/rev-plugin-sdk/src/`: bounded client/schema decomposition and documented
  public admission contracts, with unchanged wire/API/state versions.
- `crates/rev-plugin-sdk/tests/`: original cases with borrowed framing support.

## Validation

The existing native plugin suite passed all 86 tests with:

`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`

This includes protocol, callbacks, captured contexts, cancellation/restart,
retirement, keymap precedence, atomic publication, state and owned process/PTY
resource assertions. Runtime was 97.31 seconds. `git diff --check` passed.

The initial full Slice 2 quality selection failed on the existing selected
profile and source diagnostics. T2.2 cleanup remains in progress; this is not a
passing slice-quality claim or final clean-HEAD validation receipt.

The SDK-only owner is clean: all workspace targets compile without baseline
warnings, and the selected/source policy covers all 11 SDK source files and both
of its actual crate roots with zero blocking diagnostics. These commands passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope crates/rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`

The SDK passed all 12 existing tests and its doctest. A fresh source-only scan
of the complete frozen Slice 2 selection still reports 41 diagnostics in native
plugin/session owners, with none in the SDK. The complete Slice 2 selected
Clippy profile and ordered final checks remain pending.

## Risks and Follow-ups

Native plugin/session ownership/conversion cleanup and decomposition remain
required under T2.2. T2.3 and the final exact ordered five-command Looper validation
remain pending. No canonical PR is ready yet.
