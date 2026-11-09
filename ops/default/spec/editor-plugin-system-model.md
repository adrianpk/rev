# Plugin Identity, Protocol and State

Status: Approved
Date: 2026-11-08
Version: 0.1
Specification: [Plugin System](editor-plugin-system.md)
Decision state: approved host/API experiment with selected protocol, invocation and persistence contracts
Persistence authority: package-owned schema-1 state; explicit compatible schema-2 macro saves containing plugin calls

## Model boundary

The native editor owns canonical text, revisions, history, selections, views and
command dispatch. An external process receives copied values and opaque handles.
It requests operations through the admitted API; it never owns a Rust pointer,
borrow, rope, view reference or native callback address.

The contracts below describe the approved implementation target, not a claim of
existing runtime behavior. Rust module/type layout remains an engineering choice.
Plan: [Plugin System Delivery](../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../tracker/rev-plugin-system.md)

The first delivery covers host/API behavior and package-owned state access,
evaluated with explicit local programs and declarations. Catalog records,
downloaded artifacts and installation/update/removal transactions belong to a
separate distribution specification and delivery set. They are not prerequisites
for evaluating the host's buffer, command, keymap and event contracts.

## Selected execution architecture

Each active plugin package has one external process per independent rev instance,
shared by that instance's admitted buffers. Instances do not share plugin processes
or runtime handles. Connections use language-independent JSON-RPC over stdin/stdout.
Stdout is reserved for protocol messages; stderr carries diagnostics. Process
generations and request identities remain native lifecycle boundaries.

Use JSON-RPC 2.0, UTF-8 NDJSON and the exact API revision rev.plugins/0.1.
The host initiates initialize; the package identifies itself and agrees to the
supplied session, generation, capabilities, scalar units and finite profile.
No batch arrays or unsolicited pre-handshake operations are admitted.

The first SDK is an independent Rust client library for plugin programs. Its
boundary is the public wire API, without importing private native editor types.
The selected source layout is crates/rev-plugin-sdk/ in the editor repository,
managed as a Cargo workspace member with its own manifest and package version.
Library identity and version remain distinct from editor release identity and
the negotiated host API revision.

