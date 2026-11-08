# Plugin System

Status: Draft
Date: 2026-11-08
Version: 0.1
Format: OpenSpec capability proposal
Decision state: experimental design for review and local evaluation
Implementation authority: proposal only; no delivery set or prototype is started
Review scope: canonical experimental proposal; independent of completed native deliveries
Inspection baseline: 39f69965e8a5
Model: [Plugin Identity, Protocol and State](editor-plugin-system-model.md)
Exploratory case: a separate Development Workflow Plugin proposal tests the public contract; additional API needs require review
Related exploration: [External Extensions](editor-external-extensions.md)
Baseline: [rev Editor](rev-editor.md)

## Purpose

Let users develop, install and share packages that extend rev through a public
editor API. The product direction is general editor extensibility. The first
experiment concentrates on multiple text buffers, commands, keybindings and
events so that the foundation can be evaluated with concrete programs.

This is a reviewable design, not an announcement of a supported plugin system.
An experiment may change the API, execution mechanism or package format after
testing. Publishing an experimental artifact for scrutiny does not make it a
stable interface or approve its adoption into ordinary editor releases.

## Decisions and proposals

### Owner direction

- Third parties author their own extensions; the task is to build their host.
- Packages and catalogs should follow the familiar discover/install/update/remove
  experience of the other maintained editors.
- Rev will not implement its own extension language and interpreter.
- Extensions should eventually support substantial editor behavior, rather than
  being limited permanently to text filters.
- Initial capabilities include reading and editing several buffers, creating
  new buffers, contributing commands, binding keys and separating namespaces.
- Evaluate and iterate before adopting an official plugin system.

### Candidate for the first experiment

- A language-independent, bidirectional API over an external process connection.
- JSON-RPC 2.0 over stdin/stdout, with one complete JSON message per line.
- At most one lazily started process per activated package in the initial host.
- A native broker owns editor operations, revision checks, input and rendering.
- A small authoring SDK can wrap the protocol. Rust is a convenient first test
  client; the wire contract must remain usable from another language.
- Passive manifests describe commands and activation conditions. Listing packages
  does not run their executables.

These are proposals to test. They do not select a dependency, runtime, compiler
requirement or package manager. The supplied rust-script link is research input;
neither it nor any other execution mechanism is adopted by this document.

## Relationship to the earlier exploration

The earlier external-extension draft deliberately admits only a small set of
one-shot text outputs. This proposal explores a broader, persistent API with
commands, buffer-local bindings and event subscriptions. It is a candidate
successor for that exploration, not an implicit amendment of its boundaries.

Existing native shell commands, LSP and AI integrations keep their own contracts.
Their process owners and wire protocols do not automatically become the plugin
host. A future approved change must reconcile the editor baseline and the older
extension draft before declaring this interface supported.

## Current native foundation

The inspected source already provides:

- Process-local buffer IDs that are not reused, revision stamps and qualified
  source ranges: [buffer identity](../../../src/buffer.rs).
- Creation of independent text buffers, separate from opening files or showing
  them in a pane: [buffer collection](../../../src/buffer.rs).
- Preparation and publication of a checked edit transaction for one buffer:
  [editing](../../../src/buffer/edit.rs).
- A native command catalog with names and descriptions:
  [commands](../../../src/command.rs).
- Validated contextual key sequences and bindings:
  [keymaps](../../../src/input/keymap/map.rs).

These are implementation building blocks, not an existing public plugin API.
Dynamic command admission, plugin-scoped lifecycle, process RPC and atomic
multi-buffer publication require design and implementation. The first experiment
must demonstrate those boundaries rather than assuming a serialization wrapper
is sufficient.

## Proposed API surface

The following names are illustrative protocol methods, not current M-x commands.
The model companion defines their candidate inputs, results and ownership.

| Area | Candidate methods | Observable contract |
| --- | --- | --- |
| Discovery | initialize | Negotiate experimental API revision, capabilities, coordinate units and limits. |
| Buffers | buffers.list, buffers.open, buffers.create, buffers.describe | Address several independent text buffers through explicit IDs. |
| Text | buffers.read, edits.apply | Read a revision-qualified range; submit checked edits to one or more buffers. |
| Display | buffers.show | Display an existing buffer in an admitted pane without changing its identity. |
| Lifecycle | buffers.save, buffers.close | Use ordinary native policies; never silently save or discard changed text. |
| Commands | commands.register, commands.unregister, commands.list | Add described commands under the package identity and inspect admitted commands. |
| Invocation | commands.invoke, command.invoke, command.cancel | Dispatch admitted editor commands or call a package handler with captured context. |
| Bindings | keymaps.bind, keymaps.unbind | Bind a command globally or within an explicitly identified text buffer. |
| Events | events.subscribe, events.unsubscribe | Receive selected buffer lifecycle and revision notifications. |
| Feedback | messages.show | Present a bounded native message without writing to the terminal directly. |

