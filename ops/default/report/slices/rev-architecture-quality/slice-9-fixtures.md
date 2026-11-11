# Slice 9: Fixtures and developer examples

Status: reviewing
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-09-fixtures`
PR: [#23](https://forge.adrianpk.com/rev/rev/pulls/23)

## Purpose

Keep native test and developer example ownership within the existing selected
Clippy profile and source bounds, preserving behavioral and resource acceptance.

## Delivered Behavior

Shared fixture snapshots now clone their `Arc` or `Rc` owner explicitly. Values
that have no later use move into their existing recipient, and duplicate plugin
handle JSON borrows the retained handle. Concurrent publication fixtures start
both barrier participants eagerly before joining either thread.

Read-only fixture helpers borrow configuration snapshots, RPC payloads, capture
choices, catalog names and key-event phases. Numeric geometry, scalar positions,
TZif counts and timestamps use checked conversions. The existing test data,
wire records, resource ceilings and acceptance outcomes are unchanged.

Geometry fixtures separate policy and viewport setup from per-frame checks for
bookmark fields, gutter partitions, completion hints, tree presentation and Org
source positions. Existing case matrices and golden expectations are retained.

All 704 selected Rust files now pass formatting, all-target compilation, scoped
Clippy admission and native source policy. Both workspace packages and all 70
selected Cargo roots retain the existing profile and source limits. The four
selected-profile diagnostics outside this slice remain explicitly deferred.

## Implementation Notes

The baseline covers 704 of 1,836 first-party Rust files, both workspace packages
and 70 of 73 Cargo roots. It reports 201 blocking selected Clippy diagnostics,
four selected-profile diagnostics deferred outside this scope, and 123 native
source findings.

The first ownership cohort replaces 91 implicit reference-counted clone sites
with explicit shared ownership operations or the final move. It removes unused
fixture clones in manifests, JSON, context roots, staged contents and timezone
bytes. Git snapshot identity and freshness checks keep their existing inputs.

The command-usage and context publication races use eager array mapping instead
of collecting an intermediate vector of thread handles. Both threads still
start before either join, protecting the two-party barrier, lock contention,
publication-order assertions and existing two-second retry deadline.

The cohort preserves all existing test identities and assertion-site counts in
every changed Rust file. The complete selected baseline contains 2,889 test
identities and 19,672 assertion sites. No assertions, varied Unicode/numeric
inputs, PTY behavior, deadlines or resource limits are removed or relaxed.

The second cohort resolves the remaining selected Clippy diagnostics. Shared
completion setup still clones the settings snapshot into its owning buffer
collection, while views borrow the same settings observation. RPC request and
response helpers borrow JSON values or slices during synchronous serialization;
retained caller values no longer require temporary clones. All package protocol
and native source freshness checks keep their previous payloads and sequence.

Fixture numeric conversions check their existing small geometry/count bounds
rather than truncating them. The independent TZif encoder checks u32 header
counts and v1 i32 transition seconds without changing any encoded test input.
Three executable-size reporting statements retain floating-point ratio
conversion under a local, reasoned `cast_precision_loss` expectation. Only those
informational ratios use it; byte totals remain exact and signed deltas use i128.
No numeric or resource assertion is covered by those expectations.

Static comparison preserves all original string, character and numeric literals
in the changed Rust files. Only the three explicit ratio reasons are added.
Existing test identities and assertion-site counts remain unchanged.

Publication output has explicit file projection, asset copy and HTML finishing
phases. A private Markdown projection owner keeps its source, cursor, fence,
output and fragment map together, with named literal, image, table and ordered
step handling. Fence transitions, duplicate-anchor allocation and code-span cell
scanning retain their original boundaries. The canonical/reference configuration
sample comparison is a separate catalog concern.

The wrapping latency probe owns its native session, original scalar position and
measurements. Scrolling, queued reversal, far reversal, resize, editing and exact
source restoration remain in their original order, with the same event counts,
512-batch settle ceiling, idle checks and timing boundaries. Both developer
example crate roots now declare `#![forbid(unsafe_code)]`.

The publication and catalog unit-test bodies remain byte-identical. All five
wrapping inputs and their invocation order remain byte-identical, and assertion
counts are unchanged in all three files.

The geometry cohort separates marker admission from field partitioning, wrapping
labels from gutter policy combinations, and viewport setup from heading/cursor
hit-map checks. Tree entry roles, selected cells, labels and isolated NO_COLOR
journeys keep their independent expected values. Wrapped overlay checks still
distinguish active-file tree selection from source search and region spans.

All 59 existing case identities and 487 assertion sites remain in the 12 changed
Rust files. Static comparison preserves every text and numeric literal with its
original multiplicity. Native event order, geometry combinations, source bytes,
timeouts and resource bounds remain unchanged. A wrapping view moves into its
final clipped observation instead of cloning an owner that has no later use.


