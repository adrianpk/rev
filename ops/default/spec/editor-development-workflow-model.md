# Workflow Context, Artifacts and Execution

Status: Draft
Date: 2026-11-08
Version: 0.3
Specification: [Development Workflow Plugin](editor-development-workflow.md)
Parent model: [Plugin Identity, Protocol and State](editor-plugin-system-model.md)
Decision state: formal domain/conformance model; physical codecs and storage access remain open
Host contract: Plugin System draft 0.1; independently built package using its public interface

## Package and host boundary

The package may be built and maintained in another repository. Its host adapter
uses public methods, session handles, source units and error/lifecycle rules from
the parent model. It does not import private native types or retain editor pointers.
The implementation language and SDK choice do not change the authority boundary.

The proposed wire-version string in the parent manifest remains illustrative.
Select an exact API revision and finite capability profile before a prototype
executes, and record them with acceptance evidence. Compatibility is negotiated
behavior, not the fact that two documents link to one another.

## Domain records

| Record | Candidate fields | Relationship and invariant |
| --- | --- | --- |
| Workflow | Stable ID, explicit project/root identity, current drafting state, view bindings | Package domain object; not a new native editor workspace type. |
| Context revision | Revision ID, dump/question/answer snapshots, manual document edits, explicit decisions, selected observations and provenance | Captured input for one generation; snapshot history does not make the live drafting buffers read-only. |
| Question revision | Stable question ID, revision, text, reason, context revision, answer state | Wording is editable before Start; earlier answers remain related to the question revision they addressed. |
| Answer revision | Question ID/revision, answer, source buffer stamp, context revision | Editing the answer creates new context; earlier wording and evidence remain distinguishable. |
| Generation | Generation ID, context revision, Pi invocation/session observation, state, diagnostics | One user generation action may contain multiple internal model turns. |
| Artifact candidate | Generation ID, kind, source text, content digest, document revision, user-edit observation | Kinds include specification, optional model, slices, plan and one tracker. |
| Candidate bundle | Bundle ID/digest, context revision, complete artifact set, shared requirement/slice/task identities, consistency result | Candidate publication does not imply execution approval. |
| Frozen run bundle | Retained source context and artifact contents, per-artifact digests, original source observations, initial tracker, exact execution rules | Created at Start; not replaced by later disk contents, progress updates or another draft. |
| Admission | Frozen bundle identity/digest, explicit project/root, execution/validation/integration contract, Start action | Binds one run to exact scope; not inherited by a regenerated bundle. |
| Run | Run ID, admission, active slice/task, observed worker/validation state, checkpoint reference | Plugin-owned controller domain; one authoritative owner at an admitted boundary. |
| Host capability profile | Exact API revision, generation, public methods, units, limits, storage/save/lifecycle support, diagnostics | Determines supported authoring/execution behavior; not inferred from access to editor source. |
| Artifact publication | Bundle/context identity, explicit target paths/handles, expected observations, per-artifact write outcomes, reconciliation state | Individual saves are recoverable steps, not an atomic filesystem transaction; incomplete publication cannot admit a run. |
| Execution profile | Exact project/root, Pi/worker grants, checks/integration rules, finite budgets/deadlines, ownership/settlement policy | Required before Start; RPC command deadlines are separate from the admitted package job's lifetime. |
| Execution observation | Run/candidate identity, operation identity, outcome, evidence reference, reconciliation state | Determines progress; model prose is not a successful validation receipt. |
| View binding | Workflow ID, role, draft/run/progress presentation, native handles, last rendered domain revision | Drafts are editable before Start; run contract views are read-only; progress is derived separately. |
| Source observation | Artifact identity, source path, native stamp when present, last known disk digest, last plugin-write observation | Detects divergence; neither a path nor a filesystem notification alone is the frozen contract. |

These are conceptual fields, not an instruction to add a database, one file per
record or a durable event framework. Choose the smallest storage that supports
the admitted recovery behavior after testing the model.

## State and freshness

Keep drafting state, document freshness and execution state independent.

- Authoring remains an editable draft. Capturing, questioning, generating and
  reviewing are revisitable activities, not mandatory one-way stage states.
- A bundle may be current, stale, invalid or admitted relative to one context.
- A run may be preparing, running, paused, waiting, blocked, canceling, completed
  or failed under a future explicit controller contract.

These are candidate terms, not final serialized enum values. Before Start, edits
update dependency observations immediately; Generate captures the current text
without a separate mandatory submission for every buffer. Out-of-date derived
documents remain editable and inspectable. Generated output retains the source
revision it actually used even if that source changed while the model ran.

Start creates the immutable run bundle and freezes its package-managed contract
views. An explicitly separate draft can prepare later work while the run remains
on its original bundle. A view being visible or hidden is not a run transition.

## Artifact and checkpoint ownership

| Concern | Candidate owner and placement |
| --- | --- |
| Project specification/model | Target project's normal operational specification area. |
| Slice definitions and plan | Project planning convention; one shared ordered identity map. |
| Human progress tracker | One Markdown file using the project's accepted tracker format. |
| Workflow settings | Namespaced package settings in the ordinary configuration concern. |
| Context revisions and recovery checkpoint | Package-scoped mutable state; exact format and project qualification remain open. |
| Frozen run contract | Retained exact source/artifact contents and digests, including the initial tracker; storage is open but paths alone are insufficient. |
| Pi transcript | Pi-owned session artifact bound to this workflow/session grant. |
| Validation and integration evidence | Controller-owned observations and references with candidate identities. |
| Disposable derived output | Package cache or bounded transient buffer, according to recoverability needs. |

