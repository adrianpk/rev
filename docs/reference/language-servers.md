# Language servers

Optional installed stdio servers supply semantic completion through the native
popup, canonical buffer and undo owner. LSP is disabled by default. The default
Rust entry runs `rust-analyzer` with no arguments and uses `Cargo.toml` as its
root marker. Registration reuses existing executables. Explicit managed Rust, Go
and web installation can acquire and verify servers without enabling analysis.
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
| `lsp-setup` | Choose a language and installed profile, supported managed recipe, or native-only source; save and apply the admitted settings. |
| `lsp-reload-settings` | Read the selected configuration root and includes, applying only LSP settings and file associations. |
| `lsp-servers` | Open a read-only source/catalog snapshot without running helpers or checking remote versions. |
| `lsp-select-server` | Choose a compatible configured profile, ready managed installation or native-only source. |
| `lsp-install-server` | Acquire and verify managed Rust, Go, TypeScript/JavaScript or HTML/CSS/JSON, or reuse a ready installation; save its choice when configuration sources still match. |

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

## Catalog observations and setup presets

`lsp-servers` creates a read-only snapshot through the normal buffer/view owner.
It lists at most 128 language rows, all admitted profiles (at most 32), and the
finite Rust, Go, TypeScript/JavaScript/TSX, HTML/CSS/JSON, Markdown, Ruby, Python and
Odin recipe metadata. The visible result is bounded to 64 KiB. Org and arbitrary
custom languages retain generic registration without a managed recipe.

The catalog's managed acquisition target is Linux x86-64. Generic installed-server
registration is independent of that target. Missing optional prerequisites and
unsupported recipe platforms do not block external server selection. Local PATH
inspection checks at most 128 entries without running them; relative launch/search
paths report that a client working root is required. Environment values are never
included in the snapshot. Executable prerequisites do not verify runtime features,
transport readiness or semantic quality. Verified managed facts are listed separately
with version, readiness, installation ID and selected configured profiles.

The `recipe:<launch-profile>` choices in `lsp-setup` reuse an observed installed
executable. A configured profile with that ID remains authoritative. New presets
use their recipe's command/arguments while preserving the selected profile's root
markers, initialization options, settings and literal environment overrides.
Arguments and markers remain editable before publication. Known associations are
reused; no registry fact or recipe creates a language allowlist. Viewing the catalog
and normal startup never acquire or update packages.

## Managed Rust installation

`lsp-install-server` offers `rust-analyzer` acquisition on GNU/Linux x86-64.
The equivalent setup choice is language `rust`, then `install:rust-analyzer`.
An explicit operation resolves one exact official stable release, checks its
asset URL and length, verifies the release SHA-256, and unpacks one executable.
The digest comes from the same official release channel; it proves artifact
integrity, not independent publisher identity or semantic quality.

Only this operation requires installed `curl`, `gzip` and `sha256sum`. Missing
Rust toolchain or sources are reported separately; rev never installs or upgrades
a compiler. The candidate must complete the native client's initialize/shutdown/
exit exchange in an owned empty root without user documents. Installation does
not change global LSP enablement or buffer exclusions. Use `lsp-mode` separately
when continuous analysis is desired.

## Managed Go and web installation

`lsp-install-server` also offers `gopls`, `typescript-language-server` and
`vscode-langservers-extracted` on Linux x86-64. Shared packages ask which
language to select. The equivalent setup choice is a compatible language,
then `install:<recipe>`. Generic registration remains available for every
admitted language identity.

| Recipe | Optional tools | Acquisition and verification |
| --- | --- | --- |
| `gopls` | Go | Resolve an exact stable module version through the official Go proxy, verify checksum-database module provenance, build with owned `GOBIN`/module/build/temp caches and `GOTOOLCHAIN=local`, and compare the binary's module identity. |
| `typescript-language-server` | Node and npm | Resolve exact server and stable TypeScript 6 versions, install into an owned npm prefix with scripts disabled and retain the package lock. One `--stdio` profile serves TypeScript, JavaScript and TSX. |
| `vscode-langservers-extracted` | Node and npm | Resolve an exact package version into one owned npm prefix. Three independent `--stdio` profiles serve HTML, CSS and JSON; each must pass its own probe. |

An insufficient Go compiler or incompatible Node/npm is an actionable failure;
rev never downloads another compiler or upgrades the shared runtime. npm uses
owned configuration and cache/temp roots, exact saved dependencies, strict engine
checks and scripts disabled. Lock admission requires exact package identities,
registry HTTPS sources and SHA-256/512 integrity. The lock, its SHA-256 and concrete
runtime/package versions are retained. Go retains module sums, compiler identity
and binary build information. Package provenance is distinct from native release
artifact digests. Every launch completes the native empty-root probe without
user documents before its installation becomes ready.

Selecting a shared TypeScript profile inherits the explicitly chosen language's
current options. Independent web profiles retain each language's settings.
Reusing a ready record for another language preserves that language's current
settings and known associations. Acquisition and reuse preserve runtime analysis
enablement and ordinary editing.

## Managed storage and publication

Payloads live under the resolved data root at
`lsp/installations/<recipe>/<installation-id>/`; a candidate is created at its
final unique path and never moved. `lsp/installations.toml` stores schema-1 facts,
with at most 32 records and 2 MiB. It records exact version, target, relative
payload and launch paths, source, digest or package evidence, runtime identity, readiness
and creation/verification timestamps. It contains no language preferences.
The cache root's `lsp/` holds disposable owned acquisition directories; state
`lsp/mutation.lock` serializes settings changes. A shared data-directory lock
also excludes concurrent managers using different state roots.

One native management worker owns acquisition, verification and publication;
its phase appears in `lsp-servers`. Native artifacts are bounded to 256 MiB,
candidate payloads to 1 GiB, disposable cache to 2 GiB, and captured helper output
to 1 MiB with at most 64 KiB retained stderr. The total operation deadline is
900 seconds. Helpers use structured argv, owned directories and finite deadlines.
They execute with user permissions and are not a security sandbox. Confirmed
editor exit cancels the worker; owned groups receive a two-second stop grace
before bounded forced stop/reap, with cleanup uncertainty reported separately.

Ready facts are published before the independent configuration transaction.
It preserves known associations and selected profile options/environment, and
creates another profile if a preset name already belongs to custom settings.
A source conflict leaves the payload ready and unselected. Use `lsp-servers`
to find its ID, then choose `ready:<id>` in `lsp-install-server`, `lsp-setup`
or `lsp-select-server`. Reuse checks ready facts under the mutation locks and
runs no acquisition helpers or probe. Configured commands remain authoritative;
there is no implicit managed/PATH precedence.

A failed candidate remains incomplete and preserves earlier ready installations.
Preparing records from an interrupted job are shown as incomplete and never
resume automatically. Invalid, unknown-schema, duplicate, escaping-path or
unverified-ready registry data blocks managed changes while ordinary editing,
existing-server registration and configured external servers remain usable.

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
