# Slice 1: Width policy

Status: reviewing
Delivery set: rev-tree-viewport
Plan: `ops/default/plan/rev-tree-viewport.md`
Tracker: `ops/default/tracker/rev-tree-viewport.md`
Branch: `feat/rev-tree-viewport-01-width`
PR: pending

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

The tree owns desired width and runtime automatic/manual ownership. Retained
listing publication observes complete unpanned labels using the same sanitized
Glyphs display columns as rendering. Width reads, drawing, navigation and resize
perform no measurement or filesystem discovery. Existing fraction and small-frame
geometry own actual width. No dependency or persisted session field was added.

Legacy clipping/geometry fixtures explicitly select fixed width; their existing
cell, source and pointer assertions remain intact. New transition cases cover
automatic defaults, Unicode/control sanitization, offscreen/folded rows, explicit
reset, minimum saturation, manual widths, missing cached paths and context changes.
The source ticket records its real start and remains in_progress for both slices.

## Contracts Added or Changed

- tree.auto_width is a boolean, default true, with partial merge, include/root
  precedence, origin tracking, atomic rejection, reference and complete-sample coverage.
- tree.width remains 1..=512: automatic minimum or fixed startup/reset preference.
  Runtime desired columns saturate at native cell limits and actual geometry
  retains the existing tree.max_fraction and small-frame behavior.
- Four canonical tree commands and their local/prefixed defaults are registered;
  the catalogs contain 688 commands and 300 configuration entries.
- Command/configuration/key references, the User Guide Markdown and generated
  Spore projection, and Unreleased describe the usable width controls.

## Files of Interest

- `src/tree/width.rs`, tree listing/root/workspace owners and `src/session/contexts.rs`.
- `src/config.rs`, `src/config/merge.rs` and `src/config/project.rs`.
- `src/command/tree.rs`, `src/session/tree.rs` and `src/input/keymap/defaults.rs`.
- `tests/tree/width.rs`, context transitions, effective hints and resized pointer targets.
- Existing Better File Tree native/PTY acceptance and `docs/reference/file-tree.md`.

## Validation

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

The six width cases passed in 6.27 s, context transitions in 2.40 s, resized
pointer targets in 2.34 s and selector hints in 3.03 s. Full native source policy
passed for 1840 files and 73 Cargo roots; Clippy acceptance remains with the
required lint command and active commit hook.

Better File Tree passed its unchanged native/NO_COLOR/PTY bounds. Actual PTY
journeys took 1.19 s and 1.41 s with zero idle CPU ticks, unchanged source/config
bytes and restored TTY state. Width actions run in the existing PTY per profile;
visual variants retain the native cell/pointer matrix without redundant replay.
Early development runs exposed selector text clipping, sample-map counts, a
Markdown table boundary and excess native action replay; focused corrections
passed without increasing timeouts or weakening existing assertions.

Documentation passed with 106 reachable pages and 743 local links/anchors.
The User Guide built through its canonical Markdown-to-Spore projection.
Cargo execution uses the task-owned same-UID PID namespace runner with a real
PID1 reaper (effective UID 1000), single-job/no-incremental/no-debug settings and
private checkout/hook caches. Runner checksum and local logs remain outside
versioned artifacts. The existing foreign crossterm warning is unchanged.

## Risks and Follow-ups

Horizontal panning and intentional arrow rebinding belong to Slice 2. Physical
font appearance retains the existing native-cell/PTY evidence limit. The
controller owns merge, integrated acceptance and delivery-set closure.
