# Slice 10: Native Conflict Resolution

Status: reviewing
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-10-conflicts`
PR: `#267`

## Purpose

Deliver GIT12 and its GIT17-GIT19 contracts through T10.1-T10.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T10.1: git-conflicts refreshes actual Git sequence state and exact selectable
  unmerged paths. Operation Return inspects the selected path's available
  Base/Current/Applied stages, full objects, modes and bounded read-only text.
- Roles reflect the actual operation: rebase Current is its destination/onto
  side and Applied is the original replayed commit; revert Base is the reverted
  commit and Applied is its selected parent. Missing lineage remains explicit.
- Missing, binary, structural, non-text and partial sides remain visible.
  Conflict Return visits the canonical editable result through the existing
  file owner, preserving unsaved text. q returns to Operation without changing
  index entries or Git-owned sequence state.
- T10.2: 1/2/3 prepares complete compatible Base/Current/Applied working-file
  replacement while preserving unmerged index stages. K reviews removal of one
  exact supported result from disk and index, staging its deletion. x prepares
  staging of captured saved text or saved absence without changing disk.
- Exact native review accepts y, C-Enter or C-c C-c; Return alone does not
  accept. Changed saved bytes, file identity, modes, stages, HEAD, sequence or
  checkout/clean conversion facts invalidate confirmation before effects.
- Modified visiting result buffers are protected. Unrelated unsaved native
  buffers remain unchanged and need no hidden save. Stage and deletion preserve
  every other index entry, observed path state and bounded working-tree evidence.
  Continuing remains a separate explicit operation.
- T10.3: H (git-conflict-explain) copies the captured operation, exact path,
  roles, full object IDs, available side text and saved or visiting unsaved result
  into the existing optional AI owner. Its result is a read-only explanation;
  applying it cannot edit, stage or continue a Git operation.
- Missing Pi, stale capture or exceeded complete-input admission leaves manual
  inspection and resolution available. Changed disk/index/native sources mark
  the copied context historical; unsaved native text and return ownership survive.

## Implementation Notes

The existing repository worker and sequence fingerprint own inspection. Actual
metadata, refs, HEAD, complete index records and working-tree evidence are
checked before and after reading immutable blobs. Raw operating-system paths
and full SHA-1/SHA-256 identities supply authority; escaped labels never do.
Unmerged index entries without an operation marker are also observable without
inventing sequence lineage. Inspection creates no repository mutation.

Available ordinary-file stages share the configured detail byte cap, limited
by the existing 1 MiB scratch bound. Payloads, retained line ranges, path
selection and visible formatting participate in the same 32 MiB Git admission
budget. Existing repository/context counts, owned child lifetime, output caps,
deadlines and 5-second fixture waits remain unchanged. Unsupported modes retain
exact metadata without treating them as editable text. No dependency, provider
or general process owner was added.

Conflict targets travel with wrapped Operation rows into keyboard and pointer
selection. The full-canvas Sequence mapping is distinct from Status's fixed
header offset; painted state includes its viewport and conflict inspection.
Canonical result ingress rejects missing files, symlinks and paths resolving
outside the captured worktree, and reuses native file-buffer ownership.

Resolution preparation retains a bounded complete saved-result capture with
ordinary file type, mode, inode/device, length and nanosecond modification/change
facts. It rejects symlink ancestors, unsupported encodings/modes and partial
observations. Git checkout and clean conversion remain authoritative: converted
side bytes and expected saved-result object identities are captured and checked
again before the sole changing child. Anonymous hash input uses existing sealed
stdin and keeps its reservation with retained child cleanup.

Postconditions verify actual index entries, saved file content/identity, other
observed paths and bounded diff evidence, refs and Git sequence metadata. Failed
or uncertain effects retain the changing argv and diagnostics without replay,
rollback or inferred continuation. Native ordered-application lineage survives
resolution so a separately confirmed continue can finish the stopped revision.

The explanation worker rechecks the captured sequence and ordinary index before
publishing a bounded saved-result snapshot. Session serialization reserves the
existing Git budget before copying text and rejects the complete request when
its configured input cap is exceeded. Unsupported and binary content keeps its
metadata and explicit incomplete status; it is never silently promoted to text.

The existing Pi process/request/result owner receives fresh input-only context
with discovery disabled and actual --no-tools/--no-extensions arguments. Its
qualified native point retains return ownership without source-write authority.
The Git view remains usable while the optional request starts or fails. A ready
result changes focus only while its captured native origin still qualifies;
otherwise it remains available without stealing a changed source view.

## Contracts Added or Changed

- Eight functional native commands: git-conflicts, git-conflict-visit,
  git-conflict-use-base/current/applied, git-conflict-delete, git-conflict-stage
  and git-conflict-explain.
- keys.git_sequence has 33 active entries, including Return inspection.
- keys.git_conflict has 25 active entries for inspection, result visits,
  checked saved resolution and optional explanation in both interaction profiles.
- Command and configuration catalogs contain 547 commands and 269 entries.

