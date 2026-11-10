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
been introduced.

Macro retained-memory admission now borrows typed `Arc` identities for the duration
of one check. `Arc::ptr_eq` deduplicates definition allocations, shared step bodies
and region anchors. Stored definitions and active recording/replay state have
separate accounting walks, with the original byte charges, reservations, traversal
order and 48 MiB ceiling. Shared bodies count once; equal content in distinct
allocations counts separately. The new `shared_bodies` regression exercises both
properties through named snapshots and a one-entry ring.

Command and session helpers that only inspect an input now borrow it. Shared
ownership uses explicit `Arc::clone`; layout preparation moves its budget owner
instead of retaining an unnecessary extra reference. Command ranking constructs
owned promoted names directly, preserving recent/frequent ordering while removing
an intermediate collection. Command-history and bookmark guards end after their
last protected reads.

Checked numeric conversions retain the existing name/identity, prefix, file-size,
catalog-revision and candidate-row bounds. Position conversions follow the
existing clamping and positive-argument admission. Macro ring rotation keeps
Euclidean normalization, and zero/negative repeat counts retain their unbounded
iteration representation. The remaining selected source-bound corrections are
pending; selected Clippy admission now passes.

## Contracts Added or Changed

One private completion owner replaces nine independent session fields. Public
package signatures, completion observations, input commands and keybindings remain
unchanged. Private inspection/publication helpers borrow their immutable inputs;
existing admission, publication and freshness checks retain authority.

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

Initial completion-owner checks, retained from checkpoint
`e0a2cf6becc0ef49e0bc88796a8948933105ac60`:

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

Current ownership and representation checks:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission passes with zero blocking diagnostics; 121 native source
  findings remain. Coverage stays at 380 of 1,835 first-party Rust files.
- Frozen native regression command: 188 passed and 16 failed in `commands`, with
  the same failure identities as the retained completion-owner checkpoint.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases, including the new `shared_bodies` case.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with two failures.
- `target/debug/examples/source-check --workspace --scope src/macros/names.rs --scope src/session/macros/tests.rs --scope src/command/argument.rs`: passed all three selected source files.
- `git diff --check`: passed.

The modal failures are `contexts::colon_ranking` and `keymaps::state_hints`.
Their assertions expect only a command name and binding; the observed rows also
include the command description. Those assertions and fixtures remain unchanged.

## Risks and Follow-ups

Preserve native input/view context, stale completion rejection, transient
ownership, semantic macro replay/history and the retained macro body identity.
The remaining 121 source findings, resolution of the 16 command and two modal
failures, canonical PR preparation and exact clean-HEAD evidence remain pending.
F6 and shared macro-body accounting have focused regression coverage; Slice 8 is
not ready for integration.
