# Slice 7: Native and PTY Acceptance

Status: delivered
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-07-acceptance`
PR: [#254](https://forge.adrianpk.com/adrianpk/rev/pulls/254)

## Purpose

Verify real shells and installed lazygit under the native session and nested
host PTY boundaries, with finite fixtures and measured resource ownership.

## Delivered Behavior

Native sessions and real host PTYs exercise interactive Bash and installed
lazygit in both editing profiles. The journeys retain shell/tool identity, cwd
and shell variables while moving between lower panel and main buffers. They
check actual alternate-screen help interaction, multiple sessions, exact
literal/mouse/bracketed-paste bytes, retained-output copying, protected source
admission, declined shutdown, confirmed live-buffer closure and actual exit 7.
Normal exit and one injected host write fault restore real terminal attributes,
alternate screen, paste and cursor modes and reap owned children.

A two-session resource scenario measures a real 1 MiB output flood beside a
responding peer and a non-reading input owner. It checks atomic queue rejection,
64 KiB service bounds, retained rows/extras, cleanup and exact FD/thread return.

## Implementation Notes

Reuse the existing Rust acceptance layout, Linux PTY owner, metrics and source
checker. Disposable Git/HOME/XDG stores remove inherited Git selectors; lazygit
uses explicit configuration with fetch/update disabled. No network or user Git
state is needed. Fixture deadlines constrain validation only; live commands have
no runtime cap.

Four host cases use 80x24 geometry with a 15-row tool pane. Each owns an unchanged
eight-second, 1 MiB stream; a fifth case injects one output write failure. The
aggregate PTY budget is 45 seconds. Hidden-cursor frames require the owned
geometry's final row to be fully painted. The older visible-cursor path is
covered by the shared shell-command PTY regression.

Shell transitions acknowledge persisted fixture state and fresh per-shell
prompt generations in the active presentation. Restored alternate-screen text
and the inactive panel cannot acknowledge readiness. The actual lazygit exit
status must be zero before sending the next shell command. Modal copying uses an explicit
five-character operand and retains the exact child-byte assertion. Idle
measurement follows a full 350 ms quiet epoch within the existing two-second
settling deadline, then checks another 350 ms of silence and at most one CPU tick.

Live host commands use explicit CSI-u Alt-x, assert the actual M-x prompt and
verify resulting placement/PID. Legacy ESC/x bytes can instead decode as separate
keys; a separately reported Escape correctly belongs to the child. This fixture
does not claim transport equivalence. Standalone child Escape and source/copy
Meta entry are exercised separately.

## Contracts Added or Changed

The real PTY run reproduced a live session incorrectly failing when an owned
host wake interrupted the zero-time pidfd observation. The process owner now
defers that inconclusive observation to its next service turn. It performs no
retry loop, retains the child and changes no resource limit, dependency or gate.
The failing receipt is `4VJUJIZITXHW3I5K2CSIVNDJZM` at
`b8165967f5029ca9ccbb1cfeedd51168a00d00aa`; correction is `4e9bbcbdd69b`.

## Files of Interest

- `examples/integrated-terminal-acceptance.rs` and its modules: native, host,
  fault and resource journeys with shared isolated setup.
- `scripts/check-integrated-terminal.sh`: normal-release build, focused source
  coverage and isolated bounded execution.
- `examples/completion_acceptance/terminal.rs`: shared bounded host stream owner.
- `src/term/process.rs`: interrupted child-exit observation.

## Validation

Final canonical receipt `C4HWXE56WCZ757B54BGMDKQ333` passed at
`47bc4911ee8a6bd1886d153ddb87d8e5b8b8c6d2`, which canonical PR #254 merged unchanged into dev:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test integrated_terminal -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The serial integration target passed 45 tests. Both native/host profiles,
normal/error restoration and exact FD/thread retirement passed.

Focused receipt `TF7COIF6DUMGCENUZH4237HRMM` passed at
`dee525cecdd8daf5da0e905c166cecf202179a9a`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh`
- `git diff --check`

This includes both native profiles, all five host cases and the resource
scenario after fresh-prompt synchronization. The shared shell PTY regression
also passed at `0555c3af914d6e85a95bdbd7972b812a8b4f9d89` in receipt
`7MUL6VQNUYUSO4W434E6KGL6NL`, using the same profile prefix with
`bash scripts/check-shell-commands.sh --pty`. Its shared fixture has not changed
since that regression.

Canonical six-command batch `XTDL6KAYI3BTSO3YTMY73UW6PE` passed format, Clippy
and 45 tests before failing in the modal tool case. The fresh prompt-generation
assertions replace that ambiguous restored-screen readiness observation; the
full focused script above passed afterward. No limits or assertions were relaxed.

The PR declares exactly the six configured commands in canonical order. The
controller retains their final clean-HEAD validation receipt separately. Focused
shared-regression evidence remains separate from that fixed command list.

Local normal-release observations from the focused receipt:

| Observation | Result |
| --- | --- |
| Binary | 22515240 bytes; +938512 bytes (+4.3496%) against Slice 1's recorded 21576728-byte pre-terminal release at `507862798782d15235f3758868ab33cd3611c57e`. The baseline was not rebuilt here. |
| Host idle | Both profiles: 350 ms silence, zero CPU ticks; peak editor RSS 16928 / 18516 kB. |
| Two-session idle | 350 ms, zero owner-plus-shell CPU ticks; peak owner RSS 6864 kB. |
| Active flood | 1048576 producer bytes; 1064495 serviced bytes across 138 turns in 40674 microseconds; peer response 126 microseconds; owner-plus-shell CPU 3 ticks at 100 ticks/second. |
| Retention/queues | 64 retained rows, 168800 reserved cells, zero extra bytes, 49664 input bytes still queued; full 65536-byte queue rejected overflow atomically. |
| Active peak RSS | 8924 kB in the resource driver. |
| Cleanup | Two leaders reaped in 39188 microseconds; FDs 4 to 4 and threads 1 to 1. |
| Host streams | Tool: 780328 / 805428 bytes; input/closure: 912334 / 885979 bytes, all below 1 MiB. |

The cold acceptance release build took 7 minutes 40 seconds. Rebuilding the
changed process owner took 4 minutes 17 seconds; focused driver relinks took
17 and 13 seconds. These are local observations, not general throughput or
latency guarantees; active CPU covers the owner and two shell leaders.

## Risks and Follow-ups

The fault case injects one write error inside a real controlling PTY. It proves
cleanup and restoration orchestration with subsequent writes available, not
recovery from an unavailable physical terminal. Canonical PR #254 and the later
guidance slice and readiness repair are integrated. The unchanged controller
gate passed at `52da8df25937c2165d72916603ef048712b3abbb`, receipt
`IYS47Y2HH6YVYH3AMLMDEOZHVQ`; the
[Slice 8 closure](slice-8-guidance.md#delivery-set-closure) records combined
acceptance and final resource observations. The measurements above retain their
original slice-local candidates.
