# Plugin System

Status: Approved
Date: 2026-11-08
Version: 0.1
Format: OpenSpec capability proposal
Decision state: host/API scope approved; experimental implementation profile selected
Implementation authority: one host/API delivery set, canonical slice PRs and controller integration into dev; distribution remains separate
Review scope: experimental API for evaluation; no stable plugin ecosystem is announced
Inspection baseline: a398dcb77dd0ce036d989f038068ac942fc68ebd
Approved: owner delegated remaining host/API engineering decisions and requested execution
Plan: [Plugin System Delivery](../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../tracker/rev-plugin-system.md)
Model: [Plugin Identity, Protocol and State](editor-plugin-system-model.md)
Exploratory case: [Development Workflow Plugin](editor-development-workflow.md) tests the public contract; additional API needs require review
Related exploration: [External Extensions](editor-external-extensions.md)
Baseline: [rev Editor](rev-editor.md)

## Purpose

Define the host and public editor API through which users develop packages that
extend rev. The product direction is general editor extensibility. The first
experiment concentrates on multiple text buffers, commands, keybindings and
events so that the foundation can be evaluated with concrete programs.

This is a reviewable design, not an announcement of a supported plugin system.
An experiment may change the API, execution mechanism or package format after
testing. Publishing an experimental artifact for scrutiny does not make it a
stable interface or approve its adoption into ordinary editor releases.

## Delivery scope

The owner selected separate delivery sets for the host/API and package
distribution. This specification owns the first set: external process lifecycle,
the public protocol, multi-buffer operations, commands, keybindings, events and
package-owned state access. Evaluate it with explicitly supplied local plugin
programs and declarations.

Catalog discovery, package publication, downloading, installation, updates and
removal belong to a separate specification and delivery set. That work depends
on the evaluated host/API contract and receives its own plan, tracker and
acceptance. Catalog and installer design do not block planning the host/API set.

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

### Selected execution architecture

The owner selected one external process per active plugin package within each
independent rev instance, communicating with its native host through JSON-RPC
over stdin/stdout. That process serves all admitted buffers in the instance.
Separate rev instances own separate plugin processes, sessions and handles.

The protocol remains language-independent. Stdout carries protocol messages and
stderr carries diagnostics. The selected profile below fixes framing, handshake,
local activation, compatibility and finite limits for the approved experiment.

### Selected activation and concurrency policy

Read passive declarations to advertise command names, descriptions and admitted
keybindings before starting a plugin process. Activate on first use of a command
or keybinding, or on a declared file-type or event trigger. Startup activation
requires explicit configuration. Simultaneous triggers for one package in one
host session join one activation rather than starting duplicate processes.

Once active, reuse the process until explicit deactivation, editor shutdown or
process termination. The initial policy does not unload a process merely because
it has been idle or its last relevant buffer has closed.

Requests capture their session, generation, buffer and revision as applicable.
Independent plugin work may proceed concurrently; native edit publication stays
serialized and revision-checked. Changing focus does not redirect a response.
Transactions operate within one editor instance, without claiming atomic changes
across independent rev processes.

Session-local state remains isolated between editor instances. Persistent state
shared through the host-provided package-state API requires coordinated updates
and version checks that prevent lost updates across instances. The selected model
fixes package-owned TOML records, compare-and-swap and nonblocking coordination. When instances visit the
same source file, native [external-change and save-conflict policies](editor-external-file-changes.md)
still apply.

### Selected keybinding policy

