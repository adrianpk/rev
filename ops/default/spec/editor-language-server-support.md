# Language Server Support

Status: Approved
Date: 2026-10-27
Reviewed: 2026-10-31
Approved: product owner, 2026-10-31; implementation, preparation commit/push and ordered Looper execution requested
Format: OpenSpec capability proposal
Source: product-owner discussion and approved implementation scope
Delivery boundary: successor to native completion in the approved Looper sequence
Completion owner: [Native Autocompletion](editor-autocompletion.md)
Installation owner: [Language Server Installation](editor-language-server-installation.md)
Model: [Language Server Support Model](editor-language-server-support-model.md)
Controls: [Editor Configuration](editor-configuration.md)
Readiness: ready for development under the bounded delivery profile below
Plan: [Optional LSP Delivery](../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../tracker/rev-language-server-support.md)

## Purpose

Add optional native LSP client support for semantic completion. Keep the client
in rev's Rust binary and language servers outside it, with no editor addon or
public programmable-provider layer required.

## Confirmed direction

- LSP is optional and must not make ordinary editing depend on a server.
- LSP is disabled by factory default and can be enabled through TOML settings
  or compiled M-x commands.
- Once enabled, opening or activating a compatible file automatically requests
  its configured installed language server unless the buffer is manually excluded;
  no per-file connect command is needed.
- Compiled M-x commands can disconnect and restore only the current buffer.
  An excluded buffer stays on native completion until explicitly restored.
- Compatible buffers sharing the same server configuration and resolved root
  reuse one connection/process within the current rev session.
- By default, closing the last associated buffer retains the server until LSP
  is disabled or rev exits. Shutdown on last-buffer close is configurable.
- For an individually opened file with LSP enabled, root detection uses
  appropriate language-specific project markers and falls back to the file's
  containing directory. Detection does not enable LSP by itself.
- Enabled, operational LSP takes precedence over Tree-sitter completion.
- Disabling or losing a server leaves native fallback available within its
  stated coverage: Tree-sitter for selected languages and native Rust Org contexts.
- A valid empty completion response also selects the admitted native
  fallback without treating the server as unavailable.
- Completion is the selected first LSP capability. Diagnostics and other IDE
  features do not enter this scope merely because a server supports them.
- Rust is the primary language. The installation draft owns the secondary
  language list and easy connection of additional servers.

## ADDED Requirements

### Requirement: LSP01 Optional resource ownership

The client SHALL be disabled by factory default, with activation available
through TOML settings and compiled M-x commands. Rev SHALL own the
language-server processes it starts and keep their work optional. When LSP is
disabled, rev SHALL NOT start servers or associated
analysis work. Server absence or failure SHALL preserve normal editing and
allow the native completion owner to select its fallback.

Once LSP is enabled, opening or activating a compatible, non-excluded file SHALL request
the appropriate installed, configured server automatically. A separate manual
connection command SHALL NOT be required for each file.

Compatible buffers with the same admitted server configuration and resolved
root SHALL share one owned connection/process within the current rev process.
Opening another compatible buffer or view SHALL NOT by itself start a duplicate
server for that configuration and root.

By default, a healthy owned connection SHALL remain available after its last
associated buffer closes, until LSP is disabled or rev exits. Users SHALL be
able to select shutdown when the last associated buffer closes instead.
Closing a buffer SHALL retire its document-specific work independently of
whether the shared connection remains available.

#### Scenario: Edit with LSP disabled

- **GIVEN** a supported project and LSP disabled by the user
- **WHEN** rev starts and the user edits source files
- **THEN** no language-server process is started by rev
- **AND** editing and admitted native completion remain usable.

#### Scenario: Activate an eligible file with LSP enabled

- **GIVEN** LSP enabled, a non-excluded buffer and a compatible installed, configured server
- **WHEN** the user opens or activates a supported file
- **THEN** rev begins the server connection automatically
- **AND** no separate per-file connection command is required.