The planned interoperability client is a Go example maintained in the separate
[rev/plugin-examples project](https://forge.adrianpk.com/rev/plugin-examples), whose
repository is provisioned and public. That project owns its sources, build
requirements and acceptance execution. Each
evaluation identifies both repository commits and the negotiated host API
revision; the editor's normal build/test requirements stay independent.

## Identity and lifetime

| Entity | Candidate fields | Qualification and lifetime |
| --- | --- | --- |
| Package identity | publisher, package name, package version | Stable author namespace and distribution identity; version is distinct from API revision. |
| Package declaration | package identity, required API revision, executable and argv, declared commands, activation conditions | Passive installed metadata; registration/listing does not execute it. |
| Host session | opaque session ID, API revision, supported capabilities and limits | One independent editor lifetime; handles from another instance/session are invalid. |
| Process generation | session, package identity, generation, owned process identity, connection state | At most one admitted process for a package in a host session; restart invalidates old callbacks and requests. |
| Buffer handle | session, opaque buffer ID | Refers to the existing native buffer identity; no path/name aliasing or numeric position in a list. |
| Buffer snapshot | buffer handle, revision, range, source text, resource kind and flags | In-memory source at one revision; separate from disk saved state and display layout. |
| Pane handle | session, opaque view identity | Refers to an existing native view; the view displays a buffer but does not own its source. |
| Command contribution | canonical ID, display name, description, generation, handler ID, argument/recording policy | Owned registration; its stable identity is separate from the current process handler. |
| Binding contribution | registration ID, command ID, key sequence, editing context, global or buffer scope, provenance | Native admitted contribution; a valid declaration can remain shadowed or conflicted instead of effective. |
| Binding resolution | context/scope, normalized sequence, effective target, underlying mappings, conflicting contributions and diagnostics | Context precedes global fallback; within a scope user configuration precedes plugin defaults, then factory defaults. |
| Subscription | subscription ID, generation, event kinds, buffer filters, queue cursor and overflow state | Opt-in, bounded connection-local state; removed on deactivation. |
| Invocation | invocation ID, generation, command ID, captured buffer/view/point context, argument, deadline, state | Cancelable operation; focus changes do not redirect its captured targets. |
| Edit batch | operation ID, expected stamps, ordered disjoint replacements per buffer | Complete proposed mutation; all target preparation precedes publication. |
| Shared package-state record | package identity, concern, record identity, version, content | Persistent schema-1 TOML records coordinate across instances through version-checked package-state methods. |

Namespacing does not grant operating-system isolation or editor authority by
itself. Activation and admitted capabilities remain separate from identity.
The canonical forge's rev/{repo-name} organization convention identifies source
repositories, not protocol package IDs. A source repository change preserves installed
package, command and state identities unless a separate reviewed migration changes
those contracts explicitly.

## Selected local manifest

This is the approved experiment schema to implement; it is not an installer.
Reject unknown fields and invalid identity/path/capability metadata before spawn:

```toml
[plugin]
id = "field.notes"
version = "0.1.0"
api = "rev.plugins/0.1"

[process]
program = "./notes-plugin"
args = []

[[commands]]
name = "collect"
description = "Collect task lines from open text buffers."
prefix = "none"
recording = "reject"

[activation]
filetypes = []
events = []
```

The host derives field.notes.collect from the admitted package ID and command
name. Relative program paths resolve against the declaring manifest directory; argv
entries are literal. Command use always permits lazy activation; nonempty native
filetype/event declarations add their corresponding activation triggers. Explicit
[plugins].startup selects startup activation, not a manifest side effect.

## Operation contracts

| Method | Input and successful result | Important rejection or lifecycle rule |
| --- | --- | --- |
| initialize | Host supplies session/generation, expected package, API, implemented capabilities, units and limits; package acknowledges matching identity/API and handlers. | Reject incompatible versions or undeclared handlers before executable publication. |
| buffers.list | Optional native text-buffer filter; returns bounded descriptors and explicit continuation when needed. | A result may include hidden buffers; no implicit disk scan. |
| buffers.open | Explicit file reference; returns a native buffer handle. | Reuse native path/open identity; do not open a second canonical copy of the same file. |
| buffers.create | Proposed name, initial text and supported text-buffer attributes; returns a new handle and revision. | Creates an in-memory buffer, not a file or an implicit visible pane. |
| buffers.describe | Handle; returns name, optional visiting path, kind, revision, read-only and modified state. | Missing/closed handles fail; names are not stable identifiers. |
| buffers.read | Handle, source range and optional expected revision; returns exact text and its stamp. | Native scalar bounds and finite response size apply. |
| edits.apply | Complete finite edit batch with expected revision for every target; returns operation ID and new stamps. | Reject the entire batch for stale, missing, read-only, overlapping or invalid targets. |
| buffers.show | Handle and admitted native pane/placement choice; returns the view identity. | Validate text resource and view lifetime; no arbitrary terminal drawing. |
| buffers.save | Explicit buffer handle and any required native file reference; returns new saved-state observation. | Native read-only/path/conflict rules apply; no automatic save on editing. |
| buffers.close | Explicit handle; returns closed or a native refusal/confirmation outcome. | No implicit forced discard; modified content returns confirmation_required. |
| commands.register | Own-namespace command declarations; returns registration handles. | Validate complete declarations and duplicates before atomic publication. |
| commands.unregister | Own registration handles. | Does not remove another package's or a native command. |
| commands.list | Admitted command metadata under a bounded query. | Canonical ID and display name remain distinct. |
| commands.invoke | Admitted command ID, typed arguments and explicit invocation context. | Define prompt/reentrancy, prefix and recording policy before enabling each invocation class. |
| keymaps.bind | Registered command ID, native key sequence, context and explicit scope; returns an owned contribution and its resolution outcome. | A buffer-local binding needs a live buffer handle. Conflicting defaults remain ineffective and inspectable; native prefix/recovery rules still reject invalid contributions. |
| keymaps.unbind | Own binding registration handle. | Restore the underlying effective mapping; do not overwrite user changes made after registration. |
| events.subscribe | Admitted event kinds and buffer filters; returns subscription ID and cursor. | Finite queues and declared overflow behavior are required. |
| events.unsubscribe | Own subscription ID. | Already queued callbacks lose authority when the subscription generation retires. |
| messages.show | Finite text and supported severity. | Native feedback only; raw escape sequences do not become terminal instructions. |

Host-to-package calls command.invoke and command.cancel carry the invocation and
generation identities. Buffer lifecycle and revision events are notifications
with subscription ID, sequence, qualified buffer and causal origin. Replies
correlate through request IDs; stdout is protocol-only and stderr is bounded
diagnostic output.

buffers.save requires expected_revision and expected_disk, both native observations
returned with buffer metadata. expected_disk distinguishes absence from a changed
existing file; no inferred force or fresh observation can replace the caller
precondition. Modified-close and prompt-requiring calls return confirmation_required.
The selected profile in the specification owns process-group cleanup and state access.

### Typed errors

At minimum distinguish unsupported_api, unsupported_method, invalid_arguments,
namespace_conflict, command_unavailable, missing_buffer, unsupported_resource,
stale_revision, read_only, invalid_range, edit_conflict, confirmation_required,
cancelled, limit_exceeded and disconnected. Use standard JSON-RPC errors for malformed envelopes/methods/params; application
errors carry a stable data.kind and request/handle context. A useful failure identifies the request and affected handle without
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

The coordinator is required implementation work, not current evidence. A failed
or canceled request cannot acquire a new target from the current active pane.
An accepted batch creates a native undo unit in each affected buffer; the shared
operation ID is correlation metadata, not a cross-buffer undo implementation.

## Registration and execution lifecycle

Passive metadata advertises commands and admitted bindings without starting a
process. Command/keybinding use or a declared file-type/event trigger starts
activation. Startup requires explicit configuration. Concurrent triggers join
one activation for the package and host session. Activation negotiates the API
and validates the complete initial contributions before executable handlers become
available. Later contributions require the same checked admission.

Reuse the active process until deactivation, editor shutdown or process
termination. Idle time and closure of the last relevant buffer do not trigger
automatic unloading in the initial policy. Work from several buffers may overlap;
every request retains its captured targets and revision preconditions, and native
edit publication remains serialized within one editor instance.

Use lifecycle states such as installed, activating, active, stopping, inactive
and failed as candidate runtime observations, not a second durable installer
state machine. Deactivation stops new callbacks, cancels pending work, retires
the generation and reaps the owned process. Automatic restart is not assumed.
Created buffers retain their native lifetime and unsaved text.

Stopping the direct child alone does not prove descendant settlement. Own one
process group, retire authority before cancellation, signal only that owned group
with the selected grace/escalation and reap it. Report escaped/unobservable
descendants as uncertain cleanup; no sandbox guarantee is made.

Registrations must have handles so cleanup removes exactly the generation's
contributions. Removing a local binding reveals the currently valid underlying
map rather than restoring an old snapshot over newer user choices.

## Selected binding resolution

Resolve native recovery and transient input ownership before ordinary plugin
maps. Buffer-local context precedes global fallback, including customized global
bindings. Within one applicable scope, explicit user configuration precedes
plugin defaults, then factory defaults.

If plugin defaults conflict without an explicit user choice, retain their
provenance and report the ambiguity while leaving all involved assignments
ineffective. The result is independent of activation or registration arrival
order. A shortcut conflict does not retire otherwise admitted command identities;
command discovery remains available and effective-key hints omit the conflicting
suggestions. Recompute the current underlying mapping when a contribution retires.

These are native resolution observations, not package-controlled priorities or
a new persistence schema. Wire results expose effective, shadowed or conflicting status plus provenance;
Slice 1 publishes complete typed envelopes and method schemas under these contracts.

## Configuration, installed data and mutable state

Preserve the existing XDG concern split. The approved host/API set uses explicit
local programs and these selected concern boundaries:

| Concern | Candidate placement and owner |
| --- | --- |
| User preferences | Existing rev configuration area: opt-in, enabled packages, user bindings and explicitly selected package settings. |
| Installed package content | Rev data area under XDG_DATA_HOME; version-qualified package content and passive manifests. |
| Host activation records | Rev state area under XDG_STATE_HOME; native owner, separate from package-owned state. Installation records belong to the separate distribution delivery. |
| Package mutable state | XDG_STATE_HOME/rev/plugins/packages/<publisher.package>/; schema-1 TOML, package/concern/record identity, versioned CAS under a nonblocking native lock. |
| Rebuildable package caches | Rev cache area under XDG_CACHE_HOME, scoped by package and relevant version. |

Package settings do not share a flat key space with native settings. A package
cannot use the state-storage API to choose another package's directory or the
host's internal record paths. Ordinary process filesystem access remains governed
by the separately chosen operating-system trust/isolation model.

Do not add a package archive codec, signing format, registry schema or migration
merely to exercise the RPC API. The first delivery resolves explicit local
fixtures; catalog/install design belongs to a separate delivery after the
runtime and lifecycle are evaluated. Disk persistence is not required for
ephemeral request IDs, generation handles, snapshots or subscriptions.

## Shared package-state contract

| Method | Input | Result and rejection |
| --- | --- | --- |
| state.list | Own concern, bounded cursor/limit | Record IDs and versions; no foreign namespace or implicit full-content listing. |
| state.get | Own concern and record ID | Exact JSON-compatible value and opaque version, or explicit absence. |
| state.put | Own concern/record, expected_version and value | New version after atomic publication; null creates only an absent record. |
| state.delete | Own concern/record and expected_version | Version-checked deletion; never delete a newer record or sibling concern. |

Use lexical concern and record IDs without path separators; the host derives the
filesystem path. The store envelope contains schema_version=1, a store revision,
and bounded records ordered by concern/record identity, each carrying revision and
JSON-encoded value in TOML. Keep JSON data inert and roundtrip scalar/string/array/
object types. Null is representable as JSON content, distinct from absent records.

Lock the package store nonblockingly, re-read current records, verify the target
version, preserve unrelated records, prepare output and atomically publish. Busy,
stale, invalid/symlink or uncertain-write cases preserve authority and evidence;
none silently overwrites a newer value. Do not persist ephemeral sessions,
handlers, requests or subscription cursors. Storage I/O runs outside native input.

## Wire, configuration and macro compatibility

Use request IDs that are strings, opaque session-qualified buffer/view handles,
and decimal-string revisions/generations. Mutation methods always carry expected
source stamps. Responses correlate to the correct generation; retired callbacks
cannot become current by reusing an ID. JSON-RPC error data carries a stable kind;
add busy, state_conflict and unsupported_invocation to the typed failures above.

The startup [plugins] table contains enabled=false, manifests=[] and startup=[].
Relative manifest paths resolve against their configuration origin, never an
incidental process working directory. User [[plugins.keybindings]] entries carry
keys, command, native context, scope and optional filetypes/buffer_owner selectors.
Buffer scope requires selectors; global scope rejects them. When both selectors
are present they intersect. Native provenance survives process retirement with
the created buffer. Command arguments are declared and validated inert values,
not executable snippets. The Rust SDK documents each concrete wire struct.

Keep native MacroName identity unchanged. A separate plugin-command step stores
qualified identity, validated arguments and accepted prefix; it never stores a
process address, replays provider-generated edits or strips namespaces. Accept
legacy macro schema 1; explicit saves containing plugin steps use schema 2.
Preserve existing macro records and refuse unknown inputs. Reading does not
migrate personal state. Document that older editors do not read schema 2.

## Finite implementation and evidence profile

The specification's finite profile is normative for the first experiment.
Admission rejects unsupported capabilities/oversized collections before process
startup or large allocation. Native limits remain additional constraints. Event
queues use resnapshot invalidation on overflow, causal depth 8, and cooperative
32-item/2-ms service passes. Command/activation/shutdown limits, aggregate queue
ceilings and package-state bounds apply across concurrent buffers and packages.

Use package- and host-qualified operation identifiers for diagnostics. Tests must
prove the publication coordinator has no fallible partial-commit path, exercise
real owned process failure/cancellation, and check cross-instance state conflicts.
Measure dormant/idle behavior separately from active request throughput; list the
candidate, fixtures and observed numbers rather than asserting zero cost.

## Evaluation record

No prototype, passing runtime tests or resource measurements are recorded yet.
Future experiment records should name the exact host revision, draft API
revision, client implementation/version, finite profile, fixture input, observed
result and unresolved defect. Keep successful structural document checks distinct
from these runtime observations. Update this model with the specification during
each reviewed API iteration.
