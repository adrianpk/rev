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

Unified patch parsing owns hunk ranges, body classification and malformed
comparison evidence. Family command definitions and input preparation, index
selection checks, sequence control lineage and typed repository action admission
now have separate native owners. History publication owns its root and reservation
before appending retained pages.

This is a partial implementation checkpoint. Worker mutation routines and session
review handlers still require source-bound corrections.

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

- Patch indexing retains 2048-byte display chunks, complete raw evidence and
  capacity admission before allocation. Malformed evidence loses every hunk ID.
- Family labels and argv templates remain exact. Input and output validation,
  shell construction and source observation preserve their original error order.
- Index groups retain complete-source, direction, hunk identity and overlapping
  path/range checks. Sequence control retains family-specific lineage proofs,
  unsupported-action rejection and the aggregate 256-action todo limit.
- History owns its captured root and admission while retaining prior shared pages;
  page reservations and retained-page limits are unchanged.

## Contracts Added or Changed

Public command/API shapes, resource limits and protocol formats are preserved.
Native behavior still needs complete verification through the frozen checks.

Six public history-inspection methods retain their owned captured-authority
parameters under narrow, reasoned Clippy expectations to preserve their APIs.
No lint baseline or source-policy exception was added.

A scope decision is required before correcting the existing public
`Repositories::prepare_family_output` signature: it has six parameters including
`self`, while the frozen source guard allows five and the approved plan requires
compatible package APIs. The proposed bounded change is a native
`FamilyOutput { family, args, output }` request and
`prepare_family_output(&mut self, directory, context, request)`, with its session
call and three existing test calls adapted. This proposal is not implemented;
all assertions, command arguments, limits and budgets must remain intact.

## Files of Interest

- `src/git/publication/`: checked publication for native operation families.
- `src/git/work/{worker,request,observation,command,completion}.rs`: admitted
  worker lifetime, launch policy, observation and child-result ownership.
- `src/git/status.rs`: bounded porcelain-v2 parsing and path validation.
- `src/session/git/{commands,updates,input,presentation,processes}.rs`: native
  dispatch, publication routing, selection authority and lazy row projection.

- `src/git/diff.rs` and `src/git/diff/patch.rs`: bounded immutable patch indexing.
- `src/git/families/{commands,preparation}.rs`: exact command definitions and
  checked input/source admission.
- Native `history`, `index`, `integration`, `refs`, `repository` and `sequences`
  owners: publication, selection and operation-specific preconditions.

## Validation

Intermediate checks; final ordered clean-HEAD Looper validation is pending:

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/git.rs --scope src/git --scope src/session/git.rs --scope src/session/git`. Formatting, all-target compilation and scoped Clippy pass: zero blocking diagnostics across 97 selected files; 889 selected-profile diagnostics outside this scope are deferred. The command fails on 145 remaining source-bound diagnostics, reduced from 170.
- Earlier partial implementation: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --test-threads=1` completed with 239 passes and three failures. This result predates the current preparation/admission cohort; the full suite has not been rerun on this checkpoint.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` (106 reachable pages, 743 local links/anchors and the command/configuration catalogs).
- Failed after patch/family extraction: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- diffs:: families::literal_admission families::clone_kinds families::explicit_init families::direct:: families::output::output_admission conflicts::resolution::sides_then_stage conflicts::explanation::historical_source sequences::bisect:: --test-threads=1` (19 passed, seven failed; all 11 diff cases passed).
- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- conflicts::resolution::sides_then_stage families::direct::explicit_shell families::output::output_admission sequences::bisect::bisect_fences --test-threads=1` (output admission passed; conflict resolution, explicit shell and bisect failures reproduced).
- Failed after native admission extraction: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- index::groups:: history::ancestry_pages history::unborn_and_detached refs::branch_guards refs::tags::tag_guards repository::stash_admission repository::worktrees::admission_and_ownership sequences::application_admission integration::integration_guards integration::guards::incompatible_sequence integration::guards::external_rebase sequences::controls::control_fences --test-threads=1` (eight passed, six failed). History pagination/unborn roots, index rejection, branch/tag guards, stash/worktree admission and ordered application admission passed.
- Passed: `git diff --check`.

The current scoped inventory covers 97 of 1817 first-party Rust files. Native
comparison, family input and repository-facing admission owners now satisfy the
five/three/ten bounds except for the existing six-parameter public output API.
The remaining worker/session findings and integration failures prevent acceptance.

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

Later focused execution also observed cleanup waits in conflict resolution,
index groups, integration controls and bisect, plus a direct-shell publication
deadline. The 14-case admission batch's six failures all expired at the existing
five-second fixture wait while a child remained owned in cleanup. These failures
are not attributed to the baseline or to an environmental cause without evidence.

No fixture, assertion, deadline or budget has been changed. Resolve integration
failures before acceptance; corrections outside the injected source selection
require explicit scope authorization. The public output API proposal also needs
an explicit exception to the approved package-compatibility constraint before
implementation. PR preparation and final clean-HEAD validation remain pending.
The controller owns merges and the complete delivery-set gate.
