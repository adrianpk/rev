# Slice 2: Horizontal navigation

Status: delivered
Delivery set: rev-tree-viewport
Plan: `ops/default/plan/rev-tree-viewport.md`
Tracker: `ops/default/tracker/rev-tree-viewport.md`
Branch: `feat/rev-tree-viewport-02-horizontal`
PR: `#31`

## Purpose

Deliver TVP04, TVP05 and TVP06 with joint acceptance of the width controls.

## Delivered Behavior

- Every admitted row shares one display-column offset. Left/Right and
  tree-scroll-left/tree-scroll-right pan by one column and stop at the current
  expanded content boundary. Indentation, disclosure, icons and names move
  together. Whole graphemes survive clipping; a partial wide cell becomes space.
- Selection, vertical scrolling, following, focus and hide/show retain position.
  Folding/removal and wider geometry clamp only excess offset. Root/context
  changes and width reset return to the beginning. Mode changes retain position
  when it still fits. Hidden/zero geometry preserves the retained viewport.
- Both profiles use focused-tree arrows for panning. Enter still opens/toggles;
  C-f/C-b still expand/collapse/select parent. Configured overrides, prefix cancel,
  command descriptions and effective hints remain with native input ownership.
- Viewport operations preserve row/path/root, source bytes, point, mark, dirty
  state, history and editing-window identity. Pointer hits remain full-row
  canonical targets after panning, resizing and loss of the observed paths.

## Implementation Notes

Implementation commit: `75078243383df1e5e1eda9c84483b22f8a200325`.
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/31

The existing tree owns cached current extent and horizontal position alongside
width ownership. Listing publication measures sanitized unpanned rows, including
expanded offscreen entries; folding updates extent without shrinking the
automatic width mark. Geometry and navigation consume this cache without
filesystem discovery. Commands use actual composed panel width, and the
existing display clipper applies the same offset to every row. Minimal and
decorated prefixes now retain their natural bounded indentation at narrow widths
so measured and rendered row shapes agree. No dependency or stored format changed.

Slice 1 is integrated through canonical PR #30. T2.1 and T2.2 share one cohesive
runtime/test change; T2.3 completes executable acceptance, references, the User
Guide Markdown/Spore projection and the single Unreleased viewport outcome.
The source ticket retained its identity through joint review and is now solved
with resolution fixed after integrated acceptance.

## Contracts Added or Changed

- Two canonical commands bring the combined inventory to 690 names; the complete
  configuration catalog remains 300 entries. The tree map retains 29 entries.
- Focused-tree Left/Right now pan; C-f/C-b preserve directory operations in both
  profiles. Complete sample, exact description inventory and binding references
  agree with the live catalogs.
- Horizontal state is session-only, bounded in terminal display columns and
  independent from filename text and pointer identity. Width reset clears it;
  automatic mode changes preserve valid position.

## Files of Interest

- `src/tree/viewport.rs`, `src/tree/width.rs` and retained listing owners.
- `src/tree/entry.rs`, `src/tree/presentation.rs` and `src/display/tree.rs`.
- `src/session/tree.rs`, `src/session.rs` and native command/keymap catalogs.
- `tests/tree/horizontal.rs`, tree geometry, pointer, display, context and hint cases.
- Existing Better File Tree native/PTY owners and `docs/reference/file-tree.md`.

## Validation

Canonical PR #31 merged into dev at
`4a4f89a0bdf8c276d7066413beccce2a474952e3`. Clean-HEAD slice receipt
`6TH5M2YS454DBGIGDPRZSUDQ3V` passed all six ordered commands, including
639 selected native tests with no failures or ignored cases. This is slice
acceptance; the completed integrated gate is recorded below.

The exact clean-HEAD controller receipt after PR-reference commits owns the
six ordered acceptance commands below. This report records implementation
checks; it does not claim the controller-owned integrated delivery-set gate.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test configuration --test commands --test pointer --test display --test contexts -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-file-tree.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

Implementation checks passed full workspace lint (1,842 first-party Rust files,
73 Cargo roots), all 93 tree cases, targeted Unicode/extent/input, pointer and
selector checks, complete documentation catalogs and User Guide publication.
Documentation reports 106 reachable pages and 743 local links/anchors. The
existing foreign crossterm warning remains unchanged.