Dispatch fixtures separate route setup from advertised-chord replay and generated
buffer frame comparisons. Counted row motion retains both native sessions and
its original uncounted reference sequence; insertion refusal keeps zero and
negative inputs under both profiles. Tree click matrices, cached-path admission,
parent-substitution faults and all seven independent label roles retain their
original cases, source/history assertions and filesystem permission restoration.

The dispatch cohort preserves all 115 case identities, 668 assertion sites and
text/numeric literal multiplicities across its 11 Rust files. Macro buffer-name
navigation computes the same direction before selecting its inventory route;
configuration provenance expresses the same root/include/omission cases directly.


Admission fixtures separate typed AI target combinations, competing context
catalog writes, publication feedback, oversized snapshot/catalog builders and
staged Output substitution. The same locks, revisions, file identities, encoded
path widths, count limits and maximum-file positive control remain observable.
Dashboard artwork checks retain the original frame/palette assertions before
resizing the same live session. Grammar samples and malformed query texts are
explicit fixture catalogs; byte/size/file-type faults retain their original
filesystem operations and cached-rejection assertions.

The admission cohort preserves all 39 case identities, 342 assertion sites and
text/numeric literal multiplicities across its six Rust files. Native event and
write order, forbidden combinations, source retention, deadlines and resource
ceilings are unchanged. The wait predicate uses the existing polling callback's
mutable-session signature without changing its accepted feedback or wait budget.


Git fixtures separate conflict data preparation and side assertions, stale
review mutations, literal index-path seeding, metadata effects, autosquash
candidate construction and complete control scenarios. Dirty/aliased sources,
protected worktree removal and wrong/missing sequence authority retain their
existing refusal and unchanged-state checks. The controlled conflict RPC peer
keeps its 4,259,840-byte input ceiling and gated five-second settlement wait.

Exit fixtures separate each profile/entry/decision case and the bounded native
cleanup wait. Return/paste cannot grant Git termination, declining either owner
keeps jobs alive, and cancellation after cleanup starts still prevents later
exit. External reload faults retain the original permission-capability exception
and cannot acknowledge missing, rebound, oversized or invalid source text.
All 69 case identities, 593 assertion sites and text/numeric literal multiplicities
remain unchanged in the 12 Rust files. A discard source path is borrowed directly
instead of cloned into its temporary load selection.

LSP completion fixtures borrow a marked document descriptor that keeps source
text and its path together across all 26 existing session calls. Resolve
retirement distinguishes local editing/view/invocation changes from connection
retirement without changing any action, version guard or late-response wait.
Go and npm fixtures separate acquisition fault preparation from the original
refusal, registry-retention and cache-cleanup assertions. Optional native
installations separate user launch overrides from required-path/option
restoration checks under both input profiles.

The LSP cohort preserves all 27 case identities, 193 assertion sites and
text/numeric literal multiplicities in its five Rust files. It retains every
negative case, exact edit/byte limit, child-process boundary and timeout. The
marked document is borrowed during setup; the native session retains the same
owned buffers and configuration.

Plugin fixtures separate result creation, pending-input cancellation, crash
retirement, retained-result navigation and save checks while keeping exactly the
two original PTY journeys. Resource fixtures retain all five fresh-process
profiles, the 524,288-byte reads, 128 delivered events, original sampling order
and owned editor/peer exit assertions. Workload observations and idle resource
samples have explicit private owners; keymap setup borrows plugin defaults and
native user bindings together.

The executable peer separates raw buffer correlation, event/journey reactions
and SDK client lifecycle ownership. It retains every message in a received
chunk, nested failure routing, typed callback retirement, controlled late-command
threads, finite request/flood loops, all original byte/time limits and exact wire
records. Oversized store catalog construction is separate from the original
corruption/refusal matrix. All 15 case identities, 92 assertion sites and
text/numeric literal multiplicities remain unchanged in the five Rust files.

Developer tree/gutter examples separate configuration rejection, per-setting
workspace projection, expected entry styling, bookmark cells, full-row hit checks,
cached-tree resize and retained-file checks. Gutter presentation keeps the same
side/number/visibility/marker/width matrix, independent expected partitions,
marker color/hit checks, completion placement and source-only copy assertions.
PTY fixtures retain the same owner and event sequence through marker visibility,
resize/wrapping, copy/yank/undo, idle checks, finish and unchanged disk/configuration.

All 75 assertion sites and text/numeric literal multiplicities are preserved
across the five example files. The zero-column tree composition retains its
original early continuation before checking the cached identity. No fixed-row
expectation, input, timeout, stream ceiling or acceptance condition is changed.

Command-ranking examples separate usage seeding, promotion, retained selection,
first invocation, session-local recall, resize and restarted dispatch. The first
PTY owner is still explicitly dropped before its replacement starts. Paired
publishers retain the same two concurrent editors, publication checks and
unchanged-file assertions.

