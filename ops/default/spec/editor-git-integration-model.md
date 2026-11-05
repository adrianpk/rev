# Native Git Model

Status: Completed
Completed: 2026-11-05
Validated candidate: `343850fb6914bf801dfa248b8cdd0e1c34ae1fef`
Gate validation: `D6BU4EM7IWRUA3UHGDBE432UK3`
Approved: owner, 2026-11-04; promoted with the complete native Git specification and delivery plan
Date: 2026-11-03
Reviewed: 2026-11-04
Version: 0.1
Specification: [Native Git Integration](editor-git-integration.md)
Existing editor model: [rev Editor Model](rev-editor-model.md)
Related file lifecycle: [External File Changes Model](editor-external-file-changes-model.md)
Authority: complete native Git model, contextual bindings, local-operation policies and bounds approved for the linked delivery
Plan: [Native Git Integration Delivery](../plan/rev-git-integration.md)
Tracker: [Native Git Tracker](../tracker/rev-git-integration.md)

Closure evidence: [Tracker](../tracker/rev-git-integration.md#closure), [Slice 14](../report/slices/rev-git-integration/slice-14-guidance.md#delivery-set-closure)

## Ownership and durability

Git owns objects, index, refs, working-tree files, configuration, reflogs,
stashes, worktree metadata and in-progress sequences. rev observes those facts
and makes explicit checked requests. It does not persist a second repository
model or infer authority from generated text.

The confirmed backend is the installed Git executable. Native Rust prepares
structured argv, owns observations and controls the process lifecycle while
Git retains repository semantics, configuration, hooks, credentials and signing.
The native fetch request updates remote facts without integrating the current
branch. A following merge/rebase request captures the explicit user choice;
configured pull policy does not supply that request or its authority.
The confirmed dedicated push request sends one source branch to its configured
upstream only after review and confirmation of the exact source, remote and
destination. Missing or contradictory tracking cannot supply an inferred target.
Implicit force or configuration-driven additional publication is not admitted.

Native tag creation offers lightweight and annotated local tags through the
same command, defaulting to lightweight. An annotation uses native message
composition; Git owns the resulting tagger metadata and applicable signing.
The observed ref retains its actual tag object and referenced target identities.
A configured signing policy that conflicts with a selected lightweight tag
requires a visible revised choice or cancellation, never a silent kind/policy
override.

A native Git session owns checked repository context, presentation, semantic
selection, retained editor context, composition drafts and bounded operations.
An operation owns the child process/group, streams, deadline, cancellation,
refresh and verification. Source buffer/window/file owners retain canonical
text, dirty state, undo, actual file association and reload policy.

The native guarded exit owner identifies active owned Git operations and asks
before terminating them. Declining retains the editor without cancelling the
operation; normal completion and deadlines still apply. Accepting performs
bounded owned cleanup before exit. It does not issue Git sequence abort/reset
or promise rollback of effects. A sequence without a live owned child remains
Git repository state rather than a process needing termination.

Except for startup preferences, records below are ephemeral. Git alone owns
durable repository effects. A commit-message draft may remain in the active
session after cancellation/failure; persistence across editor restarts requires
an explicit later retention decision, not a new hidden store.

## Repository and observation records

| Record | Fields and identity | Authority and validity |
| --- | --- | --- |
| Repository context | Process-local identity, captured directory/resource origin, Git-validated working-tree path, per-worktree Git directory, common directory, repository capabilities and object-ID format. | Not inferred from a displayed name, tree root label or presence of a .git directory alone. Linked worktrees share common storage but retain separate working-tree context. |
| Git session | Stable session identity, repository context, active Git view, presentation visibility, retained editor layout/focus/resource identities, presentation generation and lifecycle. | One visible Git interaction owner; hidden presentation does not release active operation ownership. Startup profile remains editor-owned. |
| Observation | Repository/session identity, generation, admitted status/ref/history/sequence facts, completeness, limits and diagnostic state. | Immutable accepted result; an unsuccessful replacement does not overwrite confirmed facts. |
| Status | HEAD form/object, branch/tracking facts and ordered Unmerged/Untracked/Unstaged/Staged groups. | Clean, unborn, detached, incomplete and unavailable states are explicit. |
| Path change | Repository/generation, exact Git path representation, group/layer, change kind, old/new path where applicable, mode/object facts and eligible actions. | Native path bytes are identity; escaped/lossy display text never reconstructs a filesystem target. |
| Diff | Repository/observation/path/layer, exact comparison objects or worktree evidence, bounded text/binary detail and completeness. | Read-only; parser success alone does not make stale detail current. |
| Hunk | Parent diff identity, exact old/new ranges and admitted context/patch evidence. | Supported current text only; row number and displayed header are not an action target. |
| Revision | Full Git object identity and format, parents, metadata, ancestry continuation and retained detail. | Abbreviations/subjects are presentation; not an alternative object identity. |
| Ref | Repository/generation, full ref name and kind, exact target/symbolic state, tag kind and peeled target where applicable, available upstream facts. | Fresh ref/HEAD relation is required for changing operations; an annotated tag object is not substituted for its referenced commit. |
| Remote | Repository/generation, configured name, credential-safe displayed description and private transport/configuration fingerprint. | Raw credential-bearing URL/configuration is not public operation text. |

Observations may be produced separately, but an operation must capture a
compatible set. Do not combine unrelated Status, Refs and Sequence generations
merely because they describe the same path string. Monotonic generations are
local publication identities, not substitutes for Git's current facts.

## Presentation and targeting

git-status, also bound to C-x g in both startup profiles, uses the available
editor canvas while retaining the prior layout,
focus and live resources. q from Status restores the surviving editing context;
hidden terminal sessions continue running.
An already started Git operation also continues when q restores source editing.
Its original process, deadlines, streams and result verification remain owned.
git-processes inspects that work and git-process-cancel explicitly requests its
cancellation. Hiding the presentation neither cancels nor detaches the child.

c in Status enters the normal commit draft directly. Once composition owns
input, c is message text rather than a Git-family suffix. Submission remains
a separate explicit action on the composed draft.
C-Enter and C-c C-c submit that same request; C-g and C-c C-k cancel composition
before process start while retaining the draft. These aliases keep identical
semantics across startup profiles and do not turn plain Return into submission.

The specification's binding matrix records native commands, input contexts and
complete sequences. Equivalent Git actions retain those sequences across startup
profiles. Each exception requires a concrete constraint, reason and compatible
alternative for review; a display label or command rename cannot
stand in for that interaction contract.

| Record | Fields | Invariant |
| --- | --- | --- |
| Section | Semantic kind, stable key, parent identity, expansion and current render mapping. | Selection follows semantic identity across refresh/resize; rendered coordinates are lookup input only. |
| Git view | Status, diff, History, revision/range, Refs, stash/worktree/reflog/blame, operation/conflict, composer, confirmation or Processes; originating view/selection and viewport. | Return navigation restores the retained live context without changing repository state. |
| Selected item | Repository/observation/view identity and an admitted typed target. | Missing/stale/incompatible target rejects or requests explicit selection. |
| Marked set | Bounded ordered exact path/hunk/revision/ref identities and intended operation role. | Path groups have one direction; hunk groups have one path/layer and no overlaps; ordered revision operations preserve user order. |
| Resource visit | Exact live source buffer/file identity or read-only historical/detail identity, plus retained return context. | No source rope copy becomes an independently writable file; navigation bindings do not intercept editable text. |
| Editor return context | Surviving buffer/view identities, layout, focus and terminal attachments captured before entry. | Hiding does not terminate a session or discard text; restoration uses current native lifecycle admission. |

History range-base selection and ordered revision marks have separate roles.
Appending/removing a cherry-pick or revert target does not replace the range
base, and selecting a range base does not change the revision sequence order.

Generated Git content is read-only and explicitly typed. Native operations
cannot treat it as a file-save target, a text-edit transaction, an Org document,
an LSP document or a terminal merely because a view can display it. Ordinary
source visits, explicit text transfer and commit-message composition use their
existing native owners with their own admission.

## Request and operation records

| Record | Fields and contract |
| --- | --- |
| Action request | Exact command kind, originating session/view, repository, typed targets, captured observations, proposed structured parameters and required source/Git preconditions. |
| Confirmation | Request identity, exact user-readable effect/source/destination/policy, preparation generation, accepted/cancelled/invalidated state and one-use consumption. A changed target cannot reuse old consent. |
| Push intent | Repository, exact source ref/object, configured upstream remote, destination ref and captured tracking/configuration fingerprint; the one-use confirmation binds this complete target. |
| Tag intent and draft | Repository, tag name, lightweight/annotated kind, exact target identity/type, annotation text when selected, applicable signing-policy observation and confirmation state. Default kind is lightweight. Annotation bytes count toward shared draft/session admission, remain available after failure, and are not a second persistent tag store. |
| Commit draft | Session/repository identity, normal/amend/fixup kind, canonical message text, exact HEAD/fixup target, source/index admission and retained failure/cancel state. |
| Direct-command draft | Git argv or explicitly selected shell string, program, cwd, limits and one exact review. No implicit shell parsing in Git mode. |
| Owned operation | Stable operation/generation, action request, current phase, one current child/group, stream owners, cancellation state, absolute deadline and finite cleanup state. |
| Process record | Safe operation/target summary, phase/outcome, elapsed time, actual exit evidence when known, bounded stdout/diagnostics, completeness and originating Git view. |
| Verification | Requested postcondition, accepted refreshed observations, confirmed facts, mismatch or missing evidence. Unknown effects remain unknown. |

Public phases are Checking, Executing, Refreshing, Verifying,
Cancelling and Cleaning up. They are descriptive states, not progress
percentages. One high-level operation may own several sequential checked Git
children; it must not create concurrent unowned children or an implicit task
queue. A refresh request may be coalesced while work is active.

Terminal outcomes distinguish:

- Verified success with the operation's actual postcondition evidence.
- Known failure with its observed resulting repository state.
- Conflict/in-progress state requiring explicit Git sequence control.
- Cancellation or timeout before an effect started.
- Cancellation or timeout after a possible effect, with refreshed facts.
- Verification mismatch.
- Indeterminate effect or verification.

A failed child is not proof that no effect occurred. A successful child is not
proof of a checked final repository outcome. Retained results cannot become
fresh operation authority or a replayable queue. Another terminal/process can
change Git concurrently; editor serialization is not a repository-wide lock.

## Sequence and conflict records

| Record | Identity and fields | Validity |
| --- | --- | --- |
| In-progress operation | Repository/generation, Git-owned operation family and fingerprint, origin when known, current step/revision, relevant pre-operation facts and state/completeness. | External origin is observable; missing retained lineage does not grant unchecked destructive recovery. |
| Ordered revisions | Exact ordered commits, applicable parent choice, operation kind and captured HEAD/branch/sequence context. | Confirmation binds the order; labels and rendered graph positions cannot replace it. |
| Autosquash plan | Exact source range, ordered supported actions/objects, resulting proposed sequence and review state. | A finite declarative Git plan; no arbitrary executable todo commands. |
| Bisect | Exact original context, known good/bad objects, current candidate and classification evidence. | Classification is user input; there is no inferred test result. |
| Conflict | Exact path/kind, index stages and object/mode facts, operation-relative Base/Current/Applied roles, visiting-file/revision/dirty observations and allowed resolution actions. | Missing/binary/unsupported sides remain explicit; absence of visible marker text is not resolution. |
| Resolution request | Conflict fingerprint, chosen complete side or saved result/deletion, source admission, confirmation and expected index/worktree result. | Resolution, staging and continue are separate permissions and transitions. |

Cancellation of a process is not an abort/reset of Git-owned sequence state.
Leaving the UI does not clear that state. Refresh must reconcile it before any
further compatible mutation, including recovery after a partial operation.

The native conflict-explanation action captures an immutable conflict/source snapshot
and uses the existing native AI request owner. Its text result is explanation
data; it cannot become a resolution request, index mutation or sequence control.
Source changes invalidate the snapshot's currency without overwriting source.

## Additional repository entities

- A stash record identifies its exact commit/object and current list context;
  an ordinal alone is not stable identity for apply/pop/drop/branch.
- A worktree record retains Git-validated path, per-worktree metadata, branch,
  lock/prune/busy observations and editor resource relationships. Removing it
  does not grant authority over another context's unsaved text.
- Reflog entries retain exact referenced objects plus bounded presentation
  metadata. Blame retains the exact path/revision/line observation used for
  attribution and visits.
- Clone/init and other prepared families capture explicit destinations and
  arguments. Their native execution uses the same request, operation, process
  and verification records, not a separate unchecked subprocess path.

## Configuration and persistence mapping

The complete approved startup [git] table belongs to the existing configuration
loader, sample, provenance and all-or-default validation. The sibling spec owns
the exact approved keys, defaults, cross-field ranges and hard ceilings. All
Git-owned capacities, including replacement scratch, share its aggregate byte
admission. This model does not maintain a second numeric catalog.

User keybinding entries reference the same native git- command identities as
the built-in keymaps. Built-in command collisions fail validation; configuration
does not silently replace a native command with a declarative definition.

Git owns durable effects through its installed executable and normal configured
hooks, attributes and transport. No editor database/schema migration or separate
repository configuration format is selected. Session records, observations,
process output and message drafts are released under bounded session lifetime;
no replay is inferred after restart.

## Cross-owner invariants

1. Source text, file identity, modified state, save and undo remain native buffer
   and file responsibilities; Git never bypasses them through generated text.
2. Worktree-changing actions check affected unsaved resources. Later file
   changes use the configured reload/ask behavior, with unsaved-edit protection.
3. Git UI hiding and ordinary pane changes do not kill terminal, shell, LSP or
   AI processes. Git owns only children it explicitly created.
4. Existing low-level process/group/stream/wake primitives may be shared without
   sharing higher-level authority, cancellation targets or operation records.
5. Display projections and worker results are owned, generation-bound and
   finite. No worker obtains an unowned mutable reference to the editor session.
6. Paths, object IDs and refs retain exact semantic representation. Display
   escaping, abbreviations, rows and palette roles never define an operation.
7. Startup modal/non-modal profile selection remains immutable. Git's contextual
   navigation and composition modes do not introduce another editing profile.
8. Limits account for observations, parser scratch, drafts, marks, output and
   retained contexts together, rather than bound only the visible rows.

## Readiness

The full native model and specification are approved together. The contextual
binding matrix includes complete sequences and justified compatibility choices;
equivalent actions retain those sequences in both startup profiles. The linked
approved plan and tracker define fourteen ordered slices for one Looper run.

The local Git CLI study establishes limited backend feasibility. Native runtime
implementation, resource measurements and the selected exact-candidate delivery
gate remain the controller-owned work; approval is not delivery evidence.
No durable editor migration or duplicate repository store is introduced.
