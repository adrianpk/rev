# Slice 1: Standalone Acceptance Debt

Status: delivered
Delivery set: rev-architecture-quality-acceptance
Plan: [Standalone Acceptance Correction](../../../plan/rev-architecture-quality-acceptance.md)
Tracker: [Standalone Acceptance Tracker](../../../tracker/rev-architecture-quality-acceptance.md)
Branch: `fix/rev-architecture-quality-acceptance`
PR: `#28`
Integrated: `74896fd18e90403c0961c16bc5ae86ea4932e633` through canonical PR #28.
Gate repair branch: `fix/rev-architecture-quality-acceptance-reload-1`
Gate repair PR: `#29`
Gate repair implementation: `38b76c17ee14aeb165f52c725e77b6965f8a1096`
Gate repair state: delivered
Gate repair integrated: `ed9f8dedea2a7633d529220be0bcbc7cb579cd90` through canonical PR #29.

## Purpose

Repay the acceptance debt recorded in parent Slice 9 and complete the reopened
architecture-quality delivery with real native and terminal journeys.

## Delivered Behavior

All five original standalone entrypoints pass their complete native and terminal
journeys. File-tree acceptance retains Unicode clipping and canonical hits while
recognizing the parent navigation row and active-file selection. Command ranking
accepts the current lexical inventory, including empty-query invocation.
Keybinding acceptance verifies complete name/shortcut/description rows, selected
and secondary colors, argument-prefix outcomes and canonical alias dispatch.
Wrapping acceptance recognizes directory context replacement and its fresh empty
buffer before exercising source editing and the release performance matrix.
Projects/Workspaces passes unchanged in both interaction profiles.

## Implementation Notes

The reproduced failures were stale fixture expectations against established
contracts, not runtime defects. Corrections stay within four existing example
families; no runtime, script, configuration, dependency or quality-policy changes
are required. Tree presentation checks both selected and unselected source roles.
Command rows preserve shared annotation columns, complete narrow hint omission,
clipped descriptions and name-only completion. Registered AI dispatch uses an
explicit command query rather than depending on the first catalog entry.

All original fixture counts, Unicode inputs, negative checks, byte/history/state
assertions, color/frame checks, idle and TTY checks, subprocess ownership and
time/count/byte bounds remain in force. Build targets and the staged-check cache
remain private to the worktree. The parent receipt remains historical evidence;
all seven ordered checks passed through the controller's validator on clean
published PR HEAD `74896fd18e90403c0961c16bc5ae86ea4932e633`. Forgejo and Acta
confirm that exact HEAD is now integrated into `dev`. The expanded controller
gate subsequently passed on the candidate recorded in Delivery-set closure.

## Contracts Added or Changed

All five recorded standalone acceptance families remain mandatory for final
closure. No product contract or acceptance limit changes.

## Files of Interest

- `examples/better_file_tree_acceptance/{configuration,native,pty}.rs`
- `examples/command_ranking_acceptance/native.rs`
- `examples/command_keybinding_hints_acceptance/{native,presentation,pty}.rs`
- `examples/wrap_navigation/journey.rs`

## Validation

Baseline on the approved starting revision: file-tree fails its Unicode suffix
assertion, ranking fails the empty-query order, hints fail the foreground-role
assertion and wrapping fails the directory BufferId assertion. Each was
reproduced through its original script. Projects/Workspaces passes unchanged.

Successful focused commands, in required order:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope examples`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-file-tree.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-command-ranking.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-command-keybinding-hints.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-wrap-navigation.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-projects-workspaces.sh`
- `git diff --check`

Scoped quality covers 123 first-party Rust files across 27 Cargo roots in two
packages, with no blocking Clippy diagnostics and all native policy bounds met.
Ranking includes both-profile restarts and paired publishers. Hints includes all
native origin variants and both-profile PTYs. Wrapping includes three native
configuration variants, the full PTY journey and all five performance inputs
(token, words, Unicode, regional and many-lines), with exact source restoration
and no idle deadline. Projects/Workspaces retains capacity, storage, resource and
both-profile startup/restart journeys. Owned PTYs report restored TTYs and owned
fixture directories are removed by the original scripts.

