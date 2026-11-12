# Tree Viewport Delivery

Status: Approved
Date: 2026-11-12
Delivery set: rev-tree-viewport
Slice strategy: behavior-first
Approved: product owner; proceed with the specification and established delivery workflow
Planning baseline: `6746978285aa7cd5739b704cc0ab214e47e9050c`
Specification: [File Tree Width and Horizontal Navigation](../spec/editor-tree-viewport.md)
Model: [Tree Viewport Policy](../spec/editor-tree-viewport-model.md)
Tracker: [Tree Viewport Tracker](../tracker/rev-tree-viewport.md)
Ticket: [TKT-20261108175339](../ticket/in_progress/20261108175339-improve-file-tree-panel-width-and-horizontal-navigation.md)
Active slice: Slice 2 of 2 - Horizontal navigation

## Outcome and strategy

Two vertical slices deliver the agreed tree viewport. Slice 1 makes width
ownership/configuration and native width controls usable together. Slice 2
adds shared horizontal navigation, preserves directory routes and completes
joint native/PTY acceptance. Each remains independently passing. Reuse existing
tree, keymap, presentation and acceptance owners without a new dependency.

## Prerequisites and authority

The architecture-quality sets are complete and primary dev is clean/synchronized.
The source ticket has completed alignment with no product questions outstanding.
The owner authorizes preparation commits/push on dev, the exact slice branches
and canonical PRs, controller-owned Rebase + Fast-forward merges into dev,
selected acceptance and documentation closure. Looper is the single execution
owner; slice PRs do not wait for another maintainer approval. Main integration,
release, installation and mirror publication remain separate workflows.

Preserve private per-worktree build and hook caches. Installed hooks stay active;
do not reinstall or bypass them. Remove each new clean, inactive worktree after
its HEAD is confirmed integrated into dev, using non-forced removal. Preserve
pre-existing worktrees and unrelated work.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Width policy | `feat/rev-tree-viewport-01-width` | `feat(tree): add stable automatic and manual width controls` | `ops/default/report/slices/rev-tree-viewport/slice-1-width.md` |
| Slice 2 | Horizontal navigation | `feat/rev-tree-viewport-02-horizontal` | `feat(tree): pan the shared horizontal viewport` | `ops/default/report/slices/rev-tree-viewport/slice-2-horizontal.md` |

## Slice 1: Width policy

Requirements: TVP01, TVP02, TVP03 and width-related TVP05, TVP06.
Dependency: approved preparation committed/pushed on clean dev.

- T1.1: Move the ready source ticket to in_progress with its real start time.
  Admit default-on tree.auto_width with complete merge/provenance/catalog/sample
  coverage. Implement event-owned expanded-model measurement, automatic high-water
  mark, geometry clamping, context transitions and retained manual desired width.
- T1.2: Add tree-widen, tree-narrow, tree-reset-width and tree-auto-width-mode
  with descriptions, selectors, effective hints and the approved local/prefixed
  routes in both profiles. Cover reset, mode, fixed configuration, offscreen/folded
  rows, no discovery on drawing/navigation, Unicode and unchanged source state.
  Preserve current arrow directory routes until Slice 2 intentionally replaces them.
- T1.3: Update width/configuration/command/key references, complete sample,
  relevant User Guide Markdown and its Spore projection, and Unreleased through
  changelog-normalize. Extend existing Better File Tree acceptance for usable
  width controls. Produce the canonical report and real task/commit evidence.

## Slice 2: Horizontal navigation

Requirements: TVP04, TVP05, TVP06 and joint acceptance of TVP01 through TVP03.
Dependency: Slice 1 integrated and closed on dev.

- T2.1: Add one shared horizontal viewport, bounded one-column scroll commands
  and glyph-safe clipping. Preserve offset across ordinary navigation/focus/
  visibility, reset on root/context/reset and clamp only excess offset after
  width/content changes. Align measurement with unpanned minimal/decorated rows.
- T2.2: Route focused-tree Left/Right to panning in both profiles, keep Enter
  open/toggle and C-f/C-b expand/collapse/parent. Extend existing native tree,
  configurable key/hint, pointer, display and context cases. Protect active-buffer
  following, stale-hit rejection and source bytes/point/mark/dirty/history identity.
- T2.3: Extend the existing bounded Better File Tree native/PTY journey with
  shallow/deep/long labels, actual arrow and prefix input in both profiles,
  shared alignment, cap/end/beginning, resizing/reset and terminal cleanup.
  Complete references, User Guide/Spore and Unreleased; produce the report and
  move the source ticket to reviewing with actual PR/task evidence.

## Ordered slice checks and final gate

Each slice and the final clean integrated candidate use the following exact
commands in this order. Prefix every command except git diff with:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.

1. `make lint`
2. `cargo test --locked --test tree --test configuration --test commands --test pointer --test display --test contexts -- --test-threads=1`
3. `bash scripts/check-better-file-tree.sh`
4. `cargo run --locked --quiet -p rev --example documentation-check`
5. `bash scripts/build-user-guide.sh`
6. `git diff --check`

Looper freezes these ordered slice checks and their combined final gate before
the first checkpoint. The controller runs the combined gate only after both
PRs integrate. Full strict workspace lint protects shared public configuration
and command shapes; the six selected native targets protect affected tree,
configuration, keymap, pointer, display and Project/Workspace transitions.
The existing native/PTY journey protects executable input and rendering.
Documentation and guide checks protect complete command/configuration catalogs
and both guide sources. Keep command-count/reference expectations synchronized
with delivered commands rather than weakening completeness assertions.

Expect several minutes warm plus a fresh compile in each private worktree.
The previous exhaustive architecture gate took about 61 minutes; this feature
does not change Git execution, Org, LSP, plugin protocol or hook tooling, so
those unrelated journeys and test-hooks are not appended. Record actual costs.
Do not increase timeouts, skip cases, weaken assertions or alter quality policy.

## Execution environment and recovery

Retain repository-required single-job/no-incremental/no-debug settings. Use the
previously verified same-UID PID-namespace Cargo runner with a real PID1 reaper
only through task-owned external tooling when needed by existing process
fixtures. Keep its exact command/checksum and local evidence outside versioned
documents; reports name the mechanism and effective UID without private paths.
Do not create a shared mutable Cargo target. The controller owns validation.

A failed gate is not completion. Diagnose with the smallest affected check;
runtime corrections use fix/rev-tree-viewport-validation and one canonical PR,
then the unchanged selected gate. Native bounded repair/recovery grants preserve
the run, conversation, candidate, gates and budgets. Any new product choice,
dependency, unrelated file authority or service interruption requires separate
authority; do not evade a controller stop by rewriting its checkpoint.

## Closure

After fresh exact-candidate acceptance passes, mark both reports delivered;
move the source ticket reviewing to solved with resolution fixed, actual
lifecycle/commit/PR evidence and consistent links. Mark specification, model,
plan and tracker Completed and reconcile Pending Editor Work. Closure changes
only the configured exact Markdown paths, including both lifecycle ticket paths.
No runtime, test, guide input, command catalog or configuration change follows
the gate. Verify canonical PR state, clean synchronized dev and the final commit.