The owner accepted contextual resolution with explicit user precedence inside
each applicable scope. Initial plugin scopes are global and buffer-local within
the native interaction profile and context. Existing prompt and popup owners
keep their native input grammar ahead of ordinary plugin bindings.
The [native keymap contract](../../../docs/reference/bindings.md#configuration-keymap-contracts)
defines the existing profile, prefix and recovery rules preserved by this policy.

Within the same applicable scope, resolve explicit user configuration before
plugin defaults, then editor factory defaults. A buffer-local binding can
therefore override a customized global binding. A user changes that local
behavior by configuring the matching scope, rather than relying on a global
override to suppress all contextual behavior.

When plugin defaults conflict in the same scope and no explicit user choice
resolves them, none of the conflicting plugin assignments becomes effective.
Report the involved sequences, contexts, packages and commands. Installation,
activation and request arrival order do not choose a winner. Existing native
fallback rules remain available; admitted commands remain accessible by name
through M-x or the modal selector even when their suggested shortcuts conflict.

Plugin contributions must pass native sequence normalization, prefix and recovery
validation. They cannot remove cancellation, command-discovery recovery or native
modal transitions. User configuration keeps its existing native validation rules.
Removing a contribution reveals the current underlying policy, not an old saved
mapping that could overwrite newer user choices.

Rationale: contextual maps let a table, result buffer or prompt use familiar keys
without changing unrelated editing. User precedence preserves deliberate choices
when plugins are installed or updated. Explicit collision reporting prevents
lazy activation order from changing key behavior. The accepted tradeoff is that
the user occasionally resolves an ambiguous shortcut while both commands remain
available through ordinary discovery.

### Selected authoring SDK

The first authoring SDK is a separate Rust library crate used by plugin programs.
It wraps the public JSON-RPC contract and editor operations on the client side.
The wire API remains usable by plugins written in other languages.

The SDK initially lives at crates/rev-plugin-sdk/ in the editor repository as a
member of a Cargo workspace. It has its own Cargo.toml and package version, so
the library can be published independently while host, protocol and SDK changes
are developed and tested together. A separate SDK repository is not part of this
first delivery.

The editor repository keeps its existing stack. Any Go interoperability client
belongs to a separate project outside this repository. Go sources, binaries and
build steps are not added here, and normal editor/SDK builds and tests do not
acquire a Go toolchain requirement.

The owner selected the separate [rev/plugin-examples repository](https://forge.adrianpk.com/rev/plugin-examples)
for standalone examples and interoperability checks, beginning with a Go client.
The repository is public and currently empty. It owns its source, compilation
and acceptance execution;
each result records the tested editor commit, API revision and example commit.
The SDK stays in the editor repository. Exact external acceptance commands remain
to be defined after the wire contract is settled.

### Selected first experiment

- A language-independent, bidirectional API over an external process connection.
- JSON-RPC 2.0, with one complete JSON message per line.
- A native broker owns editor operations, revision checks, input and rendering.
- Passive manifests describe commands and activation conditions. Listing packages
  does not run their executables.

These contracts define the approved experiment. The selected process and transport
architecture does not introduce a plugin compiler or package manager. The supplied rust-script link remains research input and is not
an adopted runner.

## Relationship to the earlier exploration

The earlier external-extension draft deliberately admits only a small set of
one-shot text outputs. This proposal explores a broader, persistent API with
commands, buffer-local bindings and event subscriptions. It is a candidate
successor for that exploration, not an implicit amendment of its boundaries.

Existing native shell commands, LSP and AI integrations keep their own contracts.
Their process owners and wire protocols do not automatically become the plugin
host. A future approved change must reconcile the editor baseline and the older
extension draft before declaring this interface supported.

### Repository organization

The project uses the [rev forge organization](https://forge.adrianpk.com/rev).
The editor repository is rev/rev and associated project repositories follow
rev/{repo-name}. The [repository continuation](../ticket/solved/20261108214514-move-editor-repository-to-rev-organization.md)
retains adrianpk/rev as history rather than transferring its forge records.
Package/catalog repository names still need explicit selection.

Repository ownership and plugin namespaces are separate contracts. A source
repository location does not automatically select the package publisher ID,
reserve a native command prefix or change installed package identity.

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

The following methods define the approved experimental surface; they are not
M-x command names. The model companion defines their inputs and ownership.
Implementation must negotiate only capabilities already delivered.

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
| Package state | state.list, state.get, state.put, state.delete | Access only the admitted package namespace with version-checked publication. |

A buffer and a pane are separate objects. An extension can read or edit a hidden
buffer without selecting it, and can create a buffer before deciding where to
show it. Source text, terminal emulator output and generated native views are
not interchangeable resources. The first text API admits ordinary text buffers;
unsupported resource kinds return a typed error.

### Namespaces and user interaction

The package ID is publisher.package. A command named collect in
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
The [selected keybinding policy](#selected-keybinding-policy) governs precedence,
unresolved plugin defaults and preserved native recovery. Inspection and command
hints must describe the effective binding rather than an inactive suggestion.

Command metadata must state its prefix-argument and macro-recording policy.
Unsupported counted invocation is rejected explicitly. A process instance ID
or callback address is never a durable macro command identity. The selected
profile adds compatible qualified plugin-call steps to the completed
[macro behavior](editor-keyboard-macros.md), preserving native macro names.

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
target buffer. The required rule is all-or-nothing publication across that
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
the noninteractive plugin method returns confirmation_required instead of
opening a prompt when native consent is required. It cannot discard that content.

### Process and event behavior

The host follows the selected activation policy within its editor session.
The process can handle several buffers. Request IDs, package generation and
captured buffer targets keep concurrent results separate. Changing focus does
not redirect an in-flight operation to the new active buffer.

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

The following SHALL statements define the approved experiment's acceptance
target. Approval is implementation authority, not evidence of delivered behavior.

### Requirement: PLG01 Keep the experiment optional and identifiable

The host SHALL require explicit experimental activation and SHALL report the
exact API revision. Core editing SHALL work without an installed plugin runner.
Declared commands SHALL be discoverable before process activation. Concurrent
activation triggers SHALL share at most one package process per host session.

#### Scenario: Ordinary launch
- **GIVEN** experimental plugins are disabled
- **WHEN** rev starts and edits a file
- **THEN** it starts no plugin process and requires no extension runtime.

#### Scenario: Available but unused package
- **GIVEN** an enabled local package with declared commands and no matching activation trigger
- **WHEN** the user opens command discovery
- **THEN** its names and descriptions are visible without starting its process.

#### Scenario: Concurrent first use
- **GIVEN** two buffers trigger the same inactive package in one editor session
- **WHEN** activation begins
- **THEN** both requests use one admitted activation and process generation.

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
Native recovery and transient input owners SHALL retain their existing priority.
Within the same scope, explicit user bindings SHALL precede plugin defaults,
which SHALL precede factory defaults. Unresolved conflicting plugin defaults
SHALL remain ineffective independently of activation order, with diagnostics
that identify every involved contribution. Their admitted commands SHALL remain
available through ordinary command discovery.

#### Scenario: Result-buffer action
- **GIVEN** a package creates a result buffer and registers a command
- **WHEN** it binds that command inside the result buffer
- **THEN** the key invokes it there through native dispatch
- **AND** the binding does not change an unrelated source buffer.

#### Scenario: Local context precedes a global customization
- **GIVEN** a user global binding and a plugin buffer-local default for the same key
- **WHEN** that buffer owns ordinary editing input
- **THEN** its local binding takes precedence over the global customization
- **AND** other buffers retain their own effective mappings.

#### Scenario: Explicit user choice inside the same scope
- **GIVEN** user configuration and a plugin default bind the same key in the same scope
- **WHEN** bindings are resolved or the plugin is updated
- **THEN** the explicit user target remains effective.

#### Scenario: Competing plugin shortcuts
- **GIVEN** two packages propose conflicting defaults in the same scope without a user choice
- **WHEN** they activate in either order
- **THEN** neither conflicting plugin shortcut becomes effective
- **AND** both admitted commands remain invocable by name and the conflict is inspectable.

#### Scenario: Native prompt and recovery ownership
- **GIVEN** a buffer has active plugin bindings
- **WHEN** a native prompt owns input or the user invokes a native recovery route
- **THEN** the native interaction retains its admitted behavior
- **AND** ordinary plugin mappings cannot consume that input or change the underlying buffer.

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

#### Scenario: Handle from another editor instance
- **GIVEN** two editor instances have activated the same package
- **WHEN** a request in one instance supplies a handle from the other
- **THEN** the receiving host rejects it without changing any buffer.

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
- **THEN** activation fails before executable handler publication
- **AND** core editing continues normally.

### Requirement: PLG12 Separate settings, package content and mutable state

The design SHALL preserve the existing configuration/data/state distinction.
Package mutable state SHALL be namespaced separately from editor-owned state.
Updates through the host-provided persistent-state API SHALL coordinate across
editor instances and detect version conflicts rather than silently losing updates.

#### Scenario: Package stores a preference-independent cache
- **GIVEN** a package requests persistent cache or state storage
- **WHEN** the host resolves its admitted storage area
- **THEN** that area is qualified by package identity and concern
- **AND** it is not the editor configuration directory or another package's state.

#### Scenario: Two instances update shared package state
- **GIVEN** two editor instances read the same version of a package-state record
- **WHEN** one publishes an update and the other submits its obsolete version
- **THEN** the obsolete update is rejected without overwriting the newer record.

## Experiment and scrutiny

These evaluation journeys form the approved delivery acceptance. The linked plan
owns slice order and execution authority. Publish actual observations with the
tested host, API revision and client commits; document preparation is not runtime evidence.

| Journey | What must be demonstrated |
| --- | --- |
| Deferred activation | Discover commands without spawning; exercise command, keybinding, file-type and event activation, explicit startup and simultaneous first use. |
| Multi-buffer reader | Read several buffers, including hidden and unsaved text; no focus changes are required. |
| Generated buffer | Create a result buffer, show it, list it in ordinary buffer navigation and attach local command bindings. |
| Batch editor | Change two buffers; refuse both when one is stale, read-only, closed or invalid; exercise local undo explicitly. |
| Lifecycle | Activate, cancel, crash, deactivate and restart; retain edited result buffers and reject old-generation responses. |
| Independent editor instances | Reuse one package process across buffers in each instance; reject foreign handles and prevent lost persistent-state updates between instances. |
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

## Selected experimental implementation profile

The owner approved execution and delegated the remaining host/API details. These
choices define a finite first experiment; later API changes require a reviewed
contract revision. They do not select a catalog or package installer.

### Protocol and local declarations

Use JSON-RPC 2.0 over UTF-8 newline-delimited JSON, one complete object per line.
Newlines inside strings use JSON escaping. Reject batch arrays, malformed or
oversized frames. Peers can issue correlated requests concurrently; notifications
have no response. Reserve stdout for protocol traffic. Use the exact API string
rev.plugins/0.1. The host sends initialize with session, generation, expected
package, implemented capabilities, scalar units and limits; the package returns
its matching identity/API and accepted handlers before executable admission. Unsupported capabilities reject
explicitly rather than silently degrading into a different mutation.

Expose optional startup configuration under [plugins]: enabled defaults false,
manifests defaults empty and startup defaults empty. Manifest paths are explicit
trusted local files, resolved relative to their declaring configuration file;
relative executable paths resolve against the manifest directory. Launch literal
argv without a shell. A manifest declares its package/version/API, executable,
commands, descriptions, bindings and command/file-type/event activation conditions.
Do not scan PATH, run capability probes or add download/install behavior.

Use native plugin-list, plugin-activate, plugin-deactivate and plugin-restart
commands, with complete descriptions and ordinary M-x/modal discovery. A list
shows passive/active/failed status, effective bindings and bounded diagnostics.
The first SDK lives at crates/rev-plugin-sdk as the rev-plugin-sdk Cargo package,
version 0.1.0, under the repository license. Add a workspace without changing the
editor package/release version. Prefer existing serde, serde_json, rustix and the
standard library; adding a new runtime/dependency requires separate authority.

### Commands, contextual bindings and compatible recording

Package IDs have two lowercase ASCII segments publisher.package; each segment
starts with a letter and contains at most 32 letters, digits or hyphens. Command
names use the native 1..=64 lowercase ASCII letters/digits/hyphens grammar. The
qualified identity is publisher.package.command, at most 130 ASCII bytes. Reserve
publisher rev for native ownership. Display qualified names; a short alias can
only resolve when unique and cannot replace a native identity.

Every command declares a description, parameter contract, prefix policy (none or
integer), and recording policy (reject or semantic). Forward one captured prefix
value once; never guess that the host should run a handler N times. An admitted
semantic call records its qualified ID, validated literal arguments and accepted
prefix once. Generated low-level edits are not recorded a second time.
Replay waits cooperatively for completion and can be canceled. Missing packages,
changed declarations and unsupported policies stop explicitly.

Native MacroName syntax and existing native calls stay unchanged. Add a distinct
qualified plugin-call step. Readers retain schema-1 macro-library admission and
accept a schema-2 extension for those new steps. Only an explicit save requiring
plugin steps writes schema 2; merely reading an old library does not migrate it.
Preserve old record identities/content and reject unknown steps without executing
them. An older editor cannot be promised schema-2 support; document that boundary.

Plugin keymaps use native editing contexts (editor and the existing vim_* state
maps) and global or explicit buffer scope. They do not replace transient owners.
User overrides are declared under [[plugins.keybindings]] with command, keys,
context and scope. A buffer-scoped override selects filetypes, buffer_owner, or
both (intersection); buffer_owner matches native provenance from buffers.create,
not a reusable process handle. Require at least one selector for persistent
buffer overrides. Global overrides have neither selector. The existing native
user map remains authoritative within its scope. Admission checks normalized
sequences, prefix reachability, recovery and modal state transitions atomically.
The selected precedence and conflict rules above remain unchanged.

### Mutation authority and command invocation

Capture buffer, pane, source revision, point, active selection and prefix at
invocation. Positions are source Unicode scalar offsets, ranges half-open, and
revisions and generations are opaque decimal strings. Do not retarget on focus
change. Treat terminal buffers and generated native inventories as unsupported
text resources; buffers.create produces an ordinary unsaved text buffer.

All edits use a native prepare-all/publish-all coordinator. Preparation includes
history, remapped views and every fallible reservation; after final stamp checks,
publication has no fallible partial path. Successful batches give each affected
buffer its own ordinary undo unit. No implicit save or cross-buffer undo follows.

Plugin calls do not open native prompts or invent consent. commands.invoke admits
only metadata-declared noninteractive commands with validated typed arguments
and captured targets; other classes return unsupported_invocation or
confirmation_required. Per-invocation causal depth also covers nested command
calls. buffers.close rejects modified content requiring consent. buffers.save
requires expected source revision and expected native disk observation, including
an explicit absent-file observation for save-as; native conflict/read-only/path
rules remain authoritative. No force-save or implicit file creation is granted.

### Process lifetime, events and package state

Activate one owned process group per package per editor instance; concurrent
triggers join it. Ordinary I/O is asynchronous and all native publication runs at
the serialized editor boundary. Cancel/deactivate/shutdown first retire authority,
then signal only the owned group, apply bounded escalation and reap it. A failed
spawn cleans every owned descriptor/registration. Unknown escaped descendants
are reported as uncertain cleanup, not claimed as a sandbox or silently killed.
There is no automatic restart loop or idle unload.

Subscriptions receive post-publication sequence/revision/origin notifications.
On overflow, discard incomplete deltas and deliver one explicit resnapshot
invalidation; inability to deliver that invalidation retires the connection.
Propagate a causal chain through event-triggered edits and nested commands;
reject depth above 8. Drain at most 32 plugin items or 2 ms per event-loop service
pass, whichever occurs first, then return to native input. These are scheduling
ceilings, not timing guarantees on every host. Idle connections must not cause
polling wakeups when no work is pending.

state.list/get/put/delete operate only on the caller package namespace. Resolve
schema-1 TOML state under XDG_STATE_HOME/rev/plugins/packages/<publisher.package>/
(or the existing native XDG fallback), separate from configuration and other
stores. Store concern/record IDs and monotonically increasing opaque versions;
put/delete require the observed version, with null meaning absent for creation.
Use the existing native nonblocking locking and checked atomic-publication
patterns. Contention returns busy; an obsolete record version returns
state_conflict. Prepare/read/CAS/publication run outside the input loop, preserve
unrelated concurrent records and retain failure/uncertain-write evidence. Reject
traversal, symlink substitution, invalid schemas and oversized records.

### Finite initial profile

| Resource | Initial ceiling |
| --- | --- |
| Declared packages / active package processes | 64 / 8 per editor |
| Plugin commands | 64 per package; 512 total |
| Plugin binding contributions | 128 per package; 512 total |
| UTF-8 frame / text read or replacement payload | 4 MiB / 512 KiB |
| One edit batch | 16 buffers; 256 replacements; 512 KiB replacement bytes total |
| Pending RPC work | 16 requests per package; 64 total |
| Outbound queued data | 4 MiB per package; 16 MiB total |
| Event queue / causal depth | 256 entries per package / 8 |
| Startup / command-request deadline | 5 seconds / 60 seconds |
| Cancel or shutdown escalation grace | 2 seconds |
| Stderr diagnostic ring | 128 KiB per active package |
| State record / package store / record count | 256 KiB / 4 MiB / 256 |

Validate before spawning or allocating unbounded collections; smaller native
text/history/view limits also apply. Record baseline, dormant declaration, active
idle, large-read and event-load measurements on the tested candidate. These
values are initial engineering ceilings, not measured performance claims.
Changing a contract or weakening an assertion/limit requires review; do not raise
limits merely to hide a failing acceptance journey.

### Independent client acceptance and remaining review boundary

The separate rev/plugin-examples repository owns a small Go client and its
scripts/check-rev.sh acceptance command. That script accepts the exact editor
checkout, builds/tests Go outside it and exercises examples/plugin-interop.rs
through the public protocol. Evidence pins both source commits and rev.plugins/0.1.
The editor/SDK's normal Cargo build/test commands never invoke or require Go.

No owner question blocks this host/API delivery. Measurements, complete wire
types and coordinator proof remain implementation obligations under these rules;
they are not permission to broaden the product. Catalog/distribution, a stable
plugin ecosystem, public trust/sandboxing and the development-workflow package
retain their separate review and delivery boundaries.

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
