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

Native publication now routes checked results through operation-family owners.
Each owner retains the original context, generation and immutable-source checks
before replacing a confirmed observation. Failed cleanup still retains the
exact operation and process reservation.

Worker launch, repository observation, child capture and final record conversion
have separate owners. Session command dispatch, update routing, input authority
and visible-row projection use the existing native operation and page families.
Status parsing separates header validation, tracked-path decoding and bounded
path admission while continuing to validate records after truncation.

This is a partial implementation checkpoint. Operation-specific preparation and
mutation routines still require source-bound corrections.

## Implementation Notes

- A worker owns its captured directory, request, policy, cancellation, progress,
  reservation and wake callback until transfer into the named thread. Timeout
  and private-output classifications retain their existing operation families.
- An invocation transfers exact OS argv and staged stdin to one child. Capture
  keeps aggregate output accounting, diagnostic redaction and retained cleanup
  separate from exit-status admission.
- Completion preserves cancellation/deadline checks, indeterminate outcomes,
  malformed comparison diagnostics and reservation transfer to published data.
- Session routing retains first-match operation-ID precedence, prompt/focus
  guards and every painted selection token. Row iterators remain lazy; only
  visible labels are allocated.
- The worker retains request authority while private operation helpers borrow
  captured plans and observations. Consent takes ownership of its admitted
  message; small intent enums remain explicit copyable values.
- Explicit shared-handle clones expose ownership. Guarded file-length
  conversions retain their existing byte caps; timestamp fingerprints preserve
  all signed timestamp bits, including pre-epoch values.

## Contracts Added or Changed

Public command/API shapes, resource limits and protocol formats are preserved.
Native behavior still needs complete verification through the frozen checks.

Six public history-inspection methods retain their owned captured-authority
parameters under narrow, reasoned Clippy expectations to preserve their APIs.
No lint baseline or source-policy exception was added.

## Files of Interest

- `src/git/publication/`: checked publication for native operation families.
- `src/git/work/{worker,request,observation,command,completion}.rs`: admitted
  worker lifetime, launch policy, observation and child-result ownership.
- `src/git/status.rs`: bounded porcelain-v2 parsing and path validation.
- `src/session/git/{commands,updates,input,presentation,processes}.rs`: native
  dispatch, publication routing, selection authority and lazy row projection.

## Validation

Intermediate checks on the partial implementation:

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/git.rs --scope src/git --scope src/session/git.rs --scope src/session/git`. Formatting, all-target compilation and scoped Clippy pass (zero blocking diagnostics across 94 selected files; 889 selected-profile diagnostics outside this scope are deferred). The command fails on 170 remaining source-bound diagnostics.
- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --test-threads=1` after worker and parser extraction (239 passed; three failed).
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` (106 reachable pages and 743 local links/anchors; command and configuration catalogs pass).
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --locked -p rev --lib` after worker-boundary extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- presentation --test-threads=1` (66 cases) after publication and session extraction.
- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- lifetime refs::presentation families::output::remote_review repository::acceptance::native_process_owner sequences::controls::ordered_controls --test-threads=1` after borrowing corrections (eight passed; the same three failures reproduced).
- Passed: `git diff --check`.

The current native source inventory selects 94 of 1814 first-party Rust files;
170 source-bound diagnostics remain in operation-specific owners. Selected
ownership and conversion diagnostics pass within the current Git scope. Final
ordered Looper validation remains pending on the complete clean published candidate.

## Risks and Follow-ups

Complete the remaining operation-family preparation, verification, mutation and
session review handlers within this slice. Keep test coverage, immutable-source
authority, hook/signing policy, budgets and frozen command strings unchanged.
Three frozen Git integration failures reproduce in isolation:

- `families::output::remote_review`: the admitted preparation reaches its
  three-second transport deadline before publication, with no effects started.
- `repository::acceptance::native_process_owner`: fixture directory loading
  reaches a terminal-departure confirmation before the expected Git guard.
- `sequences::controls::ordered_controls`: the five-second fixture wait expires
  while the application child remains owned in cleanup.

No fixture, assertion, deadline or budget has been changed. Resolve these
failures before acceptance; fixture corrections outside the injected source
selection require explicit scope authorization. PR preparation and final
clean-HEAD validation remain pending. The controller owns merges and the
complete delivery-set gate.
