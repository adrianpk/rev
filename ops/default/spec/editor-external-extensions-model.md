# External Extensions Model

Status: draft
Date: 2026-10-29
Reviewed: 2026-10-31
Version: 0.2
Specification: [External Extensions](editor-external-extensions.md)
Decision state: refined candidate model only; no adopted schema, storage or protocol

## Ownership

rev owns validated user declarations, native command registration, captured
editor context, invocation identity, each child/stream lifetime and checked edit
publication. Native UI owns the fixed output surfaces. The external program
owns computation and any downstream communication; an independently managed
service does not become rev-owned because a wrapper calls it.

These are semantic records, not Rust types, wire field names or database tables.
The optional draft does not modify the current TOML schema or native command
catalog.

## Candidate durable records

| Record | Fields to resolve | Contract |
| --- | --- | --- |
| Program declaration | Command name, program reference, argv, input kind, output kind, exchange mode, timeout/resource preferences and cwd/environment policy. | Central configured record and proposed invocation trust grant. Native loading/listing is passive. Names cannot replace built-ins; exact keys and resolution rules remain open. |
| Shared execution preferences | Any selected global defaults and finite input/output/diagnostic/concurrency bounds. | Part of the native configuration contract if needed; no values or additional configuration file is selected. |
| User command bindings | Admitted command identity and contextual key sequence. | Existing native binding ownership; no extension-owned keymap or interception hook. |

Command descriptions are part of the declaration, not data obtained by running
--describe. The minimal profile has no discovery-candidate record, capability
cache, endpoint registry or executable-probe result. An optional static
manifest would require a separate policy before adding such durable structure.

Text snapshots, invocation history and output are not persisted automatically.
No extension package database, session archive or new installation directory
is selected by this model.

## Ephemeral records

| Record | Fields and observations | Lifetime and authority |
| --- | --- | --- |
| Resolved declaration | Validated configuration observation, resolved program/argv and fixed input/output/exchange policy. | Resolves an explicit invocation. The declaration is not permission to reinterpret output as more program definitions. |
| Invocation | Unique identity, resolved declaration, captured target/revision, deadlines and state. | One explicit operation; no recursive editor calls or retained session. |
| Input snapshot | Buffer identity/revision, point/selection, admitted text and any permitted JSON metadata such as visiting path/language. | Captured editor state is distinct from disk. Input scope and editing target remain separate; supplying input does not save it. |
| Input grant | Declared selection, buffer, point or none policy and admitted size/metadata. | Defines what rev sends. It does not restrict the child's operating-system account permissions or permit additional context requests. |
| Owned child | Process identity, argv, stdin/stdout/stderr, cwd/environment observation, deadline and exit/cleanup state. | One child per invocation, with a defined descendant-cleanup policy. Downstream services remain outside ownership. |
| One-shot exchange | Input completion/EOF, bounded complete stdout/stderr, exit status and optional JSON schema version. | Text and structured modes are distinct; no JSON-RPC request routing, connection registry or session framing in the current profile. |
| Result proposal | Invocation/buffer/target/revision binding, fixed output kind and complete text or permitted edit batch. | Exit zero is necessary but insufficient. Native validation and atomic publication as one undo step remain authoritative. |
| Candidate list | Invocation, original buffer/revision/target, complete bounded entries and native UI selection state. | Optional JSON choose result. Revalidate at acceptance, including after the child has exited; cancellation does not edit. |
| Diagnostic observation | Bounded stderr or native exchange failure, provenance and presentation state. | Separate from document text and protocol output; retention and redaction policy remain open. |

Text mode provides selection/buffer/no input and replace/insert/show/message
outputs. A point is an insertion target independent of input text. Structured
mode can carry admitted point metadata, edit batches and choose candidates
under its future exact schema. Highlight/gutter marks are an open candidate,
not an admitted result variant.

A static show buffer becomes ordinary native read-only text. It does not retain
an extension UI owner, action routing or custom keymap. A candidate list uses
existing native interaction; it is not an automatic third completion source.

## Candidate invocation lifecycle

An explicit command captures its declaration, original buffer/target/revision,
input snapshot and budgets. Passive validation resolves the program and
arguments. Startup creates the owned child only for that invocation; no listing
or registration probe precedes it.

The job drains bounded stdout/stderr while supplying admitted input, then
closes stdin when input is complete. This work does not block native input.
Text results require complete valid UTF-8; structured results additionally
require one compatible JSON object.
EOF and successful child exit make the result eligible for native checks, not
automatically published. Nonzero exit, malformed/incomplete output, excess
limits, stale source or cancellation prevent publication.

Editing results are checked against the original target, declared coordinate
units, revision and read-only state, then published as one native undo step.
No implicit rebasing or target redirection is assumed. For choose, native UI
owns an intermediate candidate-selection state and repeats those checks when
the item is accepted. One accepted candidate can publish once; canceling or
late callbacks cannot publish it again.

show and message use native presentation rather than document mutations.
Process cleanup remains owned after success, failure, cancellation and editor
exit. No idle child, automatic retry or persistent extension session is part
of the current profile. A separate service called by the child stays outside
its process ownership.

## Invariants and excluded model growth

- Program identity and argv are data from admitted user configuration. Native
  command collisions are rejected; output cannot register another program.
- Buffer identity/revision and explicit coordinate units govern editing.
  Display rows are not implicitly source ranges.
- Complete valid output and successful exit do not bypass native validation.
- Candidate lists retain their original target across UI delay and focus changes.
- No model record grants an extension-owned interactive view, streaming
  picker, custom layout, key interception or automatic completion source.
- Configuration as an execution grant and input minimization are not a sandbox.
- Optional persistent sessions and event subscriptions need separate adoption;
  they are not latent fields in the current one-shot model.

## Readiness

Resolve the specification's adoption, configuration, wire/coordinate, output,
candidate UI, budget and cleanup questions together. Reconcile the baseline
external-program exclusion before implementation planning. The model has no
migration, delivery order or active controller work attached to it.