## Risks and Follow-ups

Integration is verified through canonical PRs #28 and #29. The expanded controller
gate and both delivery-set closures are accepted below. The first cold
release build took about nine minutes. Two development file-tree runs reached the
existing 20-second native deadline during concurrent compilation; a serial rerun passed without increasing
that deadline. The existing foreign crossterm parentheses warning remains outside
first-party lint admission and does not block the focused checks.

## Gate repair 1: Merge source settlement

Retained gate failure `4QJ5NG6X2SBKG32XY4J7ITM45B` on
`ad1834fdd47957d4c8dcb251f1f4105dfe829dc9` passed 241 Git cases and failed
`commits::effects::cancelled_child`: the earlier namespace PID 1 did not reap
adopted children. This failed batch remains historical evidence. The approved
same-UID runner now uses an actual PID 1 reaper, with the unchanged test binary
as PID 2 and preserved exit/signal status. The retained corrected-environment
Git run then passed 241/242 cases in 427.56 seconds and exposed the separate
`integration::presentation::native_merge` failure: the editor still held
`LOCAL_SOURCE_SENTINEL\n` when the final assertion required `changed source\n`.

The unchanged exact native-merge command reproduces that source failure in this
repair worktree in 0.63 seconds. Git's integration result explicitly precedes
file-policy reload; the original wait only observed its transient message. The
fixture now remembers that observed result and waits for the exact expected
source in the same five-second budget. The existing input loop continues to
service both asynchronous owners. No second wait, deadline, sleep, runtime
change or weakened assertion is added. Both interaction profiles, explicit
submit aliases, full target review, pre-submit rejection, exact target/HEAD and
source assertions, Git dismissal and sequence-status checks remain unchanged.

Only `tests/git_integration/integration_presentation.rs` changes executable test
code. The exact corrected native case passes, including both profiles, in
1.74 seconds. Scoped quality passes all 55 Git fixture files with zero blocking
or deferred Clippy diagnostics and all native source bounds met. The complete
Git target passes all 242 cases, with no failures or ignores, in 400.03 seconds;
this includes the previously failing cancelled-child and native-merge cases.
The installed staged hook remains active. All four ordered repair checks passed
through the controller's validator after PR-reference publication on clean
published HEAD `ed9f8dedea2a7633d529220be0bcbc7cb579cd90`. Forgejo and Acta
confirm that exact HEAD is integrated into `dev` through canonical PR #29.
The expanded controller gate subsequently passed as recorded below.

Successful repair commands, in canonical order:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests/git_integration`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='unshare --user --map-current-user --pid --fork --mount-proc -- python3 /home/adrian/Projects/playground/rev/20261112-1158-git-metadata-gate/pid_init.py' cargo test --locked -p rev --test git_integration -- integration::presentation::native_merge --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='unshare --user --map-current-user --pid --fork --mount-proc -- python3 /home/adrian/Projects/playground/rev/20261112-1158-git-metadata-gate/pid_init.py' cargo test --locked -p rev --test git_integration -- --test-threads=1`
- `git diff --check`

Runner identity for the clean-HEAD evidence: the plan's retained `pid_init.py`,
SHA-256 `57ca92a038ef6348588f0dbf49cbb817141392e4888fe42911b756dec0aae608`.
A direct probe verifies UID 1000 and child PID 2. The runner and its retained
diagnostic files are unchanged. No environment failure is relabeled as a pass;
every unchanged full-gate requirement subsequently passed after integration.

## Delivery-set closure

