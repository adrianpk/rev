# Slice 5: Atomic edits and history

Status: delivered
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-05-transactions`
PR: `#5`
Integrated: `9df6c5beb7dd2df7eece84a4f90b1c87671cc724`
Controller evidence: all eight ordered commands passed on the exact clean integrated HEAD; canonical PR contract and merge verified.

## Purpose

Deliver T5.1-T5.3: publish checked scalar replacements across explicit native
buffer targets as one all-or-nothing batch, with ordinary local undo/redo.

## Delivered Behavior

- The edits capability admits edits.apply against captured session-qualified
  buffer stamps. Active focus never supplies or refreshes a target.
- The whole batch is refused for stale, foreign, closed, unsupported, read-only,
  invalid, overlapping or excessive targets, or retired request authority.
  Refusal preserves source text/revisions, history/redo, modified state, marks
  and native views.
- Native preparation reserves every participating history queue and completes
  all source candidates, shared points, marks, anchors, viewports, observations
  and response stamps before the first source is published.
- Final admission rechecks every native participant and complete view capture,
  then checks request authority. The following commit phase has no recoverable
  partial failure path. Native observers run after all sources and views commit.
- Each target receives one ordinary local undo unit containing its replacements.
  Undo/redo in one target leaves the other targets unchanged. Hidden and unsaved
  text remains editable without selecting its pane or writing a file.
- Successful publication invalidates stale native completion/resolve work.
  Refused publication preserves still-current completion and redo state. Native
  departure/context source decisions and active macro work refuse batches.

## Implementation Notes

Implementation: `0d8ec0cf5e3d2447c8e245cc3901599a3a12caa7` implements T5.1-T5.3 and introduces
this report. Canonical [PR #5](https://forge.adrianpk.com/rev/rev/pulls/5) targets dev.

The new native Atomic coordinator lives beside existing source-batch ownership.
It prepares candidates through Buffer::prepare, maps every native shared view,
reserves history and materializes source publication state before final checks.
It retains the caller's explicit identities throughout preparation. All work is
serialized by Session under the existing finite plugin service budget.

The buffer owner now separates checked publication preparation from infallible
state installation. The existing single-buffer path shares that same installer
and retains its typing/history/automatic-group behavior. An atomic reserved edit
is restricted to a standalone native edit; it cannot inherit a typing group or
an active automatic macro iteration. Its native history insertion consumes the
reserved queue slot rather than invoking another fallible publish operation.

Final admission rechecks stamps, read-only state, marks/selection, source anchors,
macro-line ownership and every shared point/viewport membership. Request authority
at the entrance to the infallible commit is the publication linearization point.
The coordinator publishes all sources and mapped views, then emits native
observations. A failed precondition or canceled admission emits no observation.
Cancellation after commit admission cannot reverse an accepted batch.

The supervisor still owns protocol decoding/encoding and transport. The native
adapter validates opaque handles, expected revisions and scalar ranges, and
prepares the complete response before committing. Origin package metadata cannot
claim another package's identity. The result echoes the operation ID and returns
new stamps in target order. It grants no cross-buffer undo or save authority.

## Contracts Added or Changed

- Initialization advertises commands, buffers, display and edits. Package-to-host
  commands, save/close, bindings, events, messages and state remain with later slices.
- A batch admits at most 16 buffers, 256 total replacements and 512 KiB of total
  replacement UTF-8 bytes. Ranges refer to one captured base revision per target;
  equal starts, overlaps, duplicate targets and empty target/replacement lists fail.
- Terminal/generated/transient buffers are not editable text resources. Native
  resource/read-only policies apply before source preparation.
- Each successful target has an independent ordinary undo record; publication
  breaks typing continuity for that target without recording generated user commands.
- The existing 16-per-package/64-per-editor RPC permits and generation/deadline
  retirement cover queued edit batches. No child I/O, wait or filesystem write
  is added to native publication.

## Files of Interest

- `src/buffer/edit/publication.rs`, `src/buffer/edit.rs`: shared native installer,
  reserved standalone edits, complete admission and deferred observations.
- `src/session/source_batch/atomic.rs`, `src/window/points.rs`: all-participant
  preparation, reservation, rechecks and infallible source/view publication.
- `src/session/plugins/transactions.rs`: public handle/range/revision adapter,
  reply preparation, native completion invalidation and ownership guards.
- `src/plugins/protocol.rs`, `src/plugins/lifecycle/editor.rs`: implemented
  capability/method admission and typed supervisor/native exchange.
- `tests/plugins/transactions.rs`, `src/session/source_batch/atomic/tests.rs`:
  real child RPC journeys and native post-preparation failure/cancellation tests.
- `docs/reference/plugins.md`, `CHANGELOG.md`: current transaction contracts and
  usable experimental behavior.

## Validation

All eight frozen ordered commands passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- transactions --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The transaction selection passed eight real child/Session tests in 3.80 seconds
after compilation. It covers both profiles, hidden targets, Unicode replacements,
shared views/marks, the maximum 16-buffer/512-KiB batch, invalid/stale/read-only/
foreign/unsupported targets, retained redo and completion state, queued retirement,
native departure consent, per-buffer undo/redo and unchanged files on disk.

The SDK passed 12 tests, editing passed 114 tests, and the required completion
selection passed four tests. Documentation validation passed 684 commands per
combined inventory, 299 configuration entries, 100 reachable pages and 684 local
links/anchors. Editing took 62.20 seconds including compilation and 59.83 seconds
in its test binary; the completion selection took 5.52 seconds including compilation.

Fresh combined plugin regression passed all 39 protocol/declaration/lifecycle/
buffer/transaction tests in 32.96 seconds. Standalone fixture formatting passed.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- transactions buffers lifecycle declarations protocol --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

Focused validation also passed eight native tests, including the three atomic
coordinator tests and the retained buffer edit/anchor admission tests:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- buffer::edit session::source_batch::atomic --test-threads=1`

Those tests took 1.02 seconds after 57.79 seconds of compilation. They exercise
late cancellation with a retained redo branch, final participant rechecks and
complete-batch observer visibility. The final PR-reference commit receives exact
clean-HEAD controller evidence for the eight frozen commands before handoff.

## Risks and Follow-ups

The native source owner excludes recoverable partial publication; process abort
or allocation failure that terminates the editor is not a recoverable transaction
outcome. Private candidates and native retained history follow existing rope and
history policies; the wire replacement ceiling is not an RSS ceiling. Scheduling
limits apply between bounded operations, not as a timing guarantee within one
multi-buffer preparation. No throughput benchmark is claimed.

File lifecycle, noninteractive command admission, semantic macro calls, dynamic
bindings, revision event delivery and state remain with their approved later
slices. Full delivery-set validation and merge remain controller-owned.
