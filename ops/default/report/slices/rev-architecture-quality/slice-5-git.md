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

Commit composition now separates draft construction, captured-index admission,
HEAD/parent/message/tree proofs and result publication. Index changes separate
path and clean-filter preparation, hunk content/patch preparation, entry grammar
and batched postconditions. Conflict resolution and discard separate complete
file capture, confirmed-source rechecks, child execution and postconditions.
Session mark admission and discard source capture remain explicit owners.

Prepared-family requests now retain their captured directory, source and argv
through destination resolution, typed proofs and operation-specific verification.
Output publication owns the parent descriptor and staging anchor together.
Submodule child capture restores its cwd before returning every success or error;
subtree checks separate remote anchors, lineage, prefix trees and local sources.
Patch/mailbox decoding and private-index predictions remain independent of the
accepted changing child. Session family source guards, parsing, submission,
preparation publication and review projection have distinct owners.

Reference observation separates HEAD compatibility, admitted ref catalogs, exact
record fields and credential-free remote-name discovery. Branch actions retain
one captured before snapshot through action-specific argument preparation,
launch rechecks and postconditions.

Tag preparation retains the exact requested identity through name/target and
signing checks. Fetch and push separate captured configuration/upstream admission,
the accepted child and actual local/remote verification. Integration retains one
reviewed request and lineage through launch, stopped-state classification and
completed merge/rebase verification. Sequence file capture and metadata decoding
have independent owners; autosquash source decoding and target grouping remain
bound to the captured linear range.

Session composition separates input source modes, command/review routing and text
projection. Comparison replacement maps only surviving exact patch units before
publishing the new detail. History inspection renders each native page lazily;
ordered marks, conflict review and manual bisect/tag review retain their original
authority and user-visible behavior.

This is a partial checkpoint. Native repository/worktree and ordered-application
owners, plus remaining ref/repository session handlers, still require source-bound
corrections.

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
- Commit preparation preserves amend/fixup messages and parents, full-index
  admission, the no-active-sequence guard and rechecks before the sole mutator.
  HEAD reference/advance, exact composition and complete tree proofs retain
  their original error order and admission calculations.
- Index parsing retains exact NUL framing, stage/type checks and count limits.
  Hunk parsing keeps both byte sides and prior-prefix state for no-newline
  markers; descending application, mode handling and patch preflight are intact.
  Literal path manifests, clean filters, gitlinks and read batches are unchanged.
- Saved conflict capture checks ordinary canonical parents, complete text and
  file identity through open/read. Resolution still revalidates conversion,
  unrelated paths, index entries and exact sequence metadata after the child.
- Discard retains NOFOLLOW verification, executable/symlink checks and complete
  bytes. Session consent captures the same visiting owners, checked stamps,
  admission and confirmation authority; modified sources still reject.
- Prepared-family input preserves directory/destination canonicalization,
  source/ref authority, shell policy and exact argv resolution. Creation checks
  retain unborn, bare, shallow, mirror and source-publication postconditions.
- Output publication retains descriptor-relative staging, atomic nonoverwriting
  hard links, cancellation/deadline checks and complete-byte reinspection.
  Notes and sparse operations retain their independent ref/configuration,
  materialization and exact selection checks.
- Submodule checks retain gitlink/name/URL admission, child saved evidence and
  cwd restoration. Subtree checks preserve remote anchors, manual sequence
  classification, exact tree/ancestry and unchanged outside-prefix sources.
- Patch input still requires complete ordinary bytes and bounded mailbox
  envelopes. Mailinfo field/date/message admission, ordered private-index
  prediction, exact commit lineage and retained workspace cleanup are unchanged.
- Reference parsing preserves exact NUL/newline framing, eight bounded fields,
  byte identities, max-ref/remote truncation and before/after HEAD guards.
  Branch checks preserve clean switching, checked deletion ancestry, exact
  targets and separate verified-failure versus uncertain-effects publication.
- Tags preserve name/target checks, signing policy, exact annotation headers,
  signature/body bytes and unchanged HEAD checks. Transport preserves explicit
  configured upstream selection, endpoint redaction, exact argv, configuration
  rechecks, no implicit retry and unresolved remote effects.
- Integration preserves policy/error order, the sole changing child, retained
  cleanup, exact active lineage and completed branch/ancestry/parent/tree proofs.
  Sequence discovery retains directory sentinels, ordinary-file checks, the
  64-file/detail/scratch ceilings, cancellation and complete before/after checks.
- Autosquash retains its 256-commit linear range, bounded complete subjects,
  exact/hash-prefix/subject-prefix target precedence and ordered todo ownership.
- Session composer source modes preserve buffer admission, retained view/input
  restoration and submit/cancel behavior. Comparison mapping preserves exact
  bytes and path/layer identity before lazy reveal. History pages preserve row
  indices, selection identities and 2048-byte chunks; review messages are intact.
- Session family routing preserves visiting-buffer/native-directory protection,
  bounded JSON, captured composer authority and all original review messages.
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
- `src/git/work/families/` and `src/session/git/families.rs`: prepared requests,
  bounded file/mail decoding, family-specific execution and native review.
- `src/git/work/refs/{observation,branch}.rs`: complete catalog/remote observation
  and exact local branch preparation/verification.
- `src/git/work/refs/{tag,transport}.rs`, `src/git/work/refs/transport/push.rs`
  and `src/git/work/integration/`: checked tag/transport requests, sequence
  capture/decoding, integration lineage and autosquash grouping.
