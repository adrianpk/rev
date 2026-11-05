# Native Git Capability and Evidence Map

Status: coverage references prepared; final delivery-set gate pending
Specification: [Native Git Integration](editor-git-integration.md)
Model: [Native Git Model](editor-git-integration-model.md)
Plan: [Delivery plan](../plan/rev-git-integration.md)
Tracker: [Delivery tracker](../tracker/rev-git-integration.md)

## Evidence identities

| Delivery | Exact tested candidate | Owning evidence |
| --- | --- | --- |
| Original editor | `b56d705c7a84b823bf069fc874bc762938d6d897` | [Editor baseline](rev-editor.md); original requirements retain their historical meaning. |
| Pre-Git comparison baseline | `772051f7a70b4be66d86ccf4fdea27f9b6c2da68` | [Slice 13](../report/slices/rev-git-integration/slice-13-acceptance.md#validation); fresh archived source, same release compiler/profile, separate target directories. |
| Prepared command families | `7f4214caf64eee0337f8507a7a0862a523fac795` | [Slice 12](../report/slices/rev-git-integration/slice-12-commands.md#validation), controller receipt `ZQMQ2STQDLUHSC67WGG2VNVRTA`: 48 family and shared-owner cases under its ordered checks. |
| Combined native/PTY/resources | `d15ab4469f3a05604117276340d87d0f834c871f` | [Slice 13 integration](../report/slices/rev-git-integration/slice-13-acceptance.md#integration), controller receipt `YQZ3ALMQDPBYYUWCT7PUZ2CREN`; five ordered checks including the complete Git acceptance driver. |

These receipts belong to their exact candidates and command lists. They do not
combine into a delivery-set gate for a later head. The controller records that
candidate, unchanged selected gate and final receipt in the tracker after the
last slice integrates. Guidance checks prove documentation/projection integrity,
not a new execution of every runtime scenario below.

## Requirements and variants

Each assertion name resolves in the linked native test owner under the existing
`git_integration` target. Related variants share a fixture when it checks their
separate observable effects. The matrix maps coverage; it is not execution
evidence by itself. The complete [command catalog](../../../docs/reference/commands.md#native-git-status),
[binding catalog](../../../docs/reference/bindings.md#git-defaults-and-availability)
and [startup preferences](../../../docs/reference/defaults.md#git-preferences)
remain the user-facing authorities.

| Requirement / variants | Native assertion owner | Observable contract |
| --- | --- | --- |
| GIT01: ordinary, linked, bare, unborn, detached, unavailable; SHA-1/SHA-256; raw paths | [status.rs](../../../tests/git_integration/status.rs): repository_context, worktree_identity, exact_path_bytes, unavailable_and_bare; [refs.rs](../../../tests/git_integration/refs.rs): ref_limits_and_heads | Exact repository/Git/common directories, HEAD and path bytes; explicit unavailable state. |
| GIT02: canvas, return, hidden source/terminal | [presentation.rs](../../../tests/git_integration/presentation.rs): native_canvas, terminal_retention; [diff_presentation.rs](../../../tests/git_integration/diff_presentation.rs): terminal_return_and_visit | Retained native owners/layout survive entry, q and visits. |
| GIT03: status groups, bounded/failed refresh, selection | [status.rs](../../../tests/git_integration/status.rs): status_layers, bounded_replacement, rejected_publication; [presentation.rs](../../../tests/git_integration/presentation.rs): semantic_pointer, display_bounds | Immutable layer-qualified paths; prior confirmed state and failed freshness are distinct. |
| GIT04: staged/unstaged/unborn/untracked; text, binary, rename, mode, gitlink; detail/visit | [diffs.rs](../../../tests/git_integration/diffs.rs): layers_and_hunks, unborn_and_path_bytes, special_files, bounded_detail; [diff_presentation.rs](../../../tests/git_integration/diff_presentation.rs): layers_and_visits, exact_current_sources | Actual comparison bytes and source identity; unsupported/partial detail cannot authorize an edit. |
| GIT05: whole paths and hunks, stage/unstage | [index.rs](../../../tests/git_integration/index.rs): path_roundtrip, hunk_roundtrip, unborn_paths, renamed_path, hunk_file_modes, gitlink_path | Actual index blobs/modes change; working/unsaved sources remain protected. |
| GIT05: marked whole paths and nonoverlapping hunks | [index_groups.rs](../../../tests/git_integration/index_groups.rs): paths, hunks, rejection | Complete compatible group checked before launch; stale/mixed/duplicate targets reject. |
| GIT05: confirmed whole-path discard | [index_discard.rs](../../../tests/git_integration/index_discard.rs): restore, filters, failures; [index_presentation.rs](../../../tests/git_integration/index_presentation.rs): native_discard_guards | Saved file restored from unchanged index, conversions and visiting-source checks retained. |
| GIT06: normal, amend, fixup; submit/cancel/draft; hook/signing failure | [commits.rs](../../../tests/git_integration/commits.rs): normal_objects, normal_failure_evidence; [commits_modes.rs](../../../tests/git_integration/commits_modes.rs): amend_objects, fixup_objects; [commits_effects.rs](../../../tests/git_integration/commits_effects.rs): changed_effects, signing_failure; [commits_acceptance.rs](../../../tests/git_integration/commits_acceptance.rs): failed_draft_recovery, cancelled_review | Checked index/HEAD/target, actual tree/message/parents and retained drafts without hidden save/stage. |
| GIT07: ancestry pages, revision/parents, exact ordered ranges and historical renames | [history.rs](../../../tests/git_integration/history.rs): ancestry_pages, revisions_and_parents, changed_ancestry; [history_inspection.rs](../../../tests/git_integration/history_inspection.rs): exact_ranges; [history_limits.rs](../../../tests/git_integration/history_limits.rs): deleted_binary_and_helpers | Full IDs and bounded actual object content without checkout or source edits. |
| GIT08: branch create/switch/rename/non-forced delete, stale refs | [refs.rs](../../../tests/git_integration/refs.rs): branch_lifecycle, branch_guards, head_between_reads; [refs_acceptance.rs](../../../tests/git_integration/refs_acceptance.rs): branch_protections | Exact ref/object and source guards; unrelated refs preserved. |
| GIT08: lightweight default, annotated message, signing, local deletion | [refs_tags.rs](../../../tests/git_integration/refs_tags.rs): tag_kinds, signing_and_draft, signed_annotation, tag_guards; [refs_acceptance.rs](../../../tests/git_integration/refs_acceptance.rs): moved_tag_at_lock | Actual tag kind/blob/peeled object, Git-owned identity and failure drafts; local effects only. |
| GIT09: separate selected fetch and reviewed upstream push | [transport.rs](../../../tests/git_integration/transport.rs): selected_fetch, fetch_admission; [transport_push.rs](../../../tests/git_integration/transport_push.rs): exact_upstream, push_policy, stale_push, receiver_rejection; [transport_acceptance.rs](../../../tests/git_integration/transport_acceptance.rs): changed_fetch_config, unavailable_verification, failed_credentials, bounded_transport | Local bare endpoints, exact destination object, no inferred integration/force/expanded publication; private transport diagnostics. |
| GIT10: fast-forward-only/ordinary/no-fast-forward merge, ordinary rebase, autosquash | [integration.rs](../../../tests/git_integration/integration.rs): merge_policies, ordinary_rebase, reviewed_autosquash, conflict_stops | Actual parents/tree/sequence state and finite reviewed todo without external interactive editor. |
| GIT10: matching continue/skip/abort, external state and incompatible policy | [integration.rs](../../../tests/git_integration/integration.rs): sequence_controls; [integration_guards.rs](../../../tests/git_integration/integration_guards.rs): external_rebase, incompatible_sequence, control_failure, autosquash_admission | Fresh actual sequence lineage; no hidden autostash or incorrect family action. |
| GIT11: ordered cherry-pick/revert, merge mainline, completed prefix and controls | [sequences.rs](../../../tests/git_integration/sequences.rs): ordered_application, explicit_mainlines, retained_order; [sequences_controls.rs](../../../tests/git_integration/sequences_controls.rs): ordered_controls, next_conflict, control_uncertainty | Actual commit order/parents and saved effects; retained conflict or uncertain prefix never silently replayed. |
| GIT11: manual bisect start/good/bad/skip/reset, ambiguity | [sequences_bisect.rs](../../../tests/git_integration/sequences_bisect.rs): manual_bisect, bisect_ambiguity, bisect_fences | Actual checked candidates and original restoration; no automatic test command. |
| GIT12: exact side roles, partial/missing/structural inspection and editable result | [conflicts.rs](../../../tests/git_integration/conflicts.rs): operation_sides, partial_and_missing, stale_inspection, external_index | Actual modes/index stages and operation-relative labels; no generated writable source. |
| GIT12: Base/Current/Applied replacement, deletion, saved staging, separate continuation | [conflicts_resolution.rs](../../../tests/git_integration/conflicts_resolution.rs): sides_then_stage, saved_and_deleted, changed_review, effects_remain_visible, converted_text, ordered_continuation; [conflicts_resolution_presentation.rs](../../../tests/git_integration/conflicts_resolution_presentation.rs): unsaved_result | Actual saved bytes/index and conversions; unsaved/stale guards and retained effects. |
| GIT12: optional copied-input AI explanation, unavailable provider/historical answer | [conflicts_explanation.rs](../../../tests/git_integration/conflicts_explanation.rs): optional_ai, copied_request, historical_source | Read-only result, saved and unsaved snapshots distinct, no edit/stage/continue authority. |
| GIT13: stash list/show/create/apply/pop/branch/drop; changed ordinal/conflict | [repository.rs](../../../tests/git_integration/repository.rs): saved_roundtrip, pop_and_branch, changed_review, retained_conflict, stash_ordinal | Exact full stash objects and actual saved state; failed restoration retains stash. |
| GIT13: worktree list/create/visit/lock/unlock/prune/remove | [repository_worktrees.rs](../../../tests/git_integration/repository_worktrees.rs): checked_roundtrip, protected_removal, stale_reviews, checked_prune, admission_and_ownership; [repository_worktrees_presentation.rs](../../../tests/git_integration/repository_worktrees_presentation.rs): visiting_owner | Distinct linked identity and exact registry/locks; clean non-forced removal protects retained owners. |
| GIT13: reflog and rename-aware blame | [history_inspection.rs](../../../tests/git_integration/history_inspection.rs): reflog_objects, blame_renamed_source; [history_inspection_presentation.rs](../../../tests/git_integration/history_inspection_presentation.rs): native_inspections | Exact historical object/line attribution and canonical current-source visits. |
| GIT14: regular/shallow/bare/mirror clone and init | [families.rs](../../../tests/git_integration/families.rs): clone_kinds, explicit_init, stale_preparation, unborn_source, empty_clone | Each variant proves its actual Git state at an explicit absent destination. |
| GIT14: .gitignore and common info/exclude visits | [families_presentation.rs](../../../tests/git_integration/families_presentation.rs): ordinary_ignore, outside_repository | Actual ordinary native file owners, no inserted pattern or save. |
| GIT14: submodule add/list/update/sync/deinit | [families_submodule.rs](../../../tests/git_integration/families_submodule.rs): roundtrip, stale_and_dirty, literal_target; [families_presentation.rs](../../../tests/git_integration/families_presentation.rs): native_submodules, submodule_sources | Actual gitlink, checkout, recursive listing, URL and non-forced deinit; protected child/source owners. |
| GIT14: subtree add/merge/pull/push/split | [families_subtree.rs](../../../tests/git_integration/families_subtree.rs): roundtrip, stale_and_policy, conflict_state, unavailable | Actual imported/split trees and remote object; unavailable optional command and conflicts stay explicit. |
| GIT14: notes add/remove/prune, including bare | [families_metadata.rs](../../../tests/git_integration/families_metadata.rs): notes_roundtrip, notes_checked_state | Actual note blob/map, literal message after Git cleanup and unchanged unrelated refs. |
| GIT14: sparse enable/disable/reapply/set/add | [families_metadata.rs](../../../tests/git_integration/families_metadata.rs): sparse_roundtrip, sparse_literal_paths, sparse_protected_state, sparse_linked_worktree | Actual cone selection, skip flags, tracked-file materialization and private linked patterns. |
| GIT14: format-patch, save-diff, request-pull | [families_output.rs](../../../tests/git_integration/families_output.rs): patch_roundtrip, request_pull, checked_output, output_admission, changed_producer, remote_review | Complete actual output and consumers, absent destinations, reproduced bytes and already-published remote anchor; no mail/push. |
| GIT14: plain patch and mailbox application | [families_patch.rs](../../../tests/git_integration/families_patch.rs): saved_application, payload_semantics, actual_hook_changes, mailbox_conflict, protected_input, signing_policy | Actual index/files/trees, decoded authors/dates/messages and stopped prefix; From provenance cannot establish equivalence. |
| GIT15: literal Git argv and separately selected shell, cancellation/outside context | [families_direct.rs](../../../tests/git_integration/families_direct.rs): literal_git, explicit_shell, outside_context, incompatible_refresh, direct_protection, direct_lifetime | Actual executable/cwd/output/exit; arbitrary postconditions stay unknown even on zero exit. |
| GIT16: process records, selected cancellation, verified/in-progress/unknown effects | [lifetime.rs](../../../tests/git_integration/lifetime.rs): selected_cancellation, retained_diagnostics; [commits_effects.rs](../../../tests/git_integration/commits_effects.rs): changed_effects, cancelled_child | Bounded retained facts and cleanup ownership; cancellation is not repository rollback. |
| GIT17: generation/config/index/HEAD/file/sequence changes through review | [index.rs](../../../tests/git_integration/index.rs): stale_preflight, failed_after_effect; [refs.rs](../../../tests/git_integration/refs.rs): head_between_reads; [conflicts_resolution.rs](../../../tests/git_integration/conflicts_resolution.rs): changed_review; [families.rs](../../../tests/git_integration/families.rs): stale_preparation | Relevant complete evidence rechecked, stale authority cannot retarget a mutator. |
| GIT18: both profiles, M-x/colon, complete prefixes, pointer/resize | [matrix](../../../examples/git_integration_acceptance/matrix.rs), [frozen bindings](../../../examples/git_integration_acceptance/bindings.tsv), [host PTY](../../../examples/git_integration_acceptance/pty.rs), [presentation.rs](../../../tests/git_integration/presentation.rs): configured_bindings, semantic_pointer | 289 expected routes per profile plus real decoded entry, pointer, resize and composer input. |
| GIT19: unsaved sources, reload, q/declined exit, accepted bounded cleanup | [lifetime.rs](../../../tests/git_integration/lifetime.rs): guarded_exit; [integration_presentation.rs](../../../tests/git_integration/integration_presentation.rs): integration_reload; [owned lifetime](../../../examples/git_integration_acceptance/lifetime.rs), [resources](../../../examples/git_integration_acceptance/resources.rs) | Actual saved/unsaved text, leader/descendant cleanup, descriptor/thread return, quiet ownership and fair service. |
| GIT20: startup catalog, missing Git, bounded observations and optional behavior | [status.rs](../../../tests/git_integration/status.rs): configuration_catalog, unavailable_and_bare, byte_admission, retained_contexts; [catalog check](../../../examples/documentation_check/catalogs.rs) | Complete startup preferences/origins, no startup child, explicit capacity refusal and ordinary editing independence. |

## Combined acceptance and evidence limits

The [native entry](../../../examples/git_integration_acceptance/native.rs) and
[journeys](../../../examples/git_integration_acceptance/journeys.rs) use real
installed Git, isolated identity/configuration, both object formats and both
profiles. Representative semantic journeys cover hunks/index/commit, local
fetch/reviewed push and conflict replacement/abort. The host PTY covers actual
entry and q, pointer/resize, composer cancellation/submission, a live terminal,
declined exit, source restoration and TTY restoration. The routing matrix
complements those journeys; routing alone does not prove an operation's effects.
Per-variant native assertions above supply the detailed semantic coverage.

Resource/lifetime phases verify four concurrent real repositories, explicit
truncation and retention refusal, real oversized output, quiet CPU/wakes and
leader/descendant cleanup. Malformed porcelain uses an explicit parser double
because installed Git cannot reproducibly generate corrupt porcelain; it proves
refusal/freshness behavior, not real Git corruption. Startup/RSS/executable
measurements and their phase costs belong to the exact Slice 13 report/receipt.
Logical charged bytes and observed RSS retain different meanings.

## Guidance and final closure

The independent [Git User Guide](../../../docs/tutorials/user-guide/git/index.md),
[task guides](../../../docs/how-to/index.md#git), [reference](../../../docs/reference/git.md)
and [ownership explanation](../../../docs/explanation/git-ownership.md) cover
observable controls and bounds. `documentation-check` checks all local docs
links/anchors, reachability, editor/Git/Org navigation and live command/settings
catalogs. `scripts/build-user-guide.sh` projects that same Markdown through
Spore strict/link checks and renders an owned local output.

The [tracker closure fields](../tracker/rev-git-integration.md#closure) reserve
exact validated candidate, controller gate receipt and closure commit identities.
They remain pending until the controller completes the selected unchanged gate.