A buffer and a pane are separate objects. An extension can read or edit a hidden
buffer without selecting it, and can create a buffer before deciding where to
show it. Source text, terminal emulator output and generated native views are
not interchangeable resources. The first text API admits ordinary text buffers;
unsupported resource kinds return a typed error.

### Namespaces and user interaction

The candidate package ID is publisher.package. A command named collect in
field.notes has the canonical identity field.notes.collect. Rev derives this
identity from the admitted package; a child cannot claim another package's ID.
The rev namespace is reserved for native identities. Existing native command
names and bindings retain their current behavior.

M-x and the modal command selector use the ordinary command catalog, descriptions,
completion and effective key hints. A short display name is presentation; a
stored binding or macro refers to the canonical command identity. Ambiguous
short names must not dispatch according to installation order.

Bindings refer to registered commands, not process addresses or simulated
keystrokes. Buffer-local bindings are important for newly created result buffers.
The exact precedence between factory bindings, package defaults and explicit
user overrides is an open review decision. Until that rule is accepted, a
collision is reported rather than silently replacing an unrelated binding.
Existing native cancellation routes remain available.

Command metadata must state its prefix-argument and macro-recording policy.
Unsupported counted invocation is rejected explicitly. A process instance ID
or callback address is never a durable macro command identity. This proposal
does not amend the completed [macro specification](editor-keyboard-macros.md).

The dotted identities proposed here are not currently accepted macro names.
The native [macro name value](../../../src/command/macro_name.rs) admits lowercase
ASCII letters, digits and hyphens, with a 64-byte ceiling. Plugin integration
must explicitly resolve canonical command admission and recorded/persisted call
compatibility before claiming that a package command can be replayed. Do not
silently strip its namespace or change the saved macro schema.

### Editing and history

A read returns text and a stamp containing session, buffer identity and revision.
Candidate coordinates are half-open Unicode scalar offsets in the actual source
text. Display rows, terminal cells, UTF-8 bytes and UTF-16 code units are different
units. Text is not silently normalized during exchange.

edits.apply takes a complete finite batch and the expected revision of every
target buffer. The candidate rule is all-or-nothing publication across that
batch: every target and edit is prepared before any source or history changes.
Missing, closed, stale, read-only or invalid targets reject the entire batch.
Rebasing stale edits is not an implicit fallback.

This is a new native coordination requirement. Existing single-buffer prepare
and publish methods alone do not prove atomic publication across several buffers.
The experiment must establish that no fallible step after the first publication
can leave a partially applied batch.

Successful publication creates one ordinary undo unit per affected buffer and
returns a shared operation ID for inspection. Undo in one buffer affects that
buffer. A cross-buffer undo command is not claimed by this experiment. The
difference between atomic publication and local undo must be visible in its
tests and eventual reference documentation.

Saving files is a separate explicit request. Creating or editing a buffer does
not save it. Closing changed content follows the native confirmation policy;
an initial noninteractive method may return confirmation_required instead of
opening a prompt, but cannot discard that content. Resolve this interaction
choice before implementing buffers.close.

### Process and event behavior

The candidate host starts a package when one of its admitted commands or event
conditions needs it. The process can handle several buffers. Request IDs,
package generation and captured buffer targets keep concurrent results separate.
Changing focus does not redirect an in-flight operation to the new active buffer.

All child I/O and computation remain outside the input/render loop. Native
requests enter the editor's normal serialized operation boundary. Events are
emitted after publication, with buffer revision, sequence and causal origin.
Callbacks cannot reenter a half-published transaction. Revision invalidation
must be explicit if change notifications are coalesced or dropped under pressure.

Subscriptions are opt-in. Define queue bounds, callback scheduling and a finite
policy for event-induced edit cycles before admitting write-producing hooks.
Merely tagging a notification with its origin does not prevent two packages
from repeatedly triggering each other.

Canceling an invocation prevents late results from publishing. It does not
silently reverse edits already committed by earlier explicit requests. A child
that ignores cancellation is stopped according to native process ownership.
Disconnection removes callbacks and registrations from that process generation;
ordinary buffers and their unsaved text survive.

