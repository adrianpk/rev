# Slice 1: Width policy

Status: delivered
Delivery set: rev-tree-viewport
Plan: `ops/default/plan/rev-tree-viewport.md`
Tracker: `ops/default/tracker/rev-tree-viewport.md`
Branch: `feat/rev-tree-viewport-01-width`
PR: `#30`

## Purpose

Deliver TVP01, TVP02, TVP03 and the width controls and presentation portions of
TVP05/TVP06. Keep horizontal navigation for the approved second slice.

## Delivered Behavior

- Default-on tree.auto_width fits every admitted expanded row, including
  offscreen rows. Folded descendants do not contribute. Expansion and refresh
  can grow a retained high-water mark; folding, navigation, focus and visibility
  changes do not shrink it. Geometry caps preserve desired columns for restoration.
- tree-widen/tree-narrow change desired width by four columns with session-only
  manual ownership. Root, Directory, Project, Workspace and standalone transitions
  retain manual choices and start fresh automatic measurement when appropriate.
- tree-reset-width restores the admitted startup policy with fresh measurement;
  tree-auto-width-mode enables a fresh fit or freezes the current desired width.
  Runtime controls do not rewrite TOML or alter source identity/history.
- Both profiles expose tree-local >/< and C-x t >/< /= through native keymaps,
  command descriptions and effective selector hints. Ordinary source input remains
  available. Existing arrow directory routes are retained for Slice 2.

## Implementation Notes

Implementation commit: `77f96b3b69c959fdeff8f0f9590ac074d0c0a7c6`.
Canonical PR: https://forge.adrianpk.com/rev/rev/pulls/30

The tree owns desired width and runtime automatic/manual ownership. Retained
listing publication observes complete unpanned labels using the same sanitized
Glyphs display columns as rendering. Width reads, drawing, navigation and resize
perform no measurement or filesystem discovery. Existing fraction and small-frame
geometry own actual width. No dependency or persisted session field was added.

Legacy clipping/geometry fixtures explicitly select fixed width; their existing
cell, source and pointer assertions remain intact. New transition cases cover
automatic defaults, Unicode/control sanitization, offscreen/folded rows, explicit
reset, minimum saturation, manual widths, missing cached paths and context changes.
The source ticket records its real start for both slices and is now solved with
resolution fixed after integrated acceptance.

## Contracts Added or Changed

- tree.auto_width is a boolean, default true, with partial merge, include/root
  precedence, origin tracking, atomic rejection, reference and complete-sample coverage.
- tree.width remains 1..=512: automatic minimum or fixed startup/reset preference.
  Runtime desired columns saturate at native cell limits and actual geometry
  retains the existing tree.max_fraction and small-frame behavior.
- Four canonical tree commands and their local/prefixed defaults are registered;
  the catalogs contain 688 commands and 300 configuration entries.
- The exact command description inventory and command/configuration/key references, the User Guide Markdown and generated
  Spore projection, and Unreleased describe the usable width controls.

## Files of Interest

- `src/tree/width.rs`, tree listing/root/workspace owners and `src/session/contexts.rs`.
- `src/config.rs`, `src/config/merge.rs` and `src/config/project.rs`.
- `src/command/tree.rs`, `src/session/tree.rs` and `src/input/keymap/defaults.rs`.
- `tests/tree/width.rs`, context transitions, effective hints and resized pointer targets.
- Existing Better File Tree native/PTY acceptance and `docs/reference/file-tree.md`.

## Validation

Canonical PR #30 merged into dev at
`03b0d22b322a97b7d374568b7bcbeb807bc38e73`. Final clean-HEAD receipt
`EXJXXB4JIRODJ7SWQL3EMTUVBW` passed all six ordered commands and all
635 selected native tests. The controller retains the exact command logs.

The controller receipt for the final clean PR HEAD owns acceptance for this
exact ordered list. It is recorded after PR-reference commits; this report does
not substitute development checks for that receipt or the later integrated gate.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree --test configuration --test commands --test pointer --test display --test contexts -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-file-tree.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `git diff --check`

