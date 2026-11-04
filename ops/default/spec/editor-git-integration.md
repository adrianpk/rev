# Native Git Integration

Status: Approved
Approved: owner, 2026-11-04; complete specification, binding matrix, local policies, resource catalog and one fourteen-slice Looper delivery through dev
Date: 2026-11-03
Reviewed: 2026-11-04
Version: 0.1
Format: OpenSpec capability specification
Source: product-owner request for built-in Rust Git workflows, a structured visual interface and descriptive git-prefixed commands
Decision state: complete native scope, contextual binding matrix, confirmation/local-operation policies and resource bounds approved
Priority: after the integrated-terminal delivery, before keybinding hints in M-x and the remaining presentation follow-ups
Delivery boundary: approved implementation through one fourteen-slice Looper run after completed terminal delivery
Model: [Native Git Model](editor-git-integration-model.md)
Plan: [Native Git Integration Delivery](../plan/rev-git-integration.md)
Tracker: [Native Git Tracker](../tracker/rev-git-integration.md)
Baseline: [rev Editor](rev-editor.md)
Configuration: [Editor Configuration](editor-configuration.md)
Related: [File Navigation](editor-file-navigation.md), [External File Changes](editor-external-file-changes.md), [Modal Editing](editor-modal-editing.md), [Shell Commands](editor-shell-commands.md), [Integrated Terminals](editor-integrated-terminal.md), [Native AI Integration](editor-ai-pi-integration.md)

## Purpose

Work with Git through a native, structured interface inside rev. Inspect the
working tree, index, history and refs; select meaningful repository objects;
and perform explicit Git operations without leaving the editor.

The public vocabulary is Git vocabulary. Commands use the git- prefix, such
as git-status, git-stage and git-commit. The feature has no separate product
name. Descriptive naming does not imply a smaller functional scope or a
reduced visual interface.

The implementation belongs to rev's Rust binary. Repository observations,
selection, commands, presentation and process ownership are native behavior.
User configuration selects preferences and bindings, not executable providers
that supply core Git behavior.

## Approved direction and delivery boundaries

- A dedicated Git view presents structured status, collapsible sections,
  selectable paths and hunks, readable diffs and contextual actions.
- The complete Git-management workflow families below are required native
  coverage, including their listed operations and variants. Planning must not
  substitute a reduced subset or external extension requirement.
- M-x in non-modal editing and colon commands in modal editing invoke the
  same native operations. Contextual keys improve discovery and frequent use.
- Reuse existing editor ownership and preserve ordinary buffers, unsaved
  text, panes, terminals and the startup-selected interaction profile.
- Git owns repository semantics and persistent state. rev owns presentation,
  checked requests, bounded jobs and their observed results.
- The backend uses the installed Git executable with structured arguments;
  native Rust owns the interface, observations and operation lifecycle.
- Fetch updates remote observations without integrating changes. Merge or
  rebase is a separate explicit choice afterward, not an inferred pull policy.
- The dedicated push action sends one branch to its configured upstream.
  It reviews the exact source, remote and destination before explicit
  confirmation and does not introduce implicit force.
- t opens native tag creation with lightweight selected by default. The same
  action offers annotated tags with a user-composed message and Git-owned
  tagger identity and date.
- git-status takes the available editor canvas and retains the prior layout;
  q from Status restores the surviving editing context without stopping terminals.
- q from Status leaves an already started Git operation running under its
  existing owner and limits. git-processes inspects that work; cancellation
  remains an explicit git-process-cancel action.
- Application exit identifies active owned Git work and asks before stopping
  its process. Declining keeps the editor and operation running. Accepted
  termination uses bounded cleanup through the native guarded exit owner.
- C-x g invokes git-status in both startup profiles, alongside M-x and modal
  colon command invocation.
- c in Status opens normal commit-message composition directly. Submission
  remains a separate explicit action after the message has been edited.
- C-Enter and C-c C-c submit the composed commit request. C-g and C-c C-k
  cancel composition before process start and retain its draft in the session.
- Default bindings preserve familiar Git actions and key sequences wherever
  their contextual meanings agree. Conflicting meanings require explicit
  resolution in the approved complete binding matrix. Each exception
  must identify its context, reason and closest compatible alternative.

### Approved additive change to the delivered baseline

