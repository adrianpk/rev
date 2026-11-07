# Slice 7: Combined native and terminal acceptance

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `test/rev-projects-workspaces-07-acceptance`
PR: pending

## Purpose

Deliver T7.1-T7.3 acceptance evidence for PW01-PW22 through the existing native
cases and representative Linux PTY journeys. Close verified Slice 6 PR #303 and
activate only the approved Slice 7 branch. The combined gate, integration and
successor startup remain controller-owned.

## Delivered Behavior

- A native acceptance executable and Bash entrypoint exercise isolated real
  catalogs, source files, configuration, rendering and executable input. Existing
  context cases retain the domain, format, concurrency, fault and lifecycle
  variants rather than repeating that matrix through terminal processes.
- One serial journey per editing profile launches without a target, then restarts
  with an explicit file and directory. It verifies native dashboard actions,
  fitting RGB artwork or NO_COLOR fallback, resize, decorated/minimal trees,
  temporary opening of a registered path and unchanged saved sessions.
- Real named commands promote a directory, restore Unicode source anchors,
  duplicate/cancel/undo, create an empty Workspace, restore independent sessions,
  add/remove roots and convert a Project atomically. Typed catalog identities,
  references and snapshots are checked against the real state file. All source
  bytes, including CRLF and combining characters, and configuration stay intact.
- Native probes admit 50 artifacts/1,000 saved references, a session near 2 MiB,
  and a catalog near 8 MiB containing 256 Projects, 128 Workspaces, a 64-root
  Workspace and 512-file sessions. Oversized session/catalog candidates and a
  513-file live checkpoint preserve complete prior state and live sources.
- Three canceled catalog reads and a canceled staged restoration preserve source
  stamps/generation/state. Two completed 512-file restore/checkpoint rounds
  record resident memory and response costs. Idle scheduling, descriptors,
  threads, TTY restoration and owned cleanup have explicit observations.

## Implementation Notes

