# External Extensions

Status: draft
Date: 2026-10-29
Reviewed: 2026-10-31
Version: 0.2
Format: OpenSpec capability proposal
Source: product-owner architectural exploration and supplied extended working note
Decision state: optional exploration; the refined profile below is not an adoption or delivery commitment
Delivery boundary: optional exploration outside the lightweight editor baseline; no scheduling, slicing or implementation authorized
Model: [External Extensions Model](editor-external-extensions-model.md)
Baseline: [rev Editor](rev-editor.md), [Editor Configuration](editor-configuration.md)
Related: [Native Autocompletion](editor-autocompletion.md), [AI Commands](editor-ai-commands.md), [Pi Integration](editor-ai-pi-integration.md)

## Purpose

Explore cooperation with ordinary external programs without expanding rev into
an editor automation platform. A program could transform selected text, insert
content, return a static explanation or supply choices through an existing
native interaction. Its implementation could be compiled, interpreted or a
wrapper around another service.

The owner does not consider this capability necessary to complete the intended
lightweight editor. It remains a possible addition, without an implementation
commitment, assigned order or dependency from existing native features.

## Status and product boundary

This revision incorporates the supplied narrower exploration. The draft profile
uses explicit user configuration, stdio and one child per invocation. Simple
text filters are the smallest candidate; structured one-shot results are a
separate extension of that mechanism. These are draft design choices, not
approved changes to the running editor.

**An extension acts on admitted text buffers through fixed native surfaces.**
The program may compute arbitrary results, but its editor-facing outputs are
limited. Its implementation language or downstream architecture does not grant
new editor surfaces. Native functionality remains useful without extensions.

This boundary follows the product intent: rev is an 80/20 of Emacs, keeping
the parts used daily in a light, slightly configurable tool. Emacs's
unlimited extensibility is part of the weight rev avoids. Extensions whose
value depends on elaborate panels, interactive views or their own layout
belong in an editor willing to pay that cost; their absence is an accepted
outcome, not a gap to close later.

The baseline excludes plugins and a public extension interface. Even a native
filter command introduces external-program execution and needs an approved
baseline delta before implementation. Retaining this draft does not amend
that baseline, the AI contracts, their implementation boundaries or any active
controller. Native AI, Pi and LSP keep their own integration owners.

## Fixed editor surfaces

| Output | Meaning | Examples within the boundary |
| --- | --- | --- |
| replace | Replace the captured selection or an explicitly admitted buffer target. | Translate, sort, transform a table or rewrite prose. |
| insert | Insert at the captured point. | A UUID, timestamp, fetched ticket title or generated snippet. |
| show | Present complete output in a static read-only text buffer. | Query results, a dictionary lookup or an explanation. |
| message | Show a bounded native echo-area message. | A calculation result, count or completion notice. |
| choose | Present a complete candidate list through native selection UI; accepting one inserts or replaces the captured target. | Choose a snippet, spelling suggestion or API result. |

A show buffer retains ordinary native navigation/search. It has no
extension-owned keymap, callbacks, per-line actions or retained view-update
protocol. A choose result is an explicit picker, not a live external source
queried on every keystroke.

Revision-bound highlight or gutter marks remain an open candidate. They are
not a supported output kind in this profile until their rendering and stale
state costs are explicitly accepted.

## Separate responsibilities

| Concern | Refined draft owner |
| --- | --- |
| Registration | The native configuration loader validates a user-declared command and its fixed input/output policy. |
| Execution | A native asynchronous job owns one child, its streams, deadline and cleanup. |
| Exchange | Text mode uses UTF-8 output, EOF and exit status; optional JSON mode has a versioned single-object contract. |
| Editing | The native editor validates the captured target/revision and publishes one atomic undoable change. |
| Presentation | Existing native buffers, echo area and candidate interaction render results. Extensions do not draw or own layouts. |

These are semantic responsibilities, not a prescribed trait hierarchy,
plugin SDK or general framework. Registering or listing configured commands
does not run them.

## Mechanism levels

These levels express increasing capability, not scheduled phases or a promise
to implement every level. Add a mechanism only if actual requirements justify
its cost.

### Text filters

An explicit command declares a program and argv, an input policy, an output
kind and bounded execution preferences. A compatible ordinary program can run
without implementing a custom discovery or message protocol. Precedents are
Emacs `shell-command-on-region`, vi `!`, Kakoune `|` and Helix `:pipe`.

Illustrative proposal syntax only; these keys and the example timeout are not
part of the current TOML schema or selected factory limits:

