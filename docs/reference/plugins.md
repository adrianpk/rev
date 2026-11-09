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

The commands capability enables commands.list and host-to-package command.invoke
and command.cancel. The buffers and display capabilities enable the six native
buffer operations below. The edits capability enables checked edits.apply batches.
Invocation enables commands.invoke, and file_lifecycle enables buffers.save and
buffers.close. Bindings, events, messages and state operations still return
correlated unsupported_method errors. Requests before
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

## Native text buffers and display

Buffer and pane handles contain the host session and an opaque native identity.
Names, file paths and list positions are not handles. Closed or replaced sources
invalidate their earlier handles; a later open receives a new native identity.
A process restart does not destroy a still-live buffer or change its handle.
Foreign-session handles fail with missing_buffer; foreign or closed pane targets
fail with unsupported_invocation. Responses retain their request and generation.
Queued work loses authority on cancellation, deactivation, disconnect or expiry.

| Method | Parameters and result | Native behavior |
| --- | --- | --- |
| buffers.list | page with cursor and limit 1..256; optional kind; returns buffers and next | Enumerates current ordinary text buffers, including hidden and unsaved sources. kind is null or text; terminal/generated filters return unsupported_resource. |
| buffers.open | path; returns a buffer descriptor | Reuses the canonical native file identity, including aliases and existing unsaved text. A new visit is hidden. A missing path creates an empty visiting buffer without writing a file. |
| buffers.create | name, text, read_only; returns a descriptor | Creates an ordinary hidden unsaved text buffer. Native unique-name rules apply. Nonempty output is modified and protected by native close/quit decisions. |
| buffers.describe | buffer handle; returns a descriptor | Reports the current source revision, scalar length, name, visiting path, read-only/modified flags, disk observation and package provenance. |
| buffers.read | buffer, half-open range, optional expected_revision; returns stamp, range and text | Reads exact in-memory text independently of selection or focus. A mismatched revision returns stale_revision. |
| buffers.show | buffer, optional pane, placement; returns pane handle | Displays in an existing pane or a new split. It does not select another pane. Null pane resolves the selected pane once when native service admits the request. |

Positions count Unicode scalar values. Combining marks, astral characters, CR
and LF each occupy their own source scalar; bytes, graphemes and display cells
are different units. Invalid scalar bounds return invalid_range. Reads return
at most 512 KiB of UTF-8 text, checked before copying; larger ranges return
limit_exceeded and can be split into adjacent revision-qualified reads.

Descriptors contain stamp.buffer, stamp.revision and expected_disk. Source
revision and saved/disk state are separate observations. expected_disk is absent
or a lossless opaque token for the native disk observation, with no filesystem
probe during describe/read. Its presence grants no save or overwrite authority.
List continuations contain the host session and last native identity. They
resume the current registry in stable identity order, rather than freezing a
cross-page snapshot. A foreign or malformed cursor returns invalid_arguments.

Open uses the existing native path resolver. Relative paths follow the host's
native process directory; absolute paths avoid ambiguity with the package's
manifest working directory. Empty paths, NUL and URL schemes are invalid.
Resolution and strict regular-file UTF-8 ingress run on the supervisor, with
16 MiB per new file and 16 MiB aggregate retained ingress. Exhausted aggregate
ingress returns busy. Canonical reuse is checked both before loading and before
publication, so a concurrent open cannot replace unsaved text or duplicate a
source. File observation changes during the read return stale_revision. Invalid
UTF-8, NUL-containing files and non-regular resources return unsupported_resource.

Create accepts up to 512 KiB of text and a nonempty name up to 4000 UTF-8 bytes
without control characters. Native labels add unique suffixes on collision.
Created buffers retain package provenance after process retirement and remain
available through ordinary buffer navigation. They are not generated native
inventories and do not imply a file write. Terminal buffers, native inventories
and other transient resources are excluded from the text API.

Placement is current, split_horizontal (below) or split_vertical (right).
Native pane fit and lifetime checks precede display publication. Prompts,
terminal/tree focus, retained source interactions and macro replay refuse show
with unsupported_invocation instead of transferring their input ownership.
Pass the invocation's captured pane to target that original pane after focus
changes. Reads and creation do not change the selected source or open a prompt.
Open/create/edit return busy while native departure or context publication has frozen
its source set; reads and listing remain available during those decisions.

## Atomic source edits

edits.apply takes an operation ID, targets and causal origin. Each target contains
its captured stamp (session-qualified buffer and expected source revision) and
ordered, disjoint half-open scalar replacements. Equal starts, overlapping ranges,
duplicate targets, empty replacements/targets and invalid coordinates are refused.
The request admits at most 16 buffers, 256 replacements across all targets and
512 KiB of replacement UTF-8 bytes in total. The origin package is null or the
connected package; a foreign package returns namespace_conflict. Causal depth is
bounded by the negotiated profile.

The native coordinator prepares every source candidate, history reservation,
point, mark, live source anchor and shared-view viewport before publication. It
then rechecks all sources and view membership and checks request authority at one
serialized publication boundary. Missing/closed or foreign handles, unsupported
resources, read-only targets, stale revisions, invalid ranges, capacity failure
or cancellation publish none of the batch. History and redo contents, modified
state, marks and views remain unchanged by a refused request.

