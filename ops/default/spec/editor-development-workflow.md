# Development Workflow Plugin

Status: Draft
Date: 2026-11-08
Version: 0.3
Format: OpenSpec capability specification proposal
Decision state: formal behavioral draft; experimental host contract and execution design remain under review
Review scope: canonical formal draft; no prototype or implementation delivery is started
Implementation location: independent package repository permitted; no rev-private integration required
Host contract: Plugin System draft 0.1 and its model; proposed compatibility, not tested runtime support
Parent: [Plugin System](editor-plugin-system.md)
Model: [Workflow Context, Artifacts and Execution](editor-development-workflow-model.md)
Existing integration: [Pi Integration](editor-ai-pi-integration.md)

## Purpose

Explore a plugin that supports the development workflow used to build editor
features: capture a context dump, refine it through questions, generate a
coherent specification/slices/plan/tracker bundle, then execute the agreed work
through a controller owned by the plugin.

The product is the workflow plugin. It is also a demanding test case for the
proposed extension API. Success would demonstrate that an ordinary package can
coordinate several buffers, AI interactions, durable artifacts and live progress
without making its domain model part of the editor core.

This specification formalizes behavior independently of the repository in which
the package may be implemented. It targets the linked plugin contract through
public methods, never imports of private editor modules. The implementation
language remains open. Status remains Draft; this is not an implementation plan
or permission to operate on the current repository. The host API is experimental
and structural conformance is not a claim of tested runtime compatibility.

## Scope and ownership

| Owner | Responsibility |
| --- | --- |
| Editor host | Canonical buffers, revisions, undo, views, keymaps, extension admission, cancellation and generic resource ownership. |
| Workflow package | Live context, question/answer relationships, generation, freshness, frozen bundles, single-file tracker and execution controller. |
| Package Pi adapter | Owned Pi sessions, protocol adaptation, model/provider observations and request completion. |
| Implementation/validation workers | Work admitted by the frozen execution contract; observed outcomes return to the package controller. |

The package may be built in another repository and installed through the plugin
ecosystem. Package version, host API revision, target project, artifact generation
and execution run have separate identities. The target project is selected
explicitly; it is not inferred from the package or editor source directory.

## Owner direction

- Capture free-form development context and ask a questionnaire to refine it.
- Generate specification, slicing, plan and one traditional tracker file through
  one user action. Do not substitute a ticket system for the tracker.
- Implement the resulting work through a plugin-owned controller analogous in
  behavior to the current development loop.
- Consider the existing Pi integration without assuming it can be reused intact.
- Explore dedicated dump, question, document and progress buffers/panes.
- Keep all drafting material revisable until Start, including earlier questions
  and answers. Start fixes the execution contract within the package.
- Observe outside file changes without claiming to prevent the user editing files
  through another tool or another editor path.
- Keep this proposal independently reviewable against the plugin contract;
  implementation and the deferred UX/UI questionnaire retain their own scope.

## Candidate interaction

Use related, live text buffers rather than a one-way wizard. Before Start, the
user can add to the dump, rewrite an earlier question, revise an answer, edit
the specification and regenerate dependent documents in any order. These are
activities over one shared context, not stages that lock their predecessors.
One workflow session owns the views and their revision relationships.

The editor integration adds a persistent, directly editable representation of
the working material: the user can inspect relationships and change the text
where it lives. Requiring the user to restate every correction in a chat transcript
would miss the point of this package.

| View | Role | Candidate interaction |
| --- | --- | --- |
| Dump | Editable source context, goals and constraints. | Add or revise text whenever needed before Start; no separate context-submission step is required for every edit. |
| Questions | Identified questions, answers and unresolved decisions. | Rewrite questions and answers, including earlier ones; keep their revision relationships visible. |
| Documents | Generated specification and the related planning artifacts. | Edit and regenerate against the current context; earlier generation does not lock these buffers. |
| Execution | Current slice/task, progress, diagnostics and run controls. | Inspect progress and invoke admitted pause/resume/cancel actions through native commands and local bindings. |

Begin with ordinary buffers and existing pane navigation. Views can be shown one
at a time or together using native splits; the plugin should not force a fixed
four-pane layout. A richer status presentation is a candidate improvement after
the state and interaction contracts work.

