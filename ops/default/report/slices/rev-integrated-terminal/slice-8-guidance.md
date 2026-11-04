# Slice 8: User Guidance and Joint Integration

Status: delivered
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `fix/rev-integrated-terminal-validation-1`
PR: [#256](https://forge.adrianpk.com/adrianpk/rev/pulls/256)
Original guidance PR: [#255](https://forge.adrianpk.com/adrianpk/rev/pulls/255)

## Purpose

Make the integrated terminal discoverable through the User Guide, task guidance
and exact reference, then reconcile the accepted model and evidence.

## Delivered Behavior

The terminal chapter follows a real shell/lazygit session through panel and
main-buffer placement, multiple sessions, returning to source, copy and closure.
The task guide and dedicated reference describe all 37 commands, seven startup
settings, input/copy ownership, limits and pending/failure lifecycle behavior.

## Implementation Notes

Markdown remains the source for the Spore projection. Small text diagrams show
placement without adding an image or runtime dependency. Chapter 16 is linked
in the everyday editor path; the independent Org path remains separately
validated. Catalog checks compare all 37 terminal commands and seven settings
with the live command/schema/default owners, rather than a second manual list.

## Contracts Added or Changed

The interactive section in the command catalog links the dedicated reference;
its stable anchor and complete command inventory remain available. Accepted
baseline/model clauses now admit typed terminal resources without transferring
source/file/history authority. TRM01-TRM11 link their implementation reports;
combined-gate acceptance remains pending. Core contracts state their behavior
directly without relying on another product's capabilities.

## Files of Interest

- `docs/tutorials/user-guide/16-integrated-terminal.md`
- `docs/how-to/use-integrated-terminal.md`
- `docs/reference/integrated-terminal.md`
- `examples/documentation-check.rs` and `documentation_check/catalogs.rs`
- `publication/userguide/index.sptree`
- Terminal/editor specification and model companions under `ops/default/spec/`

## Validation

Final canonical receipt `IT5NYQHLV2YQN43Q46TFNMIGTD` passed all six configured
commands at `56862f0225b718b82c57d1c0ccd3c24fa97e1ab9`:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal --test commands --test configuration -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

All 361 selected integration tests passed. Earlier focused receipt
`Y2WNEWSV2PFZYVWAYCQOPNJ7A7` passed documentation, local guide generation and
whitespace at `10dbde18199a61eaf768bb1d81761632580d1267`.

The checker validated 76 reachable pages, 484 local links/anchors, independent
editor/Org paths, 474 commands and 247 configuration entries. The dedicated
terminal inventory covers all 37 live names and seven startup settings. Spore
strict link checks and rendering passed; the generated index and prior chapter
link chapter 16, whose HTML contains the actual move command examples.

The first focused receipt `RTY6FC3J2FVDWH4U7F2UM5KJ74` rejected a footer-only
Next link. The chapter now uses the established Next: line; no check was weakened.
Accepted TRM01-TRM11 requirement/scenario text is unchanged from the integrated
baseline. Canonical PR #255 merged into dev unchanged at
`56862f0225b718b82c57d1c0ccd3c24fa97e1ab9`; Slice 8 of 8 completed.
The controller checkpoint records the merge activity. Guide generation was local.

## Risks and Follow-ups

The unchanged combined gate remains controller-owned on integrated dev.
Slice-local documentation and catalog checks do not close terminal delivery.

## Gate Repair 1 Diagnosis

The controller's unchanged combined gate failed at
`c22102280c1cb514fc1eca1e663c126e9a12e063` in receipt
`DIMJP2WKAA5IZ4ZYCRY75TCV4W`. Native regression batches and both native tool
profiles passed. The non-modal real-PTY lazygit journey then timed out waiting
for the second M-x prompt, after moving the live tool to the main buffer.

Focused receipt `TBCKOI6QAOOQ2H3X5H5U2O3XZL` passed the unchanged PTY journeys
and whitespace on that same head, confirming an intermittent boundary failure.
A controlled real-PTY probe against the pinned Crossterm 0.29.0 reproduced the
underlying input loss: queue an owned SIGWINCH before the eight-byte CSI-u Alt-x
event, then release the event reader. The first poll/read returns Resize(80, 24);
the second 100 ms poll returns false while all eight input bytes remain unread.
The probe restores raw mode and closes its owned PTY and control socket.

Crossterm's Mio adapter returns immediately from the signal readiness token,
discarding the remaining tokens from that poll. Mio uses edge readiness, so the
unread terminal input receives no fresh edge until another input write. This
matches the missing prompt after terminal output wakes the native event owner.

Repair identity remains branch `fix/rev-integrated-terminal-validation-1`, title
`fix(terminal): repair approved integrated terminal acceptance`, and this report.
The owner approved retained incident 3 and the controller granted the exact
published archive files, readiness patch and vendor README. Crossterm remains
0.29.0 under its original MIT license; all dependency versions and graph edges
are unchanged. The checked archive and replayable patch reproduce all 77 files,
with only `src/event/source/unix/mio.rs` differing from the published bytes.

The adapter now retains the existing three-token readiness batch across native
event returns. TTY readiness persists until its queued bytes are exhausted; a
byte-count ioctl avoids blocking on an empty descriptor before the remaining
tokens are handled. Signal and event-stream wake tokens advance before return.
One index adds no queue, timer, input rewrite or retry. Registry caches,
eight-second/1-MiB PTY limits, live-command lifetime and cleanup bounds are intact.

New regression `readiness::signal_keys` uses fresh owned event-reader processes
and real controlling PTYs. It queues signal/key readiness in both orders and a
1,300-byte burst, then requires every key and resize without another input write.
The original source fails after Resize with the Alt-x bytes still queued; the
corrected source passes all three cases. This also rejected an intermediate
patch that could block on the retained empty TTY. Each probe has a three-second
deadline and 1 MiB output cap and reaps its own process.

Focused reproduction/regression command:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal readiness:: -- --test-threads=1`

Canonical repair PR: [#256](https://forge.adrianpk.com/adrianpk/rev/pulls/256).
Focused receipt `O7FTJPLHVDD6O4KVWBHSORX3IN` passed format, the new regression
and whitespace on `27fff3c81e60a46b78def641ad47b679af88097c`.

Final canonical repair receipt `BCMQANMXVO5IJO5KC7TXAZVSEM` passed all five
configured commands at `6390608f045e36d24e3510763054777067407a6f` after the
PR-reference follow-up:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal --test configuration --test commands --test windows --test panels --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

All 461 selected tests passed, including the three controlled readiness cases.
Documentation retained 474 commands, 247 configuration entries, 76 reachable
pages and 484 local links/anchors.

Focused receipt `EZFW2VPLAEB6I5MVBWYNLAZX6Y` passed on the same exact head:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh --pty`
- `git diff --check`

Both profiles passed actual lazygit relocation/PID/cwd/exit, exact child input,
protected source, normal/error host restoration and owned cleanup. Idle CPU
remained zero ticks in both profiles and every case stayed below 1 MiB under
the unchanged eight-second deadline.

Canonical PR #256 merged into dev unchanged at
`6390608f045e36d24e3510763054777067407a6f`; the checkpoint records merge activity.
All eight slices and Gate repair 1 are integrated. The unchanged combined gate
has not rerun after this repair and remains controller-owned.