Process isolation limits faults; a normally launched executable still has the
user account's operating-system permissions. The initial evaluation uses locally
supplied trusted programs. Third-party execution trust and any sandbox are a
separate decision required before public package activation is adopted.

## Requirements

The following SHALL statements define the candidate experiment's acceptance
target. They are proposed requirements, not approved implementation claims.

### Requirement: PLG01 Keep the experiment optional and identifiable

The host SHALL require explicit experimental activation and SHALL report the
exact API revision. Core editing SHALL work without an installed plugin runner.

#### Scenario: Ordinary launch
- **GIVEN** experimental plugins are disabled
- **WHEN** rev starts and edits a file
- **THEN** it starts no plugin process and requires no extension runtime.

### Requirement: PLG02 Bind registrations to their package namespace

The host SHALL derive canonical command identities from an admitted package and
SHALL reject duplicate or foreign registrations atomically.

#### Scenario: Two packages use the same short name
- **GIVEN** two packages declare a command named collect
- **WHEN** both registrations are admitted
- **THEN** their qualified identities remain distinct
- **AND** an ambiguous short name does not choose a package implicitly.

### Requirement: PLG03 Integrate commands and buffer-local bindings

Registered commands SHALL use normal command discovery and descriptions. Bindings
SHALL reference those identities and follow the accepted context/precedence rule.

#### Scenario: Result-buffer action
- **GIVEN** a package creates a result buffer and registers a command
- **WHEN** it binds that command inside the result buffer
- **THEN** the key invokes it there through native dispatch
- **AND** the binding does not change an unrelated source buffer.

### Requirement: PLG04 Operate on multiple explicitly identified buffers

The API SHALL support creating and addressing text buffers independently of
active focus, file paths and pane visibility.

#### Scenario: Collect from hidden buffers
- **GIVEN** several admitted source buffers, including a hidden unsaved buffer
- **WHEN** a command reads their snapshots and creates a result buffer
- **THEN** it receives their current in-memory text
- **AND** creating the result does not overwrite or save a source buffer.

### Requirement: PLG05 Reject stale or mismatched source coordinates

Every text mutation SHALL carry the expected source stamp and declared coordinate
units. Closed or replaced buffers SHALL invalidate their earlier handles.

#### Scenario: User edits while a plugin computes
- **GIVEN** a plugin reads revision R
- **WHEN** the user changes that buffer before the plugin submits its edit
- **THEN** the host rejects the obsolete edit without changing text or history.

### Requirement: PLG06 Publish a multi-buffer batch atomically

The host SHALL validate and prepare the entire edit batch before publishing any
target. A failed target SHALL prevent all mutations in that batch.

#### Scenario: One target is read-only
- **GIVEN** a batch edits buffers A and B and B is read-only
- **WHEN** the batch is submitted
- **THEN** both buffers retain their original text, revisions and history.

### Requirement: PLG07 Preserve ordinary history and explicit saving

A successful edit batch SHALL use native history with one undo unit per affected
buffer. Buffer creation and editing SHALL not imply a filesystem write.

#### Scenario: Undo one affected buffer
- **GIVEN** one accepted operation changed two buffers
- **WHEN** the user undoes the change in one buffer
- **THEN** that buffer returns through its native history
- **AND** the second buffer is not silently reverted or saved.

### Requirement: PLG08 Deliver ordered revision-qualified events

Subscriptions SHALL have native lifecycle ownership, revision information and
explicit overflow behavior. Event work SHALL be bounded and non-reentrant.

#### Scenario: Subscriber falls behind
- **GIVEN** a subscribed package cannot consume changes at their production rate
- **WHEN** its queue reaches the admitted bound
- **THEN** rev follows the negotiated invalidation or disconnection policy
- **AND** it neither grows the queue indefinitely nor reports lost deltas as complete.

### Requirement: PLG09 Keep external work cancellable and responsive

The host SHALL impose finite message, queue, request and process limits. Plugin
work SHALL not block ordinary input or rendering.

#### Scenario: Late response after cancellation
- **GIVEN** the user canceled a pending invocation
- **WHEN** its generation later supplies a result
- **THEN** the result cannot publish another source edit.

### Requirement: PLG10 Retire contributions without losing user text

Deactivation or process failure SHALL retire that generation's handlers,
keybindings, subscriptions and pending invocations. Created text buffers SHALL
remain available to the user under ordinary native ownership.

#### Scenario: Package exits with an edited result buffer
- **GIVEN** the user has edited a buffer originally created by a package
- **WHEN** that package process exits
- **THEN** the buffer and unsaved text remain accessible through native navigation
- **AND** stale package handlers are unavailable.