Keybinding-hint examples separate Org origin installation/retirement, modal
equivalence, terminal preparation, palette presentation and argument dispatch.
Geometry checks retain the outer session's original scalar point, and named and
bound motions still compare source, mark, visual and modal state independently.
All 99 assertion sites and text/numeric literal multiplicities remain unchanged
across the six example files. No row/color expectation, event sequence, timeout,
stream limit or process ownership boundary is changed.

Wrapping-navigation examples separate directory admission, Tab-only file
selection, Rust/Org split views, fold projection, visual editing/selection,
search cancellation, resize/wrap policy and retained-file checks. The original
scratch-buffer identity remains captured before directory loading and is shared
with the later Tab assertion. PTY startup keeps the same bounded reader thread,
channel and process owners; its extracted reader borrows the thread-owned sender.

Both wrapping files preserve all 58 assertion sites and text/numeric literal
multiplicities. Input bytes, geometry matrices, EOF behavior, stream limits,
eight-second deadline, idle interval and owned-child cleanup remain unchanged.

Bookmark examples separate record admission, edit/undo/save, alias/marker
projection, list/exit, persisted reopen, relocation, folded-source reveal and
unavailable/malformed-store protection. Resource fixtures own their source,
settings and timing observations; all 1,024 records, the overflow refusal,
512-batch projection ceiling, 128-pass relocation bound and canceled origin
checks remain intact. PTY phases keep the original byte streams, bounded reader
and explicit first-owner drop before restart. All 80 assertion sites and
text/numeric literal multiplicities remain unchanged across three Rust files.

Projects/Workspaces examples separate Project and Workspace catalog seeding,
large storage preparation/publication/measurement, canceled catalog and staged
reads, repeated restore/capture, capacity refusal and retained source checks.
The storage fixture owns the original large copies and releases them together
before native restoration, at the same point as before. Cancellation checks
borrow their existing stamp/generation/bytes snapshot across all three cycles.
PTY phases separate startup/palette/geometry, promotion and source anchors,
duplication refusal, session membership, blank Workspace creation, root removal,
conversion and explicit file/directory restarts. All 81 assertion sites and
text/numeric literal multiplicities remain in the three Rust files; serial
process drops, freshness, capacity, time and stream limits are unchanged.

Pi examples separate directory/project/configuration preparation, dirty native
snapshot provenance, atomic reload invalidation, saved command/apply/undo,
private history restoration/reset, idle measurements and missing-backend checks.
First-disconnect, reset and exit keep their original session/process retirement
sequence. PTY phases retain the same prompt-owning reload, stale-result refusal
and explicit child/terminal cleanup. Manual live mode still checks its explicit
environment guard before any filesystem or provider work; it was not executed.

Interoperability examples own disk fixtures, dirty hidden/orphan source
observations, collected result identity/generation and retired editable text
separately. The existing session guard still shuts down its owned package on
exit; native result navigation/local F8, generation, source-stamp and undo
assertions keep their original order. All 75 assertion sites and text/numeric
literal multiplicities remain unchanged across these five Rust files.

A final per-file comparison across the entire original selected inventory
preserves all 2,889 existing test identities and all 19,672 assertion sites in
704 Rust files. Only test/example implementation and this report/tracker change;
no runtime owner, public package API, wire format, policy limit or budget changes.

## Contracts Added or Changed

No runtime, public package API, protocol, policy or acceptance contract changes
are introduced.
The existing five frozen commands and final controller gate remain required.

## Files of Interest

- `tests/git_integration/`: explicit ownership of retained operation snapshots.
- `tests/command_usage/store.rs` and `tests/contexts/storage.rs`: eager concurrent
  publication fixtures with unchanged barrier and publication contracts.
- `tests/plugins/declarations.rs` and `tests/plugins/events.rs`: manifest and
  subscription fixture ownership.
- `tests/org_time/zone_bounds.rs`: final ownership of corrupted TZif bytes.
- `tests/org_time/zone_fixture.rs`: checked independent TZif encoding.
- `tests/plugins/buffers.rs` and its RPC consumers: borrowed request observations.
- `examples/completion_acceptance/support.rs`: shared borrowed configuration.
- `examples/documentation_check/catalogs.rs`: borrowed catalog comparisons and
  independent packaged/reference sample validation.
- `examples/userguide-publication.rs`: owned Markdown projection and bounded
  file/fragment/anchor finishing steps.
- `examples/wrap-performance.rs`: native probe phases with owned timings.
- `tests/bookmarks/markers.rs` and `tests/gutter/`: independent marker, number,
  wrapping and source-overlay checks across the original geometry matrices.
- `tests/display.rs` and `tests/display/`: clipped golden rows, complete hints,
  tree palette roles, selection and isolated color-policy checks.
- `tests/org_headings/geometry.rs` and `tests/org_indentation.rs`: viewport,
  source/pretty-heading and Unicode cursor/hit-map checks.