## Files of Interest

- `src/git/conflicts.rs`, `src/git/work/conflicts.rs`: exact roles, stage
  metadata, bounded blob capture and stale observation rejection.
- `src/git/work/integration/sequence.rs`: actual unmerged index evidence even
  when no sequence family marker exists.
- `src/session/git/conflicts.rs`, `src/session/git/refs/integration.rs`: native
  inspection, exact selection and canonical editable result visits.
- `src/git/conflicts/resolution.rs`, `src/git/work/conflicts/resolution.rs`:
  exact resolution plans, bounded saved snapshots, Git conversion and proof.
- `tests/git_integration/conflicts_resolution.rs` and its presentation module:
  real index/disk effects, confirmation fences, ownership and continuation.
- `tests/git_integration/conflicts.rs` and `conflicts_presentation.rs`: real
  repository stages, both object formats/profiles and source ownership.

- `src/git/conflicts/explanation.rs`, `src/git/work/conflicts/explanation.rs`:
  checked copied-source snapshots with conservative currency and no mutation plan.
- `src/session/ai/conflicts.rs` and the existing AI capture/result owners:
  bounded copied context, input-only Pi admission and native return ownership.
- `tests/git_integration/conflicts_explanation.rs`: owned Rust RPC peer and
  native request, permission, failure, historical-source and read-only assertions.

## Validation

T10.1 development passed six conflict cases (19.18 seconds): four actual
operation families in both object formats with a newline/non-UTF-8 leading-dash
path; explicit missing/binary/partial/type-change stages; external unmerged
index without sequence markers; stale disk rejection; both-profile read-only
inspection/unsaved native result visits; and exact refreshed pointer selection.
Six existing native integration cases passed in 25.06 seconds. All 21 command
and configuration reference cases, strict Clippy, formatter, documentation and
whitespace checks passed. Documentation covers 541 commands, 269 configuration
entries, 76 pages and 485 local links.

Development corrections retained the approved assertions and all deadlines:
Git relocates the ordinary-file side of a type change to another exact path,
so the fixture now selects the actual symlink path for its mode assertion.
Pointer coverage exposed and corrected the new Sequence click offset; the test
still clicks a different exact conflict and checks its inspected file.

T10.2 development passed eight resolution cases together in 77.47 seconds and
an additional external-index resolution case in 2.47 seconds: all three sides
in both object formats, separately staged saved results/absence and deletion,
stale bytes/inodes/stages/HEAD/metadata, real post-index-change hook uncertainty,
Git CRLF conversion, retained cherry-pick/revert continuation, both native
profiles and modified-result/unrelated-source admission. Expanded unsupported
admission assertions passed in 3.74 seconds. Strict Clippy, all 21 reference
checks and documentation passed at 546 commands and 269 configuration entries.
The inode fixture also passes in 8.64 seconds after replacing the file with
identical bytes and removing its temporary old name; its assertion requires
the file-identity diagnostic, independently of other Status path changes.

T10.3 development passed three behavior cases plus the child-only Rust fixture
entrypoint in 25.49 seconds. They cover both profiles and object formats, exact
raw path escaping, full roles/objects/side bytes, saved and unsaved native text,
binary metadata, actual tool-disable arguments, missing provider, input bounds,
stale captures, readonly application rejection, native return ownership and
historical context after disk/index/buffer changes. No repository/index/HEAD
mutation is authorized by the explanation. Five existing AI result-owner cases
passed in 1.28 seconds; 21 command/configuration references and documentation
passed at 547 commands, 269 configuration entries, 76 pages and 485 local links.

The controlled peer exercises the existing real RPC owner without credentials
or provider calls. It proves request, result and permission contracts, not a
provider's explanation quality or credential readiness. It reuses the compiled
Rust test executable and existing shell/rustix support, adding no Python
validation prerequisite, dependency or process framework. A fixture startup
failure came from reopening a socket descriptor through /proc as if it were a
pipe; duplicating its inherited descriptor fixed the fixture. The five-second
waits, settlement gate, source assertions and resource limits remain unchanged.
An initial raw-path assertion was corrected to the renderer's exact uppercase
hex escape; it still requires lossless newline/non-UTF-8 identity presentation.

Successful development commands:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration integration::presentation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts::resolution:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts::resolution::external_resolution -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts::partial_and_missing -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts::resolution::changed_review -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration conflicts::explanation:: -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results:: -- --test-threads=1`
- `git diff --check`

Final validation will use the configured formatter/Clippy/conflict prefix,
then native integration presentation, the five shared AI result-owner cases,
command/configuration references, documentation and whitespace. The controller
records the exact ordered checks on the clean canonical PR head after its
PR-reference follow-up commit; no final receipt is claimed here.

## Risks and Follow-ups

Combined PTY/resource acceptance remains Slice 13 and full User Guide work
remains Slice 14. Optional explanation quality depends on the configured provider;
manual inspection and checked resolution require no provider.
Integration and delivery-set closure remain controller-owned.