#### Scenario: Reuse a server for another compatible buffer

- **GIVEN** an active connection for a server configuration and resolved root
- **WHEN** another compatible buffer with that configuration and root is opened
- **THEN** rev attaches the buffer to the existing connection
- **AND** no additional server process is started for that buffer.

#### Scenario: Retain a server after closing its last buffer

- **GIVEN** an active connection and the default server lifetime policy
- **WHEN** its last associated buffer closes
- **THEN** document-specific work is retired and the connection remains available
- **AND** reopening a compatible file can reuse that connection.

#### Scenario: Select shutdown on last-buffer close

- **GIVEN** an active connection and last-buffer shutdown enabled
- **WHEN** its last associated buffer closes
- **THEN** rev shuts down the owned server connection
- **AND** a later eligible file can start a new connection while LSP remains enabled.

### Requirement: LSP02 Versioned nonblocking communication

Protocol communication SHALL leave terminal input responsive and use explicit
request identity, document versions, cancellation and finite resource bounds.
Document open/change/close synchronization SHALL reflect unsaved editor state.
Position conversion SHALL honor the negotiated encoding and existing checked
coordinate domains. Obsolete responses SHALL NOT apply edits to a newer state.

#### Scenario: A completion response arrives after another edit

- **GIVEN** an outstanding request for one buffer revision
- **WHEN** an edit or buffer/context change makes its response inapplicable
- **THEN** the response cannot mutate the new document state
- **AND** completion remains available through a current request or fallback.

### Requirement: LSP03 Shared completion and truthful capabilities

The client SHALL advertise only supported capabilities and deliver candidates
to the native completion owner. Accepted edits SHALL pass through canonical
range validation, read-only admission and atomic history. Unsupported server
features SHALL NOT become functional-looking command stubs.

#### Scenario: Use semantic candidates in the native interface

- **GIVEN** an enabled, operational server supporting completion
- **WHEN** the user requests completion and accepts a valid candidate
- **THEN** the native interface applies its admitted edits coherently
- **AND** undo restores the prior document state through the ordinary history.

### Requirement: LSP04 Native controls and ordinary servers

Appropriate TOML preferences and compiled M-x commands SHALL control server
selection, activation and runtime connection state. Additional ordinary LSP
servers SHALL be connectable without an editor plugin per language. Missing
server/runtime requirements SHALL produce clear recoverable feedback.
The factory grammar catalog SHALL NOT act as an allowlist for compatible
external language servers or their declared languages.

#### Scenario: Select an additional installed server

- **GIVEN** an external LSP server compatible with the supported client contract
- **WHEN** the user supplies its admitted executable and launch settings
- **THEN** rev can connect through its native client without a language addon
- **AND** the declared language is not assumed to have a bundled grammar.

### Requirement: LSP05 Automatic roots for individual files

For an individually opened file with LSP enabled, rev SHALL detect its project
root automatically using appropriate language-specific markers, such as
`Cargo.toml`, `go.mod` or `package.json`. If no suitable marker is found, the
file's containing directory SHALL be used as the fallback root. This workflow
SHALL NOT require per-project configuration or workspace registration.

Root detection SHALL NOT activate a disabled client. Opening an individual
file under factory defaults SHALL preserve ordinary editing without starting
a language server.

#### Scenario: Resolve a root for an individually opened file

- **GIVEN** an individually opened file and LSP enabled
- **WHEN** rev prepares its eligible configured server connection
- **THEN** root detection uses an appropriate language-specific project marker
- **AND** absence of a suitable marker falls back to the containing directory.

### Requirement: LSP06 Manual buffer exceptions

Compiled M-x commands SHALL allow the user to disconnect the current buffer
from LSP and explicitly restore its eligibility. A disconnected buffer SHALL
remain excluded from automatic connection on later activation, until restored
or closed. The exception SHALL be buffer-local and session-local.

