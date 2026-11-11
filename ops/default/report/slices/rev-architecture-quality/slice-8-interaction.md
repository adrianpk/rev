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

Input routing now separates Git context selection, startup command families and
context-specific map names, defaults and availability. Their original precedence,
command names and bindings are unchanged. Hint sequence resolution remains with
the actual resolver; terminal first refusal and tree-command normalization have
separate checks.

Prefix handling separates admission, continued chord lookup, command invocation,
literal text capture and retirement. Prompt lookup, paste and focused decision
routing delegate to their existing owners. Macro key capture retains the same
reserved slots, map lookup, prompt errors and eight-chord ceiling. Binding checks
still inspect every map after finding an occupied chord, so a later reserved
cancellation or prefix transition can reject the binding.

Plugin contribution admission separates protected native keys from ranked
eligibility, conflict classification and map publication. Eligibility for each
rank is captured before updating its statuses; local/user precedence and
same-rank conflict handling retain their existing order.

Counter formatting separates literal scanning, flags, field parsing, radix
rendering and padding. Its NUL, 256-byte format, width/precision ceiling of 128
and 512-byte output checks retain their original errors and order. Counter/status
selection uses early returns while retaining active runner, recorder, last pause
and ring/scratch precedence; an empty pause still selects scratch.

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

Ownership and representation checks from checkpoint
`0f34fa97f9af8e37651378a854994d54ab9b8011`:

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

Development checks for input routing and counter formatting:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission still passes with zero blocking diagnostics and 205
  selected-profile diagnostics deferred outside the scope. Native source findings
  decreased from 121 to 96, resolving 25 existing findings without adding any.
  Coverage remains 380 of 1,835 first-party Rust files.
- Frozen native regression command: 188 passed and 16 failed in `commands`, with
  the same failure identities as checkpoint `0f34fa97f9af8e37651378a854994d54ab9b8011`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- keymaps:: --test-threads=1`: passed all nine selected plugin-keymap cases, including precedence, conflicts, protected keys and atomic configuration; 77 other plugin cases were filtered out.
- `target/debug/examples/source-check --workspace --scope src/input.rs --scope src/input/argument.rs --scope src/input/context.rs --scope src/input/hints.rs --scope src/input/keymap/availability.rs --scope src/input/keymap/contributions.rs --scope src/input/keymap/defaults.rs --scope src/input/macro_bindings.rs --scope src/input/paste.rs --scope src/input/step_edit.rs --scope src/macros.rs --scope src/macros/counter.rs`: passed all 12 selected source files. The modified `src/input/keymap.rs` remains covered by the frozen quality command and retains its `compile_ai` complexity finding.
- `git diff --check`: passed.

## Risks and Follow-ups

Preserve native input/view context, stale completion rejection, transient
ownership, semantic macro replay/history and the retained macro body identity.
The remaining 96 source findings, resolution of the 16 command and two modal
failures, canonical PR preparation and exact clean-HEAD evidence remain pending.
F6 and shared macro-body accounting have focused regression coverage; Slice 8 is
not ready for integration.
