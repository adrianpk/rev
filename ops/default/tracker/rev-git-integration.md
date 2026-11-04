# Native Git Integration Tracker

Status: Approved
Date: 2026-11-04
Delivery set: rev-git-integration
Plan: [Native Git Integration Delivery](../plan/rev-git-integration.md)
Spec: [Native Git Integration](../spec/editor-git-integration.md)
Model: [Native Git Model](../spec/editor-git-integration-model.md)
Approved: owner, 2026-11-04; complete fourteen-slice plan and autonomous Looper execution/integration through dev
Active slice: Slice 1 - Repository context and native Status
Progress owner: after startup, Looper checkpoint, canonical PRs and exact-candidate validation receipts

## Canonical slice map

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

## Tasks and integration evidence

## Slice 1: Repository context and native Status

Status: reviewing
Branch: `feat/rev-git-integration-01-status`
Expected PR title: `feat(git): own repository context and native status`
Report: `ops/default/report/slices/rev-git-integration/slice-1-status.md`
Requirements: GIT01-GIT03, GIT16-GIT20
PR: pending

- [x] T1.1: Resolve worktree/common directories, bare/unborn/detached state, exact path bytes and SHA-1/SHA-256 objects through installed Git; add the bounded native repository/process owner and the complete startup [git] catalog, sample and origins.
- [x] T1.2: Open semantic Status with C-x g, M-x and modal colon, preserve full-canvas return context and existing source/terminal owners, and render explicit complete/truncated/unavailable groups with checked selection and pointer targets.
- [x] T1.3: Expose native active/recent process inspection and checked cancellation; preserve owned work after q, integrate guarded application exit, and verify real repositories, missing Git, malformed output, stale publication and bounded cleanup.

Validation: T1.1-T1.3 development checks pass: formatter, strict focused Clippy and 21 native Git tests. Shared development evidence includes 10 process tests, 10 command-reference tests, 11 configuration-reference tests, 41 window tests and 19 panel tests. The controller records final clean-HEAD validation after PR-reference follow-up.
Integration: pending; T1.1-T1.3 are implemented on the slice branch. Canonical PR preparation is in progress; no merge is claimed.

## Slice 2: Changes hunks and source visits

Status: pending
Branch: `feat/rev-git-integration-02-diffs`
Expected PR title: `feat(git): inspect exact diffs and visit native sources`
Report: `ops/default/report/slices/rev-git-integration/slice-2-diffs.md`
Requirements: GIT04, GIT17-GIT19
PR: pending

- [ ] T2.1: Read staged, unstaged, unborn and untracked comparisons with exact semantic identities; admit bounded text hunks and truthful binary/rename/type-change/submodule/truncated detail.
- [ ] T2.2: Add Status expansion, read-only detail, current-source visits, retained navigation and resize/pointer mappings without making generated Git content a writable source.
- [ ] T2.3: Verify both layers of one path, multiple hunks, unusual path bytes, stale refresh, modified visiting buffers and return to the surviving editor layout.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 3: Checked staging unstaging and discard

Status: pending
Branch: `feat/rev-git-integration-03-index`
Expected PR title: `feat(git): stage exact paths hunks and checked groups`
Report: `ops/default/report/slices/rev-git-integration/slice-3-index.md`
Requirements: GIT05, GIT17, GIT19
PR: pending

- [ ] T3.1: Stage and unstage exact paths or admitted text hunks; revalidate their complete comparison before changing the index.
- [ ] T3.2: Implement compatible marked path groups and nonoverlapping same-path/layer hunk groups; reject stale, mixed, duplicate or unsupported groups before launch.
- [ ] T3.3: Restore one confirmed tracked working-tree path from its index state without changing the index; protect unsaved buffers and verify actual index/worktree results and partial-effect diagnostics.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 4: Native commit composition

Status: pending
Branch: `feat/rev-git-integration-04-commits`
Expected PR title: `feat(git): compose commits amend and fixup`
Report: `ops/default/report/slices/rev-git-integration/slice-4-commits.md`
Requirements: GIT06, GIT16-GIT19
PR: pending

- [ ] T4.1: Enter normal composition directly with c and provide ordinary native text editing, explicit submission, cancellation and session-only retained drafts.
- [ ] T4.2: Compose normal, reviewed amend and exact-target fixup requests; preserve checked index/HEAD, identity, hooks, signing policy and actual messages without hidden staging or saving.
- [ ] T4.3: Verify C-Enter/C-c C-c submission, C-g/C-c C-k cancellation, literal c, failed hooks, retained drafts, post-effect uncertainty and protected source/terminal state.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 5: History ranges reflog and blame