This specification adds a native Git capability against **Scope and priority** in
[rev Editor](rev-editor.md#scope-and-priority), whose original baseline
excludes a Git UI. This approval and later delivery record that additive
change without rewriting original completed requirements or gate evidence.

It also extends the native command registry and presentation contexts under
R11, R12, R22 and R24. Generated Git content needs explicit resource admission:
it is not an ordinary writable file merely because it appears in a view.
The [model](editor-git-integration-model.md) defines that distinction.

Interactive terminals and one-shot shell commands remain separate owners.
A native Git view does not require a terminal emulator to implement status,
diffs or command composition. Their completed behavior must remain usable.

## Functional coverage

| Family | Required native behavior |
| --- | --- |
| Status and detail | Repository/branch/tracking header; Unmerged, Untracked, Unstaged and Staged sections; collapse/expand, paths, hunks, read-only detail and file visits. |
| Index changes | Stage/unstage exact paths or compatible text hunks; checked marked groups; explicit whole-path discard from the index. |
| Commits | Compose a normal commit, amend the observed HEAD with confirmation, and create a fixup commit for an exact revision. |
| History | Bounded ancestry pages, exact revision metadata/parents/changed paths/diffs, ordered ranges and retained selection. |
| Refs | Local and remote-tracking branches, tags, configured remotes, symbolic/detached/unborn HEAD and tracking state. |
| Branches and tags | Checked local branch creation/switch/rename/deletion and exact local tag creation/deletion. |
| Synchronization | Explicit fetch and reviewed single-branch push; fetch and merge/rebase remain separate operations. |
| Integration | Explicit merge policy, rebase and reviewed autosquash; observe, continue, skip or abort compatible Git-owned work. |
| Revision sequences | Ordered cherry-pick and revert with exact mainline choice; manual bisect with good/bad/skip/reset. |
| Conflicts | Exact paths, conflict kinds and available index stages; inspect/edit saved results, select a complete side, resolve deletion, stage a checked resolution and request explanation through the optional AI capability. |
| Repository inspection | Stash, linked worktrees, reflog and blame with their own selection, detail and applicable actions. |
| Additional Git families | Clone/init, ignore files, submodules, subtree, notes, apply/format patches and sparse checkout through working native argument/review routes. |
| Processes and direct commands | Active/recent operations, phases, output, diagnostics, cancellation, verified outcomes and an explicit Git or shell command route. |

Slicing may separate these families for review. It must not silently remove
one, label a placeholder as implemented, or turn native operations into an
extension installation requirement. A later scope change needs an explicit
product decision.

## Native presentation

The confirmed default entry takes the available editor canvas for the Git
interface and retains the prior editing layout. It is a presentation context,
distinct from the startup modal/non-modal choice. Source and terminal sessions
remain owned while hidden; they do not receive input intended for Git.

Status is the main entry. Other views retain their originating context and
selection. Return navigation moves from detail to its list, then to Status;
q from Status restores the prior live editor target and layout. File visits
reuse native file identity and source ownership rather than make an editable
copy of displayed Git text.

The layout is structured, not a raw Git transcript. Use existing palette roles
and terminal-cell measurements, clear selection, indentation, section counts
and meaningful diff colors. Color is supplementary; labels and diff prefixes
remain useful without special fonts. Narrow screens keep the selected object
and current action readable without changing its identity.

Illustrative arrangement, not a claim about the current implementation:

```text
Git: trailkit                         main  ahead 1

Unstaged (1)
  src/lib.rs
    @@ build_route
    - let limit = 10;
    + let limit = 20;

Staged (1)
  README.md

s stage   u unstage   c commit   Tab detail   ? actions
```

The final layout may use coordinated panes for detail where that improves
inspection. It must preserve one Git interaction owner and ordinary native
focus, mouse and resize behavior. Exact pane geometry remains a review item;
the structured visual interface is part of the requirement.

## ADDED Requirements

### Requirement: GIT01 Resolve the actual repository context

git-status SHALL resolve Git context from the captured visiting file, or the
explicit selected directory and existing launch-directory fallback when no
file supplies a location. Opening rev with a directory SHALL be sufficient;
there is no required workspace registration or saved-file ceremony.

Git SHALL validate the working tree, per-worktree Git directory and common
repository directory. Nested repositories and linked worktrees SHALL keep
their own identities. A selected tree root SHALL not be silently replaced.
Missing Git, an unavailable repository or Git's own access refusal SHALL
report the actual condition and leave the editor context intact.

#### Scenario: Enter from a directory

- **GIVEN** rev was opened on a directory inside a Git working tree
- **WHEN** git-status is invoked
- **THEN** the checked repository opens without requiring a file or project setup
- **AND** the chosen browsing root and retained source buffers remain unchanged.

### Requirement: GIT02 Own and restore the Git presentation

git-status SHALL use the available editor canvas. The Git view SHALL retain
its originating layout, focus and live resource
identities. Its keyboard and pointer input SHALL target its own visible
semantic content. Initialization failure or ordinary exit SHALL restore the
retained editor context, reconciling resources closed in the meantime through
normal native lifecycle policy. q from Status SHALL restore that retained
layout and focus. Hiding a terminal under Git SHALL not stop it.
Leaving Status SHALL retain an already started Git operation, including its
process, deadlines, streams, refresh and verification. Returning to the editor
SHALL not implicitly cancel, restart or detach it from native ownership.

#### Scenario: Return to an editing session

- **GIVEN** source panes, unsaved text and an owned terminal before Git entry
- **WHEN** the user returns to Status after inspecting History and presses q
- **THEN** the prior surviving layout and selected editing target are restored
- **AND** unsaved text and the terminal process remain owned and intact.

#### Scenario: Return to source while a Git operation runs

- **GIVEN** an owned Git operation is executing in the current repository
- **WHEN** the user presses q from Status to return to source editing
- **THEN** the prior surviving editor context is restored and the same operation continues
- **AND** git-processes can inspect it and git-process-cancel can explicitly request cancellation.

### Requirement: GIT03 Publish structured immutable status

Status SHALL show repository and HEAD/tracking facts, then only nonempty
Unmerged, Untracked, Unstaged and Staged groups in that order. A clean working
tree SHALL have an explicit useful state. Sections, paths and diff units SHALL
have identities independent of row numbers, labels and shortened object IDs.

Refresh SHALL publish one complete bounded observation or retain the last
confirmed one with a visible failure/stale indication. Expansion and selection
SHALL reconcile by compatible semantic identity. Truncation SHALL be explicit.
Generated path/subject/control text SHALL use rev's safe display boundary.

#### Scenario: Refresh while selection moves

- **GIVEN** a selected path and a pending Status refresh
- **WHEN** an external change reorders the returned paths
- **THEN** surviving selection follows that path's identity rather than its old row
- **AND** stale detail cannot remain labeled as current.

### Requirement: GIT04 Inspect exact changes and visit files

Unstaged detail SHALL compare working-tree content with the index; staged
detail SHALL compare the index with the appropriate HEAD baseline. Unborn
HEAD and untracked previews SHALL be represented explicitly. Valid tracked
text detail SHALL expose semantic hunks; binary, rename, type-change,
submodule, malformed and truncated cases SHALL remain inspectable with their
actual action availability.

File visits SHALL select the canonical current source resource when one
exists, preserving unsaved text and ordinary file admission. Revision content
and generated diffs SHALL be read-only resources. Git navigation keys SHALL
not consume characters intended for an editable source or commit message.

#### Scenario: Inspect both layers of one path

- **GIVEN** a path with staged and unstaged changes
- **WHEN** its detail is opened from each corresponding group
- **THEN** each view shows the correct Git comparison and target layer
- **AND** inspecting or visiting it does not stage or overwrite content.

### Requirement: GIT05 Apply checked index and discard actions

Stage and unstage SHALL act on one exact path or supported text hunk. Marked
groups SHALL support complete paths with one action direction, or nonoverlapping
same-path hunks from one layer. Mixed/stale/duplicate/unsupported targets SHALL
reject the complete group; there is no adjacent-row or partial-group fallback.

Discard SHALL initially restore one complete tracked working-tree path from
its checked index state, with explicit confirmation. It SHALL preserve the
index and reject a modified visiting buffer. Untracked deletion or a broader
reset SHALL not be substituted for that action. Every started action SHALL
refresh affected observations and verify what happened.

#### Scenario: Stage one hunk

- **GIVEN** two independent unstaged hunks in one tracked text file
- **WHEN** the user stages one exact current hunk
- **THEN** only that hunk is added to the index
- **AND** the other working-tree change remains unstaged.

#### Scenario: Protect unsaved text during discard

- **GIVEN** a tracked path with a modified visiting buffer
- **WHEN** whole-path discard is requested
- **THEN** the action is rejected without saving or discarding that buffer
- **AND** the index and working-tree file remain untouched by the rejected request.

### Requirement: GIT06 Compose normal amend and fixup commits

Native composition SHALL support a subject/body, ordinary text editing and
explicit submission/cancellation. In Status, c SHALL open normal commit-message
composition directly. Once the composer owns input, a subsequent c SHALL be
ordinary message text, not a pending Git-family suffix. A normal commit SHALL
use only the checked index. Amend SHALL bind the exact observed HEAD and require
confirmation;
fixup SHALL identify one exact target and SHALL not imply autosquash.

C-Enter and C-c C-c SHALL invoke git-commit-submit. C-g and C-c C-k SHALL
invoke git-commit-cancel while composing, retaining the draft before process
start. These aliases SHALL have the same semantics in both startup profiles.
Plain Return SHALL remain message input; submission through C-Enter requires
a host event that distinguishes the Control modifier. C-c C-c remains an
equivalent submission route when the terminal cannot distinguish C-Enter.

Git SHALL continue to own identity, hooks, signing and commit semantics. Failure
SHALL retain the message and expose useful diagnostics. No hidden save, stage,
stash, editor launch or signing-policy override is permitted. A cancelled draft
SHALL remain available within the active Git session.

#### Scenario: Commit only staged content

- **GIVEN** a staged change, a different unstaged change and a composed message
- **WHEN** the user submits a normal commit
- **THEN** Git receives the checked index and exact accepted message
- **AND** unstaged and unsaved editor text is not added implicitly.

### Requirement: GIT07 Browse history revisions and ranges

History SHALL expose bounded ancestry pages and exact revision identities,
metadata, parents, changed paths and read-only diffs. A range SHALL bind its
ordered exact endpoints. Unborn repositories, missing objects, changed HEAD,
page limits and truncated detail SHALL have explicit states.

Selection, paging and return navigation SHALL retain compatible identities.
Inspecting history SHALL not check out files or convert old content into a
writable visiting buffer. Object identity SHALL use the complete Git-reported
format, not an assumed abbreviation length.

#### Scenario: Inspect an older revision

- **GIVEN** an exact revision selected from a current History observation
- **WHEN** metadata, a parent comparison and a range are inspected
- **THEN** each result remains bound to its exact objects
- **AND** current source files, index and HEAD remain unchanged.

### Requirement: GIT08 Inspect and manage refs

Refs SHALL expose local branches, remote-tracking branches, tags, configured
remote names, HEAD form and available upstream/ahead/behind facts. Native
actions SHALL create, switch, rename or delete an admitted local branch and
create or delete an admitted local tag. Names and targets SHALL be validated
by Git and shown explicitly before a changing action.

Branch switch SHALL check index/worktree state and modified repository-backed
buffers. Branch deletion SHALL not silently force an unmerged/current branch.
Native tag creation SHALL offer lightweight and annotated local tags, with
lightweight selected by default through t and git-tag-create. Lightweight
creation collects the name and exact target; annotated creation additionally
collects the message through native composition. The accepted review SHALL
show the chosen kind, target and annotation where present. Git SHALL supply
tagger identity/date and retain its applicable signing policy; rev SHALL not
silently change tag kind or disable signing to make creation succeed. Failure
SHALL retain the composed annotation and show the actual diagnostic. The tag
is local until a separately explicit publication request.

Each started operation SHALL
refresh refs, status, retained history and affected file observations.

#### Scenario: Create a lightweight tag through the default entry

- **GIVEN** an exact commit target, no existing tag with the chosen name and a Git policy permitting lightweight creation
- **WHEN** the user invokes t and accepts the default lightweight kind and review
- **THEN** Git creates the local tag pointing directly to the accepted target
- **AND** tag creation does not publish it to a remote.

#### Scenario: Compose an annotated tag

- **GIVEN** an exact commit target and the annotated kind selected in native creation
- **WHEN** the user composes its message and confirms the name, kind and target
- **THEN** Git creates the annotated local tag with that message and its tagger metadata
- **AND** the observed result distinguishes the tag object from its referenced commit.

#### Scenario: Configured signing conflicts with the chosen tag kind

- **GIVEN** a lightweight tag is selected but observed Git policy requires an annotated signed tag
- **WHEN** native creation prepares its exact review
- **THEN** rev reports the policy conflict and requires an explicit revised choice or cancellation
- **AND** it neither changes the selected kind nor disables signing implicitly.

#### Scenario: Reject a stale ref confirmation

- **GIVEN** a prepared branch operation tied to one exact ref generation
- **WHEN** that ref or HEAD changes before confirmation
- **THEN** the request is rejected or recomposed from a new observation
- **AND** the old displayed label is not used as mutation authority.

### Requirement: GIT09 Synchronize through explicit Git operations

Fetch SHALL use the selected configured remote and observed transport/refspec
configuration without silently changing it. Push SHALL present an exact source,
remote and destination before starting. The confirmed dedicated push route
SHALL send one branch to its configured upstream, with explicit confirmation
of that source/remote/destination. It SHALL not infer a destination when tracking
is missing or contradictory, or introduce implicit force, deletion or multi-ref
publication. Results SHALL distinguish transport observations from verified
publication and unresolved remote state.

The request SHALL bind the exact source ref/object, configured upstream remote
and destination ref. Confirmation SHALL not resolve a different branch or remote
from its display label. Configuration such as automatic tag following or mirror
push SHALL not broaden the reviewed single-branch request; incompatible
configuration must be reported rather than silently expand publication.

Fetch SHALL not integrate changes. Merge versus rebase SHALL be an explicit
subsequent choice, not an inferred pull policy. Existing Git credentials,
helpers, agents and receiver policy remain authoritative; an unavailable
noninteractive credential/signing prerequisite SHALL be reported honestly.

Repository pull configuration SHALL not authorize automatic integration after
the dedicated fetch action. The installed backend may read that configuration,
but the native synchronization workflow still requires the separate user choice.

#### Scenario: Fetch before choosing integration

- **GIVEN** a branch tracking a configured remote
- **WHEN** the user fetches that remote
- **THEN** checked remote-tracking facts are refreshed
- **AND** the current branch is neither merged nor rebased automatically.

#### Scenario: Push one reviewed tracked branch

- **GIVEN** a selected source branch with a valid configured upstream
- **WHEN** the user reviews and confirms its exact source, remote and destination
- **THEN** only that branch is submitted without implicit force or additional tag publication
- **AND** the recorded outcome distinguishes verified publication from unavailable verification.

#### Scenario: Preserve explicit integration despite configured pull policy

- **GIVEN** the repository has a configured pull integration policy
- **WHEN** the native fetch action completes
- **THEN** the current branch is not integrated merely because of that configuration
- **AND** merge or rebase still requires a separate explicit user action.

### Requirement: GIT10 Merge rebase and observe Git-owned sequences

Native actions SHALL offer explicit fast-forward-only, merge-allowed and
merge-commit-required policies, ordinary rebase and bounded reviewed autosquash.
An in-progress view SHALL identify the Git operation, current step and exact
conflicts when available, including operations begun outside rev.

Compatible continue/skip/abort actions SHALL bind a fresh Git-owned sequence
fingerprint and any required retained lineage. Missing evidence SHALL not be
reconstructed from a label. A conflict stop is an in-progress operation, not
successful completion. No implicit autostash or arbitrary executable todo
content is introduced by an integration request.

#### Scenario: Rebase stops at a conflict

- **GIVEN** an explicitly confirmed rebase onto one exact ref
- **WHEN** Git stops at a conflict
- **THEN** the operation view exposes current step, conflicts and admitted controls
- **AND** cancellation of its process is not misreported as a rebase abort.

### Requirement: GIT11 Apply ordered revisions and run manual bisect

History selection SHALL prepare ordered cherry-pick/revert sets, including
an explicit mainline parent where required. Starts and compatible continue,
skip and abort actions SHALL be checked against exact current state.
Ordered revision marking SHALL use explicit append/remove actions, separate
from a History range base. Changing one role SHALL not overwrite the other.

Manual bisect SHALL support explicit good/bad boundaries, current-candidate
classification, skip and reset to the checked original context. It SHALL not
infer a test result or start a build/test command. Order and boundary facts
SHALL be reviewable before mutation.

#### Scenario: Apply an ordered revision set

- **GIVEN** exact compatible revisions selected in an explicit order
- **WHEN** the user confirms cherry-pick for that set
- **THEN** the order and required mainlines are retained through any conflict stop
- **AND** later recovery controls target the same Git-owned operation.

### Requirement: GIT12 Resolve conflicts without losing source ownership

Conflict views SHALL identify the exact path, conflict kind, available index
stages and operation-relative Base/Current/Applied roles. A complete-side
replacement, supported deletion or staging of a saved resolution SHALL require
checked path/state admission and the applicable explicit confirmation.

Native source editing SHALL remain available for manual resolution. Unsaved
text SHALL not be overwritten. Binary, rename/delete, type-change, submodule,
missing-stage and truncated conflicts SHALL remain visible when no text action
is valid. Removing visible conflict markers alone SHALL not establish
resolution, stage a path or continue a sequence.

An explicit conflict-explanation action SHALL use the existing optional native
AI explanation workflow with copied conflict roles/source and its existing
read-only grants. It SHALL remain presentation only: no answer grants write,
stage or continue authority. Missing AI configuration SHALL not block manual
conflict inspection or resolution. No programmable provider layer is required.

#### Scenario: Edit and stage a resolution

- **GIVEN** an unmerged text path with observable index stages
- **WHEN** the user edits and saves its resolution, then requests staging
- **THEN** the exact saved result is checked and staged through Git
- **AND** continuing the operation remains a separate explicit action.

### Requirement: GIT13 Inspect stashes worktrees reflog and blame

Each list SHALL expose bounded exact items, detail, refresh and return
navigation. Stashes SHALL support create, inspect, apply, pop, branch and drop;
worktrees SHALL support create, visit, lock, unlock, prune and remove. Changing
actions SHALL show exact targets and appropriate confirmation before launch.

Reflog and blame SHALL permit inspection of exact revisions and source
locations without treating subjects, line labels or stale ordinal stash names
as durable authority. Linked worktree operations SHALL respect Git's own
ownership and busy-state checks and SHALL NOT remove modified editor resources.

#### Scenario: Apply a selected stash after the list changes

- **GIVEN** a selected stash and an external stash-list change
- **WHEN** the user requests apply
- **THEN** its exact object and current operation context are revalidated
- **AND** another stash is not selected merely because it inherited the same ordinal.

### Requirement: GIT14 Provide additional repository command families

Clone/init, ignore files, submodule, subtree, notes, apply/format patches and
sparse checkout SHALL have native discoverable entry points. Where a dedicated
form is unnecessary, a prepared structured Git-argument editor and exact review
SHALL perform the real operation through the shared checked process owner.
Such a route is functioning behavior, not a stub or an addon requirement.

Required variants are regular/shallow/bare/mirror clone; repository init;
visits to .gitignore and .git/info/exclude; submodule add/list/update/sync/deinit;
subtree add/merge/pull/push/split; notes add/remove/prune; mailbox and plain-patch
application; format-patch, saved diff and request-pull output; and sparse
checkout enable/disable/reapply/set/add. All use native composition, exact review,
owned execution and actual result inspection. A generic prompt without working
family preparation and execution does not satisfy this coverage.

Clone and init SHALL require explicit destinations. Ignore-file commands SHALL
visit the actual ordinary file without inventing a pattern. Missing optional
Git subcommands SHALL report their availability. Patch/mail preparation SHALL
use explicit local sources/ranges and SHALL not imply sending mail.

#### Scenario: Initialize an explicitly selected destination

- **GIVEN** the user invokes git-init and chooses a destination
- **WHEN** the exact Git arguments are reviewed and accepted
- **THEN** the owned operation runs against that destination and verifies its result
- **AND** no destination is inferred from a label or hidden project configuration.

### Requirement: GIT15 Keep an explicit direct-command route

git-run-command SHALL compose structured Git arguments without implicit shell
evaluation. git-run-shell-command SHALL be a distinct explicit shell route.
Both SHALL show their executable/mode, directory and accepted input before
execution, use the same operation/cleanup owner and refresh compatible Git
observations after any possible effect.

The native interface SHALL preserve an explicit path for operations without a
dedicated form. It SHALL not replay old output as a command or infer safe
effects for arbitrary arguments. Unknown postconditions SHALL remain unknown;
the result must state what was run and what was actually observed.

#### Scenario: Cancel a composed direct command

- **GIVEN** a prepared direct command and its exact review
- **WHEN** the user cancels before execution
- **THEN** no child or repository effect is started
- **AND** ordinary source and Git observations remain available.

### Requirement: GIT16 Expose processes and truthful outcomes

Processes SHALL show the active operation first and bounded recent records
afterward, with exact identity, phase, elapsed time, outcome and available
exit/output diagnostics. Detail SHALL retain its originating Git view and
selection. Opening/closing detail or leaving a list SHALL not cancel a job.
git-processes SHALL remain invocable through native commands after the Git
presentation is hidden. git-process-cancel SHALL target an explicitly selected
owned operation through the same checked cancellation path. Inspection or view
exit SHALL not itself grant cancellation authority.

Phases SHALL distinguish checking, executing, refreshing, verifying and cleanup.
Success requires evidence appropriate to that operation, not only exit code
zero. Failure, verification mismatch, cancellation/timeout before or after
possible effect, conflict stop and indeterminate state SHALL remain distinct.
Progress SHALL not invent percentages from elapsed time or output volume.

#### Scenario: A child exits after a possible effect

- **GIVEN** a changing operation whose process exits or is interrupted
- **WHEN** the available refresh cannot prove its final repository result
- **THEN** Processes reports the actual child facts and indeterminate outcome
- **AND** it does not report success or automatically retry the mutation.

### Requirement: GIT17 Revalidate generation-bound operations

One checked repository context SHALL own at most one active high-level
operation and its current child. Requests SHALL capture semantic targets,
required observations, parameters and one-shot confirmation before launch.
Confirmation SHALL be invalidated by incompatible context, source or Git state.

Git's own precondition and locking mechanisms SHALL be used where available.
Editor serialization SHALL not claim to lock out another terminal or process.
After a possible effect, refresh and verification SHALL replace prior authority.
Retries SHALL be new checked requests; no blind replay is permitted.

#### Scenario: An external command changes the index

- **GIVEN** a prepared index action tied to one observed path/hunk layer
- **WHEN** another process changes that layer before execution
- **THEN** the stale action fails admission or Git's applicable precondition
- **AND** the user receives current facts rather than a silently retargeted action.

### Requirement: GIT18 Integrate native commands keys and pointer input

All operations SHALL use the shared native command catalog under git- names.
Built-in and declarative-command name collisions SHALL be rejected; adding a
Git capability SHALL not silently replace an existing command or user binding.
M-x and modal colon SHALL resolve the same native command and contextual target.
C-x g SHALL invoke git-status in both startup profiles. Equivalent Git actions
SHALL retain the same contextual binding and sequence shape across profiles.
Default keys SHALL preserve familiar Git interaction conventions wherever their
meanings are compatible; conflicting meanings SHALL be resolved explicitly
before binding approval rather than silently assigned a different action.
The preparation matrix SHALL record the operation, native command, input
context and complete key sequence. Every departure from the reviewed reference
interaction SHALL include its concrete constraint, reason and closest compatible
alternative for review before slicing. Convenience or a new
naming preference alone SHALL not justify a different binding.

Status/list navigation, Tab detail, contextual action dispatch, mouse selection,
scrolling and native cancellation SHALL operate on exact semantic items.
Contextual q SHALL return/exit in Git navigation, not intercept text typed in a
composer or ordinary source. Both startup profiles remain usable; Git entry
does not change the configured profile. Exact approved default keys follow below.

#### Scenario: Invoke a contextual action through a named command

- **GIVEN** a supported selected hunk in the Git view
- **WHEN** git-stage is chosen through the active profile's command interface
- **THEN** it uses the same checked target as the contextual stage key
- **AND** missing context opens selection or reports its requirement instead of guessing.

#### Scenario: Keep a learned sequence across editing profiles

- **GIVEN** a Git action and contextual sequence in the approved binding matrix
- **WHEN** the same Git context is used under the other startup profile
- **THEN** the same sequence invokes the same action on the same kind of target
- **AND** source and commit-message input keep their own editing semantics.

### Requirement: GIT19 Preserve file state and bounded operation lifetime

Git SHALL operate on Git/disk state, not silently save or stage a visiting
rope. Actions that replace worktree files SHALL check affected unsaved buffers;
branch/integration operations SHALL honor their broader dirty-state admission.
After a started action, changed files SHALL reconcile through the existing
reload/ask policy, retaining unsaved-edit protection and ordinary undo ownership.

Jobs SHALL be nonblocking, have finite output, event, retention and time bounds,
and own graceful cancellation, finite escalation, reaping and stream closure.
Clean idle Git views SHALL not add a polling busy loop. Leaving the Git UI
SHALL retain started work under its existing ownership and finite limits.
Application exit while owned Git work is active SHALL identify that work and
require explicit confirmation before stopping its process and exiting. Declining
or cancelling that decision SHALL retain the editor and SHALL not terminate the
operation. Its ordinary completion and deadline remain effective.

Accepted termination SHALL use bounded cancellation, escalation and reaping
through the existing guarded exit coordinator, alongside ordinary unsaved-buffer
and terminal decisions. Only rev-owned processes are eligible. Stopping a child
SHALL not automatically run sequence abort/reset or claim repository rollback.
A Git-owned sequence without an active owned child SHALL remain repository
state, not a process to terminate. Possible effects keep their actual Git state.

#### Scenario: Decline exit while Git is working

- **GIVEN** rev owns an active Git operation
- **WHEN** the user requests application exit and declines stopping that work
- **THEN** the editor remains open without forcibly terminating the operation
- **AND** the operation remains subject to its normal completion and deadline.

#### Scenario: Update a clean visiting file under ask policy

- **GIVEN** files.external_changes is ask and a clean visiting source file
- **WHEN** an admitted Git action changes that file
- **THEN** the existing reload policy retains its notification/acceptance behavior
- **AND** Git does not force a different policy or overwrite unsaved text elsewhere.

### Requirement: GIT20 Keep configuration native and behavior optional

Preferences and bindings SHALL use rev's startup TOML, complete sample, typed
defaults and whole-candidate validation. Operation arguments/messages are
session input, not per-project editor configuration. No package, scripting
runtime or extension provider is required for Git behavior.

Missing Git or Git configuration SHALL not disrupt ordinary editing. Use Git's
installed executable, repository configuration, attributes and hooks under
explicit user actions. Native jobs SHALL not invoke a hidden interactive Git
editor or collect credentials in generated views. Diagnostics SHALL avoid
exposing credentials embedded in transport/configuration text.

#### Scenario: Use rev without Git installed

- **GIVEN** ordinary source editing and no available Git executable
- **WHEN** the user edits normally and later requests git-status
- **THEN** editing remains independent and git-status reports Git unavailable
- **AND** no install, provider initialization or external service starts automatically.

## Native command families

These identifiers define approved native commands to deliver, not claims about
the current binary. Implementation must reconcile the complete names and
bindings with the live catalog without silently changing the approved contract. Contextual commands request missing arguments through ordinary
native selection/composition rather than accept pasted display text as identity.

| Purpose | Native commands |
| --- | --- |
| Entry and presentation | git-status, git-refresh, git-quit, git-dispatch, git-toggle-section, git-visit-file |
| Selection and index | git-mark, git-unmark, git-unmark-all, git-stage, git-unstage, git-discard |
| Diff and history | git-diff, git-log, git-show, git-show-range, git-log-next-page |
| Commit composition | git-commit, git-commit-amend, git-commit-fixup, git-commit-submit, git-commit-cancel |
| Refs and transport | git-refs, git-fetch, git-push, git-branch-create, git-branch-switch, git-branch-rename, git-branch-delete, git-tag-create, git-tag-delete |
| Integration | git-merge, git-merge-fast-forward, git-merge-no-fast-forward, git-merge-continue, git-merge-abort, git-rebase, git-rebase-autosquash, git-rebase-continue, git-rebase-skip, git-rebase-abort, git-operation-status, git-operation-continue, git-operation-skip, git-operation-abort |
| Revision sequences | git-sequence-mark, git-sequence-unmark, git-cherry-pick, git-cherry-pick-continue, git-cherry-pick-skip, git-cherry-pick-abort, git-revert, git-revert-continue, git-revert-skip, git-revert-abort, git-bisect-start, git-bisect-good, git-bisect-bad, git-bisect-skip, git-bisect-reset |
| Conflict resolution | git-conflicts, git-conflict-visit, git-conflict-use-base, git-conflict-use-current, git-conflict-use-applied, git-conflict-delete, git-conflict-stage, git-conflict-explain through the existing optional AI workflow |
| Stashes | git-stash-list, git-stash-create, git-stash-show, git-stash-apply, git-stash-pop, git-stash-branch, git-stash-drop |
| Worktrees | git-worktree-list, git-worktree-create, git-worktree-visit, git-worktree-lock, git-worktree-unlock, git-worktree-prune, git-worktree-remove |
| Inspection | git-reflog, git-blame |
| Additional families | git-clone, git-init, git-ignore, git-submodule, git-subtree, git-notes, git-apply-patch, git-format-patch, git-sparse-checkout |
| Direct command and process feedback | git-run-command, git-run-shell-command, git-processes, git-process-show, git-process-cancel |

Confirmed global entry: C-x g invokes git-status in both startup profiles.
M-x git-status and modal :git-status remain the corresponding named routes.

Approved navigation: Up/Down and C-p/C-n move, Tab expands detail, Return
visits/inspects the exact selected item, g refreshes, ? opens contextual actions
and q returns. Status uses s stage, u unstage, c commit, h History, b Refs
and $ Processes. Native command invocation remains available for every action.
Commit composition confirms C-Enter or C-c C-c for submission and C-g or
C-c C-k for cancellation. Other contextual bindings and non-composer
confirmations follow the approved matrix. Matching
familiar bindings must preserve the action and complete sequence, not only the
first key. An alias must not create ambiguous prefix/composer input.

### Binding review matrix

Global entry, Status return and commit composition controls are confirmed.
The complete matrix below is approved for delivery, not a claim about the
current binary. They follow the reviewed interaction baseline. Additional
compatibility aliases are identified explicitly. Input context is part of
every binding; a key is never reassigned merely because another view uses it.

#### Entry and Status

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| Both profiles | Open Git / git-status | C-x g | Confirmed; M-x and modal colon remain available. |
| Status | Return / git-quit | q | Confirmed; restore the retained editor context and keep started Git work running. |
| Status, History, Refs, Sequence | Previous/next item | Up/Down, C-p/C-n | Move semantic selection without changing the selected Git object. |
| Status, History | Previous/next item | p / n | Approved compatibility aliases only in contexts without a conflicting action. |
| Status | Toggle detail / git-toggle-section | Tab | Preserve section, path and hunk identity. |
| Status | Visit / git-visit-file | Return | Visit the real native source resource. |
| Git observation views | Refresh / git-refresh | g | Keep the view and compatible semantic selection. |
| Status, Refs | Action dispatcher / git-dispatch | ? | Show context-appropriate actions; Sequence retains its distinct bisect-start key. |
| Status | Stage/unstage / git-stage, git-unstage | s / u | Use the exact selected path/hunk or compatible marked group. |
| Status | Discard / git-discard | k | Prepare one checked whole-path discard; confirmation stays separate. |
| Status | Mark/unmark/clear / git-mark, git-unmark, git-unmark-all | * / - / 0 | Keep path/hunk marks separate from ordered revision marks. |
| Status | Compose / git-commit | c | Confirmed; open the composer directly. A following c is message text. |
| Status | History / git-log | h | Keep the direct history entry. |
| Status | Current-branch history / git-log | l l | Approved compatible sequence; detached HEAD prompts for an exact history root. |
| Status | Refs / git-refs | b | Keep direct ref inspection. |
| Status | Refs / git-refs | y | Approved nonconflicting compatibility alias for ref inspection. |
| Git navigation | Processes / git-processes | $ | Inspect the active operation and bounded recent records. |
| Git navigation | Direct Git command / git-run-command | ! | Open structured Git-argument composition and exact review. |
| Any eligible native command context | Direct shell command / git-run-shell-command | Named command | Choose shell mode explicitly; no implicit shell evaluation in Git mode. |
| Any eligible native command context | Conflicts / git-conflicts; active sequence / git-operation-status | Named commands | Keep these entry points available without reusing a family prefix. |
| Menus | Return without effect | q, C-g, Escape | Dismiss the innermost menu and retain its originating selection. |
| Processes | Inspect / git-process-show | Return or named command | Show output and diagnostics for the selected operation. |
| Processes with an owned active operation | Cancel / git-process-cancel | C-g or named command | Explicit checked cancellation; q only leaves the view. |

#### History and Refs

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| History | Inspect revision / git-show | Tab | Read exact metadata, parents, changed paths and bounded diff. |
| History | Set range base / git-mark | * | Retain one exact compatible range base. |
| History | Inspect range / git-show-range | Return | Use the marked base and selected endpoint in their explicit order. |
| History | Next page / git-log-next-page | f | Page history; this key does not fetch a remote in History. |
| History | Amend / git-commit-amend | A | Compose against the exact observed HEAD, then require its review. |
| History | Fixup / git-commit-fixup | F | Use the selected exact revision; no implicit autosquash. |
| History | Append/remove ordered revision / git-sequence-mark, git-sequence-unmark | a / U | Keep cherry-pick/revert order distinct from range selection. |
| History | Cherry-pick/revert / git-cherry-pick, git-revert | P / V | Prepare the ordered set and any required mainline choices. |
| History and revision detail | Return / git-quit | q | Detail/range returns to History; History returns to Status. |
| Refs | Create/switch/rename/delete branch | n / S / R / D | Use the exact selected ref and normal checked confirmation. |
| Refs | Create/delete tag / git-tag-create, git-tag-delete | t / T | t and both creation kinds are confirmed: lightweight by default, annotated by explicit selection. T retains exact local deletion. |
| Refs | Mark/clear push source / git-mark, git-unmark | * / - | Retain one exact source branch. |
| Refs | Fetch / git-fetch | & | Select the configured remote; do not integrate the result. |
| Refs | Push / git-push | p | Confirmed policy: one branch to its upstream after source/remote/destination review; key is approved in this matrix. |
| Refs | Fast-forward-only / merge-allowed / merge-required | = / i / I | Invoke git-merge-fast-forward, git-merge or git-merge-no-fast-forward. |
| Refs | Rebase / git-rebase | e | Prepare an explicit selected base. |
| Refs | Autosquash / git-rebase-autosquash | E or r a | Review the finite sequence; never execute arbitrary todo commands. |
| Refs | Return / git-quit | q | Restore Status and retain any started owned work. |

#### Sequences and conflicts

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| Sequence | Inspect selected conflict / git-conflict-visit | Return | Show the checked conflict and available roles before editing. |
| Sequence | Merge continue/abort |  > / X | Invoke git-merge-continue or git-merge-abort for that exact family. |
| Sequence | Rebase continue/skip/abort | ] / ; / Z | Invoke the corresponding git-rebase-* command. |
| Sequence | Cherry-pick continue/skip/abort | { / l / Y | Invoke the corresponding git-cherry-pick-* command. |
| Sequence | Revert continue/skip/abort | } / j / Q | Invoke the corresponding git-revert-* command. |
| Sequence | Start manual bisect / git-bisect-start | ? | Prepare explicit boundaries; no automatic test execution. |
| Sequence | Bisect good/bad/skip/reset | G / B / \| / ~ | Invoke the corresponding git-bisect-* command on the checked candidate. |
| Conflict | Select Base/Current/Applied | 1 / 2 / 3 | Prepare a complete-side replacement with its exact confirmation. |
| Conflict | Resolve deletion / git-conflict-delete | K | Review the exact supported deletion. |
| Conflict | Stage saved resolution / git-conflict-stage | x | Require a saved checked result; continuing remains separate. |
| Conflict | Explain / git-conflict-explain | H | Use the existing optional AI request owner; its answer has no edit authority. |
| Sequence, Conflict | Return / git-quit | q | Conflict returns to Sequence; Sequence returns to Status without aborting Git state. |
| Eligible sequence context | Family-independent controls / git-operation-continue, git-operation-skip, git-operation-abort | Named commands | Bind the observed exact family; reject unavailable or mismatched actions. |

