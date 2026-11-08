# Plugin Identity, Protocol and State

Status: Draft
Date: 2026-11-08
Version: 0.1
Specification: [Plugin System](editor-plugin-system.md)
Decision state: candidate model for experimental evaluation
Persistence authority: none; no configuration schema or storage migration is changed

## Model boundary

The native editor owns canonical text, revisions, history, selections, views and
command dispatch. An external process receives copied values and opaque handles.
It requests operations through the admitted API; it never owns a Rust pointer,
borrow, rope, view reference or native callback address.

Candidate names below express contracts, not Rust module/type choices or a
database schema. Protocol methods and TOML examples are not current interfaces.

## Identity and lifetime

| Entity | Candidate fields | Qualification and lifetime |
| --- | --- | --- |
| Package identity | publisher, package name, package version | Stable author namespace and distribution identity; version is distinct from API revision. |
| Package declaration | package identity, required API revision, executable and argv, declared commands, activation conditions | Passive installed metadata; registration/listing does not execute it. |
| Host session | opaque session ID, API revision, supported capabilities and limits | One editor lifetime; handles from another session are invalid. |
| Process generation | session, package identity, generation, owned process identity, connection state | One activation instance; restart invalidates old callbacks and requests. |
| Buffer handle | session, opaque buffer ID | Refers to the existing native buffer identity; no path/name aliasing or numeric position in a list. |
| Buffer snapshot | buffer handle, revision, range, source text, resource kind and flags | In-memory source at one revision; separate from disk saved state and display layout. |
| Pane handle | session, opaque view identity | Refers to an existing native view; the view displays a buffer but does not own its source. |
| Command contribution | canonical ID, display name, description, generation, handler ID, argument/recording policy | Owned registration; its stable identity is separate from the current process handler. |
| Binding contribution | registration ID, command ID, key sequence, editing context, global or buffer scope | Native admitted keymap contribution; no raw keyboard interception. |
| Subscription | subscription ID, generation, event kinds, buffer filters, queue cursor and overflow state | Opt-in, bounded connection-local state; removed on deactivation. |
| Invocation | invocation ID, generation, command ID, captured buffer/view/point context, argument, deadline, state | Cancelable operation; focus changes do not redirect its captured targets. |
| Edit batch | operation ID, expected stamps, ordered disjoint replacements per buffer | Complete proposed mutation; all target preparation precedes publication. |

Namespacing does not grant operating-system isolation or editor authority by
itself. Activation and admitted capabilities remain separate from identity.
The canonical forge's rev/{repo-name} organization convention identifies source
repositories, not protocol package IDs. A source repository change preserves installed
package, command and state identities unless a separate reviewed migration changes
those contracts explicitly.

## Candidate manifest

Illustrative data only. This file does not exist in the implemented configuration
schema and is not a command to install or activate anything:

```toml
[plugin]
id = "field.notes"
version = "0.1.0"
api = "rev.plugins/0.1-draft1"

[process]
program = "./notes-plugin"
args = []

[[commands]]
name = "collect"
description = "Collect task lines from open text buffers."
```

The host derives field.notes.collect from the admitted package ID and command
name. Relative program paths resolve against that package's installed content;
argv entries are literal arguments rather than implicit shell source. The exact
manifest structure, identifier grammar and activation fields remain proposals.

## Candidate operation contracts

| Method | Input and successful result | Important rejection or lifecycle rule |
| --- | --- | --- |
| initialize | Package/API identity; host returns session, generation, capabilities, units and limits. | Reject incompatible versions before publishing contributions. |
| buffers.list | Optional native text-buffer filter; returns bounded descriptors and explicit continuation when needed. | A result may include hidden buffers; no implicit disk scan. |
| buffers.open | Explicit file reference; returns a native buffer handle. | Reuse native path/open identity; do not open a second canonical copy of the same file. |
| buffers.create | Proposed name, initial text and supported text-buffer attributes; returns a new handle and revision. | Creates an in-memory buffer, not a file or an implicit visible pane. |
| buffers.describe | Handle; returns name, optional visiting path, kind, revision, read-only and modified state. | Missing/closed handles fail; names are not stable identifiers. |
| buffers.read | Handle, source range and optional expected revision; returns exact text and its stamp. | Native scalar bounds and finite response size apply. |
| edits.apply | Complete finite edit batch with expected revision for every target; returns operation ID and new stamps. | Reject the entire batch for stale, missing, read-only, overlapping or invalid targets. |
| buffers.show | Handle and admitted native pane/placement choice; returns the view identity. | Validate text resource and view lifetime; no arbitrary terminal drawing. |
| buffers.save | Explicit buffer handle and any required native file reference; returns new saved-state observation. | Native read-only/path/conflict rules apply; no automatic save on editing. |
| buffers.close | Explicit handle; returns closed or a native refusal/confirmation outcome. | No implicit forced discard; exact confirmation interaction remains open. |
| commands.register | Own-namespace command declarations; returns registration handles. | Validate complete declarations and duplicates before atomic publication. |
| commands.unregister | Own registration handles. | Does not remove another package's or a native command. |
| commands.list | Admitted command metadata under a bounded query. | Canonical ID and display name remain distinct. |
| commands.invoke | Admitted command ID, typed arguments and explicit invocation context. | Define prompt/reentrancy, prefix and recording policy before enabling each invocation class. |
| keymaps.bind | Registered command ID, native key sequence, context and explicit scope. | Follow accepted precedence; a buffer-local binding needs a live buffer handle. |
| keymaps.unbind | Own binding registration handle. | Restore the underlying effective mapping; do not overwrite user changes made after registration. |
| events.subscribe | Admitted event kinds and buffer filters; returns subscription ID and cursor. | Finite queues and declared overflow behavior are required. |
| events.unsubscribe | Own subscription ID. | Already queued callbacks lose authority when the subscription generation retires. |
| messages.show | Finite text and supported severity. | Native feedback only; raw escape sequences do not become terminal instructions. |

