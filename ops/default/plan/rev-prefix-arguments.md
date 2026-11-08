# Command Prefix Arguments Delivery

Status: Approved
Date: 2026-11-08
Delivery set: rev-prefix-arguments
Slice strategy: behavior-first
Approved: product owner approved the specification and requested plan/slicing
Execution authorized: product owner requested continuation and one Looper run on 2026-11-08
Planning baseline: `2bde2d1b1cccdaf536c17fbd86938eb42cbfc896`
Specification: [Command Prefix Arguments](../spec/editor-prefix-arguments.md)
Tracker: [Command Prefix Arguments Tracker](../tracker/rev-prefix-arguments.md)
Concern: [Pending Editor Work](../README.md)
Active slice: all four slices integrated; controller-owned integrated gate pending

## Outcome and strategy

Deliver a native command-argument facility before keyboard macros. Four vertical
increments supply usable movement, editing, contextual behavior and joint
acceptance/guidance. No slice consists solely of inert scaffolding. Existing
keymap, input, command, layout, source edit, kill-ring and history owners remain
authoritative; no new dependency, persistent store or general repeat interpreter
is introduced.

The complete specification owns every command policy. Until a later slice
delivers a newly declared interpretation, other commands keep the specified
one-shot fallback. Existing C-u and modal behavior must work throughout; no
slice may knowingly break those routes until another slice repairs them.

## Prerequisites and authority

The primary checkout is clean dev synchronized with origin/dev; current main
contains the completed tree-navigation delivery and parent-row follow-up.
No other delivery may own the repository. Resolve ordinary Forgejo/provider
access and available build space before controller startup.

The approved scope covers these exact branches, canonical PRs to dev, focused
checks, eventual selected integration gate and documentation closure.
The owner has now authorized startup of one four-slice Looper run. The selected
controller owns its slice reviews and Rebase + Fast-forward integration through
dev; it does not need separate owner approval of each slice PR. It also owns
the unchanged selected integration gate and documentation-only closure.
Main promotion, release, public guide deployment and global binary installation
remain separate.

Use Conventional Commits without task-ID suffixes. Record task correspondence
in reports and the tracker. Preserve retained controller identity, conversation,
consumed budgets and gate definitions during recovery.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Prefix entry and movement | `feat/rev-prefix-arguments-01-entry` | `feat(input): add prefix arguments and counted movement` | `ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md` |
| Slice 2 | Counted edits and history | `feat/rev-prefix-arguments-02-editing` | `feat(editing): apply prefix arguments to native edits and history` | `ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md` |
| Slice 3 | Contextual argument behavior | `feat/rev-prefix-arguments-03-contexts` | `feat(input): integrate contextual prefix argument behavior` | `ops/default/report/slices/rev-prefix-arguments/slice-3-contexts.md` |
| Slice 4 | Acceptance and guidance | `test/rev-prefix-arguments-04-acceptance` | `test(input): verify prefix argument workflows and document usage` | `ops/default/report/slices/rev-prefix-arguments/slice-4-acceptance.md` |

## Slice 1: Prefix entry and movement

Dependency: approved preparation committed/pushed and execution authorized.
Requirements: ARG01, ARG02, movement portion of ARG03, compatibility portion of ARG04 and ARG06.

- T1.1: Define transient argument forms, canonical helpers/alias, complete descriptions, effective bindings and checked entry/cancellation.
- T1.2: Carry arguments through native input and M-x; deliver signed character/word/vertical movement with ordinary wrapping units and preserve existing C-u routes.
- T1.3: Add prefix_arguments native cases, update current catalogs and incremental references, and introduce the canonical report.

Focused validation: formatting; lib/bin Clippy; commands prefix_arguments plus existing transient/prompt reset cases; wrapping; documentation/catalog check.

## Slice 2: Counted edits and history

Dependency: Slice 1 canonically integrated.
Requirements: editing/kill/history portion of ARG03 and atomic admission in ARG05.

- T2.1: Implement counted literal/newline/tab/deletion preparation, region precedence, generated-byte admission and one native undo unit.
- T2.2: Implement logical-line/word kill spans, earlier-entry yank, signed yank rotation and bounded native undo/redo groups.
- T2.3: Extend real native editing/ring/history fixtures and prefix_arguments command acceptance; update the report and completed user-visible Unreleased outcome.

Focused validation: formatting; lib/bin Clippy; commands prefix_arguments; complete editing target; documentation/catalog check.

## Slice 3: Contextual argument behavior

Dependency: Slice 2 canonically integrated.
Requirements: remaining ARG03 and ARG04, cooperative/resource behavior in ARG05.