### Requirement: PLG11 Negotiate an explicit experimental contract

The host SHALL reject unsupported API revisions and unadvertised methods with
bounded typed errors. A restart SHALL create a new process generation.

#### Scenario: Incompatible package
- **GIVEN** a package requires a different experimental API revision
- **WHEN** activation is requested
- **THEN** activation fails before command publication
- **AND** core editing continues normally.

### Requirement: PLG12 Separate settings, package content and mutable state

The design SHALL preserve the existing configuration/data/state distinction.
Package mutable state SHALL be namespaced separately from editor-owned state.

#### Scenario: Package stores a preference-independent cache
- **GIVEN** a package requests persistent cache or state storage
- **WHEN** the host resolves its admitted storage area
- **THEN** that area is qualified by package identity and concern
- **AND** it is not the editor configuration directory or another package's state.

## Experiment and scrutiny

These are evaluation journeys, not a slicing plan or authorization to implement.
Use small local fixtures and publish actual observations with the tested host,
API revision and client versions when an experiment is authorized.

| Journey | What must be demonstrated |
| --- | --- |
| Multi-buffer reader | Read several buffers, including hidden and unsaved text; no focus changes are required. |
| Generated buffer | Create a result buffer, show it, list it in ordinary buffer navigation and attach local command bindings. |
| Batch editor | Change two buffers; refuse both when one is stale, read-only, closed or invalid; exercise local undo explicitly. |
| Lifecycle | Activate, cancel, crash, deactivate and restart; retain edited result buffers and reject old-generation responses. |
| Competing packages | Use identical short command names and conflicting key suggestions; show deterministic inspection and diagnostics. |
| Events | Observe changes without blocking input; exercise overflow and an event-induced edit cycle. |
| Independent client | Complete the same basic journey from a second implementation language through the same wire contract. |
| Resource comparison | Measure baseline launch/idle, installed-but-unused packages, active idle processes, large reads and bounded event load. |

Review should challenge the API from both author and user perspectives:

1. Can an author build the multi-buffer journey without undocumented editor state?
2. Are reads, revisions, coordinates and errors sufficient to recover from races?
3. Can native command/keymap integration remain one coherent interaction model?
4. Does the transaction coordinator actually prove all-or-nothing publication?
5. Are activation, cancellation, crash and restart outcomes predictable?
6. Does the idle cost remain acceptable for rev, measured rather than assumed?
7. Which future extension shapes would require breaking this initial contract?

No evidence is recorded yet. Passing these journeys supports another review;
it does not automatically approve a release or stabilize the API. Adoption
requires explicit owner acceptance after scrutiny, revised specification/model,
and the normal implementation/documentation workflow.

## Open decisions

- Confirm process-per-package versus shared language hosts or another execution
  mechanism after measuring startup, isolation, author ergonomics and memory.
- Confirm JSON-RPC/framing and the exact experimental version handshake.
- Choose SDK language/order independently from allowed extension languages.
- Resolve command display/alias rules, binding precedence, prefix arguments and
  macro invocation admission with the current native contracts.
- Prove a native multi-buffer publication coordinator and retain the stated
  distinction between publication atomicity and per-buffer undo.
- Resolve source selection/point metadata and how native confirmations are
  exposed without blocking the event loop.
- Resolve public package-state access, revision-bound save preconditions and
  owned descendant-process cleanup before an execution-oriented client relies
  on them. The candidate methods above do not establish those mechanisms.
- Set measured finite defaults and a causal policy for write-producing hooks.
- Choose the third-party trust/isolation policy and package/catalog artifact
  contract before enabling public installations. Reuse distribution concepts
  without selecting another editor's runtime or persistence implementation.

## References

Behavior and architecture references; no implementation dependency is selected:

- [Neovim external plugin API](https://neovim.io/doc/user/api/): buffer handles,
  remote calls, revision notifications and compatibility contracts.
- [Neovim remote plugins](https://neovim.io/doc/user/remote_plugin/): external
  authoring and activation examples.
- [VS Code API](https://code.visualstudio.com/api/references/vscode-api): command
  registration, documents, event lifecycles and multi-document editing.
- [Xi plugin architecture](https://xi-editor.io/docs/plugin.html): historical
  discussion of asynchronous plugins and multiple-buffer process lifetimes.
- [rust-script](https://github.com/fornwall/rust-script): supplied brainstorming
  reference; not a selected runner, dependency or authoring requirement.