Disconnecting SHALL retire that buffer's pending LSP work and select the native
completion path within its coverage, including the native Org source when
applicable. Other buffers SHALL retain
their shared connection. If no attached buffers remain, the accepted server
lifetime policy SHALL apply. Restoring eligibility SHALL resume normal automatic
activation only when the client is enabled and prerequisites are satisfied.

#### Scenario: Disconnect one buffer while another shares its server

- **GIVEN** two buffers sharing an active server connection
- **WHEN** the user disconnects the current buffer through M-x
- **THEN** that buffer uses admitted native completion and rejects its retired LSP work
- **AND** the other buffer remains attached to the shared server.

#### Scenario: Preserve a manual exception across buffer activation

- **GIVEN** an open buffer manually disconnected from LSP
- **WHEN** the user leaves and later activates that buffer again
- **THEN** it remains disconnected without starting or rejoining an LSP connection
- **AND** explicit restoration returns it to the normal activation policy.

## Delivery profile

This profile closes implementation readiness within the approved scope. It
describes the delivery to build, not current shipped commands or settings.

### Transport and synchronization

Use LSP 3.17 JSON-RPC over owned child stdio with Content-Length framing.
Launch executable plus argv directly, never through a shell. Keep protocol
stdout separate from bounded stderr diagnostics. Use `serde_json` and typed
first-party payloads for the supported subset; no general LSP framework,
asynchronous runtime or agent harness is required.

Advertise UTF-8 and UTF-16 position encodings, honor a supported selection and
default to UTF-16 when omitted. Reject unsupported encodings for that connection.
Convert positions against the exact canonical revision, including CRLF and
non-BMP characters; wire offsets are not display cells or grapheme offsets.
Admit full or incremental text synchronization. Incremental edits use
pre-change positions from the canonical transaction. A server without an
admitted synchronization contract is unavailable for completion.

Support initialize/initialized, didOpen/didChange/didClose, completion,
completionItem/resolve where advertised, $/cancelRequest, shutdown/exit and
negotiated save notifications. Document versions are monotonic within a
connection/document generation. Changes precede completion for their revision;
never save a source to synchronize it.

Answer workspace/configuration from admitted settings and
workspace/workspaceFolders with the resolved root. Acknowledge bounded
workDoneProgress/create requests. Return MethodNotFound for unknown requests.
Reject workspace/applyEdit and showMessageRequest without modifying source or
inventing user actions. Bounded log/progress/diagnostic notifications do not
enable a diagnostics UI. Do not advertise dynamic registration.

### Candidate and edit admission

LSP supplies proposals to the shared completion session. Accept plain insertion
text, TextEdit, InsertReplaceEdit and supported CompletionList item defaults.
For InsertReplaceEdit choose its replacement range explicitly and validate
point/prefix containment. Use filterText/label and server sortText/order within
the common prefix/subsequence match classes. A valid empty result, no admitted
items, unavailable capability or server failure selects admitted native fallback;
keep those outcomes distinct. Incomplete lists request current server results
when the prefix changes instead of assuming local filtering is exhaustive.

Advertise snippetSupport=false and omit explicit snippet-format items; never
insert placeholder syntax literally. Resolve the selected item at most once
when required, then revalidate revision and activation/session/connection
generations. Admit non-overlapping additionalTextEdits in the current document,
such as imports, as one transaction with the primary edit. Advertise only
resolve properties actually supported. No cross-document edits, executeCommand,
commit-character acceptance or snippet expansion. Items requiring unsupported
behavior are not presented as working operations.

### Roots, settings and commands

Default `lsp.enabled = false` and `lsp.shutdown_on_last_buffer = false`.
`lsp.servers.<name>` declares `languages`, `command`, `args`, `root_markers`,
`initialization_options` and `settings` as TOML data. Existing file associations
admit unbundled languages. Default Rust launch uses installed `rust-analyzer`;
other servers use explicit entries. Missing executables produce one recoverable
status without spawning on every keystroke. No installer/download is included.

