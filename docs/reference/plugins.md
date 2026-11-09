# Experimental plugins

The experimental catalog admits explicitly configured local packages for
rev.plugins/0.1. It reads passive metadata and adds qualified command names and
descriptions to M-x and the modal colon selector without starting programs.
Command use, explicit startup and declared native triggers can activate one
owned process per package and independent editor. Contributed bindings and
package-to-host editor operations remain unavailable.

## Configuration

These startup-only values participate in the ordinary configuration include and
leaf precedence rules. An explicit empty array replaces an inherited array.

```toml
[plugins]
enabled = false
manifests = []
startup = []
keybindings = []
```

With enabled=false, manifests are not opened and no package commands are
admitted. With enabled=true, every listed declaration must validate before
any package is admitted. Failure retains the editor's complete built-in
configuration and reports one diagnostic; no partial catalog becomes visible.

Manifest paths are explicit local paths. Resolve them relative to the source
that last assigned plugins.manifests, including an included source. Resolve a
relative program path against its manifest directory. Both paths are frozen
as absolute paths at preparation. Do not expand variables, search PATH, probe
executables, scan directories or run a shell. Program arguments remain literal.

startup contains at most eight distinct declared package IDs. The native session
requests those activations after admitting its startup configuration and sources.
Library Session::new hosts honor the same explicit startup selection.

## Local manifest

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
parameters = []
prefix = "none"
recording = "reject"