#### Stashes worktrees reflog and blame

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| Status | Stashes / git-stash-list | z z or z l | Open the native stash list. |
| Status | Worktrees / git-worktree-list | Z Z or Z l | Open the native linked-worktree list. |
| Status | Reflog / git-reflog | l r | Open exact reflog entries. |
| Status | Blame / git-blame | B b or B B | Attribute the selected exact path. |
| Stash, Worktree, Reflog, Blame | Next item | n, Down or C-n | Move semantic selection. |
| Worktree, Reflog, Blame | Previous item | p, Up or C-p | p is a navigation alias here. |
| Stash | Previous item | Up or C-p | p remains the pop action in this context. |
| Inspection lists | Toggle retained detail | Return | Do not mutate the selected object. |
| Stash | Create/inspect/apply/pop/branch/drop | c / v / a / p / b / k | Use git-stash-create/show/apply/pop/branch/drop; changing actions require exact review. |
| Worktree | Create/visit/lock/unlock/prune/remove | c / v / L / U / P / k | Use the corresponding git-worktree-* command and Git's ownership checks. |
| Reflog, Blame | Inspect selected revision / git-show | v | Use its exact object identity. |
| Inspection lists | Refresh / git-refresh; return / git-quit | g / q | Keep compatible selection or return to Status. |

