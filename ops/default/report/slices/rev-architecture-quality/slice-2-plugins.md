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

Native declaration validation and editor decoding separate identity, process,
command, binding, file and state admission. The handshake keeps an explicit map
of implemented native methods, independent of the SDK schema catalog. Transport
service separates queue/deadline work, readiness, completed delivery grants,
causal admission and command completion without changing their ordering.

State snapshots own the anchored directory descriptor, OS lock and prior bytes
through read reconciliation or prepared CAS publication. Staging, replacement
checks and post-commit durability evidence have separate owners while retaining
the existing schema, monotonic versions and uncertain-publication behavior.

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

The supervisor task owns its admitted generation and bounded endpoints before
process startup. Invocation correlation and permit admission retain native and
nested origin rules and roll back the first reservation if the second fails.
Supervisor retirement still precedes cancellation output or process cleanup. Event source
conversion, subscription eligibility, delivery and invalidation remain bounded;
event locks are released before returning results or signaling unsubscribe.

Native representation changes use checked conversions at validated source/store
lengths, TOML integer bounds, OS flags and positive spawned process identities.
Reference-counted sharing is explicit. Existing publication-fault fixtures keep
all stages and assertions; their substitution helpers follow the IO boundaries.

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
- `src/plugins/declarations.rs` and `protocol.rs`: passive declaration admission
  and the explicit native capability map.
- `src/plugins/lifecycle/`: editor decoding, transport, causal event delivery and
  process generation ownership.
- `src/plugins/state.rs` and `state/`: query, catalog, descriptor IO and prepared
  publication ownership with unchanged fault/substitution cases.

## Validation

The existing native plugin suite passed all 86 tests with:

`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`

This includes protocol, callbacks, captured contexts, cancellation/restart,
retirement, keymap precedence, atomic publication, state and owned process/PTY
resource assertions. The rerun after native decomposition completed in 91.01
seconds. `git diff --check` passed.

The complete Slice 2 quality selection remains in progress under T2.2. The
passing checks below do not replace its final ordered clean-HEAD validation.

The SDK-only owner is clean: all first-party workspace targets compile without baseline
warnings, and the selected/source policy covers all 11 SDK source files and both
of its actual crate roots with zero blocking diagnostics. These commands passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope crates/rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`

The SDK passed all 12 existing tests and its doctest.

The combined SDK and native-plugin selection now passes with zero blocking
diagnostics across all 24 selected files and both SDK crate roots:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope crates/rev-plugin-sdk --scope src/plugins.rs --scope src/plugins`

All first-party workspace targets compile with the complete baseline clean. The
driver defers 1,544 emitted selected-profile diagnostics outside this selection;
this does not establish complete Slice 2 or workspace adoption.

A fresh source-only scan of the complete frozen Slice 2 selection reports 14
remaining diagnostics, all in session-plugin owners. The complete Slice 2
selected Clippy profile and ordered final checks remain pending.

The state publication fault, substitution and version-exhaustion unit cases also
passed (three tests), and the documentation catalog/link check passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib plugins::state::tests -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`

## Risks and Follow-ups

Session-plugin ownership/conversion cleanup and decomposition remain required
under T2.2. T2.3 and the final exact ordered five-command Looper validation remain
pending. No canonical PR is ready yet.