Development checks passed before report introduction:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test tree width:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts width_contexts -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test pointer resized_targets -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands context_keymaps::context_hints -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands reference -- --test-threads=1`

The six width cases passed in 6.27 s, context transitions in 2.40 s, resized
pointer targets in 2.34 s, selector hints in 3.03 s and ten reference cases in
4.04 s. Full native source policy
passed for 1840 files and 73 Cargo roots; Clippy acceptance remains with the
required lint command and active commit hook.

Better File Tree passed its unchanged native/NO_COLOR/PTY bounds. Actual PTY
journeys took 1.19 s and 1.41 s with zero idle CPU ticks, unchanged source/config
bytes and restored TTY state. Width actions run in the existing PTY per profile;
visual variants retain the native cell/pointer matrix without redundant replay.
Early development runs exposed selector text clipping, sample-map counts, a
Markdown table boundary and excess native action replay; focused corrections
passed without increasing timeouts or weakening existing assertions.

The first clean-HEAD batch, JIK24PM3BXI2LRYEZD4H4BPCE5 on
`64dd5fec4726b40633c1e7957fb38e364fd0c744`, passed lint, 204 command tests
(677.15 s) and 162 configuration tests (257.34 s), then failed one of 63
context tests: a legacy Workspace highlight expected 24 columns rather than
the new automatic 33-column cap. Its three exact highlight expectations now
assert that cap and runtime automatic ownership. A fixed-column palette sample
also explicitly selects fixed width, preserving its original RGB assertions.
The corrected targets passed: contexts 63/63 (60.28 s), display 47/47
(34.35 s), pointer 69/69 (78.50 s) and tree 90/90 (62.45 s). The successful
surface command was:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display --test pointer --test tree -- --test-threads=1`

The unchanged six-command batch must pass on the new clean PR HEAD; the earlier
failed receipt is retained as failure evidence.

The next clean-HEAD batch, BCAPNRYLLQ5VGBXBV35Z4QUCCV on
`a8ac5400005ac0ccd5c33fae8d739754b0d88ef8`, passed lint and all 635 selected
native tests, then reached the existing 20-second native acceptance deadline.
An isolated reproduction returned 124 after 20.22 s. The fixture now shares one
unchanged frame between root-selection and source-color assertions. Its full
24-case configured-geometry matrix runs in the existing NO_COLOR invocation,
which uses the same environment and its unchanged 20-second deadline. The ten
interactive presentation variants remain in the native invocation. No case,
assertion, input route or script deadline was removed or increased; the shell
producer is unchanged. Per-variant measurements identify the workload explicitly.

The bounded Better File Tree command then passed in 27.84 s across all three
existing invocations, including its build. Native interactive cases measured
0.80-1.13 s each; both PTYs passed in 1.29 s and 1.46 s with zero idle CPU ticks.
The new fixture commit requires a fresh complete six-command receipt.

Documentation passed with 106 reachable pages and 743 local links/anchors.
The User Guide built through its canonical Markdown-to-Spore projection.
Cargo execution uses the task-owned same-UID PID namespace runner with a real
PID1 reaper (effective UID 1000), single-job/no-incremental/no-debug settings and
private checkout/hook caches. Runner checksum and local logs remain outside
versioned artifacts. The existing foreign crossterm warning is unchanged.

## Risks and Follow-ups

Slice 2 delivered horizontal panning and intentional arrow rebinding through
merged PR #31. Physical font appearance retains the existing native-cell/PTY
evidence limit. The completed integrated gate is recorded below.

## Delivery-set closure

Both slices are integrated through canonical PRs #30 and #31. The controller
passed the unchanged combined gate at `1c98ee49bc6af413b99148f270d1395dc80f9512`, receipt
`DYYC4ATKJZTUCR4UA2GE5UDS43`: full lint, 639 native tests without failures or
ignored cases, bounded native/PTY acceptance, complete catalogs, local User Guide
build and whitespace checks. The
[Slice 2 closure](slice-2-horizontal.md#delivery-set-closure) records the exact
combined command, measured costs and evidence limits. Specification, model,
plan and tracker are Completed; the source ticket is solved with resolution fixed.
Closure preserves all tested runtime, test and guide inputs.
