# Slice 4: Acceptance and guidance

Status: delivered
Delivery set: rev-prefix-arguments
Plan: [`ops/default/plan/rev-prefix-arguments.md`](../../../plan/rev-prefix-arguments.md)
Tracker: [`ops/default/tracker/rev-prefix-arguments.md`](../../../tracker/rev-prefix-arguments.md)
Branch: `test/rev-prefix-arguments-04-acceptance`
PR: [#317](https://forge.adrianpk.com/adrianpk/rev/pulls/317)
Integrated: `98d4cc896ed0662c01a648745c554f187f569db4`

## Purpose

Complete T4.1 joint native acceptance, T4.2 reader guidance and discovery
reconciliation, and T4.3 reviewable evidence for the unchanged selected checks.
Canonical PR #317 is integrated through Rebase + Fast-forward. The integrated
delivery-set gate and documentation-only closure remain controller-owned and
pending.

## Delivered Behavior

- Verify the literal first User Guide gesture, C-u 5 C-n, counted insertion and
  named direct goto-line through both default non-modal and modal Insert input.
  One-shot commands with zero or positive arguments execute once and consume
  the value before a later native movement.
- Verify the exact 8 MiB generated UTF-8 boundary as one native edit/revision and
  undo group. A product just beyond it leaves text, point, mark, history and redo
  unchanged. Existing million-count, cancellation, stale-source and idle cases
  remain selected.
- Distinguish ordinary visual kill-line from explicit logical line counts on a
  genuinely wrapped source line. Verify the literal third-entry yank binding,
  backward kill-chain prepending and counted native undo/redo in both profiles.
  Newline/tab zero and negative admission are covered in both entry profiles.
- Retain raw plain-prefix/integer/negative-only forms in the native invocation
  contract. A fresh input owner starts absent, with no periodic prefix deadline.
  All 628 compiled command descriptions are present, nonempty and single-line;
  the universal alias keeps the canonical helper's description.
- Add a short guided movement/insertion exercise, named direct goto-line example
  and focused task guide. Keep raw-form distinctions, the complete command
  policies, input ownership and resource limits in reference material.
- Reconcile current bindings, packaged/reference defaults and description
  discovery without adding a preference or persistence shape. Markdown remains
  the source for the local Spore projection; no site is published.
- Close Slice 3 records after verified PR #316 integration and reconcile earlier
  reports' follow-up status without replacing their historical validation.

## Acceptance Coverage

| Requirement or policy | Native acceptance |
| --- | --- |
| ARG01 forms, bounds, sign, termination and cancellation | commands entry_forms, entry_bounds, digit_termination, counted_text_transfer and transient_reuse |
| ARG02 M-x transport, candidate edits and usage once | commands named_helpers, named_transfer, usage_identity and counted_edit_selection |
| Character/word/row movement and line positioning | commands signed_movement, counted_rows, positioned_rows and guide_journey |
| Buffer fractions, goto-line, row pages and recenter | commands absolute_positions, supplied_scrolling and supplied_recenter |
| Loaded tree rows and contextual aliases | commands loaded_tree_rows plus retained tree actions/transitions/open sequence |
| Literal/paste, native newline/tab, regions and signed deletion | editing literal_regions, literal_groups, insertion_policy, insertion_refusal, signed_deletion and deletion_regions |
| Logical and directional kills, selected yank and signed rotation | editing logical_kills, wrapped_line_count, word_chains, integrated_ring, ring_arguments and yank_orientation |
| Native history grouping, boundaries and progressive cancellation | editing history_groups and cooperative_edits plus retained full editing history cases |
| One-shot default for undeclared policies | commands one_shot_fallback, including a supplied zero |
| ARG04 existing continuations, modal counts/Escape and non-source owners | commands effective_bindings, existing_continuations, contextual_owners and terminal_ownership; four modal prefix cases and three real shell cases |
| ARG05 cooperative plans, source invalidation, limits and idle release | commands deferred_movement, repeat_and_cancel, contextual_bounds; editing byte_boundary, insertion_admission, cooperative_edits and stale_preparation; modal cooperative_cancel |
| ARG06 complete descriptions, alias identity, remapping and transient reuse | commands discovery_inventory, effective_bindings, usage_identity and transient_reuse; live documentation/catalog validation |

The selected wrapping target retains its unmodified visual/Org regression
assertions. Real shell and PTY cases use the existing isolated native fixture
owners and stop only their own children after checks. There is no provider,
network service, replacement test harness, release build or new dependency.

## Implementation Notes

No runtime policy changes are needed for this slice. T4.1 extends the existing
commands and editing fixtures; earlier modal, shell, source/history, wrapping and
ownership cases remain in the selected targets. T4.2 updates the existing User
Guide path and adds one supporting how-to page, with complete reference links.
It does not add a parallel guide source or edit generated publication files.

Commit `f6c7f72c47bfb0d4e48b4e6bc94cd4806ef1e27c` implements T4.1.
Commit `87d715d7f0ccd0f7da3022f1891adb8708646a97` implements T4.2.
Commit `db092556ac056d06b93f341e4cd93da6f32d9515` introduces this report
and completes T4.3 tracker/report reconciliation. The final PR-reference commit
`98d4cc896ed0662c01a648745c554f187f569db4` passed the ten unchanged ordered
checks through the controller's exact-HEAD validation command before integration.

## Canonical Integration Evidence

- Slice 1: PR [#314](https://forge.adrianpk.com/adrianpk/rev/pulls/314),
  `c558786beb1925902b749dd89f3a2e09810a5412`.
- Slice 2: PR [#315](https://forge.adrianpk.com/adrianpk/rev/pulls/315),
  `57afa0a31ffcb56ee9bff6da0135e7e75a628248`.
- Slice 3: PR [#316](https://forge.adrianpk.com/adrianpk/rev/pulls/316),
  `83d283bee834bdc428e48d3390af5adcf2d01d91`.
- Slice 4: PR [#317](https://forge.adrianpk.com/adrianpk/rev/pulls/317),
  `98d4cc896ed0662c01a648745c554f187f569db4`.

Canonical Forgejo state and Acta record all four integrations; their heads are
ancestors of primary dev and all reports are delivered. Each canonical PR links
the immutable commit that introduced its recorded report. The PR checks remain
evidence for their exact slice candidates; the controller must run the selected
integrated gate on clean dev after gate-preparation documentation is committed.

## Files of Interest

- `tests/commands/prefix_arguments.rs`, `tests/editing/prefix_arguments.rs`:
  eight added native acceptance cases and retained joint policy coverage.
- `docs/tutorials/user-guide/{02-edit-text,04-search-and-commands,index}.md`:
  the short guided exercise and direct named invocation.
- `docs/how-to/use-prefix-arguments.md`, `docs/how-to/index.md`: focused tasks.
- `docs/reference/{commands,defaults,index}.md`: forms, policies and discovery.
- `CHANGELOG.md`: one cohesive user-visible outcome with its task guide.
- Delivery tracker and reports: actual integration and task/evidence status.

## Validation

These are the frozen ten-command Slice 4 PR checks in canonical order. The
controller receipt binds their execution to the clean PR-reference HEAD. The
integrated primary-dev gate and delivery-set closure remain pending.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- Passed, 36 tests including 28 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments context_isolation transient_isolation prompt_reset workspace_reset tree_up_directory tree_actions tree_transitions tree_open_sequence --test-threads=1`
- Passed, 114 tests including 18 prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1`
- Passed, 26 tests: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test wrapping -- --test-threads=1`
- Passed, 28 tests including four prefix cases: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- count operator navigation prefix_arguments --test-threads=1`
- Passed, three real shell tests across both profiles: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test shell_commands prefix_arguments -- --test-threads=1`
- Passed, 628 commands, 291 settings, 92 pages and 638 links: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- Passed, strict local projection/render of 92 Markdown pages: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- Passed: `git diff --check`

## Risks and Follow-ups

Cancellation of native history is progressive and retains completed groups.
All four canonical integrations and report identities are verified. The
controller must run the unchanged selected gate on exact clean dev and perform
the approved Markdown-only closure after it passes. Main promotion, releases,
guide deployment, binary installation and successor startup are outside this
worker turn.
