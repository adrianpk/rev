# Slice 9: Revision events and backpressure

Status: delivered
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-09-events`
PR: `#9`
Implementation: f738e46cb79aa7b3c854f74e24e242dc4cc00da7
Integrated: 95ef40a9fbda4e5e32bd582b9b14eeeeaad01670 through canonical PR #9

## Purpose

Deliver opt-in native revision/lifecycle notifications with bounded transport,
explicit invalidation and causal limits across competing package callbacks.

## Delivered Behavior

Packages subscribe to selected opened, changed and closed observations for all
ordinary text buffers or explicit session-qualified handles. Each notification
carries its subscription, generation, increasing sequence, exact source stamp
and causal origin. Creation, close, undo/redo and complete multi-buffer edits
use native publication hooks. Hidden sources retain their identities and focus
does not redirect callbacks. Active subscriptions follow context replacement.

Overflow discards incomplete undelivered deltas, revokes their callback authority
and sends explicit resnapshot invalidation, including to changed-only listeners.
Affected subscriptions pause. The package replaces its subscription first, then
rebuilds its source snapshot while reconciling notifications after the new baseline. An unavailable or undeliverable invalidation retires its
connection; ordinary unsaved text survives process cleanup.

Event edits and nested commands derive authority from delivered callbacks,
preserve one host-owned root across packages and advance depth. Depth eight
cannot trigger another edit or command. Unsubscribe, resnapshot and generation
retirement revoke queued descendants. Late replies receive correlated
cancellation without retaining native request permits until their deadline.

## Implementation Notes

A metadata-only native observer is separate from the existing LSP text observer.
It only queues stamps after publication. The existing native atomic coordinator
commits every source/history/view participant before publishing observations;
no callback can reenter it. Terminal and transient resources remain excluded.
Native creation, close and context replacement attach the same editor-owned sink.

The metadata queue holds 256 publications; fan-out is bounded by 128 subscriptions.
Notification permits include fan-out staging and metadata through encoded
frames until pipe writes complete: 256 per package and 1024 aggregate. Sixteen/package and 128 aggregate
slots are reserved for invalidations, leaving 240/package and 896 aggregate for
deltas. Existing 4-MiB/package and 16-MiB aggregate byte ceilings also apply.
Retention consists of finite metadata and existing bounded transport, never a
second source-text store. These are admission bounds, not measured RSS figures.

Supervisors own all encoding, pipe readiness and writes. Canceled whole frames
are dropped; a partially written frame finishes before invalidation or its finite
deadline retires the connection. Invalidation has a five-second delivery limit;
ordinary event delivery retains the 60-second limit. Idle supervisors block on
readiness. Native request and event phases alternate, packages are round-robin,
and lifecycle notices consume one item per package. All phases share the existing
32-item/two-millisecond cooperative service budget.

Each connection retains at most 256 delivered-event callback grants. The public
BufferEvent::invocation helper derives the callback token for commands.invoke;
generation and origin must match. Edits copy the event root/depth and identify
their calling package. Independent edits start a fresh host-owned root while
their result still echoes the caller's operation ID. Existing native capture,
consent, automatic-input and revision checks remain authoritative.

## Contracts Added or Changed

- The connected host advertises events and admits events.subscribe/unsubscribe;
  wire revision rev.plugins/0.1 and existing schemas remain unchanged.
- Subscribe admits up to 16 registrations/package and 128/editor, distinct kinds
  and up to 256 distinct live ordinary text handles. Empty handle lists select all.
- A subscription baseline excludes publications captured before its admission.
  There is no implicit snapshot or replay of earlier changes.
- Resnapshot has a null stamp and pauses its invalidated subscription. Recovery
  establishes a new subscription baseline before rebuilding native snapshots.
- Callback authority is generation-owned, bounded and revocable; expired host
  operation identities cannot be submitted as fresh independent edits.
- Native source/history/view, input, command, keymap and persistence owners stay
  canonical. No dependency, release identity or package-state storage is added.

## Files of Interest

- src/buffer/events.rs and native buffer publication/lifecycle hooks: metadata
  observations independent of LSP snapshots.
- src/plugins/lifecycle/events.rs: bounded subscriptions, fan-out, invalidation,
  callback grants and notification permits.
- src/plugins/lifecycle/{worker,editor}.rs: supervisor delivery, causal admission,
  transport deadlines and canceled-request responses.
- src/session/plugins.rs, plugins/transactions.rs and contexts.rs: fair native
  service, causal publication and retained editor ownership across contexts.
- crates/rev-plugin-sdk/src/methods.rs: public event callback identity helper.
- tests/plugins/events.rs and the real Rust lifecycle peer: controlled native
  publication, causal, slow-consumer, late-callback and reconnect journeys.
- docs/reference/plugins.md and CHANGELOG.md: current experimental event contract
  and reader-facing behavior.

## Validation

All eight frozen ordered commands passed locally. Exact commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- events --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

SDK validation passed 12 tests. The complete editing target passed 114 tests in
57.75 seconds, and the selected completion target passed four tests in 4.00
seconds, excluding compilation. The event target passed nine native/child cases.
Final focused regression passed all 74 combined plugin tests in 65.62 seconds,
including the event cases and every earlier delivered plugin contract.

Coverage includes both interaction profiles; hidden atomic participants;
creation/close and undo/redo stamps; monotonic sequences; subscription filtering,
limits and overlapping authority; forged/retired callbacks; direct cross-package
edit cycles; mixed event/command/edit depth; native queue overflow; one and eight
paused pipe consumers; explicit invalidation and re-subscription; five-second
undeliverable-invalidation retirement; late cancellation with released permits;
reconnect and settled idle without a busy wake. No mocked editor or child IO is
used. Final focused commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol declarations lifecycle buffers transactions invocation keymaps recording events --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

Formatting, Clippy and documentation checks also passed after final runtime
changes. Documentation checks passed 684 commands per combined inventory, 299
configuration entries, 100 reachable pages and 685 local links. All eight commands passed final clean-HEAD controller validation at
95ef40a9fbda4e5e32bd582b9b14eeeeaad01670 before handoff. Canonical PR #9
is merged into dev at the same commit. The integrated full gate belongs to the controller.

## Risks and Follow-ups

The limits are cooperative scheduling/admission ceilings, not per-host latency
guarantees. A paused subscription requires a new subscription and explicit
resnapshot; delivered metadata is not a retained source snapshot. Locally supplied trusted
programs still have normal account permissions. Package state and later
acceptance/measurement/guidance remain in their recorded slices. The controller
owns the integrated full gate, merges and successor startup.
