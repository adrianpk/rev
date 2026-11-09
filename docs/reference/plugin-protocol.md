# Experimental plugin protocol

This is the public `rev.plugins/0.1` wire contract implemented by the native
editor and `rev-plugin-sdk` 0.1.0. The [behavior reference](plugins.md) defines
configuration, editor commands, resource admission and publication semantics.
The [Rust tutorial](../tutorials/write-plugin.md) supplies a complete executable.

## Transport and correlation

Use UTF-8 JSON-RPC 2.0 objects, one per LF-terminated line, on stdin/stdout.
Embedded newlines are JSON escapes. The frame limit excludes the framing LF;
queue byte accounting includes it. Partial reads and writes are ordinary.
EOF inside a frame, invalid UTF-8/JSON, oversized frames, batches and ambiguous
envelopes retire the connection. There is no HTTP, Content-Length header or
stdout logging. Stderr is a bounded inert diagnostic stream.

| Envelope | Fields |
| --- | --- |
| Request | `jsonrpc: "2.0"`, string `id`, string `method`, `params` object or array; omitted params defaults to `{}`. Concrete methods below require objects. |
| Notification | `jsonrpc: "2.0"`, `method`, `params`; no `id` and no response. |
| Success | `jsonrpc: "2.0"`, matching `id`, `result`. |
| Failure | `jsonrpc: "2.0"`, required `id` (string or null), `error`. Correlated application errors carry the request's string ID. |

Unknown envelope and typed-object fields refuse. Methods are nonempty strings
of at most 128 bytes without control characters. Opaque IDs contain 1..=128
ASCII letters, digits, hyphens, underscores, colons or dots. Numeric request IDs
are not accepted. `result` and `error` cannot coexist. Keep IDs distinct while
pending and correlate both connection generation and ID; a late reply cannot
resolve a canceled or retired call. There is no package-to-host cancel method.

## Initialization

The host sends `initialize` before any package requests. Its params contain:

| Field | Wire value |
| --- | --- |
| session | Opaque editor-session ID. |
| generation | Canonical unsigned decimal string for this process generation. |
| package | Object with `id` (publisher.package) and `version` (manifest's label). |
| api | Exactly `"rev.plugins/0.1"`. |
| capabilities | Distinct array of at most ten names from the capability table. |
| units | Exactly `"unicode_scalar"`. |
| limits | Object containing every field in the finite profile below. |

Reply with the same fields and an additional `handlers` array of
`{command, handler}` objects, at most 64. `command` must be an admitted qualified
command of this package. Command IDs and opaque handler IDs must each be unique.
The host requires exact agreement with its offer, including limits, capabilities,
session and generation. The SDK's `InitializeParams::acknowledge` constructs this
agreement. A declared command without an accepted handler remains discoverable
but execution reports `command_unavailable`.

| Capability | Native surface |
| --- | --- |
| commands | `commands.list`, host `command.invoke` and `command.cancel`. Static declarations only; register/unregister schemas remain unsupported. |
| buffers | list, open, create, describe and read ordinary text. |
| display | Show ordinary text in native panes. |
| edits | Checked atomic edit batches. |
| invocation | Guarded noninteractive command calls. |
| file_lifecycle | Explicit checked save and close. |
| keymaps | Generation-owned dynamic binding/unbinding. |
| events | Subscribe/unsubscribe and host buffer notifications. |
| state | Package state when native state roots are attached. |
| messages | Schema only; not advertised or implemented. |

The native application normally offers nine capabilities. Library sessions offer
eight until `Session::attach_plugin_state(&Roots)` is called before activation.
A schema's existence does not authorize calling an unsupported method.

## Common values

Fields shown as nullable accept JSON null; optional observations generally also
permit omission. Send explicit null for clarity. `state.put.expected_version`
is required even when null. All other listed nonnullable fields are required.

| Name | Shape and constraints |
| --- | --- |
| Handle | `{session, id}`; both opaque strings from this host. Buffer and pane handles have different native owners. |
| Stamp | `{buffer: Handle, revision}`. Revisions are opaque canonical unsigned decimal strings, never JSON numbers. Do not increment or compare them arithmetically. |
| Range | `{start, end}` with unsigned scalar offsets and `start <= end`, half-open. Native checks also require bounds within the observed source. |
| Page | `{cursor, limit}`; nullable opaque cursor, integer limit 1..=256. |
| Disk | `{kind: "absent"}` or `{kind: "observed", token}`. Token is an opaque native observation, not an overwrite flag. |
| Descriptor | `{stamp, name, path, kind, read_only, modified, expected_disk, owner, scalar_len}`; nullable path and owner, booleans for flags, unsigned scalar_len. Native text operations return kind `text`; schema kinds also include `terminal` and `generated`. |
| Origin | `{package, operation, depth}`; nullable package ID, opaque operation, unsigned depth at most 8. Preserve live callback ancestry. |
| Context | `{invocation, generation, stamp, pane, point, selection, prefix, origin}`; nullable stamp, pane, point, selection and signed 64-bit integer prefix. Point/selection require a stamp. Handles share its session. |
| Parameter | `{name, kind, required}`; kind is `null`, `boolean`, `integer`, `string`, `array` or `object`. Names are distinct lexical IDs. |
| Declaration | `{name, description, parameters, prefix, recording}`; prefix is `none` or `integer`, recording is `reject` or `semantic`. |
| Metadata | `{id, description, parameters, prefix, recording, available, noninteractive}`; id is a canonical native name or qualified command; flags are booleans. |
| Binding source | `{command, provenance, package}`; native/qualified command, provenance `user`, `plugin` or `factory`, nullable package ID. |

Package IDs and command/record identities use the [declaration grammar](plugins.md#local-manifest).
Strings called labels below have a 4096-byte ceiling; text has a 512-KiB UTF-8
ceiling. Native names and paths impose their additional behavior checks.
Canonical decimal strings represent 0..=u64::MAX with no leading zero except
`"0"`; native state record versions are nonzero and must come from observations.

Source positions count Unicode scalars: `e` plus a combining mark plus `🦀`
occupies three scalars and seven UTF-8 bytes. CR and LF each count separately.
Neither grapheme motion nor UTF-16/cell coordinates substitute for these offsets.

## Complete method catalog

P→H means package request to host; H→P means host request to package. Every
request gets one correlated result or error while its connection remains live.
`{}` means an empty object. Arrays are bounded, not streams. Method-specific
native behavior and refusal rules are in the linked behavior reference.

| Method | Direction / capability | Params → result |
| --- | --- | --- |
| `initialize` | H→P / handshake | Initialization object above → identical agreement plus handlers. |
| `buffers.list` | P→H / buffers | `{page: Page, kind: nullable ResourceKind}` → `{buffers: Descriptor[], next: nullable cursor}`. At most 256; kind null/text only in native host. |
| `buffers.open` | P→H / buffers | `{path: label}` → Descriptor. Resolve/visit a local file hidden; no implicit save. |
| `buffers.create` | P→H / buffers | `{name: label, text, read_only: boolean}` → Descriptor for an ordinary hidden unsaved buffer. |
| `buffers.describe` | P→H / buffers | `{buffer: Handle}` → Descriptor, including current source and disk observations. |
| `buffers.read` | P→H / buffers | `{buffer: Handle, range: Range, expected_revision: nullable revision}` → `{stamp: Stamp, range: Range, text}`. |
| `buffers.show` | P→H / display | `{buffer: Handle, pane: nullable Handle, placement}` → `{pane: Handle}`. Placement is `current`, `split_horizontal` or `split_vertical`. |
| `buffers.save` | P→H / file_lifecycle | `{buffer: Handle, path: nullable label, expected_revision, expected_disk: Disk}` → `{stamp: Stamp, expected_disk: Disk}`. No force or consent field. |
| `buffers.close` | P→H / file_lifecycle | `{buffer: Handle, expected_revision}` → `{closed: boolean}`. Modified close needs native consent. |
| `edits.apply` | P→H / edits | `{operation, targets: [{stamp: Stamp, replacements: [{range: Range, text}]}], origin: Origin}` → `{operation, stamps: Stamp[]}` in target order. At most 16 targets, 256 total disjoint ordered replacements, 512 KiB total replacement bytes. |
| `commands.register` | P→H / commands, unsupported | `{commands: [{declaration: Declaration, handler}]}` → `{registrations: [{id, command}]}`; at most 64. Schema only; native host returns unsupported_method. |
| `commands.unregister` | P→H / commands, unsupported | `{registrations: opaque ID[]}` → `{}`; at most 64. Schema only; native host returns unsupported_method. |
| `commands.list` | P→H / commands | `{page: Page, query: label}` → `{commands: Metadata[], next: nullable cursor}`; at most 256. |
| `commands.invoke` | P→H / invocation | `{command: native or qualified ID, arguments: object, context: Context}` → `{value: JSON}`. Requires a live causal callback and compatible captured context. |
| `command.invoke` | H→P / commands | `{command: qualified ID, handler, arguments: object, context: Context}` → `{value: JSON}`. Handler comes from initialization. A receipt is not displayed result text. |
| `command.cancel` | H→P / commands | `{invocation, generation}` → `{}` when sent as a request; also accepted as a notification without a response. Revokes the matching callback. |
| `keymaps.bind` | P→H / keymaps | `{command: own qualified ID, keys: label, context, scope}` → `{registration, status, effective: nullable BindingSource, underlying: BindingSource[], conflicts: BindingSource[], diagnostic: nullable label}`. Status is effective/shadowed/conflicting; each inspection array is at most 512, with truncation in diagnostic. |
| `keymaps.unbind` | P→H / keymaps | `{registrations: opaque ID[]}` → `{}`; at most 128, all owned by this generation. |
| `events.subscribe` | P→H / events | `{kinds: EventKind[], buffers: Handle[]}` → `{subscription, sequence: revision}`. Distinct nonempty kinds, at most four; at most 256 distinct live buffers, empty selects all ordinary text. |
| `events.unsubscribe` | P→H / events | `{subscription}` → `{}`. Only the caller's generation can remove it. |
| `messages.show` | P→H / messages, unsupported | `{text: label, severity: info/warning/error}` → `{}`. Schema only; native host returns unsupported_method. |
| `state.list` | P→H / state | `{concern, page: Page}` → `{records: [{record, version}], next: nullable cursor}`; at most 256, values excluded. |
| `state.get` | P→H / state | `{concern, record}` → `{status: "absent"}` or `{status: "present", version, value: JSON}`. Present null differs from absence. |
| `state.put` | P→H / state | `{concern, record, expected_version: nullable revision, value: JSON}` → `{version}`. Required null precondition means create only if absent. |
| `state.delete` | P→H / state | `{concern, record, expected_version: revision}` → `{}`. Requires the observed existing version. |
| `buffers.event` | H→P / events, notification only | `{subscription, sequence, generation, kind, stamp: nullable Stamp, origin: Origin}`; no result. Kind is opened/changed/closed/resnapshot. A resnapshot invalidates incomplete deltas and has null stamp. |

Dynamic `keymaps.bind.scope` is `{kind: "global"}` or
`{kind: "buffer", buffer: Handle}`. This differs deliberately from the persistent
manifest's string scope and selectors. Context names are `editor`, `vim_normal`,
`vim_insert`, `vim_replace`, `vim_visual_character`, `vim_visual_line` and
`vim_visual_block`. State concern and record are lexical IDs, never paths or a
way to select another package. JSON argument/result payloads are bounded to
512 KiB; state values to 256 KiB in JSON encoding. The transport's encoded frame
ceiling applies in addition to every logical payload ceiling.

## Wire examples

Each JSON block is one complete LF-terminated frame when sent. These illustrative
session and buffer identities must be replaced with live returned handles.
The first request reads three scalars under its original observation:

```json
{"jsonrpc":"2.0","id":"read-1","method":"buffers.read","params":{"buffer":{"session":"editor-1","id":"buffer-1"},"range":{"start":0,"end":3},"expected_revision":"7"}}
```

A source changed before that observation could be used:

```json
{"jsonrpc":"2.0","id":"read-1","error":{"code":-32000,"message":"StaleRevision","data":{"kind":"stale_revision","request":"read-1"}}}
```

Creating one absent package-state record is a compare-and-swap, not an overwrite:

```json
{"jsonrpc":"2.0","id":"state-1","method":"state.put","params":{"concern":"preferences","record":"last-query","expected_version":null,"value":{"query":"TODO"}}}
```

## Errors and recovery

An error is `{code: integer, message: string, data?: {kind, request?, handle?}}`.
The message is bounded to 4096 UTF-8 bytes and is diagnostic, not a stable enum.
`data.kind` is the stable snake_case application classification; request and
handle, when present, identify relevant context without unrelated source text.
`unsupported_method` uses -32601, `invalid_arguments` uses -32602, and other
application kinds use -32000. Standard internal JSON-RPC failures may omit data;
malformed framing can retire the process without a correlated failure.

| Error kind | Meaning and caller action |
| --- | --- |
| `unsupported_api` | Exact API/units agreement failed. Use the matching editor/SDK contract. |
| `unsupported_method` | Unknown, wrong-direction, unadvertised or currently unimplemented method. Check the actual offer and catalog. |
| `invalid_arguments` | Shape, unknown field, identity, duplicate or argument-type failure. Correct the request; do not retry unchanged. |
| `namespace_conflict` | Foreign package/handler/command ownership or conflicting identity. Use only the connected package's admitted namespace. |
| `command_unavailable` | No current accepted handler for that declared command. Inspect and explicitly activate/restart the package. |
| `missing_buffer` | Closed, foreign-session or absent source handle. Obtain a live handle; never select by stale list position. |
| `unsupported_resource` | Terminal, transient/generated or invalid text/file resource. Restrict the operation to ordinary text. |
| `stale_revision` | Source/file observation no longer matches. Re-read and recalculate a new proposal; do not refresh the old proposal's token silently. |
| `read_only` | Native target refuses modification. Retain its text and respect the current policy. |
| `invalid_range` | Reversed or out-of-bounds scalar coordinates. Recalculate against the observed source. |
| `edit_conflict` | Duplicate targets or overlapping/ambiguous replacements. Build one disjoint ordered batch. |
| `confirmation_required` | Save/overwrite/discard or other native consent is required. Use native interaction; there is no force flag. |
| `cancelled` | Live authority was revoked before completion. Stop descendants; committed work is not rolled back. |
| `limit_exceeded` | Finite payload, queue, causal-depth or collection admission failed. Reduce the operation or wait for owned work to finish. |
| `disconnected` | Connection, generation or callback is retired. Discard its authority and late replies. |
| `busy` | Native context, ingress capacity or nonblocking state lock is occupied. Retry only as a new bounded action after it clears. |
| `state_conflict` | Disk/CAS/list observation changed, or committed persistence has uncertain durability. Read current evidence before another explicit mutation. |
| `unsupported_invocation` | Initialization, captured context, native command class or input owner forbids the call. Do not substitute current focus or open a prompt. |

On event overflow, unsubscribe the paused subscription, subscribe for a new
baseline, then rebuild a revision-qualified snapshot while reconciling later
notifications. A `resnapshot` is not a complete delta stream. On uncertain state
publication, retain the reported candidate version, re-read and re-observe before
another CAS. Neither case permits blind retries.

## Finite profile

All fields are required positive integers. Byte limits are UTF-8/encoded bytes;
deadlines ending `_ms` are milliseconds, and service_micros is microseconds.
The SDK admits positive values no greater than these ceilings; the native
handshake requires exact acknowledgement of the offered profile.

| Field | Ceiling |
| --- | --- |
| packages | 64 |
| active_processes | 8 |
| commands_per_package | 64 |
| commands_total | 512 |
| bindings_per_package | 128 |
| bindings_total | 512 |
| frame_bytes | 4194304 |
| text_bytes | 524288 |
| edit_buffers | 16 |
| replacements | 256 |
| pending_per_package | 16 |
| pending_total | 64 |
| outbound_per_package | 4194304 |
| outbound_total | 16777216 |
| events_per_package | 256 |
| causal_depth | 8 |
| startup_ms | 5000 |
| command_ms | 60000 |
| shutdown_grace_ms | 2000 |
| stderr_bytes | 131072 |
| state_record_bytes | 262144 |
| state_store_bytes | 4194304 |
| state_records | 256 |
| service_items | 32 |
| service_micros | 2000 |

The [admission reference](plugins.md#admission-limits) also specifies manifest,
argv, file ingress, event reserve and cleanup-observation limits outside this
wire object. Scheduling ceilings do not guarantee elapsed latency on the host.

Return to [Reference](index.md), [plugin ownership](../explanation/plugin-ownership.md)
or the [optional User Guide entry](../tutorials/user-guide/plugins/index.md).