Choose the nearest ancestor with a configured marker, starting in the file's
containing directory; ties use marker order. A containing explicitly opened
tree root bounds upward lookup and is the fallback if no marker is found within
it. A file outside that root uses the individual-file policy: nearest marker
or containing-directory fallback. Empty markers use the containing directory.
Canonical file/root identities prevent duplicate connections through aliases.
No project registration, repository source scan or project-local config is needed.

Marker suggestions are Rust Cargo.toml; C compile_commands.json/compile_flags.txt;
Go go.mod/go.work; JavaScript/TypeScript tsconfig.json/jsconfig.json/package.json.
HTML/CSS/JSON can use package.json and TOML Cargo.toml. Suggestions do not
configure or start more servers. Explicit entries may define different markers.

Compiled commands: `lsp-mode`, `lsp-disconnect`, `lsp-connect`,
`lsp-restart` and `lsp-status`. Disconnect/connect affect the current canonical
buffer across its views; restore never enables globally disabled LSP. Restart
targets the current eligible connection. Status distinguishes disabled, excluded,
starting, ready, missing, failed and limited states. These commands neither
rewrite preferences nor introduce TOML hot reload.

### Resource and failure policy

Use worker IO and bounded event delivery; never block terminal input/rendering
on pipes, server waits or JSON decoding. Defaults: `request_timeout_ms = 2000`,
`initialize_timeout_ms = 10000`, `shutdown_timeout_ms = 2000`,
`max_message_bytes = 8388608`, `max_servers = 8` and
`max_pending_requests = 16`. Validate positive finite settings.
Bound frame headers to 16 KiB, queues to 64 messages and 16 MiB per connection, retained stderr to
64 KiB, sync snapshots to 8 MiB per document and 32 MiB total. Admit at most
32 server entries. Preserve common candidate limits; at most 32 additional edits
and 256 KiB total inserted text per item. Oversized documents remain editable
without sending a truncated document as if it were complete.

Keep one pending completion per buffer and one newest queued context. Edit,
view/invocation change, cancel, exclusion, connection replacement and disable
retire affected requests. Send cancellation and discard late replies even if
the text later equals its previous bytes. Overflow, malformed frames, EOF or
deadlines cause a finite unavailable state; never silently lose document changes
while claiming current synchronization. No automatic crash restart loop:
explicit restart or disable/enable permits a new launch attempt.

Healthy servers remain after last-buffer close by default; the optional policy
shuts them down instead. Disable/exit retire requests, send graceful shutdown/exit,
then terminate and reap only owned children after the finite deadline. Drain
pipes during shutdown. Reuse the existing process boundary where suitable,
without applying finite job durations to healthy persistent servers. Idle
connections block on IO rather than poll. Exit cannot hang on a server.

### Evidence boundary

Emacs/Eglot and lsp-mode remain the main behavior references; Neovim and VS Code
are secondary. This profile does not claim all their defaults are identical.
Use a bounded local protocol fixture for deterministic failures and real installed
rust-analyzer for successful unsaved semantic completion. The fixture does not
prove arbitrary-server compatibility. Record actual server/toolchain versions.

Primary reference: [LSP 3.17 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
Measure integrated executable bytes and optional server resources separately;
the grammar-only experiment excludes this client. Disabled LSP starts no
server and leaves no polling work.

## Dependencies and validation direction

The client depends on the shared completion contract, canonical documents and
revision/coordinate conversion, native settings and cancellable nonblocking
IO. It can use a compatible externally installed server; the managed installer
is not a prerequisite. The delivery profile resolves launch/root policy against
the admitted directory context without introducing workspace registration.

Validate source switching and admitted native fallback for both valid-empty and
unavailable responses with
the completion owner, unsaved document synchronization, encoding conversion,
stale replies, read-only admission, cancellation, server loss/disable/exit and
bounded queues. Include representative real Rust-server integration after
the profile above. These are delivery acceptance requirements, not
checks performed by this documentation update.