- `src/session/git/{commit,comparisons,conflicts}.rs`, `src/session/git/commit/`,
  `src/session/git/history/` and `src/session/git/refs/{bisect,tag}.rs`: composer
  source modes, exact comparison mapping and lazy inspection/review projection.
- `src/git/work/{commit,discard}.rs`, `src/git/work/conflicts/resolution/` and
  `src/git/work/index/`: mutation preparation, file capture and postconditions.
- `src/session/git/{index,discard}.rs`: captured mark and discard-source authority.
- `src/session/git/{commands,updates,input,presentation,processes}.rs`: native
  dispatch, source authority and lazy row projection.

## Validation

Intermediate checks, with focused results tied to the stated implementation
cohorts. Final ordered clean-HEAD Looper validation remains pending.

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/git.rs --scope src/git --scope src/session/git.rs --scope src/session/git`. Formatting, all-target compilation and scoped Clippy pass with zero blocking diagnostics across 112 selected files; 889 selected-profile diagnostics outside this scope are deferred. Native source policy still rejects 33 findings, reduced from 64.
- Earlier partial implementation: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --test-threads=1`. 239 passed and three failed before the current request/parser cohort. This is not current full-suite acceptance; the full suite has not been rerun on this checkpoint.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- families::output:: --test-threads=1`. Six output-family cases after the approved API adaptation.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- sequences::bisect:: sequences::presentation::native_bisect sequences::presentation::unsaved_bisect --test-threads=1`. Five native and presentation cases after bisect decomposition.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- conflicts::operation_sides conflicts::partial_and_missing conflicts::external_index conflicts::stale_inspection conflicts::explanation::historical_source conflicts::resolution::sides_then_stage history::inspection::blame_renamed_source --test-threads=1`. Seven cases after conflict-stage and quoted-path extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- history:: --test-threads=1`. 15 cases after ancestry/comparison and historical-file request extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- history::inspection::reflog_objects history::inspection::blame_renamed_source history::limits::bounded_inspection history::limits::metadata_admission history::inspection::presentation::native_inspections --test-threads=1`. Five cases after reflog/blame parser extraction and borrowing corrections.
- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- repository::acceptance::native_process_owner sequences::controls::ordered_controls --test-threads=1`. Ordered sequence controls passed; the process-owner fixture still waits at a terminal-departure prompt before Git status appears.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- commits:: index::discard:: conflicts::resolution:: --test-threads=1`. 31 mutation and presentation cases after commit, discard and conflict-resolution decomposition, before the index/session extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- index:: --test-threads=1`. All 24 index cases after path/hunk/entry extraction and session mark/discard-source separation.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- commits::normal_objects commits::modes::amend_objects commits::modes::fixup_objects commits::effects::merge_noop_amend index::path_roundtrip index::hunk_boundaries index::groups::paths conflicts::resolution::saved_and_deleted conflicts::resolution::effects_remain_visible --test-threads=1`. Nine focused cases after the final borrowing corrections, including commit consumers of the extracted entry grammar.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- families:: --test-threads=1`. All 48 prepared-family cases after worker/session decomposition, before the final borrowing corrections and reference-observation extraction. This includes native review, output, notes/sparse, nested repositories and independently decoded patch/mailbox semantics.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- refs:: families::patch::payload_semantics families::presentation::native_notes_sparse families::subtree::roundtrip families::output::request_pull --test-threads=1`. All 21 reference catalog/branch/tag and representative family regressions after reference extraction and the final borrowing corrections.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- refs::tags transport:: --test-threads=1`. All 18 tag/transport cases after their native decomposition, before final borrowing corrections and the subsequent integration/session changes.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- integration:: sequences::bisect:: sequences::controls::ordered_controls --test-threads=1`. All 23 integration, autosquash, sequence-control and manual-bisect cases after native integration/metadata extraction, before the final borrowing corrections and session extraction.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- commits::presentation:: commits::acceptance:: diffs::presentation:: history:: conflicts::resolution::presentation:: conflicts::presentation:: refs::tags::presentation:: families::presentation:: sequences::presentation:: --test-threads=1`. All 48 session/native regression cases after the complete integration, borrowing and review cohort. Commit draft/cancel/recovery and terminal retention, conflict resolution, exact hunk refresh/source visits, prepared-family composition, historical inspection/marks, tag annotations and manual sequence/bisect controls pass.
- Passed: `git diff --check`.

The current scoped inventory covers 112 of 1832 first-party Rust files. Git family
input, bisect workers, conflict inspection/resolution, commit/discard workers,
the complete worker history/index/prepared-family trees, reference observation
and branches/tags/transport, the complete native integration tree, and session
index/discard/family/composer/comparison/conflict/history/tag/bisect owners satisfy
the five/three/ten bounds. The 33 remaining source findings block the
required quality command.

## Risks and Follow-ups

Complete the remaining native repository/worktree and ordered-application owners,
plus ref/repository session handlers, within this slice. Preserve immutable-source authority,
hook/signing policy, process ownership and every frozen command and budget.

Earlier full-suite execution observed three failures. Subsequent focused checks
passed request-pull remote review and ordered sequence controls with unchanged
limits. The last focused reproduction of
`repository::acceptance::native_process_owner` reached terminal-departure consent
before Git status appeared; no fixture correction was made.
These focused results do not establish that the complete Git suite is green.
Recheck the full suite before acceptance and retain exact observed failures
without unverified baseline or environmental attribution.

Test names, fixture behavior, assertions, timeouts and resource ceilings are
unchanged. The three approved output-request call expressions and their import
were adapted. Corrections outside the approved scope still require authority.
PR preparation and final clean-HEAD validation remain pending. The controller
owns merges and the complete delivery-set gate.
