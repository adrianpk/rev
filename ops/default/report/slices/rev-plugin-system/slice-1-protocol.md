# Slice 1: Protocol and Rust SDK

Status: delivered
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-01-protocol`
PR: `#1`
Integrated: `924db7e317729b8fde1f8fd72e3038b1bfae10c6`

## Purpose

Deliver T1.1-T1.3: publish the experimental rev.plugins/0.1 wire contract and
an independent Rust client SDK before executable host capabilities are admitted.

## Delivered Behavior

- The rev-plugin-sdk 0.1.0 workspace member exposes strict JSON-RPC 2.0 envelopes,
  all 25 request/result schemas and revision-qualified buffer notifications.
- Package and command identities enforce the approved namespace grammar. Source
  ranges count Unicode scalars; revisions and generations remain decimal strings.
  Handles retain their session and state creation requires an explicit null version.
- Incremental NDJSON decoding accepts fragmented UTF-8 and escaped newlines,
  rejects malformed/batch/oversized traffic and detects truncated EOF. Output
  serialization, queued bytes and pending requests have finite ceilings.
- The client negotiates the host offer, correlates concurrent typed results,
  exposes caller-driven deadlines/cancellation and prevents retired callbacks
  from becoming current when a peer reuses a request ID.
- The native protocol boundary validates the exact initialization agreement and
  rejects editor requests until their capabilities are implemented. Its offer
  contains no host capabilities.

## Implementation Notes

Implementation: `bdaef1028c743fab1b2180f9d2a0bed9216df80d` implements T1.1-T1.3
and introduces this report. Canonical
[PR #1](https://forge.adrianpk.com/rev/rev/pulls/1) targets dev.

The SDK depends only on the existing pinned serde and serde_json versions. Its
package version and GPL-3.0-only license are independent of the unchanged editor
version 0.6.0. Cargo.lock adds the local member without upgrading dependencies.

The client performs no I/O, spawns no threads and owns no editor resources.
Its caller supplies pipe readiness, partial-write progress and a monotonic clock.
Connection-local callback tokens bind the wire ID to its admission serial,
session and generation. Cancellation retires authority before fallible output.
A malformed frame or ambiguous duplicate active request retires the connection.

The public method catalog describes schemas rather than implemented capabilities.
Native buffer/history, process lifecycle, command/keymap admission and persistence
remain owned by the later approved slices. No release-facing behavior or
changelog entry is claimed by this internal protocol foundation.

## Contracts Added or Changed

- The initialization exchange agrees to package/version, session/generation,
  exact API, implemented capabilities, Unicode scalar units and the finite profile.
- The public method schemas cover buffers, edits, display, explicit save/close,
  commands and callbacks, keymaps, events, feedback and package-owned CAS state.
- Typed errors preserve stable data.kind and bounded request/handle context.
  Mutation schemas require explicit source stamps, disk or state observations.
- Bounded collections and text types reject excess data. Whole-batch admission
  checks duplicate targets, replacement order, aggregate count/bytes and causal depth.
  These checks do not replace native transaction/publication acceptance.

## Files of Interest

- `crates/rev-plugin-sdk/src/methods.rs`: typed method catalog and admission.
- `crates/rev-plugin-sdk/src/identity.rs`, `limits.rs`, `rpc.rs`:
  qualified identities, finite profile and envelopes/errors.
- `crates/rev-plugin-sdk/src/transport.rs`, `client.rs`:
  framing, byte queues, handshake and correlation.
- `src/plugins/protocol.rs`: native protocol-only initialization boundary.
- `crates/rev-plugin-sdk/tests/protocol.rs` and its support modules:
  literal schemas and malformed, Unicode, size, correlation and cancellation cases.
- `tests/plugins/protocol.rs`: native agreement, OS byte-stream exchange and
  unavailable-capability fixtures.
- `Cargo.toml`, `Cargo.lock`, `src/lib.rs`: workspace and native module integration.

## Validation

All seven ordered implementation checks passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The SDK selection contains 12 tests, including literal input/result fixtures for
all 25 methods. The native protocol selection contains three tests and uses an
owned UnixStream pair with finite read/write deadlines. These are focused slice
checks. The command regression selection passed 52 tests. Documentation checks
passed 680 commands in each combined inventory, 295 configuration entries,
99 reachable pages and 681 local links/anchors. The controller validator owns
the exact clean-HEAD evidence for the PR-reference commit before handoff.
All seven frozen commands passed through that validator on clean pushed HEAD
`924db7e317729b8fde1f8fd72e3038b1bfae10c6`; the canonical PR contract was
verified before controller integration. Forgejo and Acta confirm PR #1 merged
through Rebase + Fast-forward into dev at that same commit.

## Risks and Follow-ups

The API is experimental. Executable plugin activation, native editor operations,
lifecycle/resource measurements and independent-client acceptance belong to the
remaining approved slices. No native capability is advertised by this slice.