#### Prepared command families

Family prefixes open a native action menu; the suffix selects a typed variant.
Unavailable actions remain visible with their reason, rather than falling
through to another operation. A source/composer character never starts a menu.

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| Status | Clone / git-clone | C C / C s / C b / C m | Regular / shallow / bare / mirror; require an explicit destination. |
| Status | Ignore / git-ignore | i t / i p | Visit .gitignore / .git/info/exclude. |
| Status | Init / git-init | I I | Prepare initialization at an explicit destination. |
| Status | Submodule / git-submodule | o a / o l / o u / o s / o d | Add / list / update / synchronize / deinitialize. |
| Status | Subtree / git-subtree | O a / O m / O f / O p / O s | Add / merge / pull / push / split. |
| Status | Notes / git-notes | T a / T r / T p | Add / remove / prune. |
| Status | Apply patches / git-apply-patch | w a / w p | Mailbox application / plain-patch application. |
| Status | Patch output / git-format-patch | W c / W s / W r | Format-patch / save diff / request-pull output; no mail sending. |
| Status | Sparse checkout / git-sparse-checkout | > e / > d / > r / > s / > a | Enable / disable / reapply / set / add. |
| Status or Refs, where eligible | Selected diff / git-diff | d d | Inspect the selected comparison without changing the worktree. |
| Refs, where eligible | Merge policies / continuation / abort | m f / m m / m n / m c / m a | Same native requests as the direct merge controls. |
| Refs, where eligible | Rebase / autosquash / continue / skip / abort | r r / r a / r c / r s / r A | Same native requests and exact sequence checks. |
| Refs with a selected remote | Refs / fetch / push | M b / M f / M p | Keep explicit remote and destination admission. |
| Status with a selected path/hunk | Mark / unmark / clear / discard | x m / x u / x c / x d | Same semantic target operations as the direct keys. |

