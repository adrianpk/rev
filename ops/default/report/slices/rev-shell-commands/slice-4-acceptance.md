# Slice 4: Joint acceptance and guidance

Status: delivered
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../../../plan/rev-shell-commands.md)
Tracker: [Native Shell Commands Tracker](../../../tracker/rev-shell-commands.md)
Branch: `feat/rev-shell-commands-04-acceptance`
PR: `#245`

## Purpose

Verify native shell commands and filters together with real local processes,
finite resource observations and both Linux PTY profiles, then document the
complete user-facing workflow through the existing User Guide and Spore path.

## Delivered Behavior

- One isolated driver executes actual sh/printf, sort, tr and cat through both
  native profiles: unsaved Unicode/CRLF input, display/insertion/filter, one-step
  undo, failure immutability, readonly admission and stale-target rejection.
- Real cancellation stops owned leaders and descendants. Atomic file replacement
  reloads the original source while its shell is still active; completed captured
  stdout remains inspectable without overwriting that newer source.
- A finite fixture sends exactly the default 8 MiB input bound through tr, checks
  complete retained output and inserted/removed native history bytes, then undoes
  once. It measures capture, collection/publication, undo, peak RSS and idle CPU.
- Two real Linux PTYs exercise non-modal keys and modal Visual/colon invocation,
  canonical Unicode selection, sort/tr, insertion and one-step undo, output
  reopening, inner cancellation before job cancellation, descendant cleanup,
  resize from 100x24 to 72x18 and terminal restoration.
- The new script retains each existing PTY's eight-second/1-MiB limits. Native,
  resource and two-PTY groups have 20-second outer guards. Completion requires
  no shell deadline, no remaining worker and quiet post-job terminal input.
- Focused source quality covers the shell policy/input/publication/process/status
  owners, crate roots and acceptance modules under unchanged safe-Rust,
  five-parameter, three-level nesting and cyclomatic-ten limits.
- A sort-and-undo User Guide chapter, focused task guide and exact reference
  explain all six commands, four sequences, five preferences, literal text,
  checked publication, cancellation and output inspection. Existing indexes,
  previous chapter, Spore navigation and the single Unreleased entry are updated.

## Implementation Notes

- `T4.1`: `db7c3fb38c81db7b7f471d667f72e16cbc130ebe` introduces the real
  native driver/script and source-quality coverage. Four functions exceeded the
  existing complexity limit; extract worker execution, history recall, context
  admission and final file-observation validation without changing their
  conditions, evaluation order, bounds or ownership.
- `T4.2`: `306286125fae9c44629a84c80d65c135c59e78b9` adds finite resource
  observations, shared terminal-owner reuse and both-profile PTY/reload journeys.
- `T4.3`: `591267724625605c25936a6309d09c8386522821` completes guidance, catalogs and Spore
  navigation. Documentation remains canonical Markdown; publication is generated
  only through the existing local guide producer.
- Fixture HOME/XDG stores and sources are private. PID markers belong to actual
  owned shell children, are refreshed between journeys and are checked after
  cancellation. The wrapper preserves termios and uses the existing timeout,
  cursor, alternate-screen and bracketed-paste restoration assertions.
- Pipe-key cells use escaped Markdown rather than literal HTML. The two catalog
  readers split canonical spaced column delimiters, preserving escaped pipes
  inside their key cell. Complete command-set equality and duplicate rejection
  stay unchanged. Extend the existing Spore table test with literal M-| code.
- No dependency, Cargo.lock, command identity, preference, runtime limit or
  acceptance assertion is removed or weakened.

## Contracts Added or Changed

- Add `examples/shell-command-acceptance.rs` and `scripts/check-shell-commands.sh`
  with complete native/resource/PTY modes and private finite fixture ownership.
- Retain the four low-footprint Cargo variables, one release build job and 1 GiB
  filesystem admission floor; never share an active mutable target.
- Source-quality failures use the existing checker and thresholds; helper
  extraction preserves the approved runtime contracts rather than adding policy.
- Add shell guidance to the ordered editor learning path and existing Spore
  projection, with supporting task/reference links and exact startup defaults.

## Files of Interest

- `examples/shell_command_acceptance/`: native cases, metrics, finite resources,
  PTY journeys and shared native support.
