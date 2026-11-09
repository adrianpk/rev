# Slice 2: Plugin and SDK ownership

Status: reviewing
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

Session dispatch separates buffer queries, file lifecycle and registration.
Input, context mutation and active source work remain explicit admission gates
with each operation's original rejection. Captured native motions prepare their
view before the final live check; atomic edits still prepare all targets and the
complete result before publication. Keymap resolution keeps its contributions,
map and statuses together for selected and hidden buffers. Semantic capture,
context capture and completion are separate steps; recent history consumes the
completed call instead of cloning it again.

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
Supervisor retirement still precedes cancellation output or process cleanup.
Event source conversion, subscription eligibility, delivery and invalidation
remain bounded; event locks are released before returning results or signaling unsubscribe.

Native representation changes use checked conversions at validated source/store
lengths, TOML integer bounds, OS flags and positive spawned process identities.
Reference-counted sharing is explicit. Existing publication-fault fixtures keep
all stages and assertions; their substitution helpers follow the IO boundaries.

Private file/query/unbind inputs borrow their existing wire values. Disk tokens
retain the hexadecimal spelling of signed native timestamp seconds, including
pre-epoch values, and check the native nanosecond bound before narrowing. A local
compatibility probe passed 15 combinations of extreme/signed seconds and valid
nanoseconds against the prior token spelling.

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

The five frozen commands passed on the working candidate in the approved order:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope crates/rev-plugin-sdk --scope src/plugins.rs --scope src/plugins --scope src/session/plugins.rs --scope src/session/plugins`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

The complete frozen selection covers all 32 SDK/native/session-plugin sources
and both SDK crate roots from an inventory of 1,799 first-party Rust files and
73 roots. Formatting, globally blocking first-party compiler/all-group checks,
selected Clippy admission and the source bounds pass with zero blocking
diagnostics. The driver defers 1,522 emitted selected-profile diagnostics outside
these owners; this is scoped adoption, not a full workspace quality claim.

The SDK passed all 12 tests and its doctest. The plugin suite passed all 86 tests
in 82.80 seconds, retaining protocol, captured contexts, cancellation/restart,
causal events, keymap precedence, semantic recording, atomic publication, state
and owned process/PTY resource assertions. Documentation passed 106 reachable
pages, 743 local links/anchors, 684 commands and 299 configuration entries.
The three state publication-fault, substitution and version-exhaustion unit cases
also passed during the native owner refactor.

Exact final clean-HEAD validation through the controller's validate command is
required after the PR-reference commit and push, before the worker handoff.

## Risks and Follow-ups

Other owners' selected-profile and source cleanup remain in their approved
slices. Inherited workspace lints, staged-index enforcement, hook activation and
the full integrated gate remain later delivery work. A passing linter does not
prove domain ownership or absence of bugs. The existing vendored dependency
warning remains outside first-party ownership. Public protocol/state versions,
resource limits and regression assertions are retained; no dependency is added.
