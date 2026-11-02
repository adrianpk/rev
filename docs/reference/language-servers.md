# Language servers

Optional installed stdio servers supply semantic completion through the native
popup, canonical buffer and undo owner. LSP is disabled by default. The default
Rust entry runs `rust-analyzer` with no arguments and uses `Cargo.toml` as its
root marker. Registration stores existing executables; no server is downloaded.
See [Use a language server](../how-to/use-language-server.md) for setup and the
[configuration catalog](defaults.md) for every setting.

## Activation and ownership

| Command | Effect |
| --- | --- |
| `lsp-mode` | Toggle optional LSP for the session; disabling retires work and stops all owned servers. |
| `lsp-status` | Observe the current file's transport state: disabled, excluded, unavailable, starting, ready, missing, failed, limited or stopping. |
| `lsp-disconnect` | Exclude the canonical buffer across its views until restored or closed. |
| `lsp-connect` | Restore buffer eligibility without enabling a disabled global mode. |
| `lsp-restart` | Replace the eligible current root's connection; preserve other roots, exclusions and global mode. |
| `lsp-setup` | Choose a language and existing profile, new installed executable, or native-only source; save and apply the admitted settings. |
| `lsp-reload-settings` | Read the selected configuration root and includes, applying only LSP settings and file associations. |
| `lsp-servers` | Open a read-only source/catalog snapshot without running helpers or checking remote versions. |
| `lsp-select-server` | Choose a compatible configured server or native-only source for any valid language identity. |

Eligible file activation uses `lsp.preferred` when present: a compatible server
name selects that profile; `false` selects native-only. Otherwise it selects the
first matching server name in lexical order. Language identities and associations
do not require a bundled grammar.
One frozen server configuration and canonical root identify one owned connection.
Split views and canonical file aliases share the document; other buffers under
the same root share the process. Manual exclusions remain buffer-local.

Setup and explicit reload preserve the session's global LSP mode, exclusions,
text, history, selections and startup interaction profile. Source changes retire
old activations before new requests; unchanged connections remain usable.
The writer preserves comments, includes and root symlinks, validates the complete
candidate and refuses concurrent edits to any captured source. A reported
post-publication durability warning means the new settings were published.

Root lookup chooses the nearest configured ancestor marker. An explicitly
opened containing directory bounds the search and supplies its fallback. Files
outside it, or individual files without a marker, use their containing directory.
An empty marker array also selects the containing directory. Detection cannot
enable a disabled client.

Healthy servers remain after the last buffer closes by default.
`shutdown_on_last_buffer = true` requests shutdown instead. Missing/failed
connections remain retained until explicit restart or disable/enable; normal
activation does not create a crash restart loop. Shutdown sends shutdown/exit,
then terminates and reaps owned children after its finite deadline. Healthy idle
connections block on IO; disabled or idle completion creates no polling timer.

## Synchronization and completion

The client admits the LSP 3.17 stdio subset with typed JSON-RPC framing, negotiated
UTF-8 or UTF-16 positions, and full or incremental document synchronization.
A server must support document open/close and changes. Unsaved canonical text
is sent before its completion request. Wire versions also carry connection,
activation and source-revision identity. Save notifications follow only
successful ordinary saves and the negotiated server capability.

Admitted semantic candidates take precedence. Valid-empty results, no supported
items, missing capability, errors, EOF, timeouts and excluded/unavailable servers
select current native fallback. Native support is detailed in
[Completion](completion.md). Transport readiness alone does not imply semantic
candidates or a fully loaded server workspace. A semantic source needs no
bundled grammar; adding a server does not add syntax coloring.

The common popup retains its filtering, navigation and candidate limit. Semantic
matching uses `filterText` or label, prefix before subsequence, then server sort
keys/order. Supported primary forms are plain insertion, TextEdit,
InsertReplaceEdit and admitted CompletionList defaults. Trigger characters can
request before the ordinary prefix minimum; incomplete lists request again on
changed input.

Only acceptance resolves an eligible selected item. The advertised mutable
resolve field is `additionalTextEdits`; primary insertion and matching/order
remain fixed. Additional edits must be valid, disjoint, current-document ranges.
The whole primary/import batch prepares against one originating revision and
publishes as one undo transaction. Failed resolution offers native fallback
without a primary-only fragment. Read-only, changed or retired targets refuse
the entire batch. Edits, motion, cancellation, exclusion, restart and disable
retire pending work; late replies cannot reopen a cancelled popup.

Snippets, indentation adjustment, command-dependent items and cross-document
edits are rejected. Server `workspace/applyEdit` is denied and server commands
are not executed. Configuration sections, workspace folders and progress-token
requests receive bounded replies. Unsupported requests return MethodNotFound.
Diagnostics, logs and progress notifications add no diagnostic UI or edit authority.

## Finite limits

| Resource | Limit |
| --- | --- |
| Frame header / JSON body | 16 KiB / at most 8 MiB |
| Queue per direction and connection | 64 messages and 16 MiB |
| Retained stderr tail | 64 KiB per connection |
| Synchronization snapshots | 8 MiB per document and 32 MiB total |
| Retained connections / configured server entries | 8 / 32 |
| Pending requests | At most 16 per connection, one completion per buffer |
| Candidates / label / primary insertion | 200 / 512 bytes / 64 KiB |
| Additional edits / total inserted text per item | 32 / 256 KiB including the primary |
| Configuration request items / retained trigger characters | 64 / 64 |
| Default initialize / request / shutdown deadline | 10000 / 2000 / 2000 ms |

The [configuration catalog](defaults.md) gives configurable ranges. Overflow,
malformed frames, EOF and deadlines make the connection unavailable; they do not
silently drop document changes while claiming current synchronization.
Oversized buffers stay editable without sending a truncated document.
Completion and connection state remain session-local, without a second document
store or background project index owned by rev.