Status: pending
Branch: `feat/rev-git-integration-05-history`
Expected PR title: `feat(git): browse history ranges reflog and blame`
Report: `ops/default/report/slices/rev-git-integration/slice-5-history.md`
Requirements: GIT07, GIT13, GIT17-GIT18
PR: pending

- [ ] T5.1: Browse finite ancestry pages, exact revisions and parent comparisons; keep selection and explicit ancestry continuation across refresh.
- [ ] T5.2: Implement exact ordered ranges, separate range-base and ordered revision marks, read-only reflog and blame inspection, and canonical source/revision visits.
- [ ] T5.3: Verify detached/unborn/merge history, paging limits, full object identities, historical file renames, line attribution and no checkout or source modification during inspection.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 6: Branches and both tag kinds

Status: pending
Branch: `feat/rev-git-integration-06-refs`
Expected PR title: `feat(git): manage branches and local tag kinds`
Report: `ops/default/report/slices/rev-git-integration/slice-6-refs.md`
Requirements: GIT08, GIT17-GIT19
PR: pending

- [ ] T6.1: Present local/remote-tracking refs, tags, remotes and upstream facts with exact identities and checked branch create/switch/rename/non-forced deletion.
- [ ] T6.2: Implement t/git-tag-create with lightweight default and annotated message composition; retain distinct tag-object/peeled-target identities, Git metadata/signing and checked local tag deletion.
- [ ] T6.3: Verify current/unmerged branch protection, modified source admission, stale confirmation, both tag kinds, existing-name failure and conflicting signing policy without implicit publication or policy overrides.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 7: Separate fetch and reviewed push

Status: pending
Branch: `feat/rev-git-integration-07-transport`
Expected PR title: `feat(git): fetch and push one reviewed upstream branch`
Report: `ops/default/report/slices/rev-git-integration/slice-7-transport.md`
Requirements: GIT09, GIT16-GIT19
PR: pending

- [ ] T7.1: Fetch an explicitly selected configured remote and refresh tracking facts without integrating the current branch.
- [ ] T7.2: Review and confirm one exact branch, configured upstream remote and destination; prevent implicit force, deletion, tags or mirror expansion and reject ambiguous tracking.
- [ ] T7.3: Verify local-bare remote outcomes, hostile/conflicting push settings, failed credentials/helpers, cancellation, bounded diagnostics and publication uncertainty; no live user remote or credential is used.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 8: Merge rebase and sequence control

Status: pending
Branch: `feat/rev-git-integration-08-integration`
Expected PR title: `feat(git): run checked merge and rebase workflows`
Report: `ops/default/report/slices/rev-git-integration/slice-8-integration.md`
Requirements: GIT10, GIT16-GIT19
PR: pending

- [ ] T8.1: Implement explicit fast-forward/normal/no-fast-forward merge policies and rebase from exact observed refs with checked worktree/source preconditions.
- [ ] T8.2: Prepare reviewed autosquash and observe Git-owned in-progress work through exact family-specific and generic status/continue/skip/abort controls; never launch an interactive external editor.
- [ ] T8.3: Verify successful and conflicted local histories, stale HEAD, missing sequence, wrong-family controls and skipped/aborted outcomes; source reload follows existing automatic/ask policy.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 9: Ordered revisions and manual bisect

Status: pending
Branch: `feat/rev-git-integration-09-sequences`
Expected PR title: `feat(git): apply ordered revisions and manual bisect`
Report: `ops/default/report/slices/rev-git-integration/slice-9-sequences.md`
Requirements: GIT11, GIT17-GIT19
PR: pending

- [ ] T9.1: Prepare ordered cherry-pick/revert requests from independent ordered marks, including exact mainline selection for merge commits.
- [ ] T9.2: Integrate their actual continuation/skip/abort states with the shared sequence owner and conflict entry; reject wrong order, duplicate/stale targets and wrong active families.
- [ ] T9.3: Provide manual bisect start/good/bad/skip/reset with exact review and real fixture verification; do not add automatic test execution.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 10: Native conflict resolution

Status: pending
Branch: `feat/rev-git-integration-10-conflicts`
Expected PR title: `feat(git): inspect and resolve checked conflicts`
Report: `ops/default/report/slices/rev-git-integration/slice-10-conflicts.md`
Requirements: GIT12, GIT17-GIT19
PR: pending