Questionnaire presentation is still open: one question at a time, an editable
question/answer buffer, or both. Every supported presentation must allow revisiting
and editing previous questions and answers. Their identities and history must
not depend on the selected layout.

## Context and generation

A context revision includes the current dump, questions, answers, explicit decisions,
manual document edits and selected project/editor observations. Unsaved text is captured by revision;
directory contents alone are not a complete representation of editor context.
The selected root is explicit and uses existing file/directory/project identity.
Collecting a dump does not create or register a project automatically.

One generate action requests a coherent bundle containing:

1. Specification, including a model companion when the project convention needs it.
2. Ordered slice definitions with dependencies and acceptance boundaries.
3. A plan using those same slice identities.
4. One tracker Markdown file covering those slices and their tasks.

This is one user operation, not a promise that the model must produce everything
in one response. The plugin may use several internal model turns, then validate
the complete bundle before publishing a candidate. All documents record the
same context/generation identity and agree on requirement, slice and task IDs.

Use the target project's artifact conventions rather than imposing a new ticket
or directory taxonomy. A tracker is a single human-readable artifact, not a
directory of tickets. Its exact table syntax remains a design choice.

Generate captures the current in-memory material without an earlier mandatory
submit/approve action for each source buffer. Incomplete context can still produce
a draft with visible open decisions. Execution readiness is checked at Start.

Generated candidates remain editable. A changed dump, question, answer, decision
or manual document edit marks the affected derived material out of date. The
plugin shows what produced the current document and which sources have changed;
it does not block further drafting. Edits alone do not trigger hidden model calls.
An explicit regenerate action creates a new candidate from the latest context.
User changes are retained, compared or explicitly replaced, never silently lost.

If context changes while generation is running, its eventual result retains the
context revision it used. The plugin cannot present that result as current or
overwrite newer user edits merely because the model finished later.

Rewriting a question preserves the earlier wording and answer relationship.
The answer text remains available; a material wording change can mark it for
review instead of silently treating it as an answer to a different question.

For example, changing an answer from "use one worker" to "allow parallel workers"
invalidates a generated execution plan that assumed serialization. Repainting the
specification view cannot make that old plan current again.

## Execution and evidence

Start is the contract-freezing boundary. It captures an exact, checked bundle
and root, including the relevant source context, questions/answers, specification,
slice definitions, plan, initial tracker and execution rules. The retained
contents and their digests govern the run; mutable file paths alone are not a
snapshot. No earlier Generate action makes those drafting buffers immutable.

Within the package, the run's contract views become read-only and regeneration
cannot rewrite them. Preparing another proposal requires an explicitly separate
draft revision. The controller retains the original run identity and contract
while it works.

This is package-level immutability, not a promise to protect files from their
owner. A user can still edit them outside the package. The plugin records source
paths/stamps/digests and checks relevant observations before reuse or publication.
It reports divergence and preserves the retained snapshot. Workers must receive
the frozen contract text, or resolve contract reads against its retained version;
reading a changed original file cannot silently redefine the run.

An outside change that conflicts with an intended write or integration requires
reconciliation before that action proceeds. The choice to keep the run, create
a new admitted run or abandon the change must be explicit. Do not claim a
filesystem lock, overwrite the user's edit or automatically adopt the new text.

The plugin owns slice ordering, worker invocation, checkpointing, validation,
integration and progress reconciliation. Pi supplies model/agent execution and
its own provider authentication/context facilities. The editor core owns buffers,
display, input and the general extension boundary.

Start acknowledges admission with a run ID and returns promptly. The package
process continues the finite admitted job and updates progress through later API
calls. It must not keep an editor RPC command open for the entire loop. Status,
pause, resume and cancel are separate bounded commands addressed to that run.

The controller is part of the package's implementation. The existing global
Looper is a behavior reference, not an assumed installed dependency or a run to
attach to. Code sharing or a reusable controller library can be evaluated later.

Progress changes must follow observed evidence: a model saying "done" is not a
passing validation, accepted commit or integrated slice. Required checks and
their candidate identities remain explicit. Repository operations, including any
automatic integration, need authority in the admitted workflow contract.

