# Slice 1: Standalone Acceptance Debt

Status: reviewing
Delivery set: rev-architecture-quality-acceptance
Plan: [Standalone Acceptance Correction](../../../plan/rev-architecture-quality-acceptance.md)
Tracker: [Standalone Acceptance Tracker](../../../tracker/rev-architecture-quality-acceptance.md)
Branch: `fix/rev-architecture-quality-acceptance`
PR: `#28`

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
the controller records fresh ordered checks against the final clean PR HEAD.

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

The controller owns integration, the expanded integrated gate and parent closure;
those outcomes remain pending. The first cold release build took about nine
minutes. Two development file-tree runs reached the existing 20-second native
deadline during concurrent compilation; a serial rerun passed without increasing
that deadline. The existing foreign crossterm parentheses warning remains outside
first-party lint admission and does not block the focused checks.
