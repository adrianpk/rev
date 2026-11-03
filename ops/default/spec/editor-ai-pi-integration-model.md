# Pi Integration Model

Status: Completed
Completed: 2026-11-03
Date: 2026-10-28
Reviewed: 2026-11-02
Version: 0.2
Approved: product owner, 2026-11-02; paired first-Pi delivery approval
Specification: [Pi Integration](editor-ai-pi-integration.md)
Parent model: [AI Commands Model](editor-ai-commands-model.md)
Related observations: [External File Changes Model](editor-external-file-changes-model.md)

## Ownership

Pi is the first delivered execution path. Its adapter owns the external child
and RPC streams; Pi owns provider calls, credentials, discovery, history and
compaction. Native API transport/context construction remains a later path,
with no prerequisite implementation or implicit fallback in this delivery.

rev owns invocation identity, editor snapshots, read/transmit grants, targets,
presentation and canonical result application. A working directory is context,
not a substitute for checking effective tools, source paths and resources.

## Configuration delta

Use the existing user TOML root and include precedence. Delivered fields are
under `ai.pi` and match the packaged defaults and configuration reference.
Defaults permit lazy explicit AI commands without starting Pi during editing.

| Field | Type and validation | Default |
| --- | --- | --- |
| `command` | Nonempty executable name/path, at most 4096 bytes; no shell expression. | `pi` |
| `model` | Optional Pi model reference, at most 256 bytes. | Unset; use Pi's resolved model. |
| `persist_sessions` | Boolean; directory contexts only. | `true` |
| `idle_timeout_ms` | Integer, 0..=3600000; zero closes after settling. | 900000 |
| `request_timeout_ms` | Integer, 1000..=1800000; generation deadline, followed only by bounded cleanup. | 300000 |
| `max_input_bytes` | Integer, 1..=4194304; editor snapshot/template envelope. | 1048576 |
| `max_result_bytes` | Integer, 1..=1048576; admitted generated text. | 262144 |

No endpoint, native API token, arbitrary launch arguments or secret environment
map is introduced. Pi resolves its existing authentication files/environment.
The adapter supplies the admitted arguments/resource policy, not a model or
prompt. A command path remains a trusted external program, not a sandbox.

An ordinary example needs only the desired override:

```toml
[ai.pi]
command = "pi"
persist_sessions = true
```

New defaults, validation, packaged sample and reference must stay consistent.
Configuration is startup-based. A scoped save of an AI definition can activate
that definition without reloading unrelated settings or the interaction profile.

## Declarative command records

Use `ai.commands.<name>` in the same TOML graph. A name is 1..=64 lowercase
ASCII letters/digits/hyphens, starting with a letter. Its callable identity is
`ai-<name>`. It cannot collide with a compiled command, alias or another
definition. Admit at most 64 definitions.

| Field | Admitted representation | Contract |
| --- | --- | --- |
| `prompt` | Nonempty literal string, at most 16384 UTF-8 bytes. | Instruction, never executable code or shell substitution. |
| `input` | `selection`, `point` or `buffer`. | Defines the native target captured on each invocation. |
| `context` | `input` or `directory`. | Separate discovery/history grant, not a different write target. |
| `output` | `explanation`, `insert` or `replace`. | Explanation is read-only; insert requires point, replace requires selection or explicit buffer input. |
| `model` | Optional bounded Pi model reference. | Resolves within Pi and crosses the matching model/session isolation boundary. |

All four main fields are required. The initial template language is literal:
editor observations are separate envelope data, not string interpolation.
Bindings use the existing profile-appropriate keymap table; no duplicate
keybinding property or new AI prefix is needed. Resolve definitions before
validating keymap references. Newly saved names can be called immediately;
new persisted keybindings follow the existing restart policy.

`ai-save-command` writes only the chosen definition with the existing
format-preserving, conflict-checked configuration publisher. Concurrent source
changes reject the write. Do not flatten includes/defaults or overwrite an
invalid configuration graph. Running an ad hoc instruction does not save a
command definition; configured Pi session history may still retain that request.

## Persistent session bindings

Pi owns its JSONL history. rev does not implement a second transcript or edit
Pi's session file internals. Use the resolved XDG state root:

| Record | Location/fields in contract order | Invariants |
| --- | --- | --- |
| Binding index | `ai/pi/sessions.toml`: schema 1, binding ID, canonical root, configuration/grant fingerprint, resolved model identity, relative Pi session path, created/last-used timestamps. | Unique binding key and ID; paths stay in the owned session directory. |
| Pi session | `ai/pi/sessions/`, actual Pi-generated JSONL. | Private history, reused only under matching directory/model/grant ownership. Never the user's arbitrary last Pi session. |
| Pending job/result | Memory only. | Not resumed as permission to publish an old edit after restart. |

The binding key includes canonical root, directory context, resolved model/
provider and effective launch/resource policy. A narrower grant cannot reuse
a broader history. Input-only operations always use a fresh `--no-session`
child, regardless of the directory-session persistence setting.

