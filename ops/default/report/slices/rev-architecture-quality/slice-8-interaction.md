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

Native session dispatch now belongs to `src/session/dispatch.rs`, with private
handlers for source, Project, Workspace, AI request/control, shell and editing
commands. All 54 dispatched variants retain their handlers, and navigation
bookkeeping still runs once before dispatch. Public execution separates macro
control admission from context/source admission in the original order. Macro
answers and terminal scalar insertion retain their pending-token and focus checks.

Counted argument handling separates control invocations, source policy, reusable
movement planning and individual motion/line/row steps. Each movement batch still
permits at most 256 steps and four milliseconds. Region lookup and zero-line
preparation retain their original early-error behavior; pending distance work
still retains its partial plan. Absolute positions, signed reversal, zero arguments and region
replacement keep the same checks. Text generation separates unit selection,
checked payload sizing and range admission, retaining the 8 MiB ceiling and
publication only after complete preparation.

Frame projection keeps source completion outside the derived-work budget and
separates composition from overlays. Event readiness, pointer retirement, Escape
expiration and native editing/macro dispatch have explicit boundaries. The input
batch still admits at most 32 events and four milliseconds. Background owners
advance once in their original order, with shutdown Exit retaining precedence.
Held layout replay still checks selection and the request-specific point or
complete-view geometry before publication. Startup/Git versus editing context
synchronization and modal component-focus admission retain their original
ordering.

Macro execution separates runner waiting, admitted input, iteration boundaries,
replay and progress publication. Trigger keys still precede captured arguments
and definition steps. Source/profile checks, consent pauses and shared undo
groups retain their original order. Host turns remain bounded by 256 steps and
four milliseconds, with the same active-time, logical-step and iteration limits.

Recording separates command selection, semantic/plugin/completion controls, named
commands and physical input. Counters retain preparation, capture, insertion and
state publication order. Step editing, menu actions and definition drafts keep
their existing ownership, pinned identity/revision and complete-line checks.
Region replay retains its 65,536-anchor ceiling and cooperative turn budget.
Library reload still validates the whole catalog before per-record admission,
quarantines independent collisions, and updates bindings only after accepting the
named snapshot. Saved-library mutation remains explicit and consent-controlled.

Persisted macro conversion separates catalog headers, record identity, entry
profiles and each native input family. Schema declarations, bounded decoding,
error precedence and retained-byte charges are unchanged. The structural bounds
case keeps all 16 named invalid inputs and every assertion; its mutations now
belong to aggregate, record, step and encoded-record helpers.

Macro publication owns its lock and anchored directory in one private guard,
retaining both through staging, replacement and observation. Candidate admission
still precedes directory creation; revision checks and schema upgrades occur only
at explicit publication. Replacement remains the commit point, with uncertain
post-replacement observation retaining the candidate and reporting a warning.

Macro-library and command-history reads separate metadata admission, bounded IO
and freshness checks without changing their order or limits. Command-history
publication keeps empty batches passive and holds its lock across rebasing and
replacement. Ambiguous completion still suspends retries to avoid duplicate
usage.

Session file admission, argument routing, consent and cancellation now delegate
to their native concern handlers. Pending values move once to the selected
handler; resource cancellation still precedes document cancellation. Prompt
questions preserve their existing consent text and macro-pause fallback.

File observation separates borrowed reconciliation, checked generation
assignment, retired-source cleanup, watcher updates and queued-event coalescing.
Reload publication retains current-source checks and the five-second deadline;
late or canceled work keeps its single worker slot until nonblocking retirement.
No observation queue, source-count, byte or debounce bound has changed.

Startup separates target presentation, nested cancellation, navigation and
Project/Workspace actions. Existing command mappings, fallback to the retained
context and pointer arming checks are unchanged. Paste, search acceptance,
argument yanking and tree outcomes retain their original admission and
publication order. Atomic source edits reserve every participant before final
checks and publish all sources/views before notifying observers.