The commit phase consumes reserved native state and has no recoverable partial
failure path. Each target receives one ordinary buffer-local undo unit, including
all its replacements. Undo or redo in one buffer does not replay another buffer.
The result echoes the operation ID and returns new stamps in target order;
this ID correlates the batch and does not imply a cross-buffer undo command.

Targets stay independent of active focus. Hidden/unsaved ordinary text is editable;
terminal and generated/transient resources return unsupported_resource. Existing
native views and anchors map through the replacements without selecting another
pane. Native observers receive publication only after every target and its views
are committed. Revision-bound completion and resolve work is invalidated after
success; rejected batches preserve a still-current completion.

Native departure/context publication returns busy for a new edit batch once its
source set has been frozen. Active macro recording/replay also returns busy until
semantic package-call integration is available. Queued retired/expired requests
cannot publish; cancellation after an already committed batch does not reverse
that batch. Creating, editing and undoing text never imply saving a file.

## Noninteractive command invocation

commands.list accepts page (cursor and limit 1..256) and a substring query. It
returns canonical native names and qualified package IDs with descriptions,
typed parameter declarations, availability, prefix/recording policy and an
explicit noninteractive flag. Continuations belong to the current editor session.

commands.invoke takes command, an inert arguments object and context. Context
must identify a live command.invoke callback in this generation and preserve its
origin. The host derives the next causal depth from that callback; callers cannot
reset or forge it. Depth above 8 is refused, and the existing combined pending
request ceiling can refuse a recursive chain earlier. The callback's source stamp,
scalar point/selection and pane are checked against their live native owners.
A stale revision is not refreshed. Native focus never substitutes a target.

The admitted native class consists of the eight text motions: forward-char,
backward-char, move-beginning-of-line, move-end-of-line, beginning-of-buffer,
end-of-buffer, forward-word and backward-word. They take an empty arguments object,
no prefix and an explicit captured text stamp, pane and point. They reuse native
Unicode/grapheme and wrapped-line motion on that pane while preserving selection
of another pane. Save, write, kill and quit commands return confirmation_required;
other native classes return unsupported_invocation. This API does not open a prompt.

Qualified calls validate the target manifest's typed arguments, activate lazily,
and dispatch through its admitted generation-qualified handler. The result or
typed failure is correlated back to the caller. A missing handler returns
command_unavailable; a retired callee returns disconnected. Prefix calls and
macro-active contexts remain unavailable until semantic recording integration.
Native prompts, capture/source interactions and context departure retain ownership.

## Explicit save and close

buffers.save requires buffer, expected_revision, expected_disk and an optional
path. Both preconditions come from buffers.describe. Saving the associated path
uses that exact native observation; a stale disk token returns state_conflict.
External changes are refused by the native file guard, without refreshing consent.
An unsaved ordinary text buffer can explicitly save to a resolved absent path
only with expected_disk kind absent. An occupied target returns
confirmation_required. Retargeting an already associated buffer requires the
native write-file workflow, including its bookmark loading/consent boundary.
No path means confirmation_required when the buffer has no associated file.

Read-only and unsupported resources are refused. Saves admit at most 16 MiB of
source bytes, sharing the 16 MiB aggregate retained-ingress budget with file
opens. Exhaustion returns busy before snapshot capture. The supervisor writes, flushes and synchronizes private temporary
output. Native publication then rechecks the source revision, association,
permissions and disk observation before replacement, and uses the existing
completion/reconciliation policy. The native final metadata checks, replacement
and directory synchronization remain synchronous; the service budget is not a
filesystem latency guarantee. A failed replacement preserves file bytes and
unsaved state. Uncertain completion returns state_conflict and retains the native
published-file warning; it never claims that a successful replacement was undone.
The result contains stamp and the new expected_disk observation. Source/history
are unchanged, and focus is preserved. Native departure/context decisions return
busy for save and close; retired or expired queued work cannot publish.

buffers.close requires buffer and expected_revision. Modified text and an active
clock require native confirmation; there is no force/discard option. A clean close
uses native removal, shared-view replacement, bookmark/location checkpoint and
completion/LSP cleanup. It does not delete the file or cancel another native
prompt. Closing the last ordinary text buffer retains the native scratch policy.

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
| Pending RPC requests | 16 per package; 64 per editor across both directions, including initialization waits, native queues and prepared replies |
| UTF-8 frame / outbound bytes | 4 MiB per frame and package queue; 16 MiB queued per editor |
| Text read or create / list page | 512 KiB UTF-8 / 256 descriptors |
| One edit batch | 16 buffers; 256 total replacements; 512 KiB total replacement UTF-8 bytes |
| Explicit save snapshot | 16 MiB per source |
| New-file ingress | 16 MiB per file and aggregate retained ingress |
| Startup / RPC deadline | 5 seconds / 60 seconds |
| TERM grace / force-reap observation | 2 seconds / 2 seconds |
| Stderr ring | 128 KiB per active package |
| Native activation observations | First 4096 ordinary text buffers in stable native identity order; explicit notice on overflow |
| Native plugin service | At most 32 items or 2 ms per pass, whichever comes first; scheduling ceilings, not host timing guarantees |

Admission validates all declarations atomically before native discovery. It
does not promise that a program exists, is executable or can complete a future
handshake. Package distribution, downloads and installation are separate work.