- T3.1: Complete line/buffer positioning, row-based supplied scrolling, recentering, direct goto-line and loaded tree-row counts.
- T3.2: Complete named shell variants, C-u continuation/remapping precedence, input-owner admission and modal count conflict/Escape behavior.
- T3.3: Verify cooperative work, cancellation, revision/owner invalidation, resource bounds and idle behavior through native command/modal/shell fixtures and record the report.

Focused validation: formatting; lib/bin Clippy; commands prefix_arguments and tree routes; modal count/operator/navigation regressions; shell_commands prefix_arguments.

## Slice 4: Acceptance and guidance

Dependency: Slice 3 canonically integrated.
Requirements: joint ARG01-ARG06 acceptance and complete guidance.

- T4.1: Complete integrated native acceptance for every command policy and both-profile ownership transitions, with representative large-count and cancellation cases.
- T4.2: Reconcile complete command/binding/default references and description inventory; add concise User Guide/how-to/reference guidance and build the Markdown-derived Spore guide.
- T4.3: Finalize reports and task evidence, verify clean canonical integration and prepare the unchanged selected controller gate; closure remains separate.

Focused validation: the complete frozen ordered selection below, on its clean PR HEAD before integrated controller validation.

## Ordered integrated gate

Prepend every Cargo command and the guide build with:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`.
Freeze these ten commands in this order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --lib --bin rev -- -D warnings`
3. `cargo test --locked --test commands -- prefix_arguments context_isolation transient_isolation prompt_reset workspace_reset tree_up_directory tree_actions tree_transitions tree_open_sequence --test-threads=1`
4. `cargo test --locked --test editing -- --test-threads=1`
5. `cargo test --locked --test wrapping -- --test-threads=1`
6. `cargo test --locked --test modal -- count operator navigation prefix_arguments --test-threads=1`
7. `cargo test --locked --test shell_commands prefix_arguments -- --test-threads=1`
8. `cargo run --locked --quiet --example documentation-check`
9. `bash scripts/build-user-guide.sh`
10. `git diff --check`

Slice checks are the stated focused subsets, with git diff --check last.
Their exact ordered selections from the numbered commands above are:

| Slice | Required command numbers, in order |
| --- | --- |
| Slice 1 | 1, 2, 3, 5, 8, 10 |
| Slice 2 | 1, 2, 3, 4, 8, 10 |
| Slice 3 | 1, 2, 3, 6, 7, 8, 10 |
| Slice 4 | 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 |

Create a prefix_arguments module in the existing commands target. Add
prefix_arguments cases to existing modal and shell_commands targets so every
selected target/filter runs real acceptance. Use existing fixture owners and
real isolated source/filesystem/shell boundaries, not provider services or
a replacement test framework. At least one real shell prefix journey is needed;
there is no live AI/LSP/provider acceptance dependency.

Protect core input ownership, source/history, wrapped navigation and modal
counts without appending every historical suite. Known unrelated broad
command-suite expectation debt is outside the selected gate and must not be
silently repaired or used to weaken a selected assertion.

## Execution cost and resources

Expect several minutes for the combined selected gate, plus a first dependency
build in each isolated slice worktree. Existing commands, editing, wrapping,
modal and shell fixtures provide the runtime coverage. Keep normal slice checks
focused; run the combined integrated gate only after the four slices integrate.

Use one Cargo job, disabled incremental output and no debug information.
Retain separate mutable target directories and avoid release builds.
Do not interrupt another build or remove source/uncertain worktrees for space.
The 1,000,000 numeric ceiling, 8 MiB generated-text bound and cooperative
execution behavior belong to the approved specification, not controller retries.

## Reports, gate preparation and closure

Each slice introduces its exact report through an immutable commit and links
it as the last Changes bullet of its canonical PR. Use Summary, Changes and
Validation in that order; git diff --check is the final Validation item.
Record only successful commands that actually ran.

Before the integrated gate, all four PRs are canonically integrated and all
reports delivered. The controller binds its unchanged selected gate to exact
clean dev. Only after it passes may closure mark the specification, plan and
tracker Completed and reconcile the index and reports with actual evidence.

The exact closure allowlist is these eight Markdown files:

- ops/default/README.md
- ops/default/spec/editor-prefix-arguments.md
- ops/default/plan/rev-prefix-arguments.md
- ops/default/tracker/rev-prefix-arguments.md
- ops/default/report/slices/rev-prefix-arguments/slice-1-entry.md
- ops/default/report/slices/rev-prefix-arguments/slice-2-editing.md
- ops/default/report/slices/rev-prefix-arguments/slice-3-contexts.md
- ops/default/report/slices/rev-prefix-arguments/slice-4-acceptance.md

Closure cannot change runtime, tests, requirements, command policies, guide
inputs or gate definitions. Macro drafts and their persistence model remain
outside this delivery and outside its closure allowlist.