#### Composition and confirmation

| Context | Operation / native command | Sequence | Behavior or review note |
| --- | --- | --- | --- |
| Commit composer | Submit / git-commit-submit | C-Enter or C-c C-c | Confirmed; one explicit submission. Plain Return is message input. |
| Commit composer | Cancel / git-commit-cancel | C-g or C-c C-k | Confirmed; retain the session draft before process start. |
| Exact action review for branch, tag, merge, rebase, sequence, bisect or conflict | Accept the checked request | C-Enter; equivalent C-c C-c | C-c C-c provides a distinguishable terminal route without changing the reviewed effect. |
| Exact action review | Cancel without effect | C-g or q | Return to the originating Git view; do not reinterpret text in an editor as cancellation. |
| Discard or stash/worktree action confirmation | Accept / decline | y / n; C-g cancels | Keep the existing yes/no interaction and exact captured target. |
| Prepared direct-argument or shell composer | Open exact review | C-Enter; equivalent C-c C-c | First acceptance opens review; it does not launch the child. |
| Prepared command exact review | Execute / cancel | C-Enter or C-c C-c / C-g | Execute only the accepted program, mode, directory and arguments. |

#### Compatibility decisions

- Direct c composition is confirmed. A c c alias cannot be interpreted as a
  family selection after the first c has transferred input to message editing.
  Amend and fixup retain their native commands and History bindings.