Command metadata now delegates to named native concerns: repository creation,
submodules/subtrees, notes, sparse checkout, patches, processes, worktrees,
stashes, integration/conflicts, references, history, commits and Status; macro
recording, playback, counters, library, definition editing and menu actions have
their own handlers. All 119 Git and 52 macro variants retain their exact names
and descriptions. Public enums, signatures, `Command::ALL`, selector aliases and
metadata for user-defined commands are unchanged.

Keymap compilation prepares inherited editing layers before independent native
maps, then moves each validated map into the effective candidate. Repository
maps compile after transient editors, tree and bookmarks; the commit overlay
retains its earlier position. The 15 independent native map calls preserve their
contexts, override sources and validation order. The private preparation holders
add no map copies or publication boundary.

## Contracts Added or Changed

One private completion owner replaces nine independent session fields. Public
package signatures, completion observations, input commands and keybindings remain
unchanged. Private inspection/publication helpers borrow their immutable inputs;
existing admission, publication and freshness checks retain authority.

## Files of Interest

- `src/session/completion/state.rs`: completion invocation ownership and explicit
  admission, cancellation and retirement transitions.
- `src/session/dispatch.rs`: native command handlers after admission and navigation.
- `src/session/arguments/` and `src/session/arguments.rs`: counted planning,
  payload/range admission and publication under the existing budgets.
- `src/session/events.rs` and `src/session/layout.rs`: ready-input ownership,
  pointer retirement and checked deferred replay.
- `src/session/macros/`: capture, replay, counters, menu actions, step editing and
  explicit library admission under the existing native owners and budgets.
- `src/macros/editing.rs`: pinned ring/named target checks before revision
  publication.
- `src/macros/library/codec.rs` and `src/macros/library/store.rs`: persisted input
  admission and guarded publication with unchanged schema and commit semantics.
- `src/command/usage/store.rs`: bounded history reads and locked pending-event
  publication.
- `src/session/file.rs`, `src/session/prompt.rs` and `src/session/save.rs`:
  native argument, cancellation and consent routing.
- `src/session/observation.rs` and `src/session/reload.rs`: source reconciliation,
  queue admission and worker retirement with unchanged freshness/deadline checks.
- `src/session/startup/` and `src/session/startup.rs`: target presentation,
  nested cancellation and native startup action families.
- `src/session/source_batch/atomic.rs`: reserved participant preparation before
  all-source publication and deferred observer notification.
- `src/command/git.rs`, `src/command/macros.rs` and `src/command.rs`: native
  metadata concerns with unchanged catalog identities and literal descriptions.
- `src/input/keymap.rs`: inherited editing preparation and ordered repository
  map compilation before effective-map construction.
- `src/session.rs` and `src/session/completion/`: composition, source restoration,
  resolver polling, stale-result checks and publication.
- `src/command/`, `src/input/`, `src/macros/` and `src/session/`: selected native
  dispatch, input and macro ownership/complexity findings.

## Validation

Final successful clean-HEAD validation is pending. The frozen ordered commands are unchanged:

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

Input routing and counter formatting checks recorded in checkpoint
`6091f3fa19cccfc315fe2bed3ec7020d01a59bb5`:

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

Development checks for session admission and dispatch:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission passes with zero blocking diagnostics; 205 selected-profile
  diagnostics remain deferred outside the scope. Native source policy still
  fails with 75 findings, down from 96. All 21 removed findings were existing;
  no new findings appeared. Coverage is 381 of 1,836 first-party Rust files,
  including the new native dispatcher.
- Frozen native regression command: 188 passed and the same 16 `commands` cases
  failed as in checkpoint `6091f3fa19cccfc315fe2bed3ec7020d01a59bb5`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test coordinates --test display --test pointer --no-fail-fast -- --test-threads=1`: passed all 24 coordinate, 47 display and 68 pointer cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- recording:: --test-threads=1`: passed all ten selected plugin-recording cases, including prefix policy, semantic calls, replay ownership and cancellation; 76 other plugin cases were filtered out.
- `target/debug/examples/source-check --workspace --scope src/session.rs --scope src/session/dispatch.rs --scope src/session/arguments --scope src/session/arguments.rs --scope src/session/events.rs --scope src/session/layout.rs --scope src/session/mode.rs --scope src/session/modal.rs`: passed all ten selected source files, covering every modified source and the existing argument-history helper.
- `git diff --check`: passed.