~~~toml
[[external.command]]
name = "ext-translate-en"
program = "/usr/local/bin/translate"
args = ["--to", "en"]
input = "selection"
output = "replace"
timeout_ms = 5000
~~~

- Input is the captured selection, explicitly admitted buffer or no text.
  It is sent to stdin, which is then closed. Point remains a separate target
  for insertion, not an implicit request for the whole buffer.
- stdout is collected under a byte limit until EOF and must be valid UTF-8.
  The supported outputs are replace, insert, show and message.
- Exit status zero, complete output and valid native checks are necessary
  for success. Exit zero alone does not authorize a stale or invalid edit.
  Failure discards the proposed result and reports bounded diagnostics.
- stderr is diagnostic data, never source text or direct terminal drawing.
- There is no directory scan, executable probe, session or JSON-RPC requirement.
- A declaration in the user's admitted configuration is the proposed trust
  grant for explicit invocation. Merely loading it does not execute it.
- The program is launched with an argument vector, without implicit shell
  interpolation. A user can explicitly select a shell as the declared program;
  returned text cannot become a new command declaration.

The absence of any installed interpreter is a normal executable failure;
rev does not bundle Ruby, Python or another external program runtime.

### Structured one-shot results

When plain output cannot represent the required result, the candidate JSON
mode exchanges one request object on stdin and one response object on stdout,
both delimited by EOF. It still uses one child per invocation, with separate
bounded stderr and an exit-status check. JSON-RPC and streaming frames are
not needed for this draft contract.

The request can contain only admitted text and metadata: captured point/target,
revision, visiting path and language. A response can contain an admitted edit
batch, a complete candidate list or another declared output. Schema versioning,
coordinate units, size limits and allowed fields must be fixed before adoption.
Unknown output kinds or multiple response objects are failures.

Illustrative proposal syntax, not a current setting:

~~~toml
[[external.command]]
name = "ext-pick-snippet"
program = "/usr/local/bin/snippets"
protocol = "json"
input = "point"
output = "choose"
~~~

No request grants access to arbitrary other buffers or additional context
because an extension asks for it. JSON mode does not expose an editor RPC API.

### Persistent processes outside the current profile

An expensive retained index or similar demonstrated need could motivate
separately considering persistent children. That would need reuse/isolation,
idle shutdown, routing and bounded event semantics. JSON-RPC or another framed
protocol could then be evaluated; neither persistent sessions nor reuse of
LSP/Pi transport code is assumed.

A one-shot program may already bridge to an independently managed service.
rev owns the adapter child, not that service. Persistent processes are not
a committed later step and are not required by the fixed surfaces above.

## Native candidate interaction

A choose invocation supplies one complete bounded candidate list. Native UI
owns filtering, highlighting, acceptance and cancellation. The accepted item
must still refer to the original invocation, buffer, target and revision when
it becomes an edit; changing focus does not redirect it to another buffer.

The [autocompletion proposal](editor-autocompletion.md) gives native candidate
interaction its own owner and defines LSP/Tree-sitter source precedence.
Reusing that interaction for an explicit command does not add a third
automatic completion source. Whether choose waits for that capability or uses
an M-x-style filtering prompt first remains open. Text filters do not depend
on either choice.

Incremental external candidates, live previews and callbacks during filtering
are outside this profile.

## Excluded extension shapes

These shapes cross the buffer-centric boundary. They are excluded, not
deferred:

- Interactive views such as a Magit-like status buffer with sections, folding,
  a view-local keymap and per-line actions routed back to the extension.
- Streaming pickers, live previews and floating layouts such as Telescope.
- Custom panels, widgets, split layouts or direct terminal drawing.

## Alternatives retained for comparison

The original directory/probe recommendation is superseded in the current
draft profile by passive declarations. The following alternatives retain their
rationale; they are not implementation obligations.

| Alternative | Why not part of the refined profile |
| --- | --- |
| Dedicated executable directory with --describe | Enumerating capabilities executes third-party code. Passive declarations avoid that startup/trust/lifecycle work; a capability cache is not required. |
| Directory with static manifests | Could reduce repetitive registration, but adds installation/discovery rules not needed by text filters. Reconsider only for a concrete usability need. |
| PATH prefix discovery | Adds scans, precedence and collisions. Configured executable resolution is a separate open policy. |
| First-class D-Bus, sockets or TCP | Introduces endpoint discovery, availability and ownership. An ordinary adapter can use these without another native transport. |
| Temporary-file exchange | Adds file ownership and cleanup; not needed by the current text/JSON stdio proposal. |
| JSON-RPC for every command | Request IDs and framed sessions add no required capability to one request/result delimited by process lifetime. |
| Embedded scripting | Introduces a scripting runtime the baseline excludes and ties extensions to one language. |
| WebAssembly | Would provide real confinement and typed interfaces, but adds a large runtime dependency outside the lightweight constraint. Reconsider only if enforced sandboxing becomes a requirement. |
| Dynamic-library plugins | Adds in-process ABI and fault coupling; outside the external-program profile. |
| General remote editor API | Widens the surface beyond the fixed text outputs. |
| Package manager or registry | Not needed to invoke user-declared programs and remains outside the baseline. |

