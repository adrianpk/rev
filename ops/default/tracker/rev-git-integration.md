# Native Git Integration Tracker

Status: Approved
Date: 2026-11-04
Delivery set: rev-git-integration
Plan: [Native Git Integration Delivery](../plan/rev-git-integration.md)
Spec: [Native Git Integration](../spec/editor-git-integration.md)
Model: [Native Git Model](../spec/editor-git-integration-model.md)
Approved: owner, 2026-11-04; complete fourteen-slice plan and autonomous Looper execution/integration through dev
Active slice: Slice 14 - Git learning path and reference
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

Status: delivered
Branch: `feat/rev-git-integration-01-status`
Expected PR title: `feat(git): own repository context and native status`
Report: `ops/default/report/slices/rev-git-integration/slice-1-status.md`
Requirements: GIT01-GIT03, GIT16-GIT20
PR: `#258`

- [x] T1.1: Resolve worktree/common directories, bare/unborn/detached state, exact path bytes and SHA-1/SHA-256 objects through installed Git; add the bounded native repository/process owner and the complete startup [git] catalog, sample and origins.
- [x] T1.2: Open semantic Status with C-x g, M-x and modal colon, preserve full-canvas return context and existing source/terminal owners, and render explicit complete/truncated/unavailable groups with checked selection and pointer targets.
- [x] T1.3: Expose native active/recent process inspection and checked cancellation; preserve owned work after q, integrate guarded application exit, and verify real repositories, missing Git, malformed output, stale publication and bounded cleanup.

