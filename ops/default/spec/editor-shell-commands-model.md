# Shell Commands Model

Status: Completed
Date: 2026-11-03
Completed: 2026-11-03
Validated candidate: `736d631947d4d379cd65dd695ff2ab00fd6c964b`
Gate validation: `5SCPQH64UEI7UZ2VFXVFZMGKWH`
Reviewed: 2026-11-03
Version: 0.1
Specification: [Shell Commands and Text Filters](editor-shell-commands.md)
Authority: selected first capability under the owner's 2026-11-03 specification and startup request

## Ownership

Native command/prompt owners capture user intent. One ephemeral invocation owns
its immutable input, original editing target, process group, output collection,
deadline and cancellation. Existing buffer/window owners alone publish edits.
Output buffers are native read-only plain text, not terminal sessions.

## Startup records

The ordinary configuration loader owns the shell table. There is no separate
configuration file, registered program catalog, executable probe or migration.

| Field | Default | Stored and effective meaning |
| --- | --- | --- |
| program | "" | Literal executable override; empty resolves startup SHELL, then /bin/sh. |
| args | ["-c"] | Declared argv prefix; append accepted command as one final argument. |
| timeout_seconds | 600 | Checked positive finite deadline, at most 600 seconds. |
| max_input_bytes | 8388608 | Input UTF-8 byte admission, at most 16777216. |
| max_output_bytes | 8388608 | Aggregate stdout/stderr admission, at most 16777216. |

Respect whole-array replacement for args and existing include/source provenance.
The sample exposes every leaf. No language- or project-local override is added.

## Ephemeral semantic records

| Record | Identity and fields | Authority |
| --- | --- | --- |
| Invocation origin | Command kind, source buffer/revision, originating view, point or linear range, visiting-file/directory observation. | Captured before opening the prompt; never inferred from whichever pane is later active. |
| Accepted command | Literal UTF-8 command string, resolved program/argv, captured cwd, startup preferences. | An explicit Return permits this one process; selected text is stdin data. |
| Input snapshot | Exact selected canonical text or no input, admitted byte length. | Does not read a stale disk copy or save the visiting buffer. |
| Native edit target | Original buffer/revision/range, insertion/replacement policy and initial writable admission. | Successful exit cannot bypass final revision, range and writable checks. |
| Owned job | Invocation generation, process-group identity, leader, streams, cancellation and absolute deadline. | One active job, no hidden queue, no retained interactive process. |
| Collected result | Separate complete stdout/stderr, aggregate byte count, EOF/exit status and completion generation. | Incomplete or failed output cannot be published as an edit. |
| Output observation | Native result-buffer identity/revision and last retained result handle. | Display only; never a new edit target or shell command. |
| Prompt history | At most 32 accepted command strings in this editor session. | No implicit persistence or replay. |

A linear region and point use existing canonical units, not display cells.
Rectangular and modal Block selections do not acquire an implicit linear form.

## Lifecycle

Capture origin, prompt, validate current target and input, resolve shell/cwd,
launch one owned job, deliver input/EOF while draining output, observe exit and
complete streams, validate and publish or report, then release process authority.
Cancellation and all error paths converge on bounded cleanup.

Background work never accesses Session through an unowned reference. Deliver
immutable, generation-bound completion observations to the native session.
A cancelled or consumed invocation cannot publish later, twice or to another view.

Normal source edits, reload, closure and read-only transitions may invalidate
the target while the child runs. Preserve valid completed output for inspection
without modifying that stale source. Output/error views and later navigation do
not extend source-edit authority.

## Publication and retention

Use existing observed replacement and undo ownership. A replacement is one edit;
empty successful stdout deletes the selected region. An empty insertion does
not create empty history. Result metadata and stderr stay out of text applied
to source.

Transfer completed text to ordinary native output buffers and release input and
collector storage. Retain no duplicate hidden archive. The last-result handle
cannot reopen a closed buffer or overwrite a buffer whose ownership changed.

## Process sharing boundary

Reuse current cancellation and Unix process-group primitives where appropriate.
Promote a genuinely shared low-level owner if required instead of making general
shell commands depend on Org configuration or granting Org new execution rights.
Existing Org adapters keep finite limits and consent; LSP and Pi keep their own
protocols and process ownership.

## Completion

The implementation and unchanged controller gate satisfy SHL01-SHL08 together.
Canonical PRs #242-#245 and repair #246 deliver the captured target, bounded job,
checked publication and retained-result lifecycle described here. The
[tracker](../tracker/rev-shell-commands.md#final-gate-and-closure) and
[Slice 4 closure](../report/slices/rev-shell-commands/slice-4-acceptance.md#delivery-set-closure)
record exact-candidate validation and native/resource/PTY observations.
The [interactive terminal model](editor-integrated-terminal-model.md)
continues to own PTY/emulation/session placement independently.