[activation]
filetypes = []
events = []
```

Every object rejects unknown fields. The plugin and process tables are required.
The commands, bindings and activation collections default to empty. Package
IDs contain two lowercase publisher.package segments of 1..=32 ASCII characters;
each starts with a letter and then permits letters, digits and hyphens. The rev
publisher is reserved. Version is a nonempty opaque label of at most 128 visible
ASCII bytes, distinct from the exact required API revision.

Command names contain 1..=64 lowercase ASCII letters, digits or hyphens. The
canonical ID is derived as publisher.package.command, at most 130 bytes. Names
and package identities must be unique. Descriptions are nonempty, single-line
UTF-8 text of at most 4096 bytes. There are no implicit short-name aliases.

Each optional parameters entry contains name, kind and required. Names are
distinct lexical lowercase ASCII letters/digits/hyphens of 1..=64 bytes. Kinds
are null, boolean, integer, string, array or object. Arguments are inert JSON
values. prefix is none or integer; recording is reject or semantic. These
declarations validate an empty argument object before command activation. Required
arguments cannot be supplied by the current native selector. Counted calls and
keyboard-macro recording remain explicitly unsupported in this increment.

Activation filetypes use existing native language IDs. Events are opened,
changed or closed. Both arrays reject duplicates. A newly observed native text
buffer or a change of its native filetype can match filetypes. Opened, changed
and closed observations match their respective event declarations. Read-only
generated inventories and terminal resources do not trigger text activation.

## Binding declarations

Each manifest [[bindings]] entry and each user [[plugins.keybindings]] entry
has these fields:

| Field | Contract |
| --- | --- |
| command | Required qualified command ID; manifest defaults must name a command declared by that same package. User targets must exist in the admitted catalog. |
| keys | Required native key sequence, at most eight normalized chords and 4096 UTF-8 bytes. Uses the existing native terminal aliases and parser. |
| context | Required editor, vim_normal, vim_insert, vim_replace, vim_visual_character, vim_visual_line or vim_visual_block. |
| scope | Required global or buffer. Global rejects selectors; buffer requires at least one selector. |
| filetypes | Optional distinct native filetype IDs, default empty. |
| buffer_owner | Optional package ID matching native provenance of package-created buffers. |

When both selectors are present they intersect. Declarations remain inspectable
and ineffective; they do not replace a native keymap or transient/recovery owner.
Native prefix reachability, recovery and conflict resolution apply before any
future binding becomes executable. No activation order selects a winner.

## Native command metadata

| Command | Description |
| --- | --- |
| `plugin-list` | Inspect configured experimental plugin packages and their availability. |
| `plugin-activate` | Activate a configured experimental plugin package. |
| `plugin-deactivate` | Deactivate a configured experimental plugin package. |
| `plugin-restart` | Restart a configured experimental plugin package. |

plugin-list opens a read-only inventory with package IDs, versions, process state,
generation, pending requests, bounded diagnostics and cleanup observations. It
also lists declared commands and ineffective bindings. Display output is bounded
to 64 KiB with an explicit truncation notice. It performs no program or PATH
probe. Qualified
plugin selections participate in session usage ranking. They are not written to
the native schema-1 command history, whose name grammar and limits remain
unchanged; subsequent native command usage continues to persist.

The management commands select an explicitly admitted package ID. Activation
joins an activating or active generation. Restart retires the old generation
and waits for its cleanup before starting a new one. Deactivation retires work
and stops the owned group. C-g cancels a selector according to ordinary native
input ownership. With editing input, C-g cancels pending plugin work and starts
its owned cleanup; idle packages without pending work stay active.

## Process and RPC lifecycle

Simultaneous startup, native triggers and command use share one activation per
package. Programs run from the manifest directory with literal argv and three
pipes, without a host shell or PATH search. Spawning, pipe readiness, framing,
deadlines, group signals, process observation and reaping run outside native
input and rendering. Idle connections wait for readiness with no periodic
polling deadline. Active packages persist across source closure and native
Project or Workspace changes. There is no idle unload or automatic restart loop.

The host sends initialize first with its session, generation, exact API, scalar
units and finite profile. It validates the complete acknowledgement and every
initial handler before publishing active state. Handlers must belong to commands
in that package's admitted manifest, with distinct command and handler IDs.
A declared command with no accepted handler reports command_unavailable.

The commands capability currently enables host-to-package command.invoke and
command.cancel. Package-to-host commands, buffer, edit, display, binding, event,
message and state operations return correlated unsupported_method errors; no
native editor operation is enabled by schema availability. Requests before
initialization return unsupported_invocation. An incompatible acknowledgement,
malformed or oversized stdout, EOF, process exit or deadline retires the generation.
Stdout is protocol-only. Stderr is a byte-bounded diagnostic ring; display replaces
control characters instead of interpreting them as terminal instructions.

Command requests retain distinct session/generation-qualified IDs and capture
the originating native text buffer revision, pane, scalar point and selection.
Focus or source changes do not refresh this capture. Generated resources have
no text capture. Replies resolve only the matching live request and generation;
stale replies cannot publish source changes. Failed packages require explicit
command use, activation or restart before another attempt; native events do not
retry failed generations automatically.

Cancel, deactivate and shutdown retire authority before best-effort cancellation
notifications and TERM to the owned process group. The host grants two seconds
for group exit, then sends KILL and allows two seconds for force/reap completion.
It retains the unreaped leader identity through all group signals. Ordinary
buffers and unsaved text have independent native lifetimes. A failed spawn or
completed stop releases its pipes, pending registrations and control descriptor.

The cleanup observation distinguishes a reaped owned group from uncertain
observation, signal or reap failures. It always identifies escaped descendants
as unobservable. Programs retain the user account's ordinary OS permissions;
this is not a sandbox or proof that escaped descendants have stopped. Uncertain
cleanup prevents admission of another generation for that package.

run_terminal restores the terminal before its noninteractive cleanup barrier.
Library hosts use Session::shutdown_plugins before discarding owned resources;
it reports uncertainty after a five-second observation bound. Dropping a library
session submits retirement to the owning supervisors without waiting on input.

## Admission limits

| Resource | Ceiling |
| --- | --- |
| Explicit manifests / source bytes | 64 / 256 KiB per regular UTF-8 TOML file |
| Commands | 64 per package; 512 total |
| Binding defaults / user overrides | 128 per package; 512 default contributions total; 512 user overrides |
| Command parameters / activation filetypes / selector filetypes | 64 per collection |
| Literal argv | 64 strings, 4096 bytes each, 64 KiB total; NUL rejected |
| Paths | 4096 UTF-8 bytes; nonempty local paths without NUL or URL schemes |
| Active processes | 8 per independent editor, including generations still stopping |
| Pending command requests | 16 per package; 64 per editor, including calls waiting for initialization |
| UTF-8 frame / outbound bytes | 4 MiB per frame and package queue; 16 MiB queued per editor |
| Startup / command deadline | 5 seconds / 60 seconds |
| TERM grace / force-reap observation | 2 seconds / 2 seconds |
| Stderr ring | 128 KiB per active package |
| Native activation observations | First 4096 ordinary text buffers in stable native identity order; explicit notice on overflow |
| Native plugin service | At most 32 items or 2 ms per pass, whichever comes first; scheduling ceilings, not host timing guarantees |

Admission validates all declarations atomically before native discovery. It
does not promise that a program exists, is executable or can complete a future
handshake. Package distribution, downloads and installation are separate work.