Validation: T1.1-T1.3 development checks pass: formatter, strict focused Clippy and 21 native Git tests. Shared development evidence includes 10 process tests, 10 command-reference tests, 11 configuration-reference tests, 41 window tests and 19 panel tests. Controller validation `ODY3HME22HUL3MRUW4F2VYWHPM` passed all 9 ordered checks for merged commit `2a6181a062428d91dd4192467b5cbb9735bed6d1`.
Integration: [PR #258](https://forge.adrianpk.com/adrianpk/rev/pulls/258) merged through controller-owned Rebase + Fast-forward into dev at `2a6181a062428d91dd4192467b5cbb9735bed6d1`. Slice 1 of 14 completed.

## Slice 2: Changes hunks and source visits

Status: delivered
Branch: `feat/rev-git-integration-02-diffs`
Expected PR title: `feat(git): inspect exact diffs and visit native sources`
Report: `ops/default/report/slices/rev-git-integration/slice-2-diffs.md`
Requirements: GIT04, GIT17-GIT19
PR: `#259`

- [x] T2.1: Read staged, unstaged, unborn and untracked comparisons with exact semantic identities; admit bounded text hunks and truthful binary/rename/type-change/submodule/truncated detail.
- [x] T2.2: Add Status expansion, read-only detail, current-source visits, retained navigation and resize/pointer mappings without making generated Git content a writable source.
- [x] T2.3: Verify both layers of one path, multiple hunks, unusual path bytes, stale refresh, modified visiting buffers and return to the surviving editor layout.

Validation: T2.1 development formatter and strict focused Clippy pass; six comparison tests pass, with an earlier 27-test shared Git run. Nine native Status/presentation/process regressions pass after T2.2. T2.3 adds five both-profile native acceptance cases and extended staged unusual-path fixtures in both object formats; all pass focused development checks. Controller validation `45HQQNKFYXODWKRVVQBKHITJD5` passed all seven ordered checks at `78e4c43cf79792337006786f8752a093badc05d6`, including all 32 Git and 21 reference tests, documentation and whitespace.
Integration: [PR #259](https://forge.adrianpk.com/adrianpk/rev/pulls/259) merged through controller-owned Rebase + Fast-forward into dev at `78e4c43cf79792337006786f8752a093badc05d6`. Slice 2 of 14 completed.

## Slice 3: Checked staging unstaging and discard

Status: delivered
Branch: `feat/rev-git-integration-03-index`
Expected PR title: `feat(git): stage exact paths hunks and checked groups`
Report: `ops/default/report/slices/rev-git-integration/slice-3-index.md`
Requirements: GIT05, GIT17, GIT19
PR: `#260`

- [x] T3.1: Stage and unstage exact paths or admitted text hunks; revalidate their complete comparison before changing the index.
- [x] T3.2: Implement compatible marked path groups and nonoverlapping same-path/layer hunk groups; reject stale, mixed, duplicate or unsupported groups before launch.
- [x] T3.3: Restore one confirmed tracked working-tree path from its index state without changing the index; protect unsaved buffers and verify actual index/worktree results and partial-effect diagnostics.

Validation: Controller receipt `WQFGAZWVDQUWWTWHRKJ5VA3LDW` passed all nine ordered checks for `d14a59b0b274bc5a376594e375e4d288c8f71594`: 24 index, 9 Status, 5 diff and 21 reference tests, shared reload, strict Clippy, formatter, documentation and whitespace.
Integration: [PR #260](https://forge.adrianpk.com/adrianpk/rev/pulls/260) merged through controller-owned Rebase + Fast-forward into dev at `d14a59b0b274bc5a376594e375e4d288c8f71594`. Slice 3 of 14 completed.

## Slice 4: Native commit composition

Status: delivered
Branch: `feat/rev-git-integration-04-commits`
Expected PR title: `feat(git): compose commits amend and fixup`
Report: `ops/default/report/slices/rev-git-integration/slice-4-commits.md`
Requirements: GIT06, GIT16-GIT19
PR: `#261`

- [x] T4.1: Enter normal composition directly with c and provide ordinary native text editing, explicit submission, cancellation and session-only retained drafts.
- [x] T4.2: Compose normal, reviewed amend and exact-target fixup requests; preserve checked index/HEAD, identity, hooks, signing policy and actual messages without hidden staging or saving.
- [x] T4.3: Verify C-Enter/C-c C-c submission, C-g/C-c C-k cancellation, literal c, failed hooks, retained drafts, post-effect uncertainty and protected source/terminal state.

Validation: Controller receipt `ZZX5LLVZNAXQS4VOZ7WVHIINQO` passed all nine ordered checks at `32a529e71460cef57f342ac83318221c0a7dba3d`: 75 Git, 138 editing/rectangle and 21 reference tests, formatter, strict Clippy, documentation and whitespace.
Integration: [PR #261](https://forge.adrianpk.com/adrianpk/rev/pulls/261) merged through controller-owned Rebase + Fast-forward into dev at `32a529e71460cef57f342ac83318221c0a7dba3d`. Slice 4 of 14 completed.

## Slice 5: History ranges reflog and blame

Status: delivered
Branch: `feat/rev-git-integration-05-history`
Expected PR title: `feat(git): browse history ranges reflog and blame`
Report: `ops/default/report/slices/rev-git-integration/slice-5-history.md`
Requirements: GIT07, GIT13, GIT17-GIT18
PR: `#262`

- [x] T5.1: Browse finite ancestry pages, exact revisions and parent comparisons; keep selection and explicit ancestry continuation across refresh.
- [x] T5.2: Implement exact ordered ranges, separate range-base and ordered revision marks, read-only reflog and blame inspection, and canonical source/revision visits.
- [x] T5.3: Verify detached/unborn/merge history, paging limits, full object identities, historical file renames, line attribution and no checkout or source modification during inspection.

Validation: Controller receipt `QVORUFWHKOYCHDV54YAVCNY2OX` passed all eight ordered checks at `3470ed3cff871c3d3d902400c8c228ce6ebc21a9`: 90 Git tests, 11 selected process tests and 21 command/configuration reference tests, formatter, strict Clippy, documentation and whitespace. Earlier cleanup failures and their focused corrections remain in the report.
Integration: [PR #262](https://forge.adrianpk.com/adrianpk/rev/pulls/262) merged through controller-owned Rebase + Fast-forward into dev at `3470ed3cff871c3d3d902400c8c228ce6ebc21a9`. Slice 5 of 14 completed.

## Slice 6: Branches and both tag kinds

Status: delivered
Branch: `feat/rev-git-integration-06-refs`
Expected PR title: `feat(git): manage branches and local tag kinds`
Report: `ops/default/report/slices/rev-git-integration/slice-6-refs.md`
Requirements: GIT08, GIT17-GIT19
PR: `#263`

- [x] T6.1: Present local/remote-tracking refs, tags, remotes and upstream facts with exact identities and checked branch create/switch/rename/non-forced deletion.
- [x] T6.2: Implement t/git-tag-create with lightweight default and annotated message composition; retain distinct tag-object/peeled-target identities, Git metadata/signing and checked local tag deletion.
- [x] T6.3: Verify current/unmerged branch protection, modified source admission, stale confirmation, both tag kinds, existing-name failure and conflicting signing policy without implicit publication or policy overrides.

Validation: Controller receipt `P6AQRLEPYKAAT3MWRQKSXFWWQG` passed all seven ordered checks at `bef97e57c7953c9ab463c59b52b15ebcea22ac06`: 107 Git tests and 21 command/configuration reference tests, formatter, strict Clippy, documentation and whitespace. Earlier development and controller failures remain recorded in the report.
Integration: [PR #263](https://forge.adrianpk.com/adrianpk/rev/pulls/263) merged through controller-owned Rebase + Fast-forward into dev at `bef97e57c7953c9ab463c59b52b15ebcea22ac06`. Slice 6 of 14 completed.

## Slice 7: Separate fetch and reviewed push

Status: delivered
Branch: `feat/rev-git-integration-07-transport`
Expected PR title: `feat(git): fetch and push one reviewed upstream branch`
Report: `ops/default/report/slices/rev-git-integration/slice-7-transport.md`
Requirements: GIT09, GIT16-GIT19
PR: `#264`

- [x] T7.1: Fetch an explicitly selected configured remote and refresh tracking facts without integrating the current branch.
- [x] T7.2: Review and confirm one exact branch, configured upstream remote and destination; prevent implicit force, deletion, tags or mirror expansion and reject ambiguous tracking.
- [x] T7.3: Verify local-bare remote outcomes, hostile/conflicting push settings, failed credentials/helpers, cancellation, bounded diagnostics and publication uncertainty; no live user remote or credential is used.

Validation: Controller receipt `67Z2NNDVAJAC4EOIYA6PBPFUMR` passed all seven ordered checks at `5db15bc1085a2575994c7ce3f100be545fa54882`: 119 Git tests, 21 command/configuration references, formatter, strict Clippy, 512-command documentation and whitespace. Earlier development failures remain recorded in the report.
Integration: [PR #264](https://forge.adrianpk.com/adrianpk/rev/pulls/264) merged through controller-owned Rebase + Fast-forward into dev at `5db15bc1085a2575994c7ce3f100be545fa54882`. Slice 7 of 14 completed.

## Slice 8: Merge rebase and sequence control

Status: delivered
Branch: `feat/rev-git-integration-08-integration`
Expected PR title: `feat(git): run checked merge and rebase workflows`
Report: `ops/default/report/slices/rev-git-integration/slice-8-integration.md`
Requirements: GIT10, GIT16-GIT19
PR: `#265`

- [x] T8.1: Implement explicit fast-forward/normal/no-fast-forward merge policies and rebase from exact observed refs with checked worktree/source preconditions.
- [x] T8.2: Prepare reviewed autosquash and observe Git-owned in-progress work through exact family-specific and generic status/continue/skip/abort controls; never launch an interactive external editor.
- [x] T8.3: Verify successful and conflicted local histories, stale HEAD, missing sequence, wrong-family controls and skipped/aborted outcomes; source reload follows existing automatic/ask policy.

Validation: Controller receipt `LV7ZCYAEGWAJMND5ME3IRVVXGK` passed all eight ordered checks at `0c4dd707485bdb53f11963406b6874f7e1079186`: 138 Git tests, 9 process tests and 21 command/configuration references, formatter, strict Clippy, documentation and whitespace. The earlier candidate receipt and sealed-driver ownership correction remain in the report.
Integration: [PR #265](https://forge.adrianpk.com/adrianpk/rev/pulls/265) merged through controller-owned Rebase + Fast-forward into dev at `0c4dd707485bdb53f11963406b6874f7e1079186`. Slice 8 of 14 completed.

## Slice 9: Ordered revisions and manual bisect

Status: delivered
Branch: `feat/rev-git-integration-09-sequences`
Expected PR title: `feat(git): apply ordered revisions and manual bisect`
Report: `ops/default/report/slices/rev-git-integration/slice-9-sequences.md`
Requirements: GIT11, GIT17-GIT19
PR: `#266`

- [x] T9.1: Prepare ordered cherry-pick/revert requests from independent ordered marks, including exact mainline selection for merge commits.
- [x] T9.2: Integrate their actual continuation/skip/abort states with the shared sequence owner and conflict entry; reject wrong order, duplicate/stale targets and wrong active families.
- [x] T9.3: Provide manual bisect start/good/bad/skip/reset with exact review and real fixture verification; do not add automatic test execution.

Validation: Controller receipt `6EUQIIPWCK2S52UQHODGSQUN7H` passed all eight ordered checks at `be4d6098b3b67a67adacd1162897d5a7a903e801`: 16 sequence cases, 19 shared integration cases, one source/history regression and 21 reference checks, formatter, strict Clippy, documentation and whitespace.
Integration: [PR #266](https://forge.adrianpk.com/adrianpk/rev/pulls/266) merged through controller-owned Rebase + Fast-forward into dev at `be4d6098b3b67a67adacd1162897d5a7a903e801`. Slice 9 of 14 completed.

## Slice 10: Native conflict resolution

Status: delivered
Branch: `feat/rev-git-integration-10-conflicts`
Expected PR title: `feat(git): inspect and resolve checked conflicts`
Report: `ops/default/report/slices/rev-git-integration/slice-10-conflicts.md`
Requirements: GIT12, GIT17-GIT19
PR: [#267](https://forge.adrianpk.com/adrianpk/rev/pulls/267)

- [x] T10.1: Observe exact conflict paths/kinds/index stages and operation-dependent Base/Current/Applied roles; provide read-only side inspection and canonical editable result visits.
- [x] T10.2: Review whole-side replacement or resolved deletion and stage only a checked saved resolution; protect modified buffers, changed stages and unsupported/partial cases.
- [x] T10.3: Route git-conflict-explain through the existing optional AI request/result owner with bounded conflict context and no automatic edit or resolution; verify no-provider behavior and request construction without mandatory live credentials.

Validation: Controller receipt `NG2AAES7H4F5XTH2YDS6SQ7H4Y` passed all eight ordered checks at `98574bb26c413f169b6cf06f1c034d55517b9315`: 19 conflict entries, 6 native integration presentation cases, 5 AI result-owner cases and 21 reference checks, formatter, strict Clippy, documentation and whitespace. Development corrections and retained bounds are recorded in the report.
Integration: [PR #267](https://forge.adrianpk.com/adrianpk/rev/pulls/267) merged through controller-owned Rebase + Fast-forward into dev at `98574bb26c413f169b6cf06f1c034d55517b9315`. Slice 10 of 14 completed.

## Slice 11: Stashes and linked worktrees

Status: delivered
Branch: `feat/rev-git-integration-11-repository`
Expected PR title: `feat(git): manage stashes and linked worktrees`
Report: `ops/default/report/slices/rev-git-integration/slice-11-repository.md`
Requirements: GIT13, GIT17-GIT19
PR: [#268](https://forge.adrianpk.com/adrianpk/rev/pulls/268)

- [x] T11.1: Implement native stash list/create/show/apply/pop/branch/drop with exact stash-object identity, checked source state, explicit destructive review and truthful conflict outcomes.
- [x] T11.2: Implement worktree list/create/visit/lock/unlock/prune/remove with distinct worktree/common identities, checked paths and no forced removal of protected or dirty resources.
- [x] T11.3: Verify stashes with conflicts, changed stash positions, linked-worktree ownership, lock reasons, stale removal and ordinary buffer/reload behavior; share existing read-only reflog/blame owners.

Validation: Controller receipt `ECS6UBJBCXLQEW6ADYPW5TIWXE` passed all eight ordered checks at `5cfe34a7376873e76a6a289fb06c8fb33bbd6030`: 19 repository cases, four shared ref cases, one reflog/blame case, 21 reference checks, formatter, strict focused Clippy, documentation and whitespace. Development corrections and retained bounds are recorded in the report.
Integration: [PR #268](https://forge.adrianpk.com/adrianpk/rev/pulls/268) merged through controller-owned Rebase + Fast-forward into dev at `5cfe34a7376873e76a6a289fb06c8fb33bbd6030`. Slice 11 of 14 completed.

## Slice 12: Prepared Git families and direct commands

Status: delivered
Branch: `feat/rev-git-integration-12-commands`
Expected PR title: `feat(git): expose prepared repository command families`
Report: `ops/default/report/slices/rev-git-integration/slice-12-commands.md`
Requirements: GIT14-GIT15, GIT16-GIT20
PR: [#269](https://forge.adrianpk.com/adrianpk/rev/pulls/269)

- [x] T12.1: Provide working native parameter/review routes for regular/shallow/bare/mirror clone, init, ignore files, submodule add/list/update/sync/deinit and subtree add/merge/pull/push/split.
- [x] T12.2: Complete native notes add/remove/prune, mailbox/plain patch application, format-patch/save-diff/request-pull and sparse-checkout enable/disable/reapply/set/add with exact targets, capability checks and observable real effects.
- [x] T12.3: Expose explicit structured direct Git argv and a separately explicit existing shell-command route; integrate complete contextual family prefixes, bound input/review and verify every listed variant using isolated local repositories.

Validation: Controller receipt `ZQMQ2STQDLUHSC67WGG2VNVRTA` passed all eight ordered checks at `7f4214caf64eee0337f8507a7a0862a523fac795`: 48 family, 21 reference, 35 shell and five process/workspace cases, formatter, strict focused Clippy, documentation and whitespace.
Integration: [PR #269](https://forge.adrianpk.com/adrianpk/rev/pulls/269) merged through controller-owned Rebase + Fast-forward into dev at `7f4214caf64eee0337f8507a7a0862a523fac795`. Slice 12 of 14 completed.

## Slice 13: Combined native PTY and resource acceptance

Status: delivered
Branch: `feat/rev-git-integration-13-acceptance`
Expected PR title: `test(git): verify native workflows and bounded ownership`
Report: `ops/default/report/slices/rev-git-integration/slice-13-acceptance.md`
Requirements: GIT01-GIT20 together
PR: [#270](https://forge.adrianpk.com/adrianpk/rev/pulls/270)

- [x] T13.1: Add one repository-native Git acceptance entry and Bash driver reusing existing Rust/Linux PTY support; share real isolated fixture builders and verify objects/files rather than command labels.
- [x] T13.2: Exercise representative both-profile journeys covering Status/diffs/index/commit, refs/transport, sequence/conflict, contextual prefixes, pointer/resize and restoration with a live terminal; assert matrix routing at the cheapest reliable native layer.
- [x] T13.3: Measure Git-only binary delta, startup/idle CPU, retained RSS/output and bounded fair service; verify process/FD cleanup, malformed/truncated/large input, ownership on q/app exit and zero dependency additions.

Validation: Controller receipt `YQZ3ALMQDPBYYUWCT7PUZ2CREN` passed all five ordered checks at `d15ab4469f3a05604117276340d87d0f834c871f`: formatter, strict focused runtime/test Clippy, the complete native/PTY/resource/lifetime acceptance driver, strict acceptance-target Clippy and whitespace. Both profiles and object formats, 289 bindings per profile, actual repository effects, bounded ownership and zero dependency additions passed. This is Slice 13 acceptance, not the delivery-set gate.
Integration: [PR #270](https://forge.adrianpk.com/adrianpk/rev/pulls/270) merged through controller-owned Rebase + Fast-forward into dev at `d15ab4469f3a05604117276340d87d0f834c871f`. Slice 13 of 14 completed.

## Slice 14: Git learning path and reference

Status: reviewing
Branch: `feat/rev-git-integration-14-guidance`
Expected PR title: `docs(git): explain native repository workflows`
Report: `ops/default/report/slices/rev-git-integration/slice-14-guidance.md`
Requirements: GIT01-GIT20 and user-visible discovery
PR: pending

- [x] T14.1: Add a short independent Git learning path, task guides, exact command/key/configuration reference and ownership explanation; show compact status/diff/commit feedback without requiring Org learning.
- [x] T14.2: Keep canonical Markdown and its Spore projection/navigation aligned; document explicit fetch/integration, local tags, reviewed push, sequence/conflict, direct commands and bounds, with no unimplemented feature claims.
- [x] T14.3: Finish user-facing Unreleased entries and per-variant coverage, update the additive baseline/index references without rewriting historical gate evidence, and prepare exact-candidate closure metadata for the controller after its selected full gate passes.

Validation: Required documentation catalog/navigation and Spore projection checks, focused formatter and whitespace are recorded through controller validation for the final clean PR-reference head.
Integration: T14.1-T14.3 implementation complete; canonical PR pending, integration controller-owned.

## Closure

Status: pending
Validated candidate: pending
Controller gate receipt: pending
Closure commit: pending
Coverage references: [Per-variant evidence map](../spec/editor-git-integration-evidence.md)
Final slice report: [Slice 14 guidance](../report/slices/rev-git-integration/slice-14-guidance.md)
Candidate source: controller-integrated final slice head; no slice-only receipt substitutes for the selected full gate.

Native Git acceptance must cover every listed family and variant, both startup
profiles, contextual keys, real object/file effects, source/terminal retention
and the complete configuration/Markdown/Spore contract. The canonical controller
checkpoint owns progress. Execution authority is the owner's approval recorded above; release authority
is separate. Do not mark a slice delivered from a test result alone.