Completed on 2026-11-12 after canonical PRs #28 and #29 integrated into `dev`.
The controller passed the complete expanded gate on clean published candidate
`422d6f66945d30f7ae640726cf1a63d17586bb1d`, validation
`VSWWUTEOGG2SGZW266X2FB7AMA`. Immutable evidence record:
`422d6f66945d30f7ae640726cf1a63d17586bb1d.json`. Retained gate log:
`422d6f66945d30f7ae640726cf1a63d17586bb1d.VSWWUTEOGG2SGZW266X2FB7AMA.log`.
Earlier failed and focused batches above keep their original candidate/outcome.

The five complete standalone commands in the correction plan and the parent's
eight frozen stages passed in that exact order through one unchanged chained
command. Every build/check stage retained one job, disabled incremental output
and zero dev/test debuginfo.

| Stage | Result on the final candidate |
| --- | --- |
| `bash scripts/check-better-file-tree.sh` | Complete configuration, native both-profile, four-setting NO_COLOR and both-profile PTY journeys pass; Unicode, parent navigation, canonical hits, source/history, idle and TTY assertions remain. |
| `bash scripts/check-command-ranking.sh` | Native ranking, both-profile restart PTYs and paired publishers pass with exact state, dispatch, source, idle and cleanup checks. |
| `bash scripts/check-command-keybinding-hints.sh` | Native presentation/context dispatch/hints and both-profile PTYs pass, including complete narrow omission, effective bindings, colors and exactly-once usage. |
| `bash scripts/check-wrap-navigation.sh` | Three native template variants, full PTY journey and all five release performance/resource scenarios pass with exact source restoration and original deadlines. |
| `bash scripts/check-projects-workspaces.sh` | Native capacity/storage/session/resource checks and both-profile startup/restart PTYs pass unchanged; owned fixtures clean up. |
| `make lint` | All 1,838 first-party Rust files, both packages and 73 Cargo roots pass formatting, strict Clippy and native source policy; zero blocking/deferred diagnostics or source findings. |
| `make test-hooks` | All 28 real Git fixtures pass, including installed linked-worktree hooks and independent cache ownership. Cold/warm checks take 29s/2s. |
| `cargo test --locked --workspace --all-targets -- --test-threads=1` | 3,078 top-level tests and seven nested child runs pass with zero failures. All 242 Git cases pass in 413.65s, including cancellation and native merge. Three existing opt-in acquisition ignores remain unchanged. |
| `cargo test --locked --workspace --doc` | The public SDK doctest passes; the editor has zero doctests. |
| `cargo run --locked --quiet -p rev --example documentation-check` | 684 commands, 299 configuration entries, 106 reachable pages and 743 local links/anchors pass. |
| `bash scripts/build-user-guide.sh` | Local Spore User Guide builds under `target/userguide`. |
| External `scripts/check-rev.sh` | Go tests and both-profile native interoperability pass against API `rev.plugins/0.1`, editor `0.6.0` at the exact candidate and client `3365b84649919492ade7e80c63139b459b5ad923`. |
| `git diff --check` | Passes on the clean integrated candidate. |

The approved same-UID PID namespace retains its PID 1 reaper and unchanged test
execution as PID 2. Runner: the retained `pid_init.py` from the plan,
SHA-256 `57ca92a038ef6348588f0dbf49cbb817141392e4888fe42911b756dec0aae608`,
verified unchanged at closure. The preparation probe verified UID 1000 and child
PID 2. Repository-local `core.hooksPath=.githooks` remains active. No owned Rev
fixtures remain active; unrelated processes and per-worktree caches are preserved.

T1.1-T1.6 and parent T9.4-T9.8/T10.4 are complete. Both delivery plan/tracker
sets, the parent specification/review and concern index reflect verified closure;
all eleven slice reports remain delivered. Only configured Markdown closure
paths differ from the tested candidate. Runtime, test assertions, acceptance
commands and resource/deadline bounds remain identical.

The three existing opt-in upstream-acquisition tests and foreign vendored
`crossterm` warning retain their documented limits. This gate executes all five
previously deferred standalone entrypoints; it does not claim manual editor use,
every other example's custom main, main promotion, release or publication.
