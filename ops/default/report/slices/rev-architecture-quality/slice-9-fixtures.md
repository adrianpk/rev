# Slice 9: Fixtures and developer examples

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-09-fixtures`
PR: pending

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

## Contracts Added or Changed

No runtime, package API, protocol, policy or acceptance changes are planned.
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
- `examples/git_integration_acceptance/`: snapshot and staged-content ownership.
- `examples/integrated_terminal_acceptance/failure.rs`: explicit shared failure
  injection state; host terminal cleanup behavior is unchanged.

## Validation

Final successful clean-HEAD evidence is pending. Frozen ordered commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros --test commands --test plugins -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

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

## Risks and Follow-ups

Selected Clippy admission is complete. The remaining 56 native source findings
require cohesive fixture/example decomposition. T9.1-T9.3 remain in progress.
The canonical PR, final ordered clean-HEAD evidence and controller integration
remain pending.