No directory location, manifest schema, cache, new transport or runtime is
selected by retaining these alternatives.

## ADDED Requirements

The following candidate obligations describe the refined draft if adopted.
SHALL does not change its optional draft status.

### Requirement: EXT01 Preserve the independent lightweight editor

Ordinary editing SHALL retain its native behavior without any external command
configuration. Declared but unused commands SHALL NOT start children or query
services during startup, registration or listing. Core commands SHALL NOT
depend on an external program, extension runtime or package manager.

#### Scenario: Keep unused commands idle

- **GIVEN** valid external declarations that the user does not invoke
- **WHEN** rev starts and the user edits with native commands
- **THEN** no extension child or capability probe is started.

### Requirement: EXT02 Register commands passively from user configuration

The draft registration contract SHALL admit program, argv and input/output
policy from the user's configuration without executing a probe. The declaration
SHALL be the invocation trust grant, not a grant from a project file, result
message or executable name discovered by scanning. Exact syntax and resolution
rules remain open. Native command collisions SHALL be rejected.

#### Scenario: List a declared command without probing

- **GIVEN** a declaration for an executable that would stall if launched
- **WHEN** configuration is loaded and available commands are listed
- **THEN** rev validates the declaration without running that executable
- **AND** native editing remains available.

### Requirement: EXT03 Bound text and structured one-shot exchanges

Text mode SHALL use the admitted stdin text, complete UTF-8 stdout, separate
stderr, EOF and exit status. If JSON mode is adopted, it SHALL define a
versioned single request/response schema and coordinate units. Both modes
SHALL enforce byte/deadline limits and reject incomplete or malformed results.
They SHALL NOT implicitly execute stdout or require JSON-RPC sessions.

#### Scenario: Reject failed output without publishing

- **GIVEN** an invocation captured a valid editing target
- **WHEN** its child exits nonzero, exceeds limits or returns invalid UTF-8
- **THEN** no partial source change is published
- **AND** rev reports a bounded diagnostic.

### Requirement: EXT04 Supply only the admitted editor context

An invocation SHALL capture its declared input, output target and revision.
Text mode SHALL supply selection, admitted buffer text or no text. Point input
in structured mode SHALL supply only its admitted snapshot metadata/context.
These choices SHALL remain distinct from other open buffers and disk contents.
Supplying input SHALL NOT save unsaved source or grant extra read requests.

#### Scenario: Transform a selection

- **GIVEN** a declared command whose input is the selected text
- **WHEN** the user invokes it
- **THEN** its input contains only the admitted selection and defined metadata
- **AND** unrelated open-buffer contents are not added automatically.

### Requirement: EXT05 Keep edit publication inside rev

Every editing result SHALL remain bound to its invocation, original buffer,
target and source revision. rev SHALL check ranges, declared coordinate units,
read-only state and current revision before atomic native publication as one
undo step. Stale, partial, canceled or invalid proposals SHALL NOT modify source.
A completed child or zero exit status SHALL NOT bypass those checks.

#### Scenario: Receive an edit after the source changed

- **GIVEN** an invocation captured a particular buffer revision
- **WHEN** an external edit arrives after that target changed
- **THEN** rev rejects source publication against the newer text
- **AND** any diagnostic or result presentation identifies the captured operation.

### Requirement: EXT06 Keep work asynchronous and finite

External execution SHALL NOT block native input or rendering. The contract
SHALL define finite input, output, diagnostic, concurrency, queue and lifetime
budgets before implementation. Stream pumping SHALL handle stdout/stderr
without blocking on a full pipe while supplying stdin. Cancellation SHALL
fence late publication and trigger cleanup of owned work.

#### Scenario: Cancel slow work while editing

- **GIVEN** a child is waiting or producing output
- **WHEN** the user continues editing or cancels the invocation
- **THEN** native input remains responsive
- **AND** a late result cannot publish the canceled operation.

### Requirement: EXT07 Own one child per invocation

The current profile SHALL start one child only on explicit invocation and
clean it up after success, failure, cancellation or editor exit. Its process
and descendant cleanup rules SHALL be explicit before implementation; it SHALL
NOT become an unmanaged persistent extension. Closing an adapter SHALL NOT
grant authority to terminate an independently managed downstream service.

