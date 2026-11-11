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

## Risks and Follow-ups

Selected Clippy admission is complete. The remaining 114 native source findings
require cohesive fixture/example decomposition. T9.1-T9.3 remain in progress.
The canonical PR, final ordered clean-HEAD evidence and controller integration
remain pending.