T7.1-T7.3 are delivered in
[`953634c3`](https://forge.adrianpk.com/adrianpk/rev/commit/953634c3341543395ef51180796d71f0e7b79d08).
Slice 6 closure and Slice 7 activation are recorded in `7432bea53500`.

Cargo discovers the new example without manifest or dependency changes. The
script builds the existing binary and example once, creates one temporary root
and supplies isolated HOME/XDG directories. Native work has a 60-second outer
bound and ten-second individual preparation bounds. Each profile journey has a
30-second outer bound; its three serial editors retain the shared eight-second
and 1 MiB limits. The wrapper checks terminal attributes and native restoration
bytes, waits for owned children and removes only its temporary root.

The existing PTY helper gains a no-target launcher and matches the basename for
nested explicit targets. Existing callers keep the same launch path, stream,
resize, idle and shutdown handling. No second terminal parser, resource framework,
service or test dependency is introduced. All-target Clippy required one existing
Workspace fixture to pass its predicate directly; its assertions are unchanged.

The capacity fixture uses available roots and real small files for the restored
session. Other large saved references remain unavailable entries. Catalog listing
retains the selected source and does not load those files. Storage preparation,
publication and restart decoding use the real aggregate codec/store. Byte limits
include serialized paths; the session probe measures the table shape charged by
the codec. Resize and painting run while the existing owned IO/cooperative
restoration/capture paths progress.

## Contracts Added or Changed

- Acceptance entrypoints only: `examples/projects-workspaces-acceptance.rs` and
  `scripts/check-projects-workspaces.sh`.
- Preserve runtime commands, schemas, configuration, admission limits,
  dependencies and the supplied artwork.
- Native policy/storage coverage and bounded real journeys remain separate;
  existing service-authority cases stay native and require no external provider.

## Files of Interest

- `examples/projects-workspaces-acceptance.rs`: native/journey dispatch using
  existing helpers.
- `examples/projects_workspaces_acceptance/catalog.rs`: ordinary, per-session
  byte and aggregate/count boundary fixtures.
- `examples/projects_workspaces_acceptance/native.rs`: real storage, cooperative
  source restoration/capture, cancellation, overflow and Linux resource probes.
- `examples/projects_workspaces_acceptance/pty.rs`: serial profile journeys,
  independent snapshots, exact source/configuration preservation and restart.
- `examples/completion_acceptance/terminal.rs`: shared optional-target launch.
- `scripts/check-projects-workspaces.sh`: build, isolation, bounded execution,
  TTY checks and owned cleanup.

## Validation

All six required implementation commands passed:

1. `cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts -- --test-threads=1`
4. `bash -n scripts/check-projects-workspaces.sh`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
6. `git diff --check`

The context suite passed all 62 cases. Strict Clippy covered all targets. The
focused command `bash scripts/check-projects-workspaces.sh` passed the native
probes and both complete profile journeys. Documentation checks passed 618
commands, 289 configuration entries, 86 reachable pages and 588 local links and
anchors. The pre-existing vendored Crossterm unused-parentheses warning remains
outside this slice.

Measured implementation observations from that complete script run:

| Probe | Actual result |
| --- | --- |
| Ordinary catalog | 50 artifacts, 1,000 file references; 325,058 encoded bytes; encoded vector capacity 458,752 bytes; longest service pass 11,570 us; resize 53 us |
| Session byte boundary | 512 files; charged session 2,094,740 bytes against 2,097,152; complete catalog 2,095,089 bytes; encoded capacity 2,281,984 bytes; oversized candidate rejected |
| Aggregate/count boundary | 256 Projects, 128 Workspaces, 64 roots in one Workspace; 8,354,858 encoded bytes against 8,388,608; source vector capacity 11,927,552 bytes; renamed candidate 8,354,610 bytes |
| Storage resident memory | 70,488 KiB before aggregate fixture; 85,216 KiB with encoded input; 132,492 KiB with prepared state; 172,328 KiB after publication; storage phase peak 176,944 KiB |
| Canceled work | Three completed canceled reads retained 172,332 KiB each; staged restoration cancellation preserved source/state; all native idle checks measured zero CPU ticks over 350 ms with no pending timer |
| 512-file rounds | Restore 4,360,350 / 4,576,594 us; capture/publication 5,051,070 / 4,567,528 us; post-close RSS 280,376 / 301,628 KiB; longest service pass 60,272 us, resize 468 us |
| Ownership and process residency | Peak 10 descriptors / 3 threads; baseline and final 4 descriptors / 1 thread; whole native peak 355,872 KiB; resident 308,856 KiB after owners dropped |
| Non-modal PTY | 4,106 ms for three serial starts; output 545,586 + 6,686 + 7,784 bytes; zero idle CPU ticks; peak editor RSS 31,436 KiB |
| Modal PTY | 4,255 ms for three serial starts; output 285,714 + 4,593 + 4,837 bytes; zero idle CPU ticks; peak editor RSS 32,568 KiB |

The native measurements include decoded catalogs, prepared expected/candidate
copies, encoded vectors, live buffers and allocator residency. RSS includes
allocator-retained pages after logical owner cleanup; it is a process observation,
not a portable live-heap limit. The observed whole-process peak is about
347.5 MiB. Counts, bytes, exact source/state preservation and handle/thread
retirement are asserted; no new RSS or latency acceptance threshold is invented.
All fixture roots were removed after their checks finished.

Final clean-HEAD evidence is recorded after report/PR references are committed
and pushed. The frozen combined gate remains controller-owned.

## Risks and Follow-ups

Boundary catalogs have materially higher resident cost than ordinary editing;
the measurements above retain that cost and the post-drop residency. These are
local Linux observations rather than a cross-platform memory guarantee.
Slice 8 retains final guidance/specification reconciliation and gate preparation.
The full combined gate and delivery-set acceptance are still pending their
controller-owned execution.