Development checks for macro lifecycle owners:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission passes with zero blocking diagnostics; 205 selected-profile
  diagnostics remain deferred outside the scope. Native source findings decreased
  from 75 to 44, resolving 31 existing findings without adding any. Coverage
  remains 381 of 1,836 first-party Rust files.
- Frozen native regression command: 188 passed and the same 16 `commands` cases
  failed as in checkpoint `98520c9b1ace08f6837648297198d526e04d3f3a`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- recording:: --test-threads=1`: passed all ten selected plugin-recording cases; 76 other plugin cases were filtered out.
- `target/debug/examples/source-check --workspace --scope src/session/macros.rs --scope src/session/macros --scope src/macros/editing.rs`: passed all 18 selected source files, including every modified source, all session macro modules and the shared definition publisher.
- `git diff --check`: passed.

Development checks for persisted input and publication:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission passes with zero blocking diagnostics; 205 selected-profile
  diagnostics remain deferred outside the scope. Native source findings decreased
  from 44 to 30, resolving 14 existing findings without adding any. Coverage
  remains 381 of 1,836 first-party Rust files.
- Frozen native regression command: 188 passed and the same 16 `commands` cases
  failed as in checkpoint `1cc980d1c67564d7418326f76fc66be0bf8d0d51`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases, including codec bounds, publication faults and revision exhaustion.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- recording:: --test-threads=1`: passed all ten selected plugin-recording cases; 76 other plugin cases were filtered out.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test command_usage -- --test-threads=1`: failed with 14 passed and one failed. All seven store cases pass, covering bounded/passive reads, lock admission, concurrent rebasing and publication faults. `startup::startup_owner` observes `abort-recursive-edit` as the first candidate where it expects `ai-apply-result`. This target was not measured at the retained checkpoint; the failure remains unresolved. Its assertions, inputs and deadlines are unchanged.
- `target/debug/examples/source-check --workspace --scope src/macros/library --scope src/command/usage/store.rs`: passed all five selected source files, including every source changed in this cohort.
- `git diff --check`: passed.

Development checks for native interaction transitions:

- Frozen scoped quality: formatting and all-target compilation passed. Selected
  Clippy admission passes with zero blocking diagnostics; 205 selected-profile
  diagnostics remain deferred outside the scope. Native source findings decreased
  from 30 to nine, resolving 21 existing findings without adding any. Coverage
  remains 381 of 1,836 first-party Rust files. Remaining findings belong to
  command metadata and keymap compilation; all session sources satisfy the bounds.
- Frozen native regression command: 188 passed and the same 16 `commands` cases
  failed as in checkpoint `c77aa353117e197f0b4e85a9f3bb2e3f74427ddb`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases, including atomic source publication and retirement of stale observation notices.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing --test search --test files --test external_changes --test contexts --test bookmarks --test tree --test configuration --no-fail-fast -- --test-threads=1`: passed all 606 cases: 114 editing, 30 search, 75 file, 21 external-change, 62 context, 58 bookmark, 84 tree and 162 configuration cases, with no ignores.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test command_usage -- --test-threads=1`: failed with 14 passed and the same `startup::startup_owner` failure as checkpoint `c77aa353117e197f0b4e85a9f3bb2e3f74427ddb`.
- `target/debug/examples/source-check --workspace --scope src/session`: passed all 297 selected session files, including every source changed in this cohort. Public signatures and existing in-module test bodies are unchanged; no assertions, inputs, timeouts or budgets were edited.
- `git diff --check`: passed.

Development checks for command metadata and keymap preparation:

- Frozen scoped quality: passed formatting, all-target compilation, selected
  Clippy admission and native source policy across all 381 selected files. The
  final nine source findings are resolved; no new findings appeared. Selected
  Clippy has zero blocking diagnostics, with 205 selected-profile diagnostics
  deferred outside this scope. Coverage remains 381 of 1,836 first-party Rust
  files across two packages.
- Frozen native regression command: 188 passed and the same 16 `commands` cases
  failed as in checkpoint `9c76fee60ac3e0b06e38ee348015536873172606`.
- Frozen documentation check: passed 684 commands, 299 configuration entries,
  106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`: passed all 186 library unit cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test completion --test macros --no-fail-fast -- --test-threads=1`: failed overall; completion passed 44 cases, macros passed 110, and modal passed 122 with the same two failure identities listed above.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test configuration -- --test-threads=1`: passed all 162 configuration cases, including inherited-map, overlay, alias, profile and startup contracts.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- keymaps:: --test-threads=1`: passed all nine selected plugin-keymap cases; 77 other plugin cases were filtered out.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test command_usage -- --test-threads=1`: failed with 14 passed and the same `startup::startup_owner` failure as checkpoint `9c76fee60ac3e0b06e38ee348015536873172606`.