- h remains direct History and ? remains the action dispatcher. The approved
  l l route offers a compatible current-branch history sequence without
  taking h away from the reviewed baseline.
- b remains direct Refs. The approved y alias adds ref inspection without
  changing b into a branch-command menu. Branch actions remain explicit in Refs.
- f is History paging, while p is push in Refs and pop in Stash. These keys
  cannot also mean fetch or previous-item in those same contexts. Up/Down and
  C-p/C-n remain available; p/n aliases are limited to nonconflicting contexts.
- The same capital key may have a different role in Status, History or Sequence
  only when those distinct contexts already define it. Native menus and the
  reference must show the active context and complete sequence.
- Control-Enter requires a distinguishable host event. C-c C-c is confirmed
  for commit submission and approved as the same-effect terminal alternative
  for other exact reviews. Plain Return is never guessed to mean Control-Enter.
- Ordinary source editing retains its own keys. q is Git return only while
  Git navigation owns input; revisiting Git uses C-x g or its named command.

Each family above must also be reachable by its native git-* command, with
missing arguments collected through native selection/composition. Sequence
family-specific names and generic operation controls share the same owner;
a family-specific request rejects a different active operation rather than
silently retargeting it.

## Rust integration and configuration direction

The confirmed backend uses the installed Git executable with structured argv,
bounded machine-readable observations and rev-owned process/stream lifetime.
This preserves Git configuration, hook and transport behavior while avoiding a
second implementation of repository semantics. No additional crate or native
library is selected by this specification. Runtime capability verification belongs to
preparation; approval of the backend does not establish untested Git behavior.