The complete Better File Tree command passed in 24.11 s with its original three
20-second invocations and eight-second PTY deadline. All ten interactive native
variants, the complete 24-case geometry matrix and four monochrome cases remain.
Native panning runs after the observed paths disappear. Each PTY uses actual
arrows, control directory routes, local/prefixed width controls, shallow/deep/long
labels, shared capped ending/beginning views, reset, full-row file activation,
40/31/64-column resizing, idle checks and TTY restoration. The two PTYs measured
1.17 s and 1.30 s in that run, with zero idle CPU ticks. No timeout, case or
semantic/resource assertion was removed or weakened. Native cells establish the
Unicode/font matrix; physical font appearance remains outside this evidence.

Early focused failures corrected fixture expectations: natural indentation,
directories-first row ordering, explicit tree focus before pan in pointer/context fixtures and the shared
M-x route while the tree owns input in both profiles. The PTY journey was split
into cohesive owners to preserve the existing complexity bound. The terminal
producer and all deadlines remain unchanged.

Cargo uses the previously verified same-UID PID namespace runner with a real
PID1 reaper (effective UID 1000), one build job, no incremental output or dev/test
debuginfo, and private checkout/hook caches. Exact runner/checksum and local logs
remain outside versioned artifacts. Final receipt logs own exact candidate and
ordered command evidence.

## Risks and Follow-ups

Both slices and the delivery set are complete. Physical font appearance retains
the existing native-cell/PTY evidence limit.

## Delivery-set closure

Delivery-set status: Completed
Tested candidate: `1c98ee49bc6af413b99148f270d1395dc80f9512`.
Controller gate receipt: `DYYC4ATKJZTUCR4UA2GE5UDS43`.
Both canonical PRs are merged: [#30](https://forge.adrianpk.com/rev/rev/pulls/30)
and [#31](https://forge.adrianpk.com/rev/rev/pulls/31).

The controller passed the following unchanged combined command on the immutable
clean integrated candidate:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test configuration --test commands --test pointer --test display --test contexts -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-file-tree.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`

Full workspace formatting, strict Clippy and native source policy passed across
1,842 first-party Rust files, 2 packages and all 73 Cargo roots. All 639 selected
native tests passed, with zero failures and zero ignored cases:

| Target | Passed | Test execution seconds |
| --- | --- | --- |
| commands | 205 | 763.71 |
| configuration | 162 | 313.63 |
| contexts | 63 | 80.30 |
| display | 47 | 43.71 |
| pointer | 69 | 92.70 |
| tree | 93 | 66.06 |

The six native targets report 1,360.11 seconds in total, excluding compilation
and the remaining gate commands. Better File Tree passed every retained native,
configured-geometry and monochrome case and both owned PTYs. Non-modal PTY
execution measured 1,361 ms and 17,581 output bytes; modal measured 1,562 ms and
17,745 bytes. Each reported zero idle CPU ticks over its 350-ms silence check;
TTY restoration and unchanged disk/configuration checks passed. Original
20-second invocations, eight-second PTY deadline, cases and resource/semantic
assertions were retained. No owned test fixture or build remained active at closure.

Catalogs passed with 690 command names and 300 configuration entries.
Documentation passed with 106 reachable pages and 743 local links/anchors; the
canonical Markdown-to-Spore User Guide built locally. The existing foreign
crossterm warning remained nonblocking. Evidence establishes native terminal
cells and the owned PTY journeys; physical font appearance retains its documented
limit. Execution retained the approved same-UID namespace/PID1 reaper (UID 1000),
single-job/no-incremental/no-debug settings and private targets. Exact runner and
checksum details remain in external task-owned evidence.

Reports remain delivered; specification, model, plan and tracker are Completed.
The source ticket is solved with resolution fixed and real lifecycle/PR/commit
evidence. Both owned slice worktrees were removed after integration; unrelated
worktrees and caches are preserved. Documentation closure changes only the
configured Markdown paths. Runtime, tests, guide inputs and command/configuration
catalogs retain the tested bytes. The original gate receipt remains evidence for
the tested candidate, with documentation closure recorded separately on dev.
