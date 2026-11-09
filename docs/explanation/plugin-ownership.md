# Plugin ownership and compatibility

A plugin is a separate local process connected to one editor by a versioned
protocol. Its declaration contributes discoverable command metadata before it
runs. The native editor retains ownership of text, views, input contexts,
history, file observations and publication. The SDK supplies public values and
bounded correlation; it cannot grant access to a rope, pointer or native callback.

## Native sources outlive processes

Handles identify live sources in one editor session. Names and paths are labels,
not authority. A command captures its original buffer revision, pane, point and
selection; moving focus does not refresh that capture. Checked reads and edits
can address hidden unsaved text. Ordinary buffers created by a package retain
native navigation, modified state and close protection after that process stops.
Terminals and generated native inventories belong to different resource owners.

Every edit batch is prepared and checked for all targets before one serialized
publication. A stale or invalid target changes none of them. Success gives each
buffer its own undo unit; undoing one buffer does not reverse another buffer.
Text creation, editing and undo never save files. Explicit save checks both source
and disk observations. Modified close and overwrite decisions stay with native
consent; the protocol contains no force switch.

## Input and process ownership

Native prompts, transient contexts, recovery keys and macro input retain their
authority. A plugin's qualified command, binding or callback is admitted only in
its applicable context. User choices and scopes resolve through the same native
maps used for hints. Removing a contribution resolves the current maps again;
it does not restore an obsolete snapshot over later user changes.

One generation owns its requests, handlers, dynamic keys and subscriptions.
Cancellation and retirement revoke queued authority before cleanup. A late reply
cannot recreate it, and cancellation cannot undo a batch already committed.
Restart waits for owned cleanup before admitting a new generation. Idle processes
remain blocked on readiness; native input does not poll them periodically.
Finite queues, deadlines and cooperative service prevent unlimited admitted work.
They are resource bounds, not a promise about OS or filesystem latency.

This is process supervision under the user's normal OS account. It is not a
security sandbox. A program may exercise that account's permissions independently
of the protocol, and escaped descendants can be unobservable. Trust and review
precede configuration. There is no runtime installer or distribution catalog here.

## Events and shared state

Events describe committed native revisions, not proposed text. Slow consumers
receive an explicit resnapshot and must establish a new subscription baseline
before rebuilding their view. Causal origins and finite depth prevent callbacks
from repeatedly resetting an edit chain. An event grants neither focus nor save
consent.

Package state is separate from source text and configuration. Independent editors
share a package's versioned JSON records under the native state root, while their
processes and handles remain independent. Compare-and-swap checks the target
record and preserves unrelated updates. A stale version refuses. Replacement is
the commit point; uncertain durability is reported with candidate-version evidence,
not claimed as rollback. Observe current state before another mutation.

## Experimental compatibility

The exact API `rev.plugins/0.1`, SDK package version `0.1.0`, editor version and
individual plugin version are distinct. Pin the editor and SDK source used for
a package build. Initialization must agree on identities, API, Unicode scalar
units, capabilities and finite limits. Unknown object fields are rejected;
there is no silent negotiation to another API revision.

Only advertised capabilities authorize calls. A schema in the SDK can describe
a future method: `commands.register`, `commands.unregister` and `messages.show`
currently return `unsupported_method`. Static command declarations and accepted
handshake handlers are the native command surface. State is advertised only when
the host attaches an explicit native state root; library sessions do not discover
personal persistence automatically.

Protocol or saved-data changes need an explicit compatibility decision. Macro
libraries retain schema-1 reads, and only an explicit save containing a plugin
call writes schema 2. Reading does not migrate data. Package state uses its own
schema-1 envelope. Neither schema number is the API version.

For practice, use the [optional plugin lesson](../tutorials/user-guide/22-experimental-plugins.md).
For exact contracts, use the [behavior](../reference/plugins.md) and
[protocol](../reference/plugin-protocol.md) references. Return to [Explanation](index.md).