- `tests/tree/geometry.rs`: standalone tree projection at each admitted width.
- `tests/configuration/ai_commands.rs`: typed target combinations and original
  invalid-name, definition and size/count boundary checks.
- `tests/contexts/management.rs`, `tests/contexts/startup.rs` and
  `tests/contexts/storage.rs`: competing writes, retained artwork and explicit
  capacity/staging fault preparation.
- `tests/language_assets/admission.rs` and `tests/syntax/preferences.rs`:
  independent grammar samples, malformed query catalogs and cached asset faults.
- `tests/git_integration/conflicts.rs` and `conflicts_explanation.rs`: bounded
  conflict fixtures, independent side assertions and gated RPC settlement.
- `tests/git_integration/families_*`, `index_*`, `integration_guards.rs`,
  `repository_worktrees.rs` and `sequences_controls.rs`: captured-input,
  metadata, path-group, worktree and ordered-control acceptance phases.
- `tests/git_integration/lifetime.rs`, `tests/recovery/quit.rs` and
  `tests/external_changes.rs`: owned cleanup, exit consent and reload faults.
- `tests/lsp/candidates.rs` and `tests/lsp/edits.rs`: borrowed marked documents
  and explicit local/connection resolve retirement.
- `tests/lsp_installation/go.rs`, `npm.rs` and `optional-native.rs`: controlled
  installer faults and native launch-restoration checks.
- `tests/plugins/lifecycle.rs` and `state.rs`: borrowed keymap setup and
  independent oversized-record catalog construction.
- `tests/plugins/peer.rs`: raw framing/correlation, event reactions and SDK
  callback/process lifecycle owners.
- `tests/plugins/faults/pty.rs` and `resources.rs`: unchanged real PTY journeys,
  bounded workloads, idle samples and owned-process cleanup checks.
- `examples/better_file_tree_acceptance/`: configuration/workspace, independent
  style/hit-map and real-input/tree-label phases with unchanged assertions.
- `examples/better_gutter_acceptance/native.rs` and `pty.rs`: explicit native
  policy/geometry and PTY copy, wrapping, resize and retained-file check phases.
- `examples/command_ranking_acceptance/native.rs` and `pty.rs`: usage promotion,
  retained selection, recall, restarted dispatch and paired publisher phases.
- `examples/command_keybinding_hints_acceptance/`: Org hint origins, modal
  equivalence, terminal setup, palette geometry and named/bound dispatch checks.
- `examples/wrap_navigation/journey.rs` and `pty.rs`: directory/file/split,
  visual editing/search, wrap/resize and bounded reader/input/idle phases.
- `examples/git_integration_acceptance/`: snapshot and staged-content ownership.
- `examples/integrated_terminal_acceptance/failure.rs`: explicit shared failure
  injection state; host terminal cleanup behavior is unchanged.

- `examples/bookmark_acceptance/`: explicit native record/store/source lifecycle,
  measured bounded projection/relocation and PTY restart phases.

- `examples/projects_workspaces_acceptance/`: count/byte catalogs, explicit
  storage/native cancellation owners and serial real-input lifecycle phases.

- `examples/ai_pi_acceptance/`: fixture preparation, native history/reload and
  PTY input/process lifecycle phases; the manual live guard remains explicit.
- `examples/plugin-interop.rs`: owned input files, original-source observations,
  collected/retired result checks and guarded native package cleanup.

## Validation

The five frozen commands below passed in order on clean published implementation
HEAD `5a2d56a3d858484a1bc6afa1ae6614b046e72099` before the PR-reference update.
Use Looper's exact clean-HEAD record for the final PR candidate. Frozen ordered
commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros --test commands --test plugins -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The native command passed all 204 command, 110 macro and 86 plugin cases, with
no failures, ignores or filtered cases. The SDK doc test passed. Documentation
checks passed 684 commands, 299 configuration entries, 106 reachable pages and
743 local links/anchors. Warm scoped quality took 14.51 seconds; the serial
native command took 924.25 seconds under the unchanged resource prefix.

Development checks for shared fixture ownership:

- The frozen scoped quality command remains failing overall: formatting and
  all-target compilation pass; blocking selected Clippy diagnostics decrease
  from 201 to 99, with four deferred outside the selected sources. No selected
  `clone_on_ref_ptr`, `redundant_clone` or `needless_collect` findings remain.
  All 123 native source findings still require correction; this is scoped
  progress, not successful full quality adoption.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test command_usage --test contexts --test plugins --test org_time --test git_integration -- concurrent_ domain::aliases_and_overlap domain::canonical_files declarations::admission_bounds events::subscriptions zone_bounds::invalid_tzif_fields zone_bounds::tzif_limits diffs::stale_generation index::groups::hunks history::ancestry_pages commits::modes::fixup_objects --test-threads=1`: passed all 13 selected cases across command usage, contexts, Git, Org time and plugins; other cases were filtered out.
- Static comparison preserves each changed file's existing test identities and
  assertion-site count. No lint suppression or policy change was introduced.
- `git diff --check`: passed.

Development checks for borrowed observations and checked numeric fixtures:

- The frozen scoped quality command still fails overall on the same 123 native
  source findings. Formatting and all-target compilation pass, and selected
  Clippy admission now passes with zero blocking diagnostics across all 704
  selected files and 70 Cargo roots. Four selected-profile diagnostics remain
  deferred outside this scope; the profile and source limits are unchanged.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`: passed all 86 plugin cases, including real PTY, consent, cancellation, persistence and resource acceptance; the isolated child case is included in that total.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --example documentation-check`: passed all seven existing catalog/schema/link-drift unit cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`: passed 684 commands, 299 configuration entries, 106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal --test pointer --test gutter --test display --test org_headings --test org_time --test wrapping --test contexts --test git_integration --test configuration -- repeat:: context_safety:: visual_safety::visual_pointer cells:: scroll_bounds:: geometry::decimal_width controls:: palette:: habits:: prefix_selection source_and_pretty_geometry source_copy_and_edit zone_ gregorian_cycle continuation_autoscroll startup:: refreshed_hunk semantic_colors semantic_pointer palette_projection template::native_equivalence --test-threads=1`: passed all 103 selected geometry, palette, prefix, startup, timezone, scalar and wrapping cases; other cases were filtered out.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test lsp --test org_io -- acceptance_aliases bounded_sequence command_continuation completion_phases configured_phases decision_actions filesystem_arguments filesystem_consent find_file_aliases key_phases path_accept_aliases prompt_actions prompt_prefixes recursive_consent replacement_consent selected_arguments tree_actions tree_open_sequence tree_transitions document_isolation candidates:: edits:: capture_ --test-threads=1`: passed all 51 selected cases: 20 command, 22 LSP and nine Org capture cases; other cases were filtered out.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test integrated_terminal -- --test-threads=1`: passed all 49 integrated-terminal cases, with no ignored cases.
- Static comparison preserves all existing test identities, assertion-site counts,
  string/character literals and numeric input literals across 62 changed Rust
  files. No input, consent guard, assertion, timeout or resource limit is relaxed.
- `git diff --check`: passed.

Development checks for publication and wrapping probe phases:

- The frozen scoped quality command still fails overall: 114 native source
  findings remain, down from 123. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `target/debug/examples/source-check --workspace --scope examples/userguide-publication.rs --scope examples/wrap-performance.rs --scope examples/documentation_check/catalogs.rs`: passed native source policy for all three changed files, including both Cargo roots; Clippy is checked separately by the scoped quality command.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --example userguide-publication --example documentation-check`: passed all 13 existing unit cases: six publication projection/anchor/table/list cases and seven documentation/catalog/schema drift cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`: passed 684 commands, 299 configuration entries, 106 reachable pages and 743 local links and anchors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example wrap-performance`: passed all five existing bounded native scenarios: token, words, Unicode, regional indicators and many lines; exact source, scalar restoration and idle assertions remain intact.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p rev --example userguide-publication`: passed; the resulting owned producer generated all 106 canonical Markdown pages.
- Compared independent before/after publication fixtures against the clean
  `8b50c254ee67d8eddb3df53c2b3053d855fbba69` producer: all 279 output files are
  byte-identical, including projected pages, fragments, assets and the book tree.
- Existing publication/catalog unit-test bodies, wrapping scenario inputs/order
  and every changed file's assertion-site count are unchanged. No policy limit,
  timeout, input, resource budget or acceptance condition is relaxed.
- `git diff --check`: passed.

Development checks for native geometry fixtures:

- The frozen scoped quality command still fails overall: 94 native source
  findings remain, down from 114. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test bookmarks --test display --test gutter --test org_headings --test org_indentation --test tree -- --test-threads=1`: passed all 260 existing native cases: 58 bookmark, 47 display, 37 gutter, 16 Org heading, 18 Org indentation and 84 tree cases. Existing child-process assertions are included in their parent cases.
- Static comparison against the clean
  `7e46584e7a04d8eabe2f03ab00f02373fcee0338` baseline preserves all 59 case
  identities, 487 assertion sites and all text/numeric literal multiplicities
  across the 12 changed Rust files. No acceptance condition, timeout, input or
  resource budget is relaxed.
- `git diff --check`: passed.


Development checks for dispatch and tree interaction fixtures:

- The frozen scoped quality command still fails overall: 82 native source
  findings remain, down from 94. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test macros --test editing --test pointer --test tree --test configuration -- context_keymaps::tree_up_directory editor_keymaps::effective_hints palette::catalog_dispatch prefix_arguments::positioned_rows prefix_arguments::insertion_refusal tree_click_policy::activation_policy tree_decoration::full_row_actions tree::precedence capture::buffer_names decoration::label_modes parent::rejected_parent --test-threads=1`: passed all 11 affected existing native cases, including every advertised command's direct/named dispatch parity.
- Static comparison against clean
  `88a0f0a1c8ce4fb12bd293b8174159f0f90cedf5` preserves all 115 existing case
  identities, 668 assertion sites and text/numeric literal multiplicities in the
  11 changed Rust files. Inputs, event order, negative coverage, deadlines and
  resource limits remain unchanged.
- `git diff --check`: passed.


Development checks for admission and retained-state fixtures:

- The frozen scoped quality command still fails overall: 75 native source
  findings remain, down from 82. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test configuration --test contexts --test syntax --test language_assets -- ai_commands::definition_bounds management::publication_rejections startup::artwork_and_resize storage::capacity_admission storage::staging_substitution preferences::grammar_policy admission::rejection_fallback --test-threads=1`: passed all seven affected existing native cases, including all original negative combinations, competing writes, storage limits and cached asset failures.
- Static comparison against clean
  `61b33ea77404cca9075ba321c40a0b6422622ba2` preserves all 39 existing case
  identities, 342 assertion sites and text/numeric literal multiplicities in the
  six changed Rust files. No input, assertion, timeout, resource ceiling or
  acceptance condition is removed or relaxed.
- `git diff --check`: passed.


Development checks for Git and exit fixture phases:

- The frozen scoped quality command still fails overall: 61 native source
  findings remain, down from 75. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration --test external_changes --test recovery -- conflicts::partial_and_missing conflicts::explanation::optional_ai conflicts::explanation::historical_source families::patch::protected_input families::presentation::native_notes_sparse index::groups::paths index::presentation::native_discard_guards integration::guards::autosquash_admission lifetime::guarded_exit repository::worktrees::protected_removal sequences::controls::ordered_controls unavailable_sources quit::quit_alias_decisions --test-threads=1`: passed all 13 affected existing native cases, including gated historical conflict settlement, index/metadata/worktree refusals, autosquash bounds, sequence controls, owned Git/terminal cleanup and all quit decision routes.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --exact index::presentation::native_discard_guards --test-threads=1`: passed after the final path-borrow correction, retaining all six dirty/alias/changed combinations under both input profiles.
- Static comparison against clean
  `a0a450cc74470d71b3df694bd70ec4f7a986bba2` preserves all 69 case identities,
  593 assertion sites and text/numeric literal multiplicities in the 12 changed
  Rust files. Resource caps, event order, consent, deadlines and negative
  coverage remain unchanged.
- `git diff --check`: passed.

Development checks for LSP ownership and lifecycle fixtures:

- The frozen scoped quality command still fails overall: 56 native source
  findings remain, down from 61. Formatting, all-target compilation and scoped
  Clippy admission pass with zero blocking diagnostics; four selected-profile
  diagnostics remain deferred outside this scope. Coverage remains 704 of 1,836
  first-party Rust files and 70 of 73 Cargo roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test lsp --test lsp_installation -- --test-threads=1`: passed all 46 LSP and 39 offline installer cases, including the changed resolve, Go, npm and optional native workflows. The three existing explicit upstream-acquisition ignores remain unchanged.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test lsp -- candidates:: edits:: --test-threads=1`: passed all 22 affected completion cases after the final marked-document borrow correction; 24 unrelated cases were filtered out.
- Static comparison against clean
  `c0b5cdd3ef2c643cf3124abd44b5c4f89c737227` preserves all 27 case identities,
  193 assertion sites and text/numeric literal multiplicities in the five changed
  Rust files. Inputs, freshness guards, negative coverage, child boundaries,
  timeouts and resource limits are unchanged.
- `git diff --check`: passed.

Development checks for plugin fixture and peer ownership:

- The frozen scoped quality command still fails overall: 46 native source
  findings remain, down from 56, all in developer examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics; four selected-profile diagnostics remain deferred outside this
  scope. Coverage remains 704 of 1,836 first-party Rust files and 70 of 73 Cargo
  roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`: passed all 86 existing plugin cases, including native PTYs, resource profiles, peer framing/handshake faults, nested invocation, subscriptions, macro recording, persistence and cleanup. The isolated child case is included in that total.
- `target/debug/examples/source-check --workspace --scope tests --scope crates/rev-plugin-sdk/tests`: passed source admission for all 582 selected test/SDK-test files and 43 Cargo roots across both packages. This direct source check does not run Clippy; the preceding scoped quality run supplies compiler/Clippy evidence.
- Static comparison against clean
  `106dfbc007b9465a03d1b3d34f595090c866a7a5` preserves all 15 case identities,
  92 assertion sites and text/numeric literal multiplicities in the five changed
  Rust files. Wire inputs, consent, stale/canceled callback refusals, deadlines,
  byte/count limits and owned process boundaries remain unchanged.
- `git diff --check`: passed.

Development checks for tree/gutter example phases:

- The frozen scoped quality command still fails overall: 33 native source
  findings remain, down from 46, all in other developer examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics; four selected-profile diagnostics remain deferred outside this
  scope. Coverage remains 704 of 1,836 first-party Rust files and 70 of 73 Cargo
  roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-better-gutter.sh`: passed before and after decomposition, including the native glyph/geometry/legacy/color/fold/completion/copy matrix, both real PTY profiles, idle checks, unchanged files and restored terminal attributes.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p rev --example better-file-tree-acceptance`: passed after the final configuration-path borrow correction.
- The standalone file-tree native command fails before and after the cohort
  at the same nine-cell Unicode-label assertion. Its unchanged PTY journey
  also fails the first folder-row frame expectation; the displayed frame
  includes the parent row above that folder. The complete PTY failure diagnostic
  is byte-identical across both runs. No assertion is removed or adjusted.
- All four isolated file-tree `NO_COLOR` settings pass before and after, with
  byte-identical successful output. These bounded mode checks use the existing
  20-second wrapper and unchanged eight-second/1-MiB PTY owner.
- Static comparison against clean
  `c3428be18edabe81aaa828c883331449ea0ae200` preserves every one of the 75
  assertion sites and all text/numeric literal multiplicities in the five
  changed Rust files. Original matrices, row expectations, source bytes, event
  order and resource/time bounds remain unchanged.
- `git diff --check`: passed.

Development checks for command-ranking and keybinding-hint example phases:

- The frozen scoped quality command still fails overall: 23 native source
  findings remain, down from 33, all in other developer examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics; four selected-profile diagnostics remain deferred outside this
  scope. Coverage remains 704 of 1,836 first-party Rust files and 70 of 73 Cargo
  roots across two packages.
- Both existing ranking PTY restart journeys and the paired-publisher mode pass
  before and after decomposition. The same 30-second outer bounds and existing
  eight-second/1-MiB PTY owners cover idle behavior, persisted ranking,
  session-local recall, resize, exactly-once dispatch, complete publication,
  unchanged files/configuration and restored terminal attributes.
- The standalone ranking native example fails before and after at its existing
  empty-query expected-command comparison. The hint native example retains the
  same foreground-color assertion failure. Their failed assertions and actual
  and expected values are identical to the clean baseline; neither is reported
  as passing acceptance.
- Both hint PTY profiles retain the existing first-candidate frame expectation
  failure. The visible row includes the command description after its hint;
  complete failure diagnostics are byte-identical before and after the cohort.
- Static comparison against clean
  `cf448f6940a5e906f74c65aace8b5a0b8cdc7731` preserves all 99 assertion sites
  and text/numeric literal multiplicities across the six changed Rust files.
  Original inputs, negative coverage, freshness checks and resource/time limits
  remain unchanged.
- `git diff --check`: passed.

Development checks for wrapping-navigation example phases:

- The frozen scoped quality command still fails overall: 20 native source
  findings remain, down from 23, all in other developer examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics; four selected-profile diagnostics remain deferred outside this
  scope. Coverage remains 704 of 1,836 first-party Rust files and 70 of 73 Cargo
  roots across two packages.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p rev --bin rev --example wrap-navigation`: passed after correcting the extracted scratch identity and borrowing the reader thread's sender.
- The existing real PTY journey passes before and after decomposition with the
  original wrapper, inputs, eight-second/1-MiB owner and 350-ms idle assertion.
  Both runs emit 75,608 output bytes and retain click/split, visual editing,
  search, resize, no-wrap, cancellation, exact-save and terminal restoration
  checks. A temporary harness calls the unchanged PTY entry point because the
  preceding standalone native journey already fails at the clean baseline.
- The standalone native journey retains its baseline first-directory assertion
  failure: actual `BufferId(2)` differs from expected `BufferId(1)`. The failed
  assertion and both values are identical before and after; it is not reported
  as passing acceptance.
- Static comparison against clean
  `590ea30976e9abff0b0e1e22b1d055dee29cb0b0` preserves all 58 assertion sites
  and text/numeric literal multiplicities across the two changed Rust files.
  No case, assertion, time/byte bound or expected value is relaxed.
- `git diff --check`: passed.

Development checks for bookmark example phases:

- The frozen scoped quality command still fails overall: 16 native source
  findings remain, down from 20, all in other developer examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics and the same four deferred diagnostics outside this scope.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --locked -p rev --example bookmark-acceptance`: passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p rev --bin rev --example bookmark-acceptance`: passed before and after decomposition.
- The existing native, resource and real restart PTY journeys pass before and
  after in isolated fixtures using the existing terminal wrapper. Resource
  acceptance retains 2,477,825 source bytes, all 1,024 records, overflow refusal,
  exact restoration, canceled relocation and idle checks. Both runs settle the
  original relocated record in 17 passes, below the unchanged 128-pass ceiling.
  PTY ownership keeps the original eight-second/1-MiB bounds and 350-ms idle
  assertion. Timing/RSS observations are development-profile measurements.
