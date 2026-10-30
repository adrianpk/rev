# Pi Integration Model

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.1
Specification: [Pi Integration](editor-ai-pi-integration.md)
Parent model: [AI Commands Model](editor-ai-commands-model.md)

## Ownership

The native API backend owns its own provider transport, credentials and
context loop. The Pi backend owns an external process adapter; Pi owns its
provider calls, authentication and admitted discovery/context machinery.
Using an API key inside Pi does not transfer those owners to native API code.

rev owns invocation identity, the editor snapshot, read/transmit grant, target,
presentation and canonical result application in either path. Effective Pi
capabilities must be checked independently of requested cwd or prompt text.

## Persistent semantic records

| Record | Fields | Contract |
| --- | --- | --- |
| Pi adapter settings | Executable reference, supported protocol/version policy, declared arguments/environment, provider/model reference, tool/resource policy and operation/session policy. | Native configuration for an optional external backend; exact schema and keys remain open. It is not native API endpoint/key configuration. |
| External authentication selection | Opaque choice of the Pi-owned provider/authentication context and readiness. | Pi resolves its own credentials. rev does not import the token/key or duplicate its authentication store. |
| Session retention policy | Fresh/reused operation scope, directory binding, optional persistence and finite retention. | Still a design decision; no session archive or location is selected here. |

A saved AI command may select an admitted model within its resolved backend.
No model spelling or credential type changes backend identity. A user may
independently configure Pi and native API to use equivalent provider access;
the adapter must not silently copy credentials or treat that as one execution
implementation. Configuration alone does not launch Pi during ordinary editing.

## Ephemeral records

| Record | Fields and observations | Lifetime and authority |
| --- | --- | --- |
| Adapter invocation | Parent invocation identity, selected Pi settings observation, directory/grant, target and editor snapshot. | One explicit editor operation. The native API path is not used for transport or hidden failover. |
| Owned child | Process identity, admitted executable/version/capabilities, cwd/environment policy, streams and lifecycle state. | Created or reused only under the selected Pi lifecycle; unrelated user processes are outside ownership. |
| Protocol request | Unique operation/request identity, command kind, bounded payload and response/completion state. | Typed admitted RPC operations; no general shell or model-directed RPC dispatch. |
| Editor input snapshot | Source buffer/revision, point/selection, admitted unsaved contents and provenance. | Distinct from disk text. Supplying it does not save the document. |
| Delegated context grant | Root/explicit sources, read/transmit/tool/resource policy and budgets. | Pi constructs context inside an enforced grant. Cwd alone is insufficient enforcement. |
| Harness observation | Reported tool/context events, known origins, accounting and completeness. | Do not label unobserved Pi context as native-assembled or fully inspected. |
| Result proposal | Parent invocation/target, admitted completed explanation or generated text and failure state. | Native application remains subject to original target/revision/read-only/undo checks. |
| Session binding | Pi process/session identity, directory/configuration observation and retained-context policy. | Context cannot silently cross a changed directory or incompatible settings boundary. |

Actual provider tokens, API keys and native credential material are not fields
of adapter requests, editor snapshots, results or diagnostics. Pi may resolve
authentication through its own files or environment according to the admitted
child policy; authentication material is not model context.

## Lifecycle

An explicit command captures the parent invocation and resolves its Pi
settings. The adapter checks the executable/protocol, effective capabilities
and context policy before admitting work. Pi receives the instruction and
editor snapshot and manages its own model/discovery iterations. A request
acceptance response is distinct from operation completion.

The adapter continuously handles bounded protocol events, cancellation and
child failure while ordinary editor input continues. Only an admitted complete
result can proceed to the parent publication policy. Streams, retries and late
events cannot reset the invocation grant or publish twice. Cancellation fences
only owned work; already published edits remain governed by normal undo.

Fresh versus reused children and in-memory versus persistent Pi sessions are
open choices. Reuse must check directory, configuration and invocation binding.
Closing a child must not close an unrelated Pi session or alter its settings.

## Readiness

Finalize the specification's process/session/authentication references,
effective tool/path/resource enforcement, snapshot encoding, wire/event
compatibility, finite limits and result contract before slicing. This model
does not add a generic harness plugin API, a secret-import feature or native
subscription transport to rev.
