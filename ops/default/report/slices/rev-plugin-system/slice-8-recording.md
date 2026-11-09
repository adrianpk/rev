# Slice 8: Prefix arguments and macro calls

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-08-recording`
PR: pending

## Purpose

Forward declared integer prefixes once and record qualified plugin commands as
inert semantic calls through the existing native macro and library owners.

## Delivered Behavior

Native bindings and accepted command selections forward one captured prefix,
including zero and negative values, under the target's current declaration.
The host never repeats the handler according to the prefix. A command that
rejects prefixes refuses before activation.

Semantic recording replaces prefix/trigger and selector input with one qualified
plugin-call step containing validated literal arguments and the accepted prefix.
Generated RPC edits are not captured as further input. Playback captures a fresh
native source context and waits cooperatively for each call's specific receipt.
Missing packages, incompatible declarations and unsupported recording policies
stop before executing another step. Native MacroName syntax stays unchanged.

One pending automatic call retains native cancellation and resize servicing.
C-g revokes the call and descendant request authority. Source changes outside
that call remain observable to the runner's existing owner check. Only attached
causal edits and admitted native motions can advance its expected source.
Nested semantic calls preserve the operation and stop at depth eight, or an
earlier existing pending-request ceiling. Unrelated RPC edits remain refused.

Readers retain schema-1 libraries and admit the schema-2 plugin-call variant.
Explicit mixed-library saves upgrade to schema 2 while preserving unrelated
legacy records' identities, revisions and contents. Reading never migrates the
library. JSON nulls, booleans, nested values and Unicode remain inert data.
Unknown variants and malformed schemas reject the aggregate without execution.

## Implementation Notes

The native prefix dispatcher and macro recorder/runner remain the only input
owners. PluginCall is a distinct private macro step; it carries no executable,
handler, process generation or source handle. Replay revalidates current command
identity, arguments, prefix and recording policy before queuing work.

The existing supervisor call queue returns a one-slot receipt, independent of UI
notices. Its cancellation flag follows causally derived native requests and
nested callbacks. Completion revokes remaining automatic-operation authority.
The runner yields without a periodic wake while waiting; its existing active
60-second deadline and generated 64-MiB text budget still apply. Generated edit
batches keep their independent native per-buffer undo units.

The existing inert TOML codec adds plugin-call with command, arguments_json and
an optional integer prefix. JSON is stored as one bounded literal string to
preserve values which TOML cannot represent directly. Schema choice occurs at
explicit save/export; native records and native typed prefixes keep their shapes.
No dependency, public SDK revision or editor release identity changes.

## Contracts Added or Changed

- Declared integer prefixes are forwarded once through native qualified dispatch
  and causal plugin-to-plugin calls; native motion calls still reject prefixes.
- recording=semantic admits one qualified step; recording=reject stops capture
  before activation. Current declarations govern replay admission.
- Native macro_wait exposes Plugin while a semantic call owns automatic input.
- Only causal requests of the live automatic call may edit or invoke while
  recording/replay owns input. Existing generation, request, frame and depth
  limits remain in force; save/close and transient owners retain their guards.
- Schema 2 adds plugin-call without changing native MacroName or schema-1 record
  fields. Plugin steps in schema 1 and unknown inputs are refused.
- Old editors do not support schema 2. Loading old libraries never upgrades them.

## Files of Interest

- src/macros/plugin.rs, src/macros/library/codec.rs and store.rs: distinct inert
  calls, structural admission and explicit compatible publication.
- src/session/plugins/recording.rs and src/session/macros/: native semantic
  admission, capture, cooperative playback, cancellation and owner checks.
- src/plugins/lifecycle.rs and lifecycle/{worker,editor}.rs: bounded receipts,
  causal lineage and descendant request liveness.
- src/session/plugins/{invocation,transactions}.rs: current declaration checks,
  causal command/edit admission and generated text accounting.
- tests/plugins/recording.rs and the existing lifecycle peer: real native/child
  prefix, recording, compatibility, failure, cancellation and causal journeys.
- docs/reference/{plugins,macro-library}.md and CHANGELOG.md: current experimental
  behavior, saved format and compatibility boundary.

## Validation

All nine frozen ordered commands passed locally. Exact commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- recording --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

SDK validation passed 12 tests. The required native command selection passed
52 tests in 237.06 seconds, the complete macro target passed 110 tests in 75.41
seconds, and the modal selection passed 45 tests in 66.83 seconds. These timings
exclude compilation. Documentation checks passed 684 commands per combined
inventory, 299 configuration entries, 100 reachable pages and 685 local links.

Final focused validation passed 65 combined plugin tests in 50.63 seconds,
including all ten recording cases. It covers both profiles; native bindings and
command selection; one prefix per callback; generated-edit capture, replay and
undo/redo; legacy/mixed libraries; literal JSON; missing/changed declarations;
unknown input; native names; cancellation without a busy wake; source ownership;
unrelated edit refusal; cross-package generated edits; and causal depth eight.
Five native library tests passed, including schema-2 literal/structural admission
and existing native publication faults. Standalone peer formatting passed.
Exact focused commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol declarations lifecycle buffers transactions invocation keymaps recording --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- macros::library:: --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

Exact clean-HEAD controller validation of all nine frozen commands is required
on the final PR-reference commit before handoff. The integrated full gate belongs
to the controller.

## Risks and Follow-ups

Older editors cannot read schema 2. Package behavior can change between recording
and replay; the host revalidates its current declaration and stops on refusal.
The native macro and RPC deadlines are finite cooperative limits, not latency
guarantees. Events, persistent package state and later acceptance/guidance retain
their recorded slices. Main promotion, releases and publication remain separate.