- Static comparison against clean
  `3ffc2b5cf8e8943e8b29b6788424d72f2a4b0cb7` preserves all 80 assertion sites
  and text/numeric literal multiplicities in the three changed Rust files.
  Inputs, event order, session drops, deadlines and resource bounds are unchanged.
- `git diff --check`: passed.

Development checks for Projects/Workspaces example phases:

- The frozen scoped quality command still fails overall: five native source
  findings remain, down from 16, in Pi and interoperability examples. Formatting,
  all-target compilation and scoped Clippy admission pass with zero blocking
  diagnostics and the same four deferred diagnostics outside this scope.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --locked -p rev --example projects-workspaces-acceptance`: passed.
- The existing `scripts/check-projects-workspaces.sh` passes native ordinary,
  session-byte and aggregate/count-bound resource checks before and after. Both
  runs retain 256 Projects, 128 Workspaces, 64 roots, 512 files, 8,354,858 source
  catalog bytes and 8,354,610 candidate bytes. Cancellation, 513th-file refusal,
  exact source retention, idle and descriptor/thread retirement assertions pass.
- Both non-modal real PTY journeys pass, including all three serial starts,
  exactly the original inputs and 858,106 + 6,687 + 7,785 output bytes in each
  run. Palette, resize, Unicode anchors, membership, atomic conversion,
  unchanged files/configuration and terminal restoration checks remain intact.
- The modal PTY reaches its existing eight-second deadline before and after
  decomposition. Its final frame differs between the two failures; this is
  retained baseline failure evidence, not a successful or byte-identical
  acceptance claim. The 30-second outer bound, eight-second/1-MiB editor owner,
  all frame predicates and original event order remain unchanged.
- Static comparison against clean
  `57de2e0f43b3662a72de10c053d6c6a967bf3a6e` preserves all 81 assertion sites
  and text/numeric literal multiplicities in the three changed Rust files.
- `git diff --check`: passed.

Development checks for Pi and interoperability example phases:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`: passed. Formatting, all-target compilation, scoped Clippy admission and native source policy all pass across 704 files, both packages and 70 selected Cargo roots. There are zero blocking diagnostics and zero source findings; the four diagnostics outside this scope remain deferred.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p rev --bin rev --example ai-pi-acceptance --example plugin-interop`: passed before and after decomposition.
- Both native Pi journeys and both real PTY profiles pass before and after with
  the installed native Pi 1.3.0 executable pinned directly. All fixtures use a
  fresh private agent directory, offline provider and disabled telemetry; live
  mode is unset. Original 35-second native wrappers, six-/12-second waits,
  five-second reap bound and eight-second/1-MiB terminal ownership are unchanged.
  No installation, acquisition, live-provider operation or user state is used.
- Interoperability under both profiles passes before and after against the same
  existing collector executable, SHA-256
  `c70e5091474aae16646b7f79e30f7171800846b6d1eac1c54c0720d7d175a4e8`.
  Successful output is byte-identical. This focused comparison neither rebuilds
  nor changes the separate client repository; the controller still owns the
  frozen full-gate rebuild and interoperability command.
- Static comparison against clean
  `eb4cc1f121d11eda79b81c89731d7a77cfc3a661` preserves all 75 assertion sites
  and text/numeric literal multiplicities across the five changed Rust files.
  Final per-file comparison with the original Slice 9 inventory preserves all
  2,889 test identities and 19,672 assertion sites across 704 selected files.
- `git diff --check`: passed.

## Risks and Follow-ups

T9.1-T9.3 are implemented in canonical [PR #23](https://forge.adrianpk.com/rev/rev/pulls/23).
Selected fixture/example Clippy and source admission is complete, and all frozen
commands pass on the implementation HEAD recorded above. Existing case,
assertion, input and resource contracts are preserved. The PR-reference commit
requires its own exact clean-HEAD Looper record. Review, integration, the unchanged
full gate and successor startup remain controller-owned.

The standalone file-tree native and PTY examples retain their pre-existing
fixed-row expectation failures recorded above. Their assertions and limits stay
unchanged; they are not reported as successful acceptance. The gutter native
and PTY checks and the isolated file-tree `NO_COLOR` matrix pass.

The standalone command-ranking native and keybinding-hint native/PTY examples
retain their baseline expectation failures recorded above. Their assertions and
bounds remain intact. Ranking's two restart PTYs and paired-publisher mode pass.

The standalone wrapping native journey retains its pre-existing directory-buffer
identity expectation failure. Its real PTY journey passes before and after with
the original checks and bounds.

Projects/Workspaces native and non-modal PTY acceptance pass before and after.
The modal PTY retains a baseline deadline failure; no budget or assertion is
adjusted to make that standalone helper pass.
