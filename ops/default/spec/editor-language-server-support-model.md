# Language Server Support Model

Status: Completed
Date: 2026-10-31
Reviewed: 2026-10-31
Completed: 2026-11-01
Validated candidate: `610f2e019a52e6b1314931b11fa070d320ecf809`
Gate receipt: `YWRNPLD3FXNU6APAKXES5AS7I5`
Approved: product owner, 2026-10-31; implementation and ordered Looper execution requested
Specification: [Language Server Support](editor-language-server-support.md)
Completion model: [Native Autocompletion](editor-autocompletion-model.md)
Installation model: [Language Server Installation](editor-language-server-installation-model.md)
Existing owner: [rev Editor Model](rev-editor-model.md)

## Ownership

The optional native client owns only the servers/connections it starts and the
protocol exchanges it admits. It supplies semantic completion proposals to
the shared native completion owner. A server's capability declaration does not
activate unrelated editor behavior or grant it arbitrary command execution.
Factory activation is disabled. TOML settings and compiled M-x commands can
enable the optional client; disabled state starts no language-server process.

## Semantic records

| Concept | Evidence and fields | Contract |
| --- | --- | --- |
| Server configuration | Language match, executable/argv, admitted settings, activation, sharing scope and server lifetime policy. | Factory activation is disabled; configuration is data, and managed installation and process activation are separate operations. |
| Resolved launch | Checked executable origin, launch context, admitted arguments/environment and prerequisites. | Existing or managed servers are valid sources; failure does not prevent ordinary editing. |
| Resolved root | Source file identity, matched marker or containing-directory fallback, and canonical root identity. | Nearest marker and explicit-directory bounds follow the specification; lookup cannot enable a disabled client. |
| Buffer activation | Canonical buffer identity, inherited eligibility or manual exclusion, and activation generation. | A manual exception is buffer-local and session-local; ordinary buffer activation cannot clear it. Restoring eligibility still requires an enabled client. |
| Connection | Server configuration/root identity, attached buffers, owned process/streams, generation, capabilities and shutdown/failure state. | Share one connection per configuration/root within rev. Retain healthy idle connections by default; optional last-buffer shutdown. Failure requires explicit restart or disable/enable. |
| Synchronized document | Canonical buffer identity, checked file URI, language, source revision and protocol document version. | Unsaved text is synchronized as unsaved text; closing/failure cannot silently save it. |
| Position mapping | Source revision, native UTF-8/grapheme domains and negotiated wire encoding. | Convert and range-check before native edit admission; wire offsets are not assumed to be terminal cells. |
| Request | Connection/document/activation generation, request ID, requested capability, deadline and cancellation state. | Late/stale responses cannot publish into a newer invocation, document or manually excluded buffer. |
| Completion response | Request identity, bounded supported candidates/edits and completeness/failure evidence. | Native completion owns filtering/application/undo; unsupported forms are rejected truthfully. |

## Lifecycle and invalidation

Once the client is enabled, opening or activating a compatible, non-excluded file requests
its configured installed server automatically. An admitted activation resolves
prerequisites, obtains an owned connection and completes the selected
initialization contract. Document open/change/close
notifications track actual canonical state and monotonically qualified versions
within that connection. Completion requests capture the corresponding version.

For individually opened files, enabled-client preparation resolves the root
using appropriate language-specific project markers, falling back to the
containing directory when none is suitable. Root detection does not enable
LSP or require per-project configuration.

Connection lookup uses the admitted server configuration and resolved root
within the current rev process. Another compatible buffer or view reuses that
connection instead of starting another server. This does not introduce sharing
between independent rev processes.

Manual exclusion detaches only the selected canonical buffer, advances its
activation generation and retires its outstanding LSP work. Native completion
remains eligible within its coverage. Other attached buffers continue using
the shared connection. The exception survives view changes until explicit
restoration or buffer close; it is not written into project or user settings.
Restoration resumes inherited eligibility without enabling a disabled client.

Closing a buffer removes its attachment and retires its document-specific work.
By default, closing the last associated buffer keeps a healthy connection
available until LSP is disabled or rev exits. An optional last-buffer shutdown
policy closes the connection when its attachment set becomes empty instead.

Disable, close, cancellation, connection loss or a changed document invalidates
affected work under the accepted lifecycle policy. The client must release its
owned resources and make availability changes visible to the completion owner.
Normal editing continues; restart/retry must not reset finite request budgets
indefinitely or reuse old connection IDs as current evidence.

## Persistence and preparation

Connection state, request maps, synchronization evidence and candidate results
are session-local. Do not persist them as a second document store. User-authored
settings retain existing TOML/XDG ownership; installation payloads and records
belong to the separate installer. No client database or migration is selected.

The specification defines stdio LSP 3.17, typed JSON payloads, UTF-8/UTF-16
conversion, full/incremental synchronization, same-document completion edits,
marker precedence and finite request/queue/shutdown policies. Literal primary
and bounded additional edits use one canonical transaction; no snippet runtime
or protocol-controlled command execution is selected.

The completed [plan](../plan/rev-language-server-support.md) and
[tracker](../tracker/rev-language-server-support.md) record six integrated slices,
two authorized repairs and the successful controller-owned full gate at the
validated candidate above. Real Rust unsaved semantic completion, resolved
edits, source fallback, shared lifetime, cancellation, resource and terminal
contracts passed. The [Slice 6 closure](../report/slices/rev-language-server-support/slice-6-acceptance.md#delivery-set-closure)
holds executed evidence for LSP01-LSP06 and joint CMP01-CMP09; this model retains
the ownership contract.