Use existing async observation/wake, checked file/reload, command/keymap,
display, selection and process primitives where their contracts fit. Git needs
its own semantic repository/operation owner, not a mutable text transcript
from which repository actions are reparsed. Retain rendering on the editor
thread; workers deliver owned generation-bound observations.

The following complete [git] preference catalog is approved for delivery,
not current configuration keys. Defaults are finite engineering choices;
the plan distinguishes measured Git CLI evidence from future editor acceptance.

| Key | Default | Accepted range or policy |
| --- | --- | --- |
| program | git | Nonempty literal executable, at most 4096 bytes; no automatic installation or shell expansion. |
| observation_timeout_seconds | 10 | 1-60 seconds; independently bounded from mutations and transport. |
| operation_timeout_seconds | 60 | 1-600 seconds; no automatic increase after a hang. |
| transport_timeout_seconds | 120 | 1-600 seconds; retain uncertain publication truth. |
| max_status_paths | 1024 | 1-65536 complete records; explicit truncation. |
| max_path_bytes | 262144 | 4096-4194304 aggregate retained Git path bytes. |
| history_page_size | 128 | 1-1024 revisions per page with exact ancestry continuation. |
| max_history_pages | 4 | 1-16 retained pages per repository context. |
| max_refs | 256 | 1-16384 complete ref records, within aggregate admission. |
| max_remotes | 32 | 1-256 complete remote records, within aggregate admission. |
| max_output_bytes | 1048576 | 4096-16777216 aggregate captured job bytes; completeness remains explicit. |
| max_detail_bytes | 524288 | 4096-8388608 diff/revision bytes; no more than max_output_bytes. |
| max_diagnostic_bytes | 65536 | 1024-1048576 bytes; no more than max_output_bytes, with credential-safe presentation. |
| max_process_records | 8 | 1-32 session-local active/recent records. |
| max_process_record_bytes | 2097152 | 65536-16777216 total retained process-record bytes; at least max_output_bytes. |

