# Language Server Support Model

Status: draft
Date: 2026-10-31
Reviewed: 2026-10-31
Specification: [Language Server Support](editor-language-server-support.md)
Completion model: [Native Autocompletion](editor-autocompletion-model.md)
Installation model: [Language Server Installation](editor-language-server-installation-model.md)
Existing owner: [rev Editor Model](rev-editor-model.md)

## Ownership

The optional native client owns only the servers/connections it starts and the
protocol exchanges it admits. It supplies semantic completion proposals to
the shared native completion owner. A server's capability declaration does not
activate unrelated editor behavior or grant it arbitrary command execution.

## Semantic records

| Concept | Evidence and fields | Contract |
| --- | --- | --- |
| Server configuration | Language match, executable/argv, admitted settings, activation and sharing scope. | Configuration is data; managed installation and process activation are separate operations. |
| Resolved launch | Checked executable origin, launch context, admitted arguments/environment and prerequisites. | Existing or managed servers are valid sources; failure does not prevent ordinary editing. |
| Connection | Owned process/streams, generation, protocol readiness, capabilities and shutdown/failure state. | No process when disabled; exact sharing/restart policy is unresolved. |
| Synchronized document | Canonical buffer identity, checked file URI, language, source revision and protocol document version. | Unsaved text is synchronized as unsaved text; closing/failure cannot silently save it. |
| Position mapping | Source revision, native UTF-8/grapheme domains and negotiated wire encoding. | Convert and range-check before native edit admission; wire offsets are not assumed to be terminal cells. |
| Request | Connection/document generation, request ID, requested capability, deadline and cancellation state. | Late/stale responses cannot publish into a newer invocation or document. |
| Completion response | Request identity, bounded supported candidates/edits and completeness/failure evidence. | Native completion owns filtering/application/undo; unsupported forms are rejected truthfully. |

## Lifecycle and invalidation

An admitted activation resolves prerequisites, starts an owned connection and
completes the selected initialization contract. Document open/change/close
notifications track actual canonical state and monotonically qualified versions
within that connection. Completion requests capture the corresponding version.

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

Before planning, fix the protocol/transport profile, capability/request subset,
position encoding, supported completion edits, process-sharing scope, finite
IO/queue/time limits, cancellation and shutdown/restart behavior. These concepts
do not select a Rust dependency or promise all features of the LSP standard.