The tracker is the single human task/progress document. Technical checkpoints,
process identities and validation receipts can exist separately so the run can
recover safely. They are operational evidence, not a replacement ticket system.
The tracker and status view must be reconciled from those observations. If their
publication fails, report the disagreement rather than inventing completion.

The frozen bundle includes the original tracker definition. Live progress is a
projection of that fixed task structure plus execution evidence: statuses and
results can change without revising the admitted work. Updating the one human
tracker does not modify the retained initial tracker snapshot. A plugin-authored
progress write has its own expected source observation, so it can distinguish
its next update from an outside manual change.

Pause stops new dispatch and settles the current owned operation at a recorded
boundary. Resume reconciles ownership, candidate, work and validation observations
before starting another owner. Cancel withdraws future work and stops the owned
operation through bounded cleanup; accepted changes remain visible. Uncertain
cleanup cannot be reported as safely paused or canceled.

Hiding a view does not stop its job. Package deactivation or editor exit follows
PLG10: retire the generation, stop/reconcile owned descendants and retain safe
checkpoint observations. Checkpoints may survive; an independent background
service is not implied. Reactivation reconciles the retained run before launching
another worker. Automatic retry authority, integration permissions and numerical
budgets must be defined by the admitted execution profile, not inferred from the
current global Looper configuration.

## Pi reuse question

The existing native adapter provides useful evidence for RPC framing, process
ownership, cancellation, provider/model identity and session binding. Its current
AI editing contract uses restricted discovery and checked result application.
That contract does not authorize a workflow to run builds, tests or Git writes.

The candidate first experiment gives the plugin its own Pi session and explicit
working context. It can reuse verified protocol knowledge and, if an appropriate
public client can be extracted, transport code. It must not borrow the native
editing command's private session or broaden its tool grant.

| Alternative | Potential benefit | Question to resolve |
| --- | --- | --- |
| Plugin-owned Pi client/session | Workflow state and execution authority remain in the plugin. | Who owns nested child cleanup, resumable sessions and the shared client contract? |
| Native reusable Pi service API | Centralized transport and lifecycle code. | Can a general service support separate execution grants without exposing private editing state or turning rev into the controller? |

No choice of library or code extraction is approved here. Authentication remains
Pi-owned; the workflow does not copy subscription credentials or ask for native
provider API keys merely because Pi is used.

## Host compatibility contract

The target is PLG01-PLG12 in Plugin System draft 0.1 and its model. The names
below are proposal methods, not current binary interfaces. Activation requires
an exact negotiated wire revision and capabilities; sharing a repository, SDK
or compiler language does not establish compatibility.

| Workflow need | Public mapping | Conformance obligation |
| --- | --- | --- |
| Read live context | buffers.list, buffers.describe, buffers.read | Use explicit session/IDs/revisions and source scalar units, including selected unsaved text. PLG04-05. |
| Create/show related material | buffers.create, buffers.show | Buffer identity remains independent of pane placement. PLG04. |
| Publish an in-memory document bundle | edits.apply | Prepare every target and publish one checked batch; reject a partial/stale batch. PLG05, PLG06, PLG07. |
| Expose workflow commands | commands.register, commands.list | Package-qualified identities, descriptions and ordinary M-x/modal discovery. PLG02-03. |
| Bind local actions | keymaps.bind, keymaps.unbind | Native precedence/context/cancellation; no direct terminal key interception. PLG03. |
| Observe context changes | events.subscribe, events.unsubscribe, buffers.describe/read | Handle revisions, invalidation and queue overflow; events are not durable complete history. PLG05, PLG08. |
| Save project artifacts | buffers.save | Native conflict rules and checked source observations; revision-bound save details must be resolved publicly before execution. PLG05, PLG07. |
| Store settings and checkpoints | Host-declared package storage mechanism | Preserve configuration/data/state/cache concerns and package namespaces; physical access remains open in the parent model. PLG12. |
| Continue asynchronous work | Package generation and bounded invocations | The job belongs to the package; commands return promptly and finite limits remain enforced. PLG09. |
| Retire execution ownership | Host generation cancellation/disconnection | Stop/reconcile owned descendants, remove contributions and retain user text. PLG09-10. |
| Admit compatibility | initialize and package API declaration | Reject unsupported revisions and missing required capabilities. PLG01, PLG11. |

