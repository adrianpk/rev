# Slice 1: Native Engine and PTY Ownership

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-01-engine`
PR: `#248`

## Purpose

Own bounded interactive PTY sessions through the pinned native Rust engine,
before exposing editor commands or presentation attachments.

## Delivered Behavior

Explicit creation owns one structured executable, PTY, emulator and stable
session identity. Real shell output becomes native cells, styles, cursor and
alternate-screen state. Accepted input remains ordered; checked resize retains
the child identity. Normal exit retains its actual status and final screen
until explicit closure releases capacity.

Session, cell, extra and input admission reject invalid candidates without
replacing existing owners. Nonblocking output uses kernel backpressure;
round-robin service handles at most 64 KiB across sessions. Coalesced descriptor
readiness blocks while idle. Closing owns shell job-control groups and child
reaping with a 500 ms grace and 2 second final deadline. Cleanup failure retains
ownership and a diagnostic.

## Implementation Notes

The only new direct crate is `alacritty_terminal =0.26.0`, with default features
disabled. The existing rustix dependency gains `termios` for checked resize;
the upstream resize API exits the host on ioctl failure. The upstream EventLoop
and its unbounded queues are not used. The lockfile adds 27 packages across
all targets; nine additions participate in the Linux dependency graph:
alacritty_terminal, arrayvec, base64, cursor-icon, home, polling, rustix-openpty,
signal-hook 0.4.6 and vte. No desktop/window/font backend is added.

Retained incident 1 reproduced a dependency SIGCHLD pipe-sender leak after a
failed spawn: an executable script with an absent interpreter increased open
descriptors from four to five. The operator authorized a minimal same-version
source override. Its sole runtime change unregisters `sig_id` before returning
the original spawn error. All 23 runtime files were compared with the
SHA-256-verified published archive; the version and Apache-2.0 license are
preserved. A clean extraction, manifest packaging adjustment and patch reproduce
the checked-in override byte for byte. Registry caches are unchanged. See
`vendor/README.md` and `vendor/terminal-spawn-cleanup.patch`.

At 80 columns, 24 rows and 2000 history rows, cell admission conservatively
reserves 323680 cells, or 7768320 bytes with the measured 24-byte Cell. This
includes both screens and up to 999 cached rows per grid. Resize keeps a
high-water reservation for retained allocation. The limits remain 1048576 cells
per session and 2097152 across the instance. Separately, the default input bound
is 262144 bytes, reads use a 4096-byte scratch buffer without an output queue,
control strings stop at 4096 bytes and combining scalars stop at 64 per cell.
Retained extras have an 8 MiB per-session budget. The engine also reserves its
fixed 2 MiB synchronization parser; synchronization is flushed at each parsing
boundary. Child clipboard and unsupported host effects remain inert.

The baseline release build at `507862798782d15235f3758868ab33cd3611c57e`
produced a 21576728-byte executable in 5 minutes 7 seconds. The candidate at
`4fe473a5827648efffbdc582e74f6c125a3030aa` produced 21698848 bytes in
4 minutes 2 seconds: an increase of 122120 bytes. The adapter is not yet wired
into editor command paths; the integrated binary delta is measured again in
the acceptance slice. The real-shell readiness marker was
observed in 5 ms. The shell accumulated zero CPU ticks during a bounded 100 ms
idle wait with no readiness notification. These are local adapter observations;
integrated terminal resource measurements belong to the recorded acceptance slice.

## Contracts Added or Changed

`term::Sessions` owns stable identities, transactional resource admission,
ordered input, fair service, checked geometry and explicit closure independently
of presentation. `term::Session` exposes read-only emulator state, truthful
lifecycle and separate resource accounting. The existing host-terminal and
one-shot process owners are unchanged. No editor command stubs are exposed.

## Files of Interest

- `src/term.rs` and `src/term/`: native registry, session, emulator, resource,
  readiness and process owners.
- `tests/integrated_terminal.rs` and `tests/integrated_terminal/`: shared real
  PTY fixtures, failed-launch regression and resource/lifecycle assertions.
- `Cargo.toml`, `Cargo.lock` and `vendor/`: pinned engine and reproducible
  authorized source correction with license and provenance.

## Validation

Development evidence:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal -- --test-threads=1 --nocapture`
  passed eight native PTY tests in 0.49 seconds, with no ignored cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal resources::rejected_launches -- --exact --nocapture`
  passed after the source override. The original four-descriptor assertion and
  missing-interpreter case are retained.
- The existing source-quality checker passed all seven new runtime files:
  safe Rust, at most five parameters, nesting at most three and cyclomatic
  complexity at most ten.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked --release`
  passed for baseline and candidate; executable measurements are above.
- Source archive checksum, unchanged runtime files/license and the documented
  clean reconstruction recipe passed comparison.

Final acceptance uses the controller receipt for the exact clean PR HEAD and
this ordered required list. Its validation ID is retained outside this report
so that recording the receipt does not change the candidate:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Risks and Follow-ups

The same-version dependency override must be retained until an approved
upstream source resolves failed-spawn cleanup. Panel, buffer, input/rendering
and integrated acceptance work continue in their recorded later slices.
