# Slice 8: Session interaction and completion owner

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-08-interaction`
PR: pending

## Purpose

Own completion admission and retirement in one session concern, and correct the
selected command, input, macro and session quality findings.

## Delivered Behavior

Completion invocation state now belongs to one private session concern. It owns
the popup, monotonic generation, admitted request, editing observation, source
outcome, worker, resolver, recent buffers and policy. Admission and retirement
update that owner explicitly; cancellation keeps the existing worker until normal
polling reaps it. Public completion observations and native editing/context
semantics retain their existing checks.

## Implementation Notes

The F6 extraction moves the related session fields into
`src/session/completion/state.rs`. Session completion handlers retain their input,
view, buffer, LSP and publication checks, while the owner coordinates request
admission, dismissal and terminal retirement. Terminal results keep the last
source observation; dismissal cancels the resolver before clearing authority and
cancelling worker work, in the original order. Recent-buffer ordering and its
existing ceiling are unchanged.

Macro playback still tests popup presence at its original admission boundary;
one-normal transitions still test admitted request presence. Existing in-module
completion cases use the new owner without changing their assertions, inputs or
deadlines. No generic job framework, dependency, public API or resource limit has
been introduced. The remaining selected native source and Clippy corrections are
pending.

## Contracts Added or Changed

One private completion owner replaces nine independent session fields. Public
package signatures, completion observations, input commands and keybindings remain
unchanged.

## Files of Interest

- `src/session/completion/state.rs`: completion invocation ownership and explicit
  admission, cancellation and retirement transitions.
- `src/session.rs` and `src/session/completion/`: composition, source restoration,
  resolver polling, stale-result checks and publication.
- `src/command/`, `src/input/`, `src/macros/` and `src/session/`: selected native
  dispatch, input and macro ownership/complexity findings.

## Validation

Final clean-HEAD validation is pending. The frozen ordered commands are unchanged:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/command.rs --scope src/command --scope src/input.rs --scope src/input --scope src/macros.rs --scope src/macros --scope src/session.rs --scope src/session`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test modal --test completion --test macros -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

Development checks on the completion owner:

- Frozen scoped quality: formatting and all-target compilation passed; admission
  still fails with 172 selected Clippy diagnostic instances and 123 native source
  findings, matching the baseline. Coverage is 380 of 1,835 first-party Rust files
  after introducing the owner.
- Frozen native regression command: failed in `commands` with 188 passed and 16
  failed. Cargo stopped before running `completion`, `modal` and `macros`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- --test-threads=1`: passed all 44 existing native completion cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- session::completion:: --test-threads=1`: passed all 12 existing completion unit cases, including worker replacement, source snapshots and stale/retired results.
- `target/debug/examples/source-check --workspace --scope src/session/completion.rs --scope src/session/completion`: passed all five selected source files.
- `git diff --check`: passed.

The unresolved `commands` cases are `command_sequences`,
`context_keymaps::context_hints`,
`context_keymaps::flows::filesystem_arguments`,
`context_keymaps::flows::recursive_consent`,
`context_keymaps::guards::filesystem_consent`,
`context_keymaps::terminal_hints`, `editor_keymaps::effective_hints`,
`flows::entry_retirement`, `keymap::editor_defaults`,
`org_catalog::default_routes`, `org_catalog::native_catalog`, `prefix_recovery`,
`selection::acceptance_rules`, `shell::shell_sequences`,
`terminal::suffix_phases` and `vertical::geometry::candidate_limits`.
Their assertions, inputs, timeouts and budgets have not been changed.

## Risks and Follow-ups

Preserve native input/view context, stale completion rejection, transient
ownership, semantic macro replay/history and the retained macro body identity.
Selected quality corrections, resolution of the command-suite failures, the
remaining native regression suites, canonical PR preparation and exact clean-HEAD
evidence remain pending. F6 has focused regression coverage; Slice 8 is not ready
for integration.
