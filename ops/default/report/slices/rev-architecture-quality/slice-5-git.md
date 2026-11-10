# Slice 5: Git operation boundaries

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-05-git`
PR: pending

## Purpose

Resolve the approved native Git ownership, representation and source-bound
findings within the existing operation-family and observation owners.

## Delivered Behavior

Native publication, worker launch, repository observation, child capture and final
record conversion have separate owners. Session dispatch, update routing, input
authority and visible-row projection retain the existing operation/page families,
generation checks and immutable source authority.

The approved `FamilyOutput` request owns exact output-family argv and destination.
Command definitions, input validation, shell construction and source admission
remain separate from launch. Patch and status parsing retain bounded complete
source evidence and invalidate unsupported changing targets.

Bisect now separates exact argument preparation, changing-child facts, ref/range
observation and family-specific postconditions. Conflict inspection reads stages
under one cumulative detail allowance. History separates request ownership,
ancestry pages, immutable comparisons, reflog records, blame attribution and
quoted-path decoding.

This is a partial checkpoint. Other operation-family mutation handlers and session
review handlers still require source-bound corrections.

## Implementation Notes

- Workers retain captured directory, request, policy, cancellation, progress,
  reservation and wake callback until transfer into the named thread. Cleanup
  still retains the exact changing child and its reservation.
- Capture preserves aggregate output accounting, timeout classification,
  redaction, malformed evidence and reservation transfer to published data.
- Session routing retains first-match operation-ID precedence, focus/prompt
  guards and all painted selection tokens. Row projection remains lazy.
- Output preparation keeps its original validation/error order and does not write
  the destination. Only the approved Rust signature and its four callers changed.
- Bisect preserves exact commands, original branch lineage, manual verdicts,
  complete-ref admission and bounded candidate graphs. Exit 2 requires actual
  graph evidence before ambiguity is accepted; no verdict or reset is inferred.
- Conflict parsing preserves duplicate-stage/error precedence, shared detail
  accounting, immutable source rechecks and 2048-byte display chunks.
- History preserves root/continuation authority, read-only path/object selection,
  parser cancellation, metadata/path ceilings, page reservations and truncation.
  C-quoted filenames retain non-UTF-8 bytes and checked octal escapes.
- Private helpers borrow retained authority and parsed views. Explicit shared
  clones, guarded size conversions and signed timestamp fingerprints remain.

## Contracts Added or Changed

The maintainer-approved `FamilyOutput { family, args, output }` request replaces
three related parameters of `Repositories::prepare_family_output`. Its session
caller and three existing test calls use the new request. User commands, SDK and
plugin protocols, source limits and resource budgets are unchanged.

Six public history-inspection methods retain their owned captured-authority
parameters under narrow, reasoned Clippy expectations. No lint baseline, broad
suppression or source-policy exception was added.

## Files of Interest

- `src/git/publication/` and `src/git/work/{worker,request,observation,command,completion}.rs`:
  checked publication, worker lifetime and child-result ownership.
- `src/git/families/` and `src/git/diff/`: exact command/input preparation and
  bounded immutable patch indexing.
- `src/git/work/bisect/` and `src/git/work/conflicts/stages.rs`: native graph,
  classification and conflict-stage evidence.
- `src/git/history/` and `src/git/work/history/`: owned historical requests,
  ancestry/comparison reads and bounded inspection grammars.
- `src/session/git/{commands,updates,input,presentation,processes}.rs`: native
  dispatch, source authority and lazy row projection.

## Validation

Intermediate checks, with focused results tied to the stated implementation
cohorts. Final ordered clean-HEAD Looper validation remains pending.

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/git.rs --scope src/git --scope src/session/git.rs --scope src/session/git`. Formatting, all-target compilation and scoped Clippy pass with zero blocking diagnostics across 103 selected files; 889 selected-profile diagnostics outside this scope are deferred. Native source policy still rejects 126 findings, reduced from 145.
- Earlier partial implementation: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --test-threads=1`. 239 passed and three failed before the current request/parser cohort. This is not current full-suite acceptance; the full suite has not been rerun on this checkpoint.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- families::output:: --test-threads=1`. Six output-family cases after the approved API adaptation.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- sequences::bisect:: sequences::presentation::native_bisect sequences::presentation::unsaved_bisect --test-threads=1`. Five native and presentation cases after bisect decomposition.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- conflicts::operation_sides conflicts::partial_and_missing conflicts::external_index conflicts::stale_inspection conflicts::explanation::historical_source conflicts::resolution::sides_then_stage history::inspection::blame_renamed_source --test-threads=1`. Seven cases after conflict-stage and quoted-path extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- history:: --test-threads=1`. 15 cases after ancestry/comparison and historical-file request extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- history::inspection::reflog_objects history::inspection::blame_renamed_source history::limits::bounded_inspection history::limits::metadata_admission history::inspection::presentation::native_inspections --test-threads=1`. Five cases after reflog/blame parser extraction and borrowing corrections.
- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- repository::acceptance::native_process_owner sequences::controls::ordered_controls --test-threads=1`. Ordered sequence controls passed; the process-owner fixture still waits at a terminal-departure prompt before Git status appears.
- Passed: `git diff --check`.

The current scoped inventory covers 103 of 1823 first-party Rust files. Git family
input, bisect workers, conflict inspection and the complete worker history tree
satisfy the five/three/ten bounds. The 126 remaining source findings block the
required quality command.

## Risks and Follow-ups

Complete the remaining operation-family preparation, verification, mutation and
session review handlers within this slice. Preserve immutable-source authority,
hook/signing policy, process ownership and every frozen command and budget.

Earlier full-suite execution observed three failures. The current focused checks
now pass request-pull remote review and ordered sequence controls with unchanged
limits. `repository::acceptance::native_process_owner` still reaches a terminal-
departure confirmation before Git status appears; no fixture correction was made.
These focused results do not establish that the complete Git suite is green.
Recheck the full suite before acceptance and retain exact observed failures
without unverified baseline or environmental attribution.

Test names, fixture behavior, assertions, timeouts and resource ceilings are
unchanged. The three approved output-request call expressions and their import
were adapted. Corrections outside the approved scope still require authority.
PR preparation and final clean-HEAD validation remain pending. The controller
owns merges and the complete delivery-set gate.