This package must not expand the parent API implicitly. Missing mandatory
mechanisms are compatibility gaps, not permission to import private host modules.
Generic API additions need their own review. The exact parent wire revision and
finite host profile must be recorded before an execution experiment starts.

### Text baseline and optional presentation

The baseline uses ordinary editable buffers for context, questionnaire, documents
and progress, with local commands. Native form widgets are not required. Progress
tables update through edits.apply with expected revisions; user edits remain
protected by the same conflict rules as other text.

At Start, supported initial read-only attributes can create separate snapshot
buffers for the run contract. Its retained contents belong to package state, not
the mutable original path. If the host cannot create the required views publicly,
keep Start unavailable rather than mutate a private read-only flag.

Forms, semantic decorations, actionable list widgets and live refresh of read-only
generated views are optional improvements. Their absence does not disable the
text baseline. No raw terminal drawing or private renderer is required.

### Artifact publication and admission

Atomic in-memory publication is distinct from writing several files. The parent
API proposes individual saves, not an atomic multi-file filesystem transaction.
The candidate remains unadmitted until every required saved artifact matches the
checked bundle. Start then retains its exact frozen contract.

Keep a publication record with expected source observations and each successful
write. A failed/conflicting save starts no worker. Recovery reconciles completed
writes without overwriting newer user changes. The crash-safe record and public
save preconditions must be resolved and tested before execution is enabled.

### Capability modes

- Authoring mode uses the public text API. Ordinary editing remains available
  without Pi. AI questions/generation additionally require an owned Pi connection,
  explicit context and bounded tool/process behavior.
- Execution mode additionally requires durable package state, retained contents,
  safe artifact publication, a finite execution profile and proven child cleanup.
  Start remains unavailable while these contracts are missing or unresolved.

These are supported package capabilities, not irreversible workflow stages. All
authoring material stays revisable until Start admits a run.

The package declaration must distinguish its mandatory authoring minimum from
conditional execution features. An incompatible wire revision or absent mandatory
minimum rejects activation under PLG11. Missing conditional features disable the
affected action; they cannot justify calling an unadvertised method.

## Package-facing commands

Illustrative identities use field.workflow. No default shortcut or final package
identity is selected; descriptions and effective key hints use the host registry.

| Command | Behavior |
| --- | --- |
| field.workflow.open | Create/reopen the related text buffers and select an explicit target root. |
| field.workflow.questions | Propose/refine questions against current material while preserving edits. |
| field.workflow.generate | Generate/regenerate the coherent candidate from current in-memory context. |
| field.workflow.start | Check and retain the contract, admit one run and return its ID. |
| field.workflow.status | Show context freshness or observed run progress and diagnostics. |
| field.workflow.pause | Stop dispatch and settle the owned current operation. |
| field.workflow.resume | Reconcile a checkpoint and continue the same admitted run. |
| field.workflow.cancel | Withdraw future work and stop the owned operation without implicit rollback. |

The root comes from explicit invocation arguments or editable package context.
A native directory/project shortcut is optional when public invocation context
provides it. Prefix arguments, macro recording and binding scope follow the
parent command contract, without introducing a second dispatch system.

These dotted command IDs rely on the parent's unresolved command-admission and
macro-call compatibility contract. They are illustrative package identities,
not names that the current native macro library or command registry accepts.

## Requirements

These requirements define the proposed package behavior and compatibility target.
They remain Draft pending review and experimental evidence.

### Requirement: WFP01 Preserve context across workflow views

The plugin SHALL keep dump, question, answer and document buffers editable in
any order before Start. Their revisions and dependencies SHALL remain independent
of focus, layout and the last model operation.

#### Scenario: Remember another constraint before specification generation
- **GIVEN** a questionnaire already exists and no specification has been generated
- **WHEN** the user adds a remembered constraint directly to the dump
- **THEN** that change becomes part of the next captured generation context
- **AND** the user does not restart a wizard or resubmit the entire conversation.

#### Scenario: Improve an earlier question
- **GIVEN** the user has answered a question and continued drafting
- **WHEN** they rewrite that earlier question or its answer directly in the buffer
- **THEN** the changed material remains available to later generation
- **AND** affected answers or derived documents expose their freshness without locking drafting.