Use existing complete-sample, origin and all-or-default validation, including
cross-field checks. Changing a preference cannot bypass hard admission or
silently evict a live operation. Completed records may be evicted explicitly
under the retention policy; active ownership is retained until cleanup.

Hard engineering ceilings approved with this catalog are four retained
repository contexts with one child each, sixteen retained navigation contexts,
256 marked targets, 64 structured argv tokens, 4096 bytes per token or exact
path, 65536 combined argv bytes and 4096 bytes per metadata/control string.
Use a checked NUL input manifest where the installed Git operation supports it
instead of expanding a marked group beyond argv admission. An unrepresentable
request fails before launch; no partial group is substituted.

Commit and tag drafts each admit at most 65536 bytes per repository context.
Parser scratch admits at most 1048576 bytes per worker. Retained observations,
history, drafts, marks, projections, stream bytes, result queues and process
records share a 33554432-byte editor-wide Git admission budget. Charge owned
capacities and conservative metadata costs before publication, including
temporary replacement candidates; allocator/RSS measurements remain separate.
Native source buffers retain their existing independent ownership and limits.

Service at most 65536 stream bytes and 64 coalesced events across repositories
per editor turn, with fair progress and readiness wakes. Quiet Git state creates
no polling timer. Cancellation uses a 500 ms grace and a 2 second cleanup
deadline through the existing process owner; unresolved cleanup stays visible
and owned. These are design bounds, not measured responsiveness guarantees.
Slice 1 verifies admission and capabilities; joint acceptance measures actual
binary/startup/idle/allocation cost. No limit silently removes a workflow family.

Git remains the durable store. rev persists only approved preferences and any
separately agreed draft-retention policy; no repository database, replayable
operation queue or hidden persistent command archive is introduced.

## Approval and implementation readiness

The owner approved the complete specification/model, contextual binding matrix,
local branch and whole-path discard policies, startup catalog, engineering
ceilings and fourteen-slice plan/tracker on 2026-11-04, and explicitly requested
one Looper run through dev. The approved aliases and confirmation controls are
part of that contract. Scope or behavioral changes require a concrete reviewed
decision; execution must not silently substitute a smaller family inventory.

A successful isolated Git CLI study is recorded in the plan. It verifies only
the stated backend capabilities; native process/UI/credential/sequence
acceptance remains delivery work. Cancelled commit and tag drafts retain the
approved session-only scope.

Approval authorizes preparation publication, one canonical slice PR at a time,
controller-owned integration and the selected final gate and closure. It does
not establish native runtime delivery or a passing integration gate.

## Acceptance direction

Use native Rust tests and isolated real Git repositories with local remotes.
Exercise status/index/hunks, commit/amend/fixup, history/ranges, refs/transport,
branch/tag changes, sequences/conflicts, stash/worktree/reflog/blame and the
additional/direct command routes. Verify actual repository objects and files,
not only messages, exit codes or terminal screenshots.

Include initial/unborn/detached repositories, linked worktrees, stale targets,
external changes, unusual path bytes, modified visiting buffers, failed hooks,
missing credentials, cancellation after possible effect, truncated output and
partial/unsupported conflicts. Native failure injection may supplement real
boundaries where a reproducible effect/verification failure needs it.

Representative Linux PTY journeys must cover both profiles, section/diff
navigation, pointer selection, resize, source/terminal context restoration,
composition/cancellation and no idle busy loop. Do not contact the user's
repositories or remotes during acceptance. Keep Rust/Bash validation selected
for affected contracts rather than chain every historical feature gate.

Trace every functional-coverage row and listed variant to its native command,
slice tasks and acceptance evidence. Exercise the approved binding matrix in
both profiles, including sequence meaning, composition and return behavior.
Neither successful staging/commit alone nor a command-name inventory establishes
delivery of the remaining Git-management operations.

As part of this approved delivery, add a progressive Git learning
path and task/reference pages with small visual examples. Markdown remains
the source for the Spore guide; Git learning must remain independent of Org
learning and ordinary file editing. Update Unreleased when behavior is usable,
not merely when this specification exists.
