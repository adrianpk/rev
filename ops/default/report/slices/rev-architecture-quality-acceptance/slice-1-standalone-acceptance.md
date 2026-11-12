# Slice 1: Standalone Acceptance Debt

Status: delivered
Delivery set: rev-architecture-quality-acceptance
Plan: [Standalone Acceptance Correction](../../../plan/rev-architecture-quality-acceptance.md)
Tracker: [Standalone Acceptance Tracker](../../../tracker/rev-architecture-quality-acceptance.md)
Branch: `fix/rev-architecture-quality-acceptance`
PR: `#28`
Integrated: `74896fd18e90403c0961c16bc5ae86ea4932e633` through canonical PR #28.
Gate repair branch: `fix/rev-architecture-quality-acceptance-reload-1`
Gate repair PR: pending
Gate repair state: reviewing

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
confirm that exact HEAD is now integrated into `dev`. This delivery status
records the slice merge; the expanded gate and delivery-set closure remain pending.

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

Integration is verified through canonical PR #28. The controller owns the expanded
integrated gate and parent closure; those outcomes remain pending. The first cold
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
The installed staged hook remains active. Final clean-HEAD evidence is recorded
separately by the controller's validator after PR-reference publication. The
expanded gate remains controller-owned.

Successful repair commands, in canonical order:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests/git_integration`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='unshare --user --map-current-user --pid --fork --mount-proc -- python3 /home/adrian/Projects/playground/rev/20261112-1158-git-metadata-gate/pid_init.py' cargo test --locked -p rev --test git_integration -- integration::presentation::native_merge --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='unshare --user --map-current-user --pid --fork --mount-proc -- python3 /home/adrian/Projects/playground/rev/20261112-1158-git-metadata-gate/pid_init.py' cargo test --locked -p rev --test git_integration -- --test-threads=1`
- `git diff --check`

Runner identity for the clean-HEAD evidence: the plan's retained `pid_init.py`,
SHA-256 `57ca92a038ef6348588f0dbf49cbb817141392e4888fe42911b756dec0aae608`.
A direct probe verifies UID 1000 and child PID 2. The runner and its retained
diagnostic files are unchanged. No environment failure is relabeled as a pass;
the controller must run every unchanged full-gate requirement after integration.