#### Scenario: Finish through a service adapter

- **GIVEN** an invoked program communicates with an existing external service
- **WHEN** rev completes or cancels the invocation and cleans up its child
- **THEN** it does not terminate the unrelated service.

### Requirement: EXT08 Distinguish fault isolation from a sandbox

A child failure SHALL NOT corrupt native editor state or publish a partial
edit. Documentation SHALL distinguish the configured program's operating-system
permissions from the limited text rev supplies. User configuration as a trust
grant SHALL NOT be described as enforced filesystem/network confinement.

#### Scenario: Describe the actual trust boundary

- **GIVEN** ordinary external program execution without enforced confinement
- **WHEN** its permissions and admitted input are documented
- **THEN** the description states that limiting supplied text does not sandbox
  the program's independent operating-system access.

### Requirement: EXT09 Keep outputs inside the fixed native surfaces

The admitted outputs SHALL be replace, insert, static read-only show, bounded
message and explicit choose as available. Unknown output kinds SHALL be rejected.
Programs SHALL NOT override built-ins, intercept input, draw custom layouts,
own interactive views or subscribe to editor events in this profile.
Highlights/gutter marks remain unadopted and SHALL NOT be silently admitted.

#### Scenario: Present static output without creating an interactive view

- **GIVEN** a declared show command returns complete text
- **WHEN** rev presents its result
- **THEN** a native read-only buffer contains the static output
- **AND** no extension keymap, per-line callback or custom layout is installed.

### Requirement: EXT10 Keep external candidate selection explicit

If choose is adopted, an explicit command SHALL supply a complete bounded list
to native filtering/selection UI. Acceptance SHALL revalidate the captured
editing target/revision and publish through EXT05; cancellation SHALL leave
source unchanged. The integration SHALL NOT alter automatic LSP/Tree-sitter
precedence or query the extension as a third source while the user types.

#### Scenario: Reject a choice after concurrent editing

- **GIVEN** a candidate list is open for a captured target and revision
- **WHEN** that source changes before the user accepts an item
- **THEN** the candidate is not applied to the changed source or a newly focused buffer.

#### Scenario: Leave automatic completion ownership intact

- **GIVEN** an external choose command is registered but not invoked
- **WHEN** ordinary autocompletion runs
- **THEN** the external program is not queried or inserted into its source policy.

## Open decisions

1. Adopt external text commands at all, given the baseline exclusion. The
   feature is optional and has no assigned delivery order.
2. Exact user-configuration shape, command namespace, program resolution,
   contextual bindings and trusted include behavior. Proposed examples are
   not current settings.
3. Missing-selection behavior, input/target combinations, whole-buffer scope
   and acceptance/preview policy for editing results.
4. Whether structured JSON mode is justified; its schema/versioning, coordinate
   units and permitted edit batches. A plain filter does not depend on it.
5. Whether choose waits for native autocompletion UI or can use an M-x-style
   prompt; candidate fields, limits and stale-list handling.
6. Whether revision-bound marks fit the fixed surface and resource budget.
   They are not included merely because ordinary rendering can draw them.
7. Finite sizes, deadlines, concurrency, cwd/environment, interpreter failures,
   process-descendant cleanup and diagnostics retention.
8. Optional Linux hardening could be investigated, including Landlock/seccomp.
   No sandbox dependency or enforcement claim is selected.
9. Passive manifests or persistent sessions require a demonstrated need and
   separate consideration. Automatic completion sources, interactive views,
   streaming pickers and custom drawing are excluded from the current profile,
   not promised as future work.

## Validation direction and readiness

A future adopted design needs representative compiled-program and
interpreted-script filters, adapter ownership, each admitted output kind,
malformed/non-UTF-8/oversized/nonzero-exit output, pipe pressure, slow/crashed
children, cancellation, editor exit, Unicode, read-only targets and stale edits.
Undo must restore exact prior source in one step. Candidate acceptance needs
the same checks after UI delay, not only when the child finishes.

Measure startup and ordinary editing with no declarations and with declared
but unused commands separately from active invocations. The idle guarantee is
no external child or capability query; identical measured latency or literally
zero native configuration overhead is not assumed. No such measurements have
been performed by this draft.

This specification and model remain drafts, not ready for slicing. Adoption,
the finite selected mechanism, wire and lifecycle details and the baseline
delta must precede implementation planning. The work index records this exploration without scheduling or adopting it.
No accepted specification, execution tracker, controller configuration or
runtime behavior is changed by this draft.
