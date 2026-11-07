# File Tree Parent Navigation Delivery

Status: Approved
Date: 2026-11-07
Delivery set: rev-tree-parent-navigation
Slice strategy: behavior-first
Approved: product owner requested specification, slicing, plan and Looper
Planning baseline: `6fde63810e95b8e3a0e61b72714696354993d9c0`
Specification: [File Tree Parent Navigation](../spec/editor-tree-parent-navigation.md)
Tracker: [Parent Navigation Tracker](../tracker/rev-tree-parent-navigation.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 of 1, pending controller startup

## Outcome and authority

Deliver the complete upward-navigation interaction in one vertical slice.
Existing native tree, context and command owners supply the boundaries;
splitting root policy, keyboard integration and documentation would leave
parts of this small behavior unusable.

Preparation is committed and pushed on clean dev. The requested workflow
authorizes the recorded slice branch/worktree and canonical PR targeting dev,
Looper-owned Rebase + Fast-forward integration, the selected exact-candidate
gate and documentation-only closure. No additional slice-PR approval is
required. Main promotion, releases, guide deployment, global installation and
mirror publication remain separate.

No durable schema, new configuration, new dependency or model companion is
needed. Preserve automatic file-parent following and all existing fixed roots.
Future sandboxing is not implemented by this set.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Parent navigation | `feat/rev-tree-parent-navigation-01-parent` | `feat(tree): navigate to unanchored parent directories` | `ops/default/report/slices/rev-tree-parent-navigation/slice-1-parent.md` |

## Slice 1: Parent navigation

Dependency: approved preparation published; clean synchronized dev, no other
controller owner, sufficient build capacity.
Requirements: TUP01-TUP04.

- T1.1: Add native root eligibility and transactional parent observation.
  Preserve explicit directory, Project, Workspace and cwd roots; retain the
  previous state on errors. Select and reveal the directory just left.
  Reuse existing traversal limits and path/symlink policies.
- T1.2: Register tree-up-directory, its complete description and tree-local ^
  binding in both profiles. Extend existing native tree and command fixtures
  for automatic versus fixed roots, repeated upward movement, filesystem root,
  failures, selection/scroll, remapping and source/view immutability. Include
  a meaningful command acceptance test matching tree_up_directory. Reuse
  fixtures; no new test harness or external service is required.
  Check the decorated Nerd Fonts projection after upward navigation, including
  existing icons, semantic colors, connectors, selected rows and narrow widths.
- T1.3: Update the file-tree reference, default binding/command references and
  live command-description inventory/counts. Preserve historical delivery
  claims. Add brief guidance to the file-tree User Guide and rebuild its
  Markdown-derived Spore counterpart. Use changelog-normalize for the completed
  Unreleased behavior. Introduce the canonical report with task, commit, PR
  and validation evidence; reconcile current baseline guidance where needed.

Use Conventional Commits without task-ID suffixes. Record task correspondence
in the tracker/report. Public PRs use Summary, Changes and Validation in that
order; include the immutable report link as the last Changes bullet.
Do not announce routine changelog maintenance as a product change.

## Ordered checks

Prepend every Cargo command and the guide build with:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
Freeze these nine checks in this order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --lib --bin rev -- -D warnings`
3. `cargo test --locked --test tree -- --test-threads=1`
4. `cargo test --locked --test files directories -- --test-threads=1`
5. `cargo test --locked --test commands -- tree_up_directory catalog_reference description tree_actions tree_transitions tree_open_sequence --test-threads=1`
6. `cargo test --locked --test pointer tree -- --test-threads=1`
7. `cargo run --locked --quiet --example documentation-check`
8. `bash scripts/build-user-guide.sh`
9. `git diff --check`

Each named acceptance filter must execute real tests. Tree tests must cover
all fixed-root variants, not merely directory roots. Native command fixtures
exercise actual input and named routing; pointer checks protect selected-entry
and scroll behavior. Existing unrelated command-suite debt is outside this
set; do not silently expand scope to repair it or weaken assertions.

## Integrated gate and execution cost

The controller runs the same nine ordered checks on the exact clean integrated
dev candidate. Focused native tests protect root/context, command, selection
and source boundaries. Lib/bin Clippy covers changed runtime owners; selected
test binaries compile and execute their own contracts. Do not append historical
delivery journeys or an unrelated exhaustive suite.

Allow several minutes for the isolated worktree's first dependency build.
Use one Cargo job, no debug information and no incremental output; preserve
separate mutable target directories. Do not build a release binary or remove
source, uncertain worktrees or another active build to obtain capacity.

## Gate and closure

Before gate preparation the slice PR is canonically integrated and its report
is delivered. The controller records the unchanged selected gate against exact
clean dev. Only after it passes may closure mark the spec, plan and tracker
Completed and reconcile the index/report with actual commit, PR and receipt
evidence.

The exact closure allowlist is the index, this plan, its tracker, its
specification and its one canonical report. Closure must not modify runtime,
tests, requirements, selected command definitions, user-guide inputs or gate
commands. Preserve retained run identity and consumed budgets during recovery.
