# Slice 3: Owned process lifecycle

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-03-lifecycle`
PR: `#3`

## Purpose

Deliver T3.1-T3.3: activate and supervise one local package process per independent
editor, with bounded bidirectional RPC and owned cancellation and cleanup.

## Delivered Behavior

- Passive discovery stays dormant. Explicit startup, declared native filetype
  and opened/changed/closed observations, command use and management commands
  share one lazy activation per package. Active processes persist across source
  closure and native context changes.
- Initialization validates the exact session, generation, API, units, finite
  profile and complete initial handler set. Only declared qualified commands
  acquire handlers. Missing handlers remain explicitly unavailable.
- Command callbacks capture their original source revision, pane, scalar point
  and selection. Distinct request IDs retain session and generation identity;
  later focus or source changes do not refresh the capture.
- Native editing remains available during activation and callback work. C-g
  honors transient input ownership before canceling pending plugin work. Deferred
  completion feedback cannot replace a native question.
- Deactivate, restart, shutdown, EOF, crash, protocol failure and deadlines retire
  authority before cleanup. Restarts wait for a settled old group and allocate a
  new generation. Ordinary buffers and unsaved text retain native ownership.

## Implementation Notes

Implementation: `71857f4a05786909e1d7beb58d8845b2f0cccc05` implements T3.1-T3.3 and introduces
this report. Canonical [PR #3](https://forge.adrianpk.com/rev/rev/pulls/3) targets dev.

A small native Host owns admitted generations and finite request/output budgets.
Each active package has one supervisor thread. It alone spawns the literal local
program, negotiates initialization, services nonblocking pipes and process-exit
readiness, and signals and reaps its owned group. Input submits requests through
a bounded nonblocking channel. No child I/O, process observation, signal, wait or
join runs on input or rendering.

The supervisor uses the existing native wake mechanism. Idle pipes block on
readiness with no timer deadline. Native service shares a 32-item/2-ms budget
between lifecycle observations and activation work. Failed generations do not
restart from ordinary event observations; explicit command use or management can
request a new attempt.

Request permits cover calls waiting for initialization and running callbacks.
Owned frame accounting releases both partial writes and discarded output. Final
state, cleanup evidence and its notice publish together, after pending authority
retires. Finished supervisors release their control descriptors; finite inert
inspection history retains diagnostics without retaining live pipes.

The unreaped direct leader pins group identity through TERM, the two-second
grace, KILL and the two-second force/reap observation. A scan across the grace
boundary cannot suppress escalation. Cleanup settlement is a typed observation,
independent of its diagnostic wording. The receipt reports the owned group's
settlement and separately states that escaped descendants are unobservable.
Uncertain owned cleanup prevents another generation and leaves child reaping
with its supervisor. The noninteractive barrier reports uncertainty after five
seconds rather than blocking native input on an unobservable child.

Transport byte ceilings permit at most 8 x 4 MiB of decoder input, 16 MiB of
aggregate outbound data and 8 x 128 KiB of active stderr rings. JSON value
overhead, finite declaration metadata and external program memory are additional;
this is a resource sketch, not a measured RSS ceiling. Native activation retains
at most 4096 ordinary text-buffer observations and reports overflow explicitly.

## Contracts Added or Changed

- The implemented commands capability supports host-to-package command.invoke
  and command.cancel. Package-to-host editor methods still return correlated
  unsupported_method errors. No buffer, edit, display, binding, event-delivery,
  message or state capability is advertised by this increment.
- Native callbacks validate an empty argument object before activation. Required
  arguments, counted invocation and macro recording remain explicitly unavailable.
- Limits include 8 active/stopping generations, 16 pending requests per package,
  64 total, 4 MiB frames/package output, 16 MiB aggregate output, five-second
  initialization and 60-second command deadlines, and 128 KiB stderr rings.
- Library hosts can inspect ephemeral package status and call
  Session::shutdown_plugins outside input/rendering. run_terminal restores its
  host terminal before collecting cleanup receipts. Library Drop submits stop
  without joining supervisors on input.

## Files of Interest

- `src/plugins/lifecycle.rs`: native generation owner, budgets, nonblocking
  submission, finite inspection and noninteractive completion barrier.
- `src/plugins/lifecycle/worker.rs`, `process.rs`: bidirectional framing,
  handshake/correlation, readiness, deadlines and owned group cleanup.
- `src/plugins/protocol.rs`: complete initial handler admission against the
  passive declaration catalog; `src/window.rs` supplies native pane IDs in the
  public opaque-token grammar.
- `src/session/plugins.rs`, `src/session/plugins/activation.rs`: management,
  captured command calls, passive inventory and cooperative native triggers.
- `src/session/events.rs`, `contexts.rs`, `quit.rs`, `region.rs`, `startup.rs`:
  native wake, editor lifetime, source decisions and cancellation ownership.
- `tests/plugins/lifecycle.rs`, `peer.rs`: real Rust child/SDK fixtures and native
  Session integration; `declarations.rs` retains dormant discovery coverage under
  the newly executable startup policy.
- `docs/reference/plugins.md`, `docs/reference/index.md`, `CHANGELOG.md`:
  current runtime contracts, limits and observable user behavior.

## Validation

All seven frozen ordered implementation checks passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- lifecycle --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The lifecycle selection passed 11 tests, including both native profiles,
coalesced first use with frozen captures, every configured trigger, cancellation
and restart, native context lifetime and prompt ownership, crash/EOF/malformed
and incompatible peers, actual startup expiry, pending/process limits, bounded
stderr and isolated failed-exec descriptor baselines. The SDK passed 12 tests and
native command regressions passed 52 tests. Documentation validation passed 684
commands per combined inventory, 299 configuration entries, 100 reachable pages
and 684 local links/anchors.

Additional focused validation passed all 22 protocol/declaration/lifecycle tests:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- lifecycle declarations protocol --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

The real lifecycle selection takes about 15 seconds, including the actual
five-second startup deadline and owned-group grace/escalation. The required
native command selection took 242.20 seconds. These are observed test costs,
not idle/throughput measurements. All owned fixtures complete their cleanup
before their resources are discarded. The controller validator owns the exact
clean-HEAD evidence for the final PR-reference commit before handoff.

## Risks and Follow-ups

These are trusted local programs with ordinary account permissions, not a
sandbox. The owned-group receipt does not prove escaped descendant settlement.
No package-created buffer journey is claimed before the buffer API exists;
current cleanup fixtures preserve ordinary native buffers and user edits.
Native editor operations, binding installation, semantic prefix/macro calls,
event delivery and persistent package state remain with their recorded later
slices. The controller owns full-gate acceptance and integration. No performance
benchmark or delivery-set completion is claimed here.