#### Scenario: Return to the dump
- **GIVEN** the user answered questions and is reviewing a generated specification
- **WHEN** they return to the dump buffer and revise a constraint
- **THEN** their earlier answers remain available
- **AND** dependent generated candidates are marked stale.

### Requirement: WFP02 Generate one consistent artifact bundle

One user action SHALL generate and validate a candidate specification, slicing,
plan and single-file tracker from the same captured current context. Generation
SHALL NOT freeze its inputs or imply permission to execute.

#### Scenario: Generated slice identities disagree
- **GIVEN** the generated tracker names a slice absent from the plan
- **WHEN** the plugin checks the generated bundle
- **THEN** it retains the candidate for correction
- **AND** does not present it as ready to execute.

### Requirement: WFP03 Preserve edits during regeneration

Regeneration SHALL retain user changes or obtain an explicit replacement decision.
It SHALL not overwrite an independently modified specification or plan silently.

#### Scenario: User edited a generated specification
- **GIVEN** a generated document has subsequent user edits
- **WHEN** new questionnaire answers trigger an explicit regeneration request
- **THEN** the new candidate is distinguishable from the edited document
- **AND** the existing edits remain recoverable.

### Requirement: WFP04 Bind execution to the admitted candidate

The controller SHALL execute an explicitly admitted bundle and SHALL not silently
replace its scope when source files change. Start SHALL retain an immutable
contract snapshot and make that run's contract views read-only inside the package.

#### Scenario: Start fixes the reviewed contract
- **GIVEN** the user freely revised the working material and the bundle is current
- **WHEN** they invoke Start and admission succeeds
- **THEN** the package retains the exact input contents and contract identities
- **AND** package editing/regeneration cannot replace that run's contract.

#### Scenario: Draft changes during a run
- **GIVEN** a run is implementing an admitted candidate
- **WHEN** the user explicitly opens a separate draft and revises its context
- **THEN** the active run retains its original candidate identity
- **AND** using the new scope requires a defined transition and fresh admission.

### Requirement: WFP05 Derive completion from execution evidence

Slice/task completion SHALL follow observed validation and integration outcomes.
The progress view and tracker SHALL identify unavailable or unreconciled evidence.

#### Scenario: Worker reports success but validation fails
- **GIVEN** a worker claims the slice is complete
- **WHEN** the configured candidate validation fails
- **THEN** the slice remains incomplete
- **AND** the status view exposes the failed check and retained candidate.

### Requirement: WFP06 Keep Pi workflow authority independent

A workflow Pi session SHALL have its own identity, context and execution grant.
It SHALL not widen the existing native AI editing session's authority implicitly.

#### Scenario: Native explanation and workflow coexist
- **GIVEN** native explanation and a workflow session use Pi
- **WHEN** the workflow needs to run a build in its admitted project
- **THEN** that permission belongs to the workflow execution path
- **AND** the native explanation command retains its original editing contract.

### Requirement: WFP07 Retain jobs independently of presentation

The plugin SHALL distinguish closing a view, pausing a job, canceling an invocation
and resuming a checkpoint. Failed or uncertain cleanup SHALL remain observable.

#### Scenario: Close the execution view
- **GIVEN** a job has an owned implementation worker
- **WHEN** its progress buffer is hidden or its pane is closed
- **THEN** the job follows its admitted lifetime policy
- **AND** the user can reopen its status without starting a duplicate worker.

### Requirement: WFP08 Remain an ordinary plugin

The package SHALL exercise documented extension capabilities and identify
missing surfaces explicitly. The editor core SHALL not own workflow-specific
specification generation, slicing or task-state decisions.

#### Scenario: Status needs a capability absent from the API
- **GIVEN** the package needs actionable progress rows
- **WHEN** the current API only supports ordinary text buffers
- **THEN** the experiment either uses documented text interaction or records an API gap
- **AND** does not claim a private core callback is a public extension feature.

### Requirement: WFP09 Observe outside changes without adopting them

The plugin SHALL distinguish the retained run contract, its mutable source files
and plugin-authored progress updates. Observed external changes SHALL not revise
the running contract or justify overwriting the user's file.

