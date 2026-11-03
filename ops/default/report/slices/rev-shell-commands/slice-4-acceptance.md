# Slice 4: Joint acceptance and guidance

Status: delivered
Delivery set: rev-shell-commands
Plan: [Native Shell Commands Delivery](../../../plan/rev-shell-commands.md)
Tracker: [Native Shell Commands Tracker](../../../tracker/rev-shell-commands.md)
Branch: `fix/rev-shell-commands-validation-1`
PR: `#246`

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

Gate repair 1 addresses controller receipt `RSHBYQUHAMXM7FQYNTWOPSLFQS`
on failed base `858c50f64105cde67ee128e1cb1d0c90b96cc9df`. Format,
strict Clippy and 163 command tests passed, then configuration's existing
`reference::sample_maps` rejected the packaged VimNormal map. Focused execution
of the retained configuration binary reproduced the absent M-! and M-| entries.
The other modal maps derive from the same editor defaults and need their matching
declarations too. Add four compiled shell bindings to Editor/Insert/Replace and
two M- bindings to Normal/Visual character/line/block. This repairs the packaged
factory template without changing compiled behavior or extending modal C-u
motions. Keep both complete declared-map and effective-map equality assertions.

The repair uses the original gate-repair policy, the pinned branch/title and
this existing report. Canonical repair PR:
[#246](https://forge.adrianpk.com/adrianpk/rev/pulls/246).
Focused controller receipt `ZLBTP4HTQ6ZICAKCSZP7MAB3GN` passed both
packaged-template regressions on `353d33651463f85f76e46471dc938aeb01eb9454`:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration reference::sample_ -- --test-threads=1`: sample_defaults and sample_maps passed, covering packaged bytes, scalar defaults and complete declared/effective maps.
- `git diff --check`: passed.

The canonical PR receipt owns the exact clean head and five-command repair
prefix after the PR-reference follow-up. That prefix includes all shell,
configuration and command tests plus formatting, strict all-target Clippy,
documentation and whitespace checks. The unchanged combined gate remains the
controller's responsibility.

Repair receipt `7O7DA2WCJ7EMSOOOOYYYDL3DBH` on
`c390a68ecdfa38974a7acc7ff08135b17aa376c3` passed format, strict Clippy
and all 349 shell/configuration/command tests, including complete sample maps.
Documentation then rejected `keys.editor: compiled map count drift`: the seven
existing keymap rows in defaults.md still described the smaller incomplete
template. Their exact counts were corrected to 80/139/81/80/121/121/117 for
Editor/Normal/Insert/Replace/Visual Character/Line/Block, preserving the count
assertion and both complete-map assertions.

Canonical repair PR #246 merged into dev at
`02e7e77b57ecb219f01e33b5d93d6dd7124b1bc7`. Final controller receipt
`VCK427Z65VUKAXGHWR7UJ5F3S2` passed the exact five-command repair prefix
on that clean candidate:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands --test configuration --test commands -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

All 163 command, 151 configuration and 35 shell tests passed. Documentation
validated 437 commands, 240 preferences, 73 reachable pages and 458 local
links/anchors. The controller's unchanged combined gate remains pending;
preparation did not run it.

## Risks and Follow-ups

Terminal bytes prove protocol behavior; glyph appearance belongs to the terminal.
The finite 8 MiB case exercises the default bound and does not claim unlimited
streaming. Existing conservative disk observation cannot prevent a later external
write after observation. The controller still owns the unchanged combined gate;
final documentation closure requires its successful receipt.
