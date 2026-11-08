# Slice 4: Queries, regions and nested execution

Status: delivered
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-04-queries`
PR: [#322](https://forge.adrianpk.com/adrianpk/rev/pulls/322)
Integrated: f996b57938532efce6a7735715dc901b39efcce9 on dev through canonical PR #322
Implementation: [f421903f80a6](https://forge.adrianpk.com/adrianpk/rev/commit/f421903f80a606c529190e45cd51ccdecbbabdc6)

## Purpose

Deliver T4.1-T4.3: native query decisions and manual editing pauses, original
region-line execution and shared outer limits across admitted nested runs.

## Delivered Behavior

- C-x q records a query marker without interrupting initial recording. During
  replay, Space/y continues, Backspace/n skips this iteration's remaining steps,
  Enter/q ends the remaining run and C-r enters manual editing.
- A supplied query prefix enters an unconditional manual editing pause during
  both recording and playback. Manual input is excluded from capture. C-M-c
  resumes execution or returns to the same confirmation query; C-] or C-g aborts
  while retaining real source effects. Native modal editing stays with its owner.
- Pauses show their depth, retain source/view/profile ownership and schedule no
  macro polling deadline. Four suspended frames are admitted; exceeding that
  bound or losing the checked source aborts the owned continuation.
- Manual playback edits and user-entered nested runs retain the outer
  iteration's native undo group and share its work, iteration, generated-text
  and active-time budgets. Explicit query, consent and manual wait time is
  excluded. Counter edits address the suspended working counter state.
- C-x C-k r runs once at each original logical line beginning in the selected
  half-open region, up to 65,536 targets. A next-line endpoint excludes that
  line. Display wrapping adds no targets. Inserted lines add none, deleted
  targets are skipped and partial character replacements retain their line.
  Each region iteration keeps its native undo unit and shares the outer limits.

## Implementation Notes

Native input owns typed query answers and confirmation phases, including
release/repeat suppression. The runner stores only confirm/edit markers.
Suspending capture moves its recorder out of the active input path; manual keys,
prompt values and source edits cannot enter the reusable definition.

The macro owner retains bounded manual frames and pins each definition. Source
point/revision checks distinguish admitted manual edits from external changes.
A shared budget follows nested manual replays, with a paused clock and one text
allowance. Native automatic groups keep partial, manual and resumed edits
undoable without rolling back source effects.

The canonical buffer edit owner prepares lightweight original-line anchor maps,
rechecks active owners before publication and publishes them with source state.
Anchors track source line identity through insertion, deletion and native retained
permutations. They carry no disk-reopening authority and are absent from undo
records, avoiding anchor-count multiplied by edit-count retention. At most five
live target sets cover the current run and four suspended manual frames.
Skipped-target work yields within the existing 256-input/four-millisecond turns.

## Contracts Added or Changed

- Four canonical command identities and descriptions: region application, query,
  resume and abort.
- Three shared global gestures; C-] and its M-x hint belong only to manual macro
  editing. Startup configuration has no new leaves.
- Public typed query/manual wait observation, pause/region/iteration limit stops
  and shared outer limits. Existing step, text and active-time ceilings remain
  unchanged.
- Command, binding, description and factory-default catalogs, plus the existing
  Unreleased macro outcome.

## Files of Interest

- `src/input/macros.rs`, `src/session/macros/queries.rs`: native confirmation,
  bounded manual ownership, resume/abort and scoped hints.
- `src/macros/execution.rs`, `src/session/macros/runner.rs`: shared limits,
  cooperative execution and native history ownership.
- `src/buffer/edit/macro_lines.rs`, `src/session/macros/regions.rs`: checked
  original line targets and canonical publication maps.
- `tests/macros/queries.rs`, `tests/macros/regions.rs`,
  `src/session/macros/tests.rs`: native workflows and admission boundaries.

## Validation

All eight required commands passed on clean final PR head f996b57938532efce6a7735715dc901b39efcce9
with receipt ZAI7MBMIESFSLEZGAISN2H6LAF. Forgejo and Acta confirm integration
through PR #322. These checks do not establish the integrated delivery-set gate
or PTY acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- queries regions runner --test-threads=1` — pass, 22 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — pass, 52 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1` — pass, 114 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1` — pass, 45 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — pass, 648 commands and 293 configuration entries; 92 pages and 639 links/anchors.
- `git diff --check` — pass.

Additional development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording capture ring counters --test-threads=1` — pass, 36 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- session::macros::tests queries_clock regions_admission --test-threads=1` — pass, 12 cases covering native admission, shared budgets and pause timing.

## Risks and Follow-ups

Nested fixtures exercise user-entered runs during manual editing. Dynamic named
calls and their cycle/depth admission remain Slice 5. Persistent library and
definition/menu editing retain their recorded later slices. Full PTY and
large-buffer acceptance remain Slice 8.
