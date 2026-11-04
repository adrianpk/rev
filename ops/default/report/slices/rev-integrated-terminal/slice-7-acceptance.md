# Slice 7: Native and PTY Acceptance

Status: reviewing
Delivery set: rev-integrated-terminal
Plan: [Integrated Terminal Delivery](../../../plan/rev-integrated-terminal.md)
Tracker: [Integrated Terminal Tracker](../../../tracker/rev-integrated-terminal.md)
Branch: `feat/rev-integrated-terminal-07-acceptance`
PR: pending

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

Shell transitions acknowledge persisted fixture state and completed host prompts,
rather than matching stale prompt text. Modal copying uses an explicit
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

Focused receipt `LSXFV5ULDTILOGEM6NGIHJYDIS` passed at
`972eee56a2f2c7b0df026652f3086f635363b242`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh --pty`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-integrated-terminal.sh --resources`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh --pty`
- `git diff --check`

T7.1 native receipt `PGKUQXPYN44HYS2BXYNCZKH7VC` passed at
`8f0a16c510ce54cd90197c639f72bcb2efd6bb30` with the profile prefix and
`bash scripts/check-integrated-terminal.sh --native`, then `git diff --check`.
The final PR head uses the six configured commands in canonical order, plus
the focused shared shell PTY regression before whitespace. The controller owns
the exact clean-HEAD receipt; integration remains pending that validation.

Local normal-release observations from the focused receipt:

| Observation | Result |
| --- | --- |
| Binary | 22515240 bytes; +938512 bytes (+4.3496%) against Slice 1's recorded 21576728-byte pre-terminal release at `507862798782d15235f3758868ab33cd3611c57e`. The baseline was not rebuilt here. |
| Host idle | Both profiles: 350 ms silence, zero CPU ticks; peak editor RSS 17952 / 19040 kB. |
| Two-session idle | 350 ms, zero owner-plus-shell CPU ticks; peak owner RSS 6988 kB. |
| Active flood | 1048576 producer bytes; 1064495 serviced bytes across 139 turns in 40454 microseconds; peer response 123 microseconds; owner-plus-shell CPU 3 ticks at 100 ticks/second. |
| Retention/queues | 64 retained rows, 168800 reserved cells, zero extra bytes, 49664 input bytes still queued; full 65536-byte queue rejected overflow atomically. |
| Active peak RSS | 8984 kB in the resource driver. |
| Cleanup | Two leaders reaped in 42381 microseconds; FDs 4 to 4 and threads 1 to 1. |
| Host streams | Tool: 801782 / 821112 bytes; input/closure: 964885 / 938531 bytes, all below 1 MiB. |

The cold acceptance release build took 7 minutes 40 seconds. Rebuilding the
changed process owner took 4 minutes 17 seconds; focused driver relinks took
17 and 13 seconds. These are local observations, not general throughput or
latency guarantees; active CPU covers the owner and two shell leaders.

## Risks and Follow-ups

The fault case injects one write error inside a real controlling PTY. It proves
cleanup and restoration orchestration with subsequent writes available, not
recovery from an unavailable physical terminal. Final configured PR validation
and controller integration remain pending. User guidance is the recorded next
slice.
