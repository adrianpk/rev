# Keyboard Macros Delivery

Status: Approved
Date: 2026-11-08
Delivery set: rev-keyboard-macros
Slice strategy: behavior-first
Approved: owner requested spec/model, plan, slicing and one Looper run; delegated remaining reference-compatible decisions
Planning baseline: c86c11383f7de33d56dec17422d87ca2fbfcdc6f
Specification: [Keyboard Macros](../spec/editor-keyboard-macros.md)
Model: [Macro Library and State Ownership](../spec/editor-keyboard-macros-model.md)
Tracker: [Keyboard Macros Tracker](../tracker/rev-keyboard-macros.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 4 of 8, queries, regions and nested execution

## Outcome, prerequisites and authority

Deliver KMA01-KMA09 and the model through eight behavior increments in one
controller run. Each slice adds usable native behavior and remains independently
valid; no inert skeleton or knowingly failing intermediary is required. Until a
later slice introduces a declared surface, unsupported macro operations stop
explicitly rather than pretending to record them. No new runtime dependency,
embedded language or generic state framework is authorized.

Command Prefix Arguments and its four-slice gate/closure are complete. Main/dev
are aligned at the planning baseline. Require clean synchronized dev, no competing
repository owner, available build capacity and working forge/provider access.

The owner authorizes the exact slice branches, canonical PRs, controller review
and Rebase + Fast-forward integration into dev, the unchanged selected integrated
gate and documentation-only closure. No separate owner PR approval is required
for these slice PRs. Main promotion, releases, mirror publication, deployed guide
publication and a later global binary installation retain separate boundaries.

Use Conventional Commits without task-ID suffixes. Reports/tracker own task
correspondence. Looper alone owns live progress; do not start a parallel worker or
write a second runtime tracker. Preserve run identity, conversation, consumed
budgets and frozen checks during recovery.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Recording and replay | feat/rev-keyboard-macros-01-recording | feat(macros): record and replay native editing input | ops/default/report/slices/rev-keyboard-macros/slice-1-recording.md |
| Slice 2 | Named commands and prompt capture | feat/rev-keyboard-macros-02-capture | feat(macros): preserve named commands and accepted prompt input | ops/default/report/slices/rev-keyboard-macros/slice-2-capture.md |
| Slice 3 | Ring, append and counters | feat/rev-keyboard-macros-03-ring | feat(macros): add ring workflows append and counters | ops/default/report/slices/rev-keyboard-macros/slice-3-ring.md |
| Slice 4 | Queries, regions and nested execution | feat/rev-keyboard-macros-04-queries | feat(macros): add query pauses and checked region execution | ops/default/report/slices/rev-keyboard-macros/slice-4-queries.md |
| Slice 5 | Named snapshots and key bindings | feat/rev-keyboard-macros-05-names | feat(macros): expose named snapshots and session bindings | ops/default/report/slices/rev-keyboard-macros/slice-5-names.md |
| Slice 6 | Explicit persistent library | feat/rev-keyboard-macros-06-library | feat(macros): persist explicitly saved definitions as TOML | ops/default/report/slices/rev-keyboard-macros/slice-6-library.md |
| Slice 7 | Definition editing and macro menu | feat/rev-keyboard-macros-07-editing | feat(macros): edit definitions and manage the macro ring | ops/default/report/slices/rev-keyboard-macros/slice-7-editing.md |
| Slice 8 | Acceptance and guidance | test/rev-keyboard-macros-08-acceptance | test(macros): verify complete workflows and document usage | ops/default/report/slices/rev-keyboard-macros/slice-8-acceptance.md |

## Slice tasks and focused acceptance

### Slice 1: Recording and replay

Dependency: approved preparation committed/pushed and controller preflight.

- T1.1: Add bounded recording/runner ownership, normalized key/text capture, F3/F4 and C-x controls, indicators, cancellation and positive prefix replay through normal input.
- T1.2: Deliver native per-iteration undo boundaries, typed stops, modal entry checks and cooperative scheduling without an idle worker; unsupported input transfers stop explicitly.
- T1.3: Introduce the native macros integration target and recording/runner fixtures; align command/default catalogs and record the first usable Unreleased outcome.

Focused checks: formatting, lib/bin Clippy, macros filters recording runner, numbered regressions 4, 5, documentation/catalog check and whitespace.

### Slice 2: Named commands and prompt capture

Dependency: Slice 1 canonically integrated.

- T2.1: Normalize accepted M-x/colon invocations to one canonical name with typed prefix and role-qualified prompt values; preserve ordinary-key current-map semantics.
- T2.2: Integrate search, paths/buffer arguments, literal completion contracts, fresh-consent pauses and typed stop/wait observation without replaying selector rows or echo parsing.
- T2.3: Verify both profiles, changed ranking/bindings, provider-independent completion, missing names, unsupported jobs and exactly-once effects in native capture fixtures.

Focused checks: formatting, lib/bin Clippy, macros filters capture, numbered regressions 4, 7, documentation/catalog check and whitespace.

### Slice 3: Ring, append and counters

Dependency: Slice 2 canonically integrated.

- T3.1: Deliver the global bounded ring, rotate/swap/delete/second-call actions, temporary repeat maps and append variants with independent named-snapshot semantics.
- T3.2: Implement current/previous/iteration-start counters, checked operations, captured formats and the two fully documented macro configuration leaves.
- T3.3: Verify counter formats/overflow, repeated controls, append failures, history grouping, bounds and catalog/default consistency through ring/counter fixtures.

Focused checks: formatting, lib/bin Clippy, macros filters ring counters, numbered regressions 4, 5, documentation/catalog check and whitespace.

### Slice 4: Queries, regions and nested execution

Dependency: Slice 3 canonically integrated.

- T4.1: Implement query continue/skip/stop choices and bounded native manual-edit pause/resume/abort, excluding manual input from recording.
- T4.2: Apply definitions to checked original logical region-line anchors and enforce shared run budgets across region iterations and admitted manual pauses.
- T4.3: Validate Unicode/wrapping, line insert/delete remapping, cancellation/partial history, source invalidation, pause ownership and idle release in native fixtures.

Focused checks: formatting, lib/bin Clippy, macros filters queries regions runner, numbered regressions 4, 5, 6, documentation/catalog check and whitespace.

### Slice 5: Named snapshots and key bindings

Dependency: Slice 4 canonically integrated.

- T5.1: Admit bounded independent named snapshots as native dynamic commands with canonical identities, optional descriptions, M-x/colon ranking and effective key hints.
- T5.2: Implement naming/replacement and session key capture, reserved slots, consent and collision precedence while preserving compiled, prompt, AI and user-configured owners.
- T5.3: Verify both profiles, pinned nested named calls, cycle/depth rejection, missing definitions, namespace/binding collisions, session-only naming and command-history behavior through names fixtures.

Focused checks: formatting, lib/bin Clippy, macros filters names, numbered regressions 4, 6, documentation/catalog check and whitespace.

### Slice 6: Explicit persistent library

Dependency: Slice 5 canonically integrated.

- T6.1: Implement schema-1 library encoding/admission and private concern-owned state publication with exact limits, nonblocking locks, revisions and fresh-record concurrency.
- T6.2: Deliver explicit save/reload/delete and inert TOML export, startup admission, optional prefixed binding inclusion and preservation of unsaved/conflicting state.
- T6.3: Verify restart, absent/invalid/oversized/symlink state, unrelated concurrent writes, stale same-record conflicts, uncertain publication and unchanged sibling stores.

Focused checks: formatting, lib/bin Clippy, macros filters library names, numbered regressions 4, documentation/catalog check and whitespace.

### Slice 7: Definition editing and macro menu

Dependency: Slice 6 canonically integrated.

- T7.1: Deliver checked TOML definition buffers, key/region helpers, 300-input lossage projection and target revision checks without implicit saving.
- T7.2: Implement bounded step editing and the read-only macro menu with stable rows, property edits, marks/flags/copy/delete confirmation and named contextual actions.
- T7.3: Verify cancelled/stale/invalid editor drafts, step changes and retained source effects, menu targeting, all local actions, both-profile focus and complete descriptions.

Focused checks: formatting, lib/bin Clippy, macros filters definition_editor step_edit menu lossage, numbered regressions 4, 5, 6, documentation/catalog check and whitespace.

### Slice 8: Acceptance and guidance

Dependency: Slice 7 canonically integrated.

- T8.1: Complete integrated KMA01-KMA09 acceptance, representative real PTY function-key/C-x journeys, restart and concurrent storage boundaries plus large-buffer cancellation/idle probes.
- T8.2: Add concise User Guide and how-to paths, complete command/default/state references and explanation; keep Markdown-derived Spore and the complete sample configuration aligned.
- T8.3: Finalize all reports and task evidence, reconcile catalogs and Unreleased outcomes, and prepare the unchanged selected integrated controller gate and documentation-only closure.

Focused checks: all ten unchanged integrated gate commands below.

## Frozen ordered integrated gate

Prepend commands 1-9 with this exact environment; do not prefix git:

    env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1

Freeze the following selection in this order:

1. cargo fmt --all -- --check
2. cargo clippy --locked --lib --bin rev -- -D warnings
3. cargo test --locked --test macros -- --test-threads=1
4. cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1
5. cargo test --locked --test editing -- --test-threads=1
6. cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1
7. cargo test --locked --test completion -- manual_edit cancel stale --test-threads=1
8. cargo run --locked --quiet --example documentation-check
9. bash scripts/build-user-guide.sh
10. git diff --check

For each earlier slice, its exact ordered required_commands are commands 1 and 2,
then the macro command below, then its numbered regressions in order, command 8
and command 10. Slice 8 uses all ten integrated commands.

| Slice | Macro command before the numbered regressions | Numbered regressions |
| --- | --- | --- |
| 1 | cargo test --locked --test macros -- recording runner --test-threads=1 | 4, 5 |
| 2 | cargo test --locked --test macros -- capture --test-threads=1 | 4, 7 |
| 3 | cargo test --locked --test macros -- ring counters --test-threads=1 | 4, 5 |
| 4 | cargo test --locked --test macros -- queries regions runner --test-threads=1 | 4, 5, 6 |
| 5 | cargo test --locked --test macros -- names --test-threads=1 | 4, 6 |
| 6 | cargo test --locked --test macros -- library names --test-threads=1 | 4 |
| 7 | cargo test --locked --test macros -- definition_editor step_edit menu lossage --test-threads=1 | 4, 5, 6 |

The new macros integration target uses the repository-native Rust framework,
existing isolated configuration/source/storage fixture patterns and real native
Session/input/history owners. Filters must select real introduced cases, never
zero-test successes. Keep integration assertions behavior-oriented with short
human test names, explanatory comments and parameterized cases where useful.

The final macros target includes real PTY journeys using the Cargo-built binary
(CARGO_BIN_EXE_rev), existing rustix terminal primitives and finite deadlines.
Reuse owned native helpers before adding infrastructure. No Python, external
macro runtime or provider service is a clone/build/test prerequisite. Completion
acceptance needs no live language server; unsupported asynchronous-job admission
is verified without launching provider jobs.

The selected gate protects source/history, shared input, modal counts, completion,
dynamic command identities and state publication. Do not append every historical
journey or repair unrelated command-catalog expectation debt. Existing broad-suite
context_hints, terminal_hints and org_catalog native_catalog issues are outside
this selection unless a changed contract genuinely requires addressing one.

## Engineering ownership and cost

Use existing input/Session, command registry, edit/history, windows/source anchors,
completion and Roots/persistence owners. Add a cohesive native macro owner and
small typed observations for step completion/wait/stop/consent. Do not parse echo
strings, synthesize terminal escapes for internal replay, or build a second
editing dispatcher or general plugin registry. Preserve public native behavior.

Expect several minutes for the selected integrated gate plus initial dependency
compilation per owned worktree. Keep slice regression selections focused. Use
one Cargo job, disabled incremental output and no debug information as above;
never run release builds for this delivery. Preserve unrelated active processes.

Build space is finite. A worker may clean only its own generated target output
after successful recorded checks, all test/probe children have exited and that
output is no longer needed by a current process. Use Cargo's owned clean command
and preserve reports/controller receipts outside target. Never clean primary
checkout/global binaries, unrelated targets, source or uncertain worktrees.
Do not silently increase runtime bounds, controller budgets or deadlines.

## Gate preparation and closure

Each slice introduces its canonical report in an immutable commit. Its PR uses
Summary, Changes and Validation, with the immutable report as the final Changes
item and git diff --check as the final Validation item. Record only commands that
actually passed. Keep Unreleased aligned with delivered product outcomes using
the changelog skill, not internal scaffolding.

After all eight PRs integrate, reconcile delivered reports and prepare the exact
clean dev candidate. Only the controller executes the unchanged integrated gate.
After it passes, closure may mark spec/model/plan/tracker Completed, reconcile the
index and record actual evidence in reports. Closure may change exactly these
thirteen Markdown paths:

- ops/default/README.md
- ops/default/spec/editor-keyboard-macros.md
- ops/default/spec/editor-keyboard-macros-model.md
- ops/default/plan/rev-keyboard-macros.md
- ops/default/tracker/rev-keyboard-macros.md
- ops/default/report/slices/rev-keyboard-macros/slice-1-recording.md
- ops/default/report/slices/rev-keyboard-macros/slice-2-capture.md
- ops/default/report/slices/rev-keyboard-macros/slice-3-ring.md
- ops/default/report/slices/rev-keyboard-macros/slice-4-queries.md
- ops/default/report/slices/rev-keyboard-macros/slice-5-names.md
- ops/default/report/slices/rev-keyboard-macros/slice-6-library.md
- ops/default/report/slices/rev-keyboard-macros/slice-7-editing.md
- ops/default/report/slices/rev-keyboard-macros/slice-8-acceptance.md

No runtime, tests, guide inputs, requirements, model data contracts, command
semantics, limits, checks or gate definitions may change in closure. An actual
integration correction follows the retained controller's supported recovery
authority; it does not justify unapproved side fixes or weakened assertions.
