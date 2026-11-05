# Native Git Integration Delivery

Status: Approved
Date: 2026-11-04
Delivery set: rev-git-integration
Slice strategy: behavior-first
Approved: owner, 2026-11-04; complete scope, matrix, policies, bounds, preparation commit/push and one autonomous fourteen-slice Looper delivery through dev
Planning baseline: `772051f7a70b4be66d86ccf4fdea27f9b6c2da68`
Spec: [Native Git Integration](../spec/editor-git-integration.md)
Model: [Native Git Model](../spec/editor-git-integration-model.md)
Tracker: [Native Git Tracker](../tracker/rev-git-integration.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration --test configuration --test commands --test windows --test panels --test files --test external_changes --test shell_commands -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal git_ -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-git-integration.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 13 - Combined native PTY and resource acceptance

## Outcome and execution boundary

Deliver GIT01-GIT20 in one ordered Looper run through dev, including the full
listed Git-management inventory. The fourteen slices separate independently
testable behavior, not fourteen approval cycles or separate controller runs.
Preparation publication and controller startup are authorized; their actual
state belongs to Git and the controller checkpoint. Approval is not a runtime
delivery or gate result.

The owner has confirmed full native coverage, maximum binding continuity,
canvas/return behavior, C-x g, direct commit composition, installed Git,
continued owned work on view exit, guarded application exit, separate fetch,
reviewed upstream push and both local tag kinds. Complete-set approval also
covers the matrix aliases, non-composer confirmation routes, branch/discard
policies and engineering defaults in the specification.

The explicit startup instruction authorizes publishing approved preparation
and delegates one branch/PR/report per slice, Rebase + Fast-forward integration
into dev, the selected exact-candidate gate and documentation closure to
Looper. Do not ask for individual slice merges. Main promotion, release tags,
global installation and public guide deployment are separate operations.

## Prerequisites and existing ownership

Integrated terminals, native shell commands, optional Pi AI, external reload,
startup modal/non-modal profiles, native completion and LSP are already delivered
at the planning baseline. No existing delivery needs restarting. Git is an
optional installed executable; ordinary editing must start without it.

Use a cohesive native Git owner fitted to src/git.rs and src/git/ where useful,
with existing session/command/input/display/configuration boundaries. Git owns
durable objects, refs, index, worktree state, hooks, signing and transport.
Rev owns typed observations, target admission, native UI and process lifetime.
No new runtime or development crate, database, plugin provider or general
process framework is authorized.

Reuse low-level process/group/stream/wake facilities where compatible. Their
current argv uses UTF-8 String and their error path is not itself the complete
Git process record. Preserve exact operating-system path bytes and retained
diagnostics through the smallest necessary boundary change; do not convert
lossy display paths back into arguments or weaken existing process callers.

One visible Git presentation owns input; retained source and terminal resources
keep their original owners. Git-generated views are typed read-only resources.
Commit/tag composition is native editable text with its own input context.
Keep q, literal message text, contextual prefixes and effective configured
bindings distinct. Register a public command only when its actual path works.

One active Git child per repository context owns its group, bounded streams,
deadline, cancellation, refresh and verification. Serialize overlapping
mutations of common repository storage across linked worktrees. Work can
continue after q without becoming detached or unowned. Integrate Git with the
existing guarded exit coordinator; do not implicitly abort a repository sequence
or terminate unrelated shell, terminal, LSP or AI work.

## Scope and acceptance mapping

| Functional family | Delivery slices | Proof required |
| --- | --- | --- |
| Context, Status, native presentation and processes | 1-2, 13 | Real repository identity, semantic selection, return ownership, bounded child/stream lifecycle. |
| Diffs, index, hunks, marks and discard | 2-3, 13 | Exact index/worktree contents, complete preflight rejection and modified-buffer protection. |
| Normal/amend/fixup commits | 4, 13 | Actual commit graph, messages, hooks, signing failures and retained drafts. |
| History/ranges/reflog/blame | 5, 13 | Exact objects, parents, paths, ancestry continuation and source immutability. |
| Refs, branches and both local tag kinds | 6, 13 | Exact refs/tag objects/peeled targets; no force or implicit publication. |
| Fetch and reviewed upstream push | 7, 13 | Local remote refs, separate integration and truthful publication verification. |
| Merge/rebase/autosquash | 8, 10, 13 | Actual Git sequence state, controls, conflicts and file/reload outcomes. |
| Ordered cherry-pick/revert and manual bisect | 9-10, 13 | Exact revision order/mainline, good/bad state and family-specific controls. |
| Conflict sides, saved resolution and optional explanation | 10, 13 | Index stages, saved bytes, deletion/staging and checked existing AI request. |
| Stashes and linked worktrees | 11, 13 | Exact stash objects, distinct worktree identities, ownership and cleanup. |
| Clone/init/ignore/submodule/subtree/notes/patch/sparse families | 12-13 | Every listed variant has a real prepared route and repository/file evidence. |
| Direct Git argv and explicit shell route | 12-13 | Checked exact parameters, separate authority and bounded process records. |
| Key/pointer contexts, configuration and user guidance | 1-14 | Matrix coverage, complete sample/origins, both profiles, Markdown and Spore parity. |

GIT16-GIT20 apply to each changing operation, not only to Slice 1. Keep a compact
variant-to-command/task/test map in the eventual native acceptance source and
slice reports. A catalog entry or a green status/commit journey does not prove
the other families. Do not introduce a second progress tracker.

## Backend preparation evidence

A bounded local study on Git 2.55.0 used newly created, isolated repositories,
no global/system Git configuration, no user remote and no live credentials.
It verified:

- porcelain-v2 NUL records for 1,503 untracked paths, including whitespace,
  newline and leading-dash names: 22,939 bytes; one local run took about 5 ms;
- lightweight direct refs versus annotated tag objects and exact peeled targets;
- common storage with distinct per-worktree Git directories and NUL worktree records;
- an installed failing pre-commit hook preserving HEAD and retaining its diagnostic;
- one reviewed local branch publication with upstream recorded and no tags;
- both SHA-1 and SHA-256 object identities;
- configured tag signing requiring additional creation input, with no tag left
  by the failed attempt. Do not silently disable policy or change the chosen type.

This is Git CLI feasibility evidence, not Rev runtime, RSS, rendering,
cancellation, credential or performance acceptance. Ignored local study files
are not a published dependency. Slice 1 verifies required installed capabilities
and finite accounting; Slice 13 measures the integrated editor.

Do not lock behavior to an assumed object length or the study's Git version.
Check the relevant machine-readable and operation capabilities. Missing Git,
an unavailable optional Git helper or unsupported operation must produce an
explicit capability result, not a placeholder success or automatic installer.

## Bounds and failure policy

Use the complete approved [git] catalog and hard ceilings in the specification.
All counts and byte caps participate in one 32 MiB editor-wide Git admission
budget. Bound parser scratch, observation/history retention, drafts, marks,
output and pending results as well as visible rows. Byte measurements are
payload observations, not a complete allocator/RSS estimate.

The native process owner starts with at most four retained repository contexts,
one child each, 64 KiB total stream service and 64 coalesced events per editor
turn. Quiet Git state creates no polling timer. Cancellation uses the existing
owned group facilities with a 500 ms grace and a 2 second cleanup deadline;
failure retains visible ownership and never claims rollback.

Reached caps, unavailable verification and cancellation after possible effects
are observable states. Do not silently drop target bytes, run a partial marked
request, retry a mutation, raise bounds or weaken assertions to make a check
pass. Any material contract or dependency change requires a reviewable decision.

## Ordered slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Repository context and native Status | `feat/rev-git-integration-01-status` | `feat(git): own repository context and native status` | `ops/default/report/slices/rev-git-integration/slice-1-status.md` |
| Slice 2 | Changes hunks and source visits | `feat/rev-git-integration-02-diffs` | `feat(git): inspect exact diffs and visit native sources` | `ops/default/report/slices/rev-git-integration/slice-2-diffs.md` |
| Slice 3 | Checked staging unstaging and discard | `feat/rev-git-integration-03-index` | `feat(git): stage exact paths hunks and checked groups` | `ops/default/report/slices/rev-git-integration/slice-3-index.md` |
| Slice 4 | Native commit composition | `feat/rev-git-integration-04-commits` | `feat(git): compose commits amend and fixup` | `ops/default/report/slices/rev-git-integration/slice-4-commits.md` |
| Slice 5 | History ranges reflog and blame | `feat/rev-git-integration-05-history` | `feat(git): browse history ranges reflog and blame` | `ops/default/report/slices/rev-git-integration/slice-5-history.md` |
| Slice 6 | Branches and both tag kinds | `feat/rev-git-integration-06-refs` | `feat(git): manage branches and local tag kinds` | `ops/default/report/slices/rev-git-integration/slice-6-refs.md` |
| Slice 7 | Separate fetch and reviewed push | `feat/rev-git-integration-07-transport` | `feat(git): fetch and push one reviewed upstream branch` | `ops/default/report/slices/rev-git-integration/slice-7-transport.md` |
| Slice 8 | Merge rebase and sequence control | `feat/rev-git-integration-08-integration` | `feat(git): run checked merge and rebase workflows` | `ops/default/report/slices/rev-git-integration/slice-8-integration.md` |
| Slice 9 | Ordered revisions and manual bisect | `feat/rev-git-integration-09-sequences` | `feat(git): apply ordered revisions and manual bisect` | `ops/default/report/slices/rev-git-integration/slice-9-sequences.md` |
| Slice 10 | Native conflict resolution | `feat/rev-git-integration-10-conflicts` | `feat(git): inspect and resolve checked conflicts` | `ops/default/report/slices/rev-git-integration/slice-10-conflicts.md` |
| Slice 11 | Stashes and linked worktrees | `feat/rev-git-integration-11-repository` | `feat(git): manage stashes and linked worktrees` | `ops/default/report/slices/rev-git-integration/slice-11-repository.md` |
| Slice 12 | Prepared Git families and direct commands | `feat/rev-git-integration-12-commands` | `feat(git): expose prepared repository command families` | `ops/default/report/slices/rev-git-integration/slice-12-commands.md` |
| Slice 13 | Combined native PTY and resource acceptance | `feat/rev-git-integration-13-acceptance` | `test(git): verify native workflows and bounded ownership` | `ops/default/report/slices/rev-git-integration/slice-13-acceptance.md` |
| Slice 14 | Git learning path and reference | `feat/rev-git-integration-14-guidance` | `docs(git): explain native repository workflows` | `ops/default/report/slices/rev-git-integration/slice-14-guidance.md` |

## Slice 1: Repository context and native Status

Prerequisite: approved and committed preparation on synchronized dev; no competing Rev controller.
Requirements: GIT01-GIT03, GIT16-GIT20.

- T1.1: Resolve worktree/common directories, bare/unborn/detached state, exact path bytes and SHA-1/SHA-256 objects through installed Git; add the bounded native repository/process owner and the complete startup [git] catalog, sample and origins.
- T1.2: Open semantic Status with C-x g, M-x and modal colon, preserve full-canvas return context and existing source/terminal owners, and render explicit complete/truncated/unavailable groups with checked selection and pointer targets.
- T1.3: Expose native active/recent process inspection and checked cancellation; preserve owned work after q, integrate guarded application exit, and verify real repositories, missing Git, malformed output, stale publication and bounded cleanup.

Validation: `cargo test --locked --test git_integration status:: -- --test-threads=1`, with the shared focused checks below.

## Slice 2: Changes hunks and source visits

Prerequisite: Slice 1 integrated into dev.
Requirements: GIT04, GIT17-GIT19.

- T2.1: Read staged, unstaged, unborn and untracked comparisons with exact semantic identities; admit bounded text hunks and truthful binary/rename/type-change/submodule/truncated detail.
- T2.2: Add Status expansion, read-only detail, current-source visits, retained navigation and resize/pointer mappings without making generated Git content a writable source.
- T2.3: Verify both layers of one path, multiple hunks, unusual path bytes, stale refresh, modified visiting buffers and return to the surviving editor layout.

Validation: `cargo test --locked --test git_integration diffs:: -- --test-threads=1`, with the shared focused checks below.

## Slice 3: Checked staging unstaging and discard

Prerequisite: Slice 2 integrated into dev.
Requirements: GIT05, GIT17, GIT19.

- T3.1: Stage and unstage exact paths or admitted text hunks; revalidate their complete comparison before changing the index.
- T3.2: Implement compatible marked path groups and nonoverlapping same-path/layer hunk groups; reject stale, mixed, duplicate or unsupported groups before launch.
- T3.3: Restore one confirmed tracked working-tree path from its index state without changing the index; protect unsaved buffers and verify actual index/worktree results and partial-effect diagnostics.

Validation: `cargo test --locked --test git_integration index:: -- --test-threads=1`, with the shared focused checks below.

## Slice 4: Native commit composition

Prerequisite: Slice 3 integrated into dev.
Requirements: GIT06, GIT16-GIT19.

- T4.1: Enter normal composition directly with c and provide ordinary native text editing, explicit submission, cancellation and session-only retained drafts.
- T4.2: Compose normal, reviewed amend and exact-target fixup requests; preserve checked index/HEAD, identity, hooks, signing policy and actual messages without hidden staging or saving.
- T4.3: Verify C-Enter/C-c C-c submission, C-g/C-c C-k cancellation, literal c, failed hooks, retained drafts, post-effect uncertainty and protected source/terminal state.

Validation: `cargo test --locked --test git_integration commits:: -- --test-threads=1`, with the shared focused checks below.

## Slice 5: History ranges reflog and blame

Prerequisite: Slice 4 integrated into dev.
Requirements: GIT07, GIT13, GIT17-GIT18.

- T5.1: Browse finite ancestry pages, exact revisions and parent comparisons; keep selection and explicit ancestry continuation across refresh.
- T5.2: Implement exact ordered ranges, separate range-base and ordered revision marks, read-only reflog and blame inspection, and canonical source/revision visits.
- T5.3: Verify detached/unborn/merge history, paging limits, full object identities, historical file renames, line attribution and no checkout or source modification during inspection.

Validation: `cargo test --locked --test git_integration history:: -- --test-threads=1`, with the shared focused checks below.

## Slice 6: Branches and both tag kinds

Prerequisite: Slice 5 integrated into dev.
Requirements: GIT08, GIT17-GIT19.

- T6.1: Present local/remote-tracking refs, tags, remotes and upstream facts with exact identities and checked branch create/switch/rename/non-forced deletion.
- T6.2: Implement t/git-tag-create with lightweight default and annotated message composition; retain distinct tag-object/peeled-target identities, Git metadata/signing and checked local tag deletion.
- T6.3: Verify current/unmerged branch protection, modified source admission, stale confirmation, both tag kinds, existing-name failure and conflicting signing policy without implicit publication or policy overrides.

Validation: `cargo test --locked --test git_integration refs:: -- --test-threads=1`, with the shared focused checks below.

## Slice 7: Separate fetch and reviewed push

Prerequisite: Slice 6 integrated into dev.
Requirements: GIT09, GIT16-GIT19.

- T7.1: Fetch an explicitly selected configured remote and refresh tracking facts without integrating the current branch.
- T7.2: Review and confirm one exact branch, configured upstream remote and destination; prevent implicit force, deletion, tags or mirror expansion and reject ambiguous tracking.
- T7.3: Verify local-bare remote outcomes, hostile/conflicting push settings, failed credentials/helpers, cancellation, bounded diagnostics and publication uncertainty; no live user remote or credential is used.

Validation: `cargo test --locked --test git_integration transport:: -- --test-threads=1`, with the shared focused checks below.

## Slice 8: Merge rebase and sequence control

Prerequisite: Slice 7 integrated into dev.
Requirements: GIT10, GIT16-GIT19.

- T8.1: Implement explicit fast-forward/normal/no-fast-forward merge policies and rebase from exact observed refs with checked worktree/source preconditions.
- T8.2: Prepare reviewed autosquash and observe Git-owned in-progress work through exact family-specific and generic status/continue/skip/abort controls; never launch an interactive external editor.
- T8.3: Verify successful and conflicted local histories, stale HEAD, missing sequence, wrong-family controls and skipped/aborted outcomes; source reload follows existing automatic/ask policy.

Validation: `cargo test --locked --test git_integration integration:: -- --test-threads=1`, with the shared focused checks below.

## Slice 9: Ordered revisions and manual bisect

Prerequisite: Slice 8 integrated into dev.
Requirements: GIT11, GIT17-GIT19.

- T9.1: Prepare ordered cherry-pick/revert requests from independent ordered marks, including exact mainline selection for merge commits.
- T9.2: Integrate their actual continuation/skip/abort states with the shared sequence owner and conflict entry; reject wrong order, duplicate/stale targets and wrong active families.
- T9.3: Provide manual bisect start/good/bad/skip/reset with exact review and real fixture verification; do not add automatic test execution.

Validation: `cargo test --locked --test git_integration sequences:: -- --test-threads=1`, with the shared focused checks below.

## Slice 10: Native conflict resolution

Prerequisite: Slice 9 integrated into dev.
Requirements: GIT12, GIT17-GIT19.

- T10.1: Observe exact conflict paths/kinds/index stages and operation-dependent Base/Current/Applied roles; provide read-only side inspection and canonical editable result visits.
- T10.2: Review whole-side replacement or resolved deletion and stage only a checked saved resolution; protect modified buffers, changed stages and unsupported/partial cases.
- T10.3: Route git-conflict-explain through the existing optional AI request/result owner with bounded conflict context and no automatic edit or resolution; verify no-provider behavior and request construction without mandatory live credentials.

Validation: `cargo test --locked --test git_integration conflicts:: -- --test-threads=1`, with the shared focused checks below.

## Slice 11: Stashes and linked worktrees

Prerequisite: Slice 10 integrated into dev.
Requirements: GIT13, GIT17-GIT19.

- T11.1: Implement native stash list/create/show/apply/pop/branch/drop with exact stash-object identity, checked source state, explicit destructive review and truthful conflict outcomes.
- T11.2: Implement worktree list/create/visit/lock/unlock/prune/remove with distinct worktree/common identities, checked paths and no forced removal of protected or dirty resources.
- T11.3: Verify stashes with conflicts, changed stash positions, linked-worktree ownership, lock reasons, stale removal and ordinary buffer/reload behavior; share existing read-only reflog/blame owners.

Validation: `cargo test --locked --test git_integration repository:: -- --test-threads=1`, with the shared focused checks below.

## Slice 12: Prepared Git families and direct commands

Prerequisite: Slice 11 integrated into dev.
Requirements: GIT14-GIT15, GIT16-GIT20.

- T12.1: Provide working native parameter/review routes for regular/shallow/bare/mirror clone, init, ignore files, submodule add/list/update/sync/deinit and subtree add/merge/pull/push/split.
- T12.2: Complete native notes add/remove/prune, mailbox/plain patch application, format-patch/save-diff/request-pull and sparse-checkout enable/disable/reapply/set/add with exact targets, capability checks and observable real effects.
- T12.3: Expose explicit structured direct Git argv and a separately explicit existing shell-command route; integrate complete contextual family prefixes, bound input/review and verify every listed variant using isolated local repositories.

Validation: `cargo test --locked --test git_integration families:: -- --test-threads=1`, with the shared focused checks below.

## Slice 13: Combined native PTY and resource acceptance

Prerequisite: Slice 12 integrated into dev.
Requirements: GIT01-GIT20 together.

- T13.1: Add one repository-native Git acceptance entry and Bash driver reusing existing Rust/Linux PTY support; share real isolated fixture builders and verify objects/files rather than command labels.
- T13.2: Exercise representative both-profile journeys covering Status/diffs/index/commit, refs/transport, sequence/conflict, contextual prefixes, pointer/resize and restoration with a live terminal; assert matrix routing at the cheapest reliable native layer.
- T13.3: Measure Git-only binary delta, startup/idle CPU, retained RSS/output and bounded fair service; verify process/FD cleanup, malformed/truncated/large input, ownership on q/app exit and zero dependency additions.

Validation: `bash scripts/check-git-integration.sh`, once on the completed native candidate, plus the shared focused checks.

## Slice 14: Git learning path and reference

Prerequisite: Slice 13 integrated into dev.
Requirements: GIT01-GIT20 and user-visible discovery.

- T14.1: Add a short independent Git learning path, task guides, exact command/key/configuration reference and ownership explanation; show compact status/diff/commit feedback without requiring Org learning.
- T14.2: Keep canonical Markdown and its Spore projection/navigation aligned; document explicit fetch/integration, local tags, reviewed push, sequence/conflict, direct commands and bounds, with no unimplemented feature claims.
- T14.3: Finish user-facing Unreleased entries and per-variant coverage, update the additive baseline/index references without rewriting historical gate evidence, and prepare exact-candidate closure metadata for the controller after its selected full gate passes.

Validation: `cargo run --locked --quiet --example documentation-check`, `bash scripts/build-user-guide.sh` and `git diff --check`; the controller owns the subsequent selected full gate.

## Focused checks and selected final gate

The future tests/git_integration.rs target, scripts/check-git-integration.sh
driver and native acceptance entry are created by this set; they are not
existing commands or current validation evidence. Build one shared Git fixture
owner and use local bare remotes, isolated configuration/identity and exact
object/file assertions. A small explicit test double is allowed only for an
unreproducible boundary failure and must state what real behavior it cannot prove.

For implementation slices, run formatter, targeted native Git module tests and
Clippy for the library/binary/changed test target. Batch related cases through
the Rust runner. Add narrowly selected shared-owner regressions only when
changed behavior reaches them. Each filter must select at least one test.
Do not repeat the combined PTY/resource journey per slice.

The selected final gate compiles/lints all targets, runs Git integration and
the affected configuration, command, window/panel, file/reload and shell
contracts, the named git_ modal-context cases and shared process regressions.
The one Git acceptance driver owns representative native/PTY journeys, source
checks and resource measurements; it must not call historical feature gates.
It verifies a live retained terminal across Git entry/exit without repeating
terminal-emulator acceptance. Optional AI explanation uses request-contract
evidence and no mandatory OAuth/network session.

Finish with the canonical documentation check, Markdown-to-Spore guide build
and whitespace validation. Vendor source is unchanged by this set. Acceptance
must neither edit dependency caches nor add Python as a validation prerequisite.

Allow roughly 10-20 minutes for a warm bounded build and selected gate on the
development host; this is an estimate, not a measured runtime or a new timeout.
Record actual phase costs in Slice 13. Investigate an unexpectedly slow check
before broadening its budget. Exhaustive release validation remains separate.

## Completion and closure

Each task normally has one Conventional Commit; each slice has one canonical
PR targeting dev and one report at the mapped path. Preserve the repository's
current task-ID message convention rather than invent another pattern. PR bodies
use Summary, Changes, Validation; the final Changes bullet links the immutable
slice report and Validation contains only exact successful commands.

After all fourteen slice PRs integrate, Looper owns the selected gate on clean
dev. Required repair changes follow a bounded fix/rev-git-integration-validation
PR with the same controller, frozen gate, retained failed evidence and explicit
repair authority. An environmental failure does not justify an empty PR.

Only after the selected gate passes may closure mark the plan/tracker Completed,
the spec/model delivered and reports delivered with the real validated candidate
and controller receipt. Do not overwrite historical delivery evidence. Final
closure paths are this plan, tracker, spec, model, ops/default/README.md and the
fourteen mapped slice reports. No runtime or public-guide changes belong in the
documentation-only closure commit.