- Static comparison confirms unchanged metadata for every Git/macro variant,
  unchanged public signatures/enums and `Command::ALL`, and the same ordered
  contexts and override sources for all 15 independent native map calls. Test
  files, assertions, inputs, timeouts and resource bounds are unchanged.
- `git diff --check`: passed.

Fixture reconciliation, following maintainer technical-recovery authorization:

- The initial focused command cohort passed 37 cases; its remaining description,
  alias and numeric-prefix assertions were corrected against the existing native
  contracts. The startup ranking regression passed independently.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test modal -- context_keymaps::terminal_hints editor_keymaps::effective_hints contexts::colon_ranking keymaps::state_hints --test-threads=1`: passed all four cases, retaining both interaction profiles.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- shell::shell_sequences --test-threads=1`: passed, including the preserved tangle continuation, cancellation, invalid Meta input and counted text insertion.
- Final ordered clean-published-HEAD validation remains required through
  `looper validate`; these focused results do not replace its evidence.

## Risks and Follow-ups

Preserve native input/view context, stale completion rejection, transient
ownership, semantic macro replay/history and the retained macro body identity.
Scoped quality and T8.2 are complete. The retained fixture failures have focused
corrections; canonical PR preparation and successful exact clean-HEAD evidence
remain pending. F6, shared macro-body
accounting, native dispatch, counted planning, macro lifecycle, publication and
keymap preparation have focused regression coverage; Slice 8 is not ready for
integration.

The maintainer authorized technical diagnosis, bounded corrections, validation
and same-run continuation without repeated routine approval pauses. The fixture
correction reconciles the following existing tests with approved behavior:

- Align catalog name grammar, numeric-prefix outcomes, the now-bound macro-query
  suffix and the four implemented Org visibility commands in `tests/commands.rs`,
  `tests/commands/editor_keymaps.rs`, `tests/commands/keymap.rs`,
  `tests/commands/org_catalog.rs`, `tests/commands/shell.rs` and
  `tests/commands/terminal.rs`.
- Include existing command descriptions in selector-row expectations in
  `tests/commands/context_keymaps.rs`, `tests/modal/contexts.rs` and
  `tests/modal/keymaps.rs`. Keep full command identities and hints while allowing
  descriptions to clip at the unchanged 80-column terminal viewport. Select
  actual filesystem paths through native tree navigation in
  `tests/commands/context_keymaps.rs`, `tests/commands/context_keymaps/flows.rs`
  and `tests/commands/context_keymaps/guards.rs`, retaining the parent row and
  every create/rename/delete, repeat, cancellation and fresh-consent assertion.
- Align empty-query selection, retirement and persisted ranking fixtures with
  the complete current catalog in `tests/commands/flows.rs`,
  `tests/commands/selection.rs`, `tests/commands/vertical/geometry.rs` and
  `tests/command_usage/startup.rs`. Preserve the case-specific acceptance and
  ranking checks rather than removing them.

These 15 test paths preserve case identities and semantic/negative coverage.
The two flow/guard files replace positional assumptions with explicit target
selection rather than changing tree defaults. Numeric text input remains a
positive counted-insertion case; an actually unbound Meta key preserves
invalid-prefix coverage. Alias hints still dispatch their canonical action.
There is no runtime behavior, API/protocol change, ignored case, removed
assertion, timeout/resource change, frozen command change, slice or gate
deferral. Every frozen Slice 8 check and the unchanged controller gate remain
required before acceptance.