- `scripts/check-shell-commands.sh`, `examples/completion_acceptance/terminal.rs`:
  owned driver wrapper and reused unchanged eight-second/1-MiB terminal boundary.
- `src/shell.rs`, `src/input/shell.rs`, `src/session/shell.rs`,
  `src/session/shell/publication.rs`: the four bounded helper extractions.
- `docs/tutorials/user-guide/15-shell-commands.md`,
  `docs/how-to/run-shell-command.md`, `docs/reference/shell-commands.md`:
  learning exercise, specific tasks and exact command contracts.
- `examples/documentation-check.rs`, `examples/documentation_check/catalogs.rs`,
  `examples/userguide-publication.rs`, `tests/commands/palette.rs`,
  `publication/userguide/index.sptree`: complete navigation/catalog/projection
  checks and escaped-key preservation.

## Validation

Successful development commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh`: initial T4.1 native/source-quality journeys passed both profiles before resource/PTY modes were added.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh --native`: both profiles passed, including concurrent atomic reload before child completion.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh --resources`: exact 8388608 input/output/history bytes, one native undo and post-job idle passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-shell-commands.sh --pty`: both real PTYs passed with 75738/75781 observed output bytes, unchanged source/disk and restored terminal state.
- `git diff --check`: passed.

Development release observations on this host: 8839 us input capture, 2242515 us
collection/publication, 61007 us undo observation and 105096 kB peak native RSS.
The native fixture retained exactly 8388608 stdout bytes and one history change
with the same inserted/removed byte counts. Native idle consumed zero CPU ticks
over 350 ms, with two threads before and after. PTY idle also consumed zero ticks
with 350 ms terminal silence; observed editor peak RSS was 12864/14384 kB.
These observations establish the finite local cases, not general latency or
memory guarantees.

Focused receipt `RXP3QWEFVRARFBCQN4JO477BVM` passed catalog, readonly-transition
and Spore pipe-cell checks on `575f47551d121495f4227838af00007bc5da4e2e`.
The first configured receipt `WY4B6AB7OJ7X5FLV2QAPO3HQ3Y` passed format,
strict Clippy, all 35 shell tests and the complete native/resource/PTY driver,
then rejected the preceding chapter's footer-only next link. The guide contract
requires a literal `Next:` line. The chapter was corrected to the established
form, preserving navigation checks; both receipts were regenerated afterward.

Canonical PR [#245](https://forge.adrianpk.com/adrianpk/rev/pulls/245) merged
into dev at `554d1b770a2cee58328863afac1f0dd2c1e0ba1d`.
Controller receipt `3DQKGFLQ5HKNUHWESKIWUZ5FNL` passed all seven configured
commands in order on that exact clean head: format, strict all-target Clippy,
35 shell tests, the complete native/resource/PTY driver, documentation validation,
the local guide build and `git diff --check`. Documentation covered 437 commands,
240 preferences, 73 reachable pages and 458 local links/anchors; Spore projected
all 73 Markdown pages. The canonical PR contract check passed.

The final resource run retained exactly 8388608 stdin/stdout bytes and matching
inserted/removed history bytes, with one undo. It observed 8530 us capture,
2248738 us collection/publication, 66333 us undo and 104684 kB peak native RSS.
Native idle used zero CPU ticks over 350 ms, with two threads before and after.
The non-modal/modal PTYs observed 75739/75782 bounded bytes and 12736/13936 kB
peak RSS; each passed 350 ms of terminal silence with zero idle CPU ticks.
These remain finite host observations, not general performance guarantees.

Focused receipt `7FMBXVR66YUNL4HKD54J6GMZUW` passed three tests on that same
clean head:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands palette::catalog_reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::shell::publication -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --example userguide-publication table_cells -- --test-threads=1`

The immutable report introduction is `591267724625605c25936a6309d09c8386522821`.
All PR-reference follow-ups were pushed before validation. The unchanged
integrated gate remains controller-owned and pending.

## Risks and Follow-ups

Terminal bytes prove protocol behavior; glyph appearance belongs to the terminal.
The finite 8 MiB case exercises the default bound and does not claim unlimited
streaming. Existing conservative disk observation cannot prevent a later external
write after observation. The controller still owns the unchanged combined gate;
final documentation closure requires its successful receipt.