- [ ] T10.1: Observe exact conflict paths/kinds/index stages and operation-dependent Base/Current/Applied roles; provide read-only side inspection and canonical editable result visits.
- [ ] T10.2: Review whole-side replacement or resolved deletion and stage only a checked saved resolution; protect modified buffers, changed stages and unsupported/partial cases.
- [ ] T10.3: Route git-conflict-explain through the existing optional AI request/result owner with bounded conflict context and no automatic edit or resolution; verify no-provider behavior and request construction without mandatory live credentials.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 11: Stashes and linked worktrees

Status: pending
Branch: `feat/rev-git-integration-11-repository`
Expected PR title: `feat(git): manage stashes and linked worktrees`
Report: `ops/default/report/slices/rev-git-integration/slice-11-repository.md`
Requirements: GIT13, GIT17-GIT19
PR: pending

- [ ] T11.1: Implement native stash list/create/show/apply/pop/branch/drop with exact stash-object identity, checked source state, explicit destructive review and truthful conflict outcomes.
- [ ] T11.2: Implement worktree list/create/visit/lock/unlock/prune/remove with distinct worktree/common identities, checked paths and no forced removal of protected or dirty resources.
- [ ] T11.3: Verify stashes with conflicts, changed stash positions, linked-worktree ownership, lock reasons, stale removal and ordinary buffer/reload behavior; share existing read-only reflog/blame owners.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 12: Prepared Git families and direct commands

Status: pending
Branch: `feat/rev-git-integration-12-commands`
Expected PR title: `feat(git): expose prepared repository command families`
Report: `ops/default/report/slices/rev-git-integration/slice-12-commands.md`
Requirements: GIT14-GIT15, GIT16-GIT20
PR: pending

- [ ] T12.1: Provide working native parameter/review routes for regular/shallow/bare/mirror clone, init, ignore files, submodule add/list/update/sync/deinit and subtree add/merge/pull/push/split.
- [ ] T12.2: Complete native notes add/remove/prune, mailbox/plain patch application, format-patch/save-diff/request-pull and sparse-checkout enable/disable/reapply/set/add with exact targets, capability checks and observable real effects.
- [ ] T12.3: Expose explicit structured direct Git argv and a separately explicit existing shell-command route; integrate complete contextual family prefixes, bound input/review and verify every listed variant using isolated local repositories.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 13: Combined native PTY and resource acceptance

Status: pending
Branch: `feat/rev-git-integration-13-acceptance`
Expected PR title: `test(git): verify native workflows and bounded ownership`
Report: `ops/default/report/slices/rev-git-integration/slice-13-acceptance.md`
Requirements: GIT01-GIT20 together
PR: pending

- [ ] T13.1: Add one repository-native Git acceptance entry and Bash driver reusing existing Rust/Linux PTY support; share real isolated fixture builders and verify objects/files rather than command labels.
- [ ] T13.2: Exercise representative both-profile journeys covering Status/diffs/index/commit, refs/transport, sequence/conflict, contextual prefixes, pointer/resize and restoration with a live terminal; assert matrix routing at the cheapest reliable native layer.
- [ ] T13.3: Measure Git-only binary delta, startup/idle CPU, retained RSS/output and bounded fair service; verify process/FD cleanup, malformed/truncated/large input, ownership on q/app exit and zero dependency additions.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Slice 14: Git learning path and reference

Status: pending
Branch: `feat/rev-git-integration-14-guidance`
Expected PR title: `docs(git): explain native repository workflows`
Report: `ops/default/report/slices/rev-git-integration/slice-14-guidance.md`
Requirements: GIT01-GIT20 and user-visible discovery
PR: pending

- [ ] T14.1: Add a short independent Git learning path, task guides, exact command/key/configuration reference and ownership explanation; show compact status/diff/commit feedback without requiring Org learning.
- [ ] T14.2: Keep canonical Markdown and its Spore projection/navigation aligned; document explicit fetch/integration, local tags, reviewed push, sequence/conflict, direct commands and bounds, with no unimplemented feature claims.
- [ ] T14.3: Finish user-facing Unreleased entries and per-variant coverage, update the additive baseline/index references without rewriting historical gate evidence, and prepare exact-candidate closure metadata for the controller after its selected full gate passes.

Validation: pending; use the exact focused commands and selected-gate policy in the plan.
Integration: pending; no commit or PR is claimed.

## Closure

Status: pending
Validated candidate: pending
Controller gate receipt: pending
Closure commit: pending

Native Git acceptance must cover every listed family and variant, both startup
profiles, contextual keys, real object/file effects, source/terminal retention
and the complete configuration/Markdown/Spore contract. The canonical controller
checkpoint owns progress. Execution authority is the owner's approval recorded above; release authority
is separate. Do not mark a slice delivered from a test result alone.