One lazy child is active at a time. A directory/model/grant change settles or
cancels its current operation, then selects the proper isolated history. The
idle timeout stops the child; persisted history remains eligible for explicit
future invocations. Editor exit stops the owned child and retains only the
configured history. No standalone service is installed.

Admit at most 16 retained bindings, 64 KiB of index data and 32 MiB per Pi
session. At a limit, report that history must be reset/removed or persistence
disabled; do not silently truncate JSONL or delete unrelated sessions.
`ai-reset-session` explicitly starts fresh owned history. Unknown index schema,
invalid links or ownership mismatch preserves files and blocks unsafe reuse;
offer a fresh ephemeral operation rather than guessing another session.

## Ephemeral records and messages

| Record | Fields in contract order | Lifetime and authority |
| --- | --- | --- |
| Invocation | Native ID/epoch, command definition, input/output/context policy, root, resolved Pi settings, target snapshot, deadline and accounting. | One explicit operation; at most one active invocation. |
| Target snapshot | Buffer ID/stamp, visiting path or unnamed identity, point/range, complete admitted text/BOM and saved/dirty state. | Native source evidence, never a model-selected target. |
| Dirty-source map | At most 16 admitted buffer snapshots within the root, path/revision and completeness. | Counts toward max input bytes; exposed reads prefer these snapshots over stale disk. |
| Child | Owned process identity/generation, executable/version observation, cwd, effective tool/resource policy and streams. | Not an unrelated interactive Pi or its session. |
| RPC request | Unique string ID, admitted command and bounded payload, response and settled state. | Typed commands only; model data cannot dispatch RPC. |
| Discovery observation | Tool kind/path, checked root, source provenance, bounded returned content and accounted size. | Data only; no shell/write/editor-command authority. |
| Result | Invocation/epoch, original target, settled outcome, version-1 text payload and provenance. | Proposal until explicit native application; read-only explanation has no publication. |

Encode editor metadata and snapshots as structured data inside the RPC
`prompt.message` string with explicit instruction/data boundaries. Do not
invent unsupported Pi RPC fields or send `@file` arguments in RPC mode.
No whole-directory dump or native relevance/discovery loop is required:
Pi decides what admitted sources to inspect.

The model-result object has exactly `version = 1` and a UTF-8 `text` string.
Do not accept extra model-supplied target paths, command dispatch or operations.
Record parsing and generated-result parsing are separate contracts. Invalid
output can be inspected without being published.

## Protocol lifecycle and finite work

Capture -> admit -> prompt accepted -> streaming/discovery -> settled ->
validated result -> explicit accept/discard -> native publication or terminal
failure. Result acceptance rechecks the original target; a disk reload or local
edit fences an older proposal. Publication occurs at most once as one undo step.

Use LF as the only stream delimiter and bounded byte-oriented readers. A record
is at most 2 MiB; queued protocol data is at most 4 MiB and stderr retains only
a 64 KiB tail. Both encoded-record and logical-text limits apply. Read stdout
while writing; never block input/rendering on a pipe.

Admit at most 8 model turns, 32 discovery tool calls and 4 MiB of returned
discovery text per invocation. A discovered regular file is at most 1 MiB.
Check these bounds before admitted tool IO where possible and abort on excess;
Pi's own compaction does not reset native deadlines/accounting. No hidden
follow-up command queue is admitted.

C-g or an owning modal Escape sends `clear_queue`, then `abort`. Await the
admitted response/idle outcome for at most 5 seconds, then close the owned child
with a 2-second stop grace and bounded forced reap. Late records are fenced by
child generation and invocation epoch. Ordinary editor cancellation remains
owned by its current input component.

## Compatibility and preparation evidence

Pin Pi 1.3.0 for the first real acceptance. Compatible 1.3.x patches need the
required RPC/event/tool checks; newer minor/major versions remain unsupported
until reviewed. Missing executables/authentication, rejected tools or invalid
history are actionable unavailable states, not automatic native API fallback.

Planning must prove the guarded tool/root profile, native snapshot precedence,
private history persistence and actual process shutdown. No watcher or AI
dependency is selected here. Measure cold/warm startup, idle CPU and RSS; no
performance number is implied by this model. Real provider checks require
authorized access; fixtures prove only their stated protocol boundaries.

## Delivery evidence

PI01-PI08 completed through canonical PRs #229-#236 in the
[Pi/reload delivery](../plan/rev-ai-pi.md). The unchanged controller full gate
passed at `ecd02abcbd1f08530d8fad1395672dc6d00ee085`, receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, on 2026-11-03. The
[Slice 8 closure](../report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
records exact commands, 578 selected top-level tests, both-profile native/PTY
acceptance and separate real-provider evidence. Existing requirement, scenario,
configuration and resource contracts remain intact.

The real provider observation used Pi-owned OAuth and the existing configured
model. Its original failed attempt and one explicitly authorized successful
continuation remain recorded. API-key variants were not exercised live. The
broader native API specification/model remain draft; direct transport and native
context construction stay deferred.
