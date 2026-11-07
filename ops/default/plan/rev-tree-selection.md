# File Tree Selection Delivery

Status: Approved
Date: 2026-11-07
Delivery set: rev-tree-selection
Slice strategy: behavior-first
Approved: product owner; prepare the plan, slicing and tracker and start one Looper delivery
Planning baseline: `fe49403440011e48e5e46b3196394cb0fc39f035`
Specification: [File Tree Selection and Active Buffer](../spec/editor-tree-selection.md)
Model: [Tree Selection Policy](../spec/editor-tree-selection-model.md)
Tracker: [File Tree Selection Tracker](../tracker/rev-tree-selection.md)
Ticket: [TKT-20261031093408](../ticket/in_progress/20261031093408-highlight-tree-selection-and-sync-active-buffer.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 - Selection and following

## Outcome and strategy

One vertical slice delivers persistent selected-row feedback and configurable
following of the active editing buffer. Implement the complete setting, source
identity policy, shared presentation, acceptance and guidance together. The
existing tree/listing, pane and palette owners already supply the needed
boundaries; separate feature and acceptance slices add no useful increment here.

Keep `tree.follow_active_buffer = true` as the startup default. Tree focus gives
manual navigation its own highlight; editing focus follows the active file.
Match only admitted loaded entries, reveal an offscreen match by scrolling,
and preserve folded directories, filtering, root ownership and hidden panes.
No new command, binding, palette field, background worker or state file is added.

## Prerequisites and authority

The existing click preferences, minimal/decorated tree, Project/Workspace roots,
native input profiles and complete command-description catalog are delivered.
The command-description controller must finish its documentation closure before
this preparation is integrated or this controller starts. Keep the two sets'
artifacts and gate evidence separate.

The owner authorizes preparation commits and push on dev, the recorded slice
worktree/branch and canonical PR, controller-owned Rebase + Fast-forward merge
into dev, the selected final gate and documentation closure. Looper is the
execution owner; the slice PR does not wait for another maintainer approval.
Main integration, release, installation and mirror publication remain separate.
Retain installed worker and recovery defaults without resetting budgets or
adding slices. Introduce no dependency or unrelated feature.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Selection and following | `feat/rev-tree-selection-01-follow` | `feat(tree): retain selection and follow the active buffer` | `ops/default/report/slices/rev-tree-selection/slice-1-follow.md` |

## Slice 1: Selection and following

Dependency: approved preparation committed and pushed on clean dev; prior set closed.
Requirements: TSV01-TSV04, configuration/model and existing native navigation.

- T1.1: Move the source ticket ready to in_progress and record its actual start.
  Add the one startup boolean across native defaults, partial admission,
  merging/provenance, the complete sample and direct configuration users. Keep
  browsing selection separate from active file/Workspace route identity. Resolve
  following on real pane, file, context and listing transitions, with no matching
  on repeated drawing or same-file point movement and no extra filesystem reads.
- T1.2: Render the appropriate single highlight after focus changes in both
  tree presentations, using existing palette/monochrome roles. Preserve focused
  cursor/footer ownership, loaded-only reveal, manual browsing/collapse, absent
  targets and checked pointer release. Extend existing native tree, pointer and
  display cases and the overlapping Workspace route case. Cover configuration
  omission/true/false/invalid values and meaningful alias/unavailable cases;
  assert source bytes, dirty state, point, mark and history are unchanged.
- T1.3: Update the tree and configuration references, complete sample, relevant
  User Guide example and Unreleased notes with changelog-normalize; project the
  Markdown into Spore. Introduce the canonical report, record real task/commit
  and PR evidence, and move the ticket to reviewing before gate preparation.
  Keep references aligned as its path changes. Conventional messages omit task
  suffixes; keep task correspondence in the report/tracker.

## Ordered slice checks

For every Cargo command and the guide build below, prepend exactly:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
The local configuration freezes these ten commands in this order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --all-targets -- -D warnings`
3. `cargo test --locked --test tree -- --test-threads=1`
4. `cargo test --locked --test pointer tree -- --test-threads=1`
5. `cargo test --locked --test display tree -- --test-threads=1`
6. `cargo test --locked --test contexts peer_tree_routes -- --test-threads=1`
7. `cargo test --locked --test commands catalog_reference -- --test-threads=1`
8. `cargo run --locked --quiet --example documentation-check`
9. `bash scripts/build-user-guide.sh`
10. `git diff --check`

Extend the full native tree target with the joint following/presentation matrix,
including both input profiles. Retain the existing pointer event and drawing
tests; extend peer_tree_routes for default following and duplicate routes.
Filters must execute their existing meaningful cases, not report zero matches.
The catalog guard protects the unchanged 618-command description contract, and
the documentation checker protects the complete setting catalog and sample.

## Final gate and cost

The controller runs the same ordered checks against the exact integrated clean
dev candidate. Checking all targets is appropriate because the public Tree
configuration shape changes. Runtime tests remain limited to affected tree,
pointer, display, Workspace and catalog contracts; do not append unrelated Git,
Org execution, LSP acquisition or historical delivery journeys.

Expect a few minutes with existing artifacts, plus first-build cost in the
slice's own worktree. Preserve separate build directories; build no release
binary for this set. Reuse native surface/input fixtures instead of creating
new PTY machinery or a per-case executable for this UI-state change.

## Closure

After the controller gate passes, mark the report delivered and the source
ticket solved with resolution fixed, real lifecycle fields and evidence. Mark
this plan, tracker, specification and model Completed, and reconcile the index.
The local configuration includes the exact ready/in_progress/reviewing/solved
ticket paths so its source and destination are authorized. No runtime, test or
published-guide input changes are allowed after the gate.