#### Scenario: Specification is edited outside the package after Start
- **GIVEN** a run uses a retained specification snapshot
- **WHEN** another editor changes the original specification file
- **THEN** the package reports the source divergence and retains the run snapshot
- **AND** it reconciles a conflicting publication or integration before proceeding.

#### Scenario: Tracker progress advances normally
- **GIVEN** Start retained the original task structure and tracker snapshot
- **WHEN** evidence confirms that a task completed
- **THEN** the live progress projection and human tracker can reflect that outcome
- **AND** the frozen task definition and execution contract remain unchanged.

### Requirement: WFP10 Negotiate public host capabilities

The package SHALL declare its required API revision and SHALL use only admitted
public methods. Missing optional presentation SHALL retain the text baseline;
missing execution prerequisites SHALL keep Start unavailable.

#### Scenario: Host cannot provide durable package state
- **GIVEN** text buffers and commands are available but durable state is unsupported
- **WHEN** the package activates
- **THEN** it exposes supported authoring with a specific compatibility diagnostic
- **AND** it starts no worker through a private host interface.

### Requirement: WFP11 Admit only complete artifact publication

The package SHALL distinguish atomic buffer edits from individually saved files.
It SHALL reconcile all required writes before admitting execution.

#### Scenario: The third artifact cannot be saved
- **GIVEN** two saves of a four-artifact candidate succeeded
- **WHEN** another save fails or its expected source observation no longer matches
- **THEN** the candidate remains unadmitted and no implementation worker starts
- **AND** recovery retains the observed writes and preserves conflicting user changes.

### Requirement: WFP12 Honor package generation retirement

The controller SHALL obey the parent lifecycle/cancellation contract. Hiding a
view SHALL differ from deactivation or host exit. Retired-generation replies
SHALL not resume or publish new work.

#### Scenario: Package deactivation during implementation
- **GIVEN** an admitted run owns an active worker
- **WHEN** the user deactivates the package
- **THEN** new work stops and owned processes enter bounded settlement
- **AND** the checkpoint distinguishes safe stop from uncertain cleanup
- **AND** later activation reconciles ownership before starting another worker.

### Requirement: WFP13 Remain independently implementable and project-scoped

The package SHALL be implementable outside the host repository through the public
plugin contract. Its target project SHALL remain distinct from package source,
editor source, buffer focus and launch directory.

#### Scenario: Package and project are separate repositories
- **GIVEN** the package was built in another repository
- **WHEN** the user selects a target project and generates its documents
- **THEN** public operations address that project and explicitly identified buffers
- **AND** private editor imports or an incidental working directory do not determine the target.

## Candidate evaluation

First edit a dump after questions exist, rewrite a previous question/answer and
revise a generated specification; verify that the package preserves these live
changes and explains freshness without a one-way stage gate. Exercise a generation
that completes after its context changed. Then test Start freezing the contract,
normal progress updates and conflicting edits made outside the package.
Only after a reviewed execution contract exists should a disposable project
exercise one simple slice through worker, validation, checkpoint and completion.
These are proposed experiments, not a delivery plan or a request to start them.

Exercise the conformance boundary too: missing optional UI, absent required
storage/save semantics, overlapping short command names, deactivation during work
and restart with uncertain old ownership. Build the package separately and use
the public protocol in acceptance fixtures. Record host API revision, capability
profile and package revision with every runtime result.

A later rich dashboard can use the same state. Its appearance must not become a
prerequisite for proving the workflow or the extension API.

No prototype, generated project, running controller or passing runtime evidence
is recorded by this document.

## Open decisions

- Questionnaire interaction and the smallest useful initial view layout.
- Target-project artifact format adapters and how project rules enter context.
- Regeneration, comparison and preservation of manual document edits.
- Retained snapshot storage and reconciliation choices for outside modifications.
- Pi ownership, public transport reuse and exact execution/resource permissions.
- Controller implementation, validation/integration protocol and recovery policy.
- Public storage, save-precondition and descendant-cleanup mechanisms required
  by the execution profile; deactivation itself follows PLG10.
- Optional UI enhancements that preserve the ordinary text baseline.

## Planning readiness

This formal draft and its model can be carried into an independent package
repository with an explicit reference to the host contract. Execution delivery
still depends on resolving the relevant parent API contracts and recording the
finite execution profile. No stable compatibility or passing prototype is implied
by structural document checks.