Host-to-package calls command.invoke and command.cancel carry the invocation and
generation identities. Buffer lifecycle and revision events are notifications
with subscription ID, sequence, qualified buffer and causal origin. Replies
correlate through request IDs; stdout is protocol-only and stderr is bounded
diagnostic output.

Exact expected-source and saved-file preconditions for buffers.save remain an
API decision. An implementation client cannot infer them from edits.apply or
from a path alone. Package-state access and owned descendant cleanup also need
public contracts before a client can enable durable background execution.

### Error candidates

At minimum distinguish unsupported_api, unsupported_method, invalid_arguments,
namespace_conflict, command_unavailable, missing_buffer, unsupported_resource,
stale_revision, read_only, invalid_range, edit_conflict, confirmation_required,
cancelled, limit_exceeded and disconnected. Final spelling and wire error codes
remain open. A useful failure identifies the request and affected handle without
exposing unrelated editor content.

## Editing transaction lifecycle

1. Capture source IDs, expected revisions, ranges and replacements.
2. Resolve all targets and validate source units, edit order, flags and limits.
3. Prepare every private native candidate, history delta and mapped source/view
   state. Reserve all resources required for publication.
4. Recheck source stamps at the editor's serialized publication boundary.
5. Publish the complete batch through a coordinator with no remaining fallible
   step that can produce partial publication; emit events only after it succeeds.
6. Return the operation ID and final stamps. A failed admission publishes none.

The coordinator is proposed work, not a current implementation claim. A failed
or canceled request cannot acquire a new target from the current active pane.
An accepted batch creates a native undo unit in each affected buffer; the shared
operation ID is correlation metadata, not a cross-buffer undo implementation.

## Registration and execution lifecycle

Installed metadata can advertise commands without starting a process. First
explicit activation starts a generation, negotiates the experimental API and
validates its complete initial contributions. Only then are executable handlers
available. Contributions added later require the same checked admission.

Use lifecycle states such as installed, activating, active, stopping, inactive
and failed as candidate runtime observations, not a second durable installer
state machine. Deactivation stops new callbacks, cancels pending work, retires
the generation and reaps the owned process. Automatic restart is not assumed.
Created buffers retain their native lifetime and unsaved text.

Stopping the direct package process alone is not proof that its descendants
settled. Record process ownership, permitted child creation, shutdown and uncertain
cleanup behavior in the finite experiment profile before enabling nested jobs.

Registrations must have handles so cleanup removes exactly the generation's
contributions. Removing a local binding reveals the currently valid underlying
map rather than restoring an old snapshot over newer user choices.

## Configuration, installed data and mutable state

Preserve the existing XDG concern split. Exact subpaths and codecs are unresolved:

| Concern | Candidate placement and owner |
| --- | --- |
| User preferences | Existing rev configuration area: opt-in, enabled packages, user bindings and explicitly selected package settings. |
| Installed package content | Rev data area under XDG_DATA_HOME; version-qualified package content and passive manifests. |
| Host activation/install records | Rev state area under XDG_STATE_HOME; native owner, separate from package-owned state. |
| Package mutable state | Rev state area under a plugin/package namespace; explicit API and versioned data contract to be chosen. |
| Rebuildable package caches | Rev cache area under XDG_CACHE_HOME, scoped by package and relevant version. |

Package settings do not share a flat key space with native settings. A package
cannot use the state-storage API to choose another package's directory or the
host's internal record paths. Ordinary process filesystem access remains governed
by the separately chosen operating-system trust/isolation model.

Do not add a package archive codec, signing format, registry schema or migration
merely to exercise the RPC API. The first experiment can resolve explicit local
fixtures; catalog/install design can reuse established product conventions after
the runtime and lifecycle are evaluated. Disk persistence is not required for
ephemeral request IDs, generation handles, snapshots or subscriptions.

## Bounds to establish before a prototype executes

The candidate host needs finite limits for manifest/contribution counts, message
and text bytes, edit targets/replacements, in-flight requests, event queues,
active processes, startup/command deadlines and shutdown grace. Native text,
history and view limits remain additional constraints rather than replacements.

Choose numerical values with a recorded local prototype profile and measurements;
they are not silently inherited from the examples or another editor. Reject an
unbounded profile before starting a child. Specify event-cycle attribution and
fair scheduling so several individually bounded plugins cannot starve native
input collectively. Ordinary editing must retain an independent service budget.

## Evaluation record

No prototype, passing runtime tests or resource measurements are recorded yet.
Future experiment records should name the exact host revision, draft API
revision, client implementation/version, finite profile, fixture input, observed
result and unresolved defect. Keep successful structural document checks distinct
from these runtime observations. Update this model with the specification during
each reviewed API iteration.