Use the host's declared package storage mechanism or an explicitly admitted
package storage location. The parent model leaves physical access unresolved;
execution must not proceed by inventing an SDK method or opening private host
state to bypass that gap. Keep settings, content, mutable state and caches separate.

The single-file tracker requirement does not imply that process identity,
credential handling or crash recovery must be encoded as editable tracker rows.
The tracker is the one human task document; technical checkpoint/evidence storage
must reconcile its observations into that document without establishing a rival
ticket registry.

Retain the initial tracker in the immutable run bundle. The one live human
tracker projects its fixed task definitions plus observed progress. Status writes
do not replace the retained original or authorize changed tasks. Compare the
expected observation before a package write so a manual outside change remains
a visible conflict rather than being overwritten as an ordinary progress update.

Generated editor buffers are not automatically durable project artifacts.
Publishing them requires explicit path resolution and public native save/conflict
rules. Track individual write outcomes and retain the candidate as unadmitted
until all required contents match the checked bundle. Partial failure may leave
generated documents on disk; it cannot start implementation. Recovery preserves
later user changes. Crash-safe records and revision-bound save behavior remain
execution prerequisites to resolve.

## Draft relationships and Start

Before Start, dump text, question wording, answers and generated documents are
live inputs. Editing one changes its revision and invalidates dependent generation
as needed. A question may be rewritten after an answer exists; retain the prior
question/answer relationship and mark a materially affected answer for review.
Do not silently reinterpret it or discard its text.

Generate captures one consistent context observation and produces a candidate.
The user may edit or regenerate again. There is no transition that locks earlier
buffers merely because a later activity happened. An asynchronous result cannot
overwrite a newer working revision or claim to represent unseen source edits.

Start checks currentness/consistency and retains the exact run bundle. That
admission is the boundary between live authoring and a fixed execution contract.
The controller and workers use the retained contents; subsequent contract reads
cannot silently resolve to a changed original file.

Use supported initial read-only attributes to create separate frozen snapshot
views if needed. Do not mutate private buffer flags. An ordinary progress buffer
can update through revision-checked edits; richer widgets and continuously
refreshed read-only output are optional capabilities rather than hidden dependencies.

Status/tracker progress then follows evidence over the frozen task definitions.
An explicitly separate next draft does not mutate the running contract. An
unfinished operation must be reconciled before a restart or another owner can
repeat it. Cancellation does not imply rollback of accepted source changes.

## External modification observations

Package-level read-only views do not prevent external tools or another editor
path from changing the original files. Preserve source observations and compare
relevant stamps/digests before using a source or publishing over it. An observed
change after Start creates a divergence record; it is not a new run admission.

Keep the frozen contract and the outside edit separately available. A conflicting
write or integration waits for explicit reconciliation. The final continue,
new-run or abandonment policy remains open. File permissions and operating-system
immutability are not implied by these records.

## Pi session identity

A candidate session binding needs workflow/project identity, model/provider
observation, admitted tool/resource grant and the current plugin generation.
The actual transcript and credential method remain Pi-owned.

Generation and implementation may require different grants. Reusing a process
or transcript between them is not automatic: inspect context exposure, authority,
state freshness and cancellation ownership first. A planning request must not
silently inherit a previously broader implementation session.

The package adapter can own its Pi process directly under the admitted external
process policy. A future public native Pi service is another possible adapter,
but access to the existing private native AI session is never assumed.

## Command, job and generation lifetime

Register commands inside the package namespace. Display names are presentation;
bindings and retained command references use canonical IDs. Every registration,
binding, subscription and pending callback belongs to a host generation.

Start returns an admitted run ID within the command deadline. The longer-running
job stays in the package process, with finite execution limits and later bounded
status/control requests. It does not require a workflow-specific native job type
or keep one RPC invocation open for the full implementation run.

The package correlates later Pi/worker callbacks with run ID, frozen bundle,
operation epoch and host generation. Cancellation retires that operation's
publication authority even while the package generation remains active. The host
still receives ordinary bounded API requests; it does not interpret workflow
states or decide whether a slice is complete.

The package declaration distinguishes a mandatory authoring minimum from
conditional execution capabilities. Incompatible API or missing mandatory minimum
rejects activation. Missing conditional capabilities leave supported text behavior
available and Start unavailable; they do not permit unadvertised API calls.

View closure preserves an active job. Pause settles at a safe recorded boundary.
Deactivation/host exit retire the generation under PLG10, stop/reconcile owned
Pi/worker descendants and retain resumable observations when safe. A crash can
leave uncertain cleanup; record that uncertainty and reconcile before another
owner resumes. No standalone service survives implicitly.

## Conformance record

A runtime evaluation should record package revision, host/API revision,
negotiated capability/limit profile, client/transport identity, selected fixture
project and observed outcomes. Document missing capabilities as unavailable, not
as successful behavior simulated through private callbacks. A separate package
build must reproduce the text baseline over the public interface.

## Evaluation gaps

No file schema, manifest, SDK method, execution budget, recovery policy or live
Pi workflow is implemented by these documents. The experiment should resolve
the minimum identities and transitions first, then retain observed failures as
inputs to revision. Presentation improvements can consume the same model later.
