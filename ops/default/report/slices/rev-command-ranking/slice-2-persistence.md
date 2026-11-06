# Slice 2: Persistent usage

Status: delivered
Delivery set: rev-command-ranking
Plan: [Command Ranking Delivery](../../../plan/rev-command-ranking.md)
Tracker: [Command Ranking Tracker](../../../tracker/rev-command-ranking.md)
Branch: `feat/rev-command-ranking-02-persistence`
PR: `#292`

## Purpose

Deliver CMR04-CMR05 and the approved usage model while retaining the shared
selector ranking and input contracts from Slice 1.

## Delivered Behavior

- Normal startup loads the user-wide XDG `rev/command-history.toml` window.
  Successful publication retains ranking across files, directories, profiles
  and editor starts. Loading or opening a selector creates no state file.
- Version 1 contains only oldest-first canonical command names, with repeated
  invocations retained. Reads and serialized output are bounded to 64 KiB,
  names to 128 ASCII bytes, and the window and pending queue to 256 events each.
- Concurrent publishers acquire a nonblocking stable lock, read the latest valid
  window, append their pending events and atomically replace a synchronized
  user-only file. Busy publication defers without recurring idle polling.
- Confirmed batches are acknowledged once. Unresolved replacement completion
  suspends persistence rather than risking duplicate invocations. Other state
  errors preserve session ranking and existing invalid files.
- Pure Input and Session construction discovers no user files. Embeddings attach
  an explicit state owner; executable startup and ordinary shutdown supply their
  owners. State failures never add an editing or exit decision.
- Navigation history remains session-local and distinct-name bounded. Existing
  settings and dependencies are unchanged; one Unreleased outcome now describes
  persistence as well as session ordering.

## Implementation Notes

T2.1 and T2.2 are implemented in
`380c07e992ba94d631f7e82d246cb1fc9f71391a`. T2.3 is implemented in
`397907bd342d83d54d83c13e7e0cb067fa2cb757`: the real-boundary test target,
reference/defaults guidance, updated Unreleased entry and this report.
Canonical [PR #292](https://forge.adrianpk.com/adrianpk/rev/pulls/292) targets
`dev` and is merged there at `ce48b3a699970eec94375ba116f100c4eedb949c`.
The report's immutable introduction is
`397907bd342d83d54d83c13e7e0cb067fa2cb757`.
Slice 1's report and tracker are closed against merged PR #291 at
`17e16f07580cc32d743e6f8d34510dda14c85c2e`; the plan and tracker activate Slice 2.

`command::usage::Store` validates bounded TOML and uses the existing concrete
`file::Output` and owned sibling temporary-file boundary. Ordinary-file checks,
nonblocking/no-follow opens and pre-replacement comparison preserve rejected
or changed inputs. New state and lock files use mode 0600; missing parents use
0700. Post-replacement synchronization or observation failure suspends retries.

`command::usage::History` owns the session window, one bounded pending queue and
at most one IO worker. Transient acknowledgment identities distinguish an
in-flight batch from later invocations without persisting counters. Publication
does not hold the memory mutex during filesystem work. One bounded batch copy
and bounded read/serialization buffers serve each job; no job queue accumulates.
The native loop services publication after input batches and polls only while a
worker is active. Final flush waits at most 250 ms for owned work and one last
attempt, returning state errors separately from terminal/editing outcomes.

Startup combines configuration and usage diagnostics so a state-root failure
cannot hide the existing configuration source or unresolved XDG variable.

## Contracts Added or Changed

- `Store::open` is read-only; publication accepts a bounded caller-owned pending
  batch and returns the rebased window. The caller discards acknowledged events.
- `History::open` supplies an explicit isolated path; its default owner performs
  no discovery or IO. Only shared named acceptance records events.
- `Session::with_command_usage`, `publish_command_usage` and
  `flush_command_usage` expose explicit embedding ownership and bounded cleanup.
- The executable persists ranking at the approved XDG state path. No new
  configuration leaf, argument recall or command vocabulary is introduced.

## Files of Interest

- `src/command/usage/history.rs`, `src/command/usage/store.rs`,
  `src/command/usage/store/format.rs`: bounded ownership, schema and publication.
- `src/input/usage.rs`, `src/input/palette.rs`: explicit owner and captured usage.
- `src/session/command_usage.rs`, `src/session/startup.rs`,
  `src/session/events.rs`, `src/session.rs`: startup, idle work and final flush.
- `tests/command_usage.rs`, `tests/command_usage/`: real storage, session and
  isolated native startup/shutdown regressions.
- `docs/reference/commands.md`, `docs/reference/prompts.md`,
  `docs/reference/defaults.md`, `CHANGELOG.md`: state and user-facing boundaries.

## Validation

All five required commands passed locally, in order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test command_usage --test commands --test configuration -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 343 tests: command_usage 15, commands 168 and configuration 160.
The new target covers schema/name/event/byte bounds, ordinary-file admission,
permissions, passive startup, stale and concurrent publishers, lock contention,
real-file publication faults before and after replacement, bounded pending usage,
selector snapshots, both editing profiles and default-constructor isolation.
One reused bounded PTY journey verifies native shutdown, saved usage, unchanged
source bytes and terminal restoration. Its owner retains the existing eight-second
and 1 MiB limits; the suites clean their fixtures and start no persistent service.

The initial required test batch exposed startup diagnostic replacement: usage
root failure hid the existing configuration environment diagnostic. Production
startup now combines both diagnostics. The unchanged regression and the extended
dual-diagnostic startup check passed in focused runs:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration startup::errors::unresolved_environment -- --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test command_usage startup::startup_owner -- --exact --test-threads=1`

The full required batch then passed. Successful command times were 2.85s
formatting, 49.86s Clippy, 385.58s tests, 1.18s documentation and 0.01s whitespace.
The first command_usage build took 1m12s; its 15 cases took 3.59s. Documentation
validation passed 85 reachable pages and 574 local links/anchors. No assertion
was weakened or disabled. Canonical clean-HEAD evidence belongs to Looper
receipts for the final PR-reference commit; these checks do not replace the
controller-owned integrated gate.

## Risks and Follow-ups

Forced termination can lose unpublished events. Filesystem work that outlasts
the final flush deadline returns a diagnostic and retains its sole worker slot.
The joint ranking matrix, paired real-publisher acceptance, representative
two-profile restart journeys, User Guide/Spore completion and baseline closure
remain Slice 3. The plan and tracker now activate that slice; the integrated
gate and delivery-set closure remain controller-owned.
