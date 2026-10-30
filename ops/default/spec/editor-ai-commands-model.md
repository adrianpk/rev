# AI Commands Model

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.2
Specification: [AI Commands](editor-ai-commands.md)
Optional adapter: [Pi Integration](editor-ai-pi-integration.md), [Pi Integration Model](editor-ai-pi-integration-model.md)
Existing owners: [Editor Model](rev-editor-model.md), [Configuration Model](editor-configuration-model.md)
Directory prerequisite: [Directory Opening](../ticket/open/20261028094803-open-directory-targets-from-cli-and-find-file.md)

## Ownership

rev owns command resolution, invocation identity, context grants, editor source
observations and native edit publication. Direct API execution additionally
owns provider IO and context discovery. Pi execution delegates provider
authentication, discovery and context management to the external harness
through a checked adapter. The model can request admitted read context and
generate content; it does not own filesystem/editor authority. Saved prompts
remain declarative records, not executable plugins.

Canonical buffers and history remain under existing editor ownership. Context
snippets and result views are observations/proposals, never a second editable
source database. A native job facility reused from another capability does not
import that capability's shell, write or executable-block operations into AI.

## Persistent records and semantic fields

The order below is semantic, not a selected TOML/JSON/database representation.
Exact storage, keys, version handling and activation behavior remain open.

| Record | Fields | Contract |
| --- | --- | --- |
| Backend selection | Native direct API or Pi execution, with a reference to the appropriate settings. | Selection is separate from authentication. Exact configuration scope/default remains open; failure never silently switches paths. |
| Native API settings | Endpoint, native credential binding, default model, admitted parameters and protocol/capability profile. | Used by rev's native transport and context loop. Does not require Pi or its authentication. |
| Pi adapter reference | External process/integration settings and Pi-owned provider/model/authentication selection. | Separate integration record. Does not contain a copy of rev's native key or make Pi-managed authentication native API support. |
| AI command definition | Stable name, prompt/template, input scope, context policy, output behavior, optional binding and model override. | Validated declarative data; invokes only the native AI engine. No credential or executable handler body. |
| Context policy | Explicit source selectors and/or discovery admission, boundary selection, allowed operations, transmission/inclusion rules and finite budgets. | Separate from target authority; exact policies and defaults are unresolved. |
| Native credential binding | Opaque reference to the native API secret source and transport use. | Native transport resolves it separately. Pi credentials remain under Pi ownership; neither store is imported into the other. |

An endpoint/model override is not inferred from contextual content. Model
overrides resolve within the selected backend; incompatible names do not cause
backend switching. A saved model override does not introduce command-level
credential/endpoint overrides. Name collision and keymap resolution must be
reviewed with the native catalog and reserved cancellation behavior.

Pi may authenticate through a supported subscription, API key or another
method it supports. That choice is internal to the Pi execution path. rev's
native API support remains its own transport, credentials and discovery loop,
with independent implementation and acceptance. It is not a request sent to Pi
with an API key. rev does not export Pi tokens to emulate its native backend.

Existing rev preferences load at startup. Saving a command during a session
therefore needs an explicit activation/persistence contract: a native definition
change must not silently claim that all startup configuration is hot-reloaded.
No definition, data/cache/state directory or credential file is created merely
because ordinary editing starts without AI configuration.

## Ephemeral invocation records

| Record | Fields and observations | Lifetime and authority |
| --- | --- | --- |
| Invocation | Identity, builtin/saved/direct command kind, instruction, definition observation, selected-backend/settings snapshot, target, read grant, budget and state. | One explicit editor action; backend iterations do not make a persistent user conversation. |
| Target snapshot | Native buffer identity/revision, selection/point/buffer kind, validated native range/anchor and relevant original-content observation. | Chosen by invocation; revalidate through native edit admission before publication. |
| Read grant | Root/explicit paths or regions, canonical path/link policy, allowed operations and transmit policy. | Native authority; cannot be extended by prompts, source text or model calls. |
| Context observation | Origin (selection/buffer/disk/structure/optional LSP), document/path identity, revision or checked file observation, range, content and completeness. | Identify the actual source version; never mislabel disk text as current unsaved buffer content. |
| Context manifest | Editor-supplied observations, read/transmit grant, selected backend, accounting and known completeness. | Direct API records its assembled context. Pi reports admitted discovery to the adapter's supported extent; rev must not pretend it assembled or fully inspected Pi's final context. |
| Discovery request | Invocation/iteration identity, requested list/search/read kind, typed parameters and requested bounds. | Request only; validate against the native grant before IO. |
| Discovery result | Returned observations/matches, source identity, consumed resources, completeness and diagnostic. | Read data only; no side channel to shell, native command dispatch or another feature's job tools. |
| Backend exchange | Native provider-request or Pi operation identity, compatibility observation, admitted input, response state and resource use. | Native API performs model requests itself; Pi owns its model exchanges. Credentials remain with the appropriate owner and budgets do not reset on retry. |
| Generated result | Explanation or generated/transformed text, invocation/target reference, completion/usability state and provenance. | Proposal/read-only output until the selected native presentation/application policy admits it. |
| Edit publication | Invocation identity, checked target revision/range, admitted replacement/insertion and normal history result. | Publish at most once through the canonical edit owner; never apply another file named by the model. |

Native source positions retain the editor's byte/grapheme/display distinction.
A Tree-sitter range must match its actual grammar/source revision. Optional LSP
coordinates must be checked and mapped rather than substituted for native text
positions. Their absence does not remove the native file/text discovery path.

## Invocation state and failure boundaries

A conceptual invocation progresses through capture, optional discovery/model
exchanges, result-ready and presentation/application. Depending on the selected
policy it may await explicit acceptance; this draft does not force preview or
automatic application. Successful explanation ends without a source edit.
A successful edit ends after one normal native publication.

Failure, unavailable capability, insufficient context, timeout, rejection and
cancellation are explicit outcomes. Until publication, they preserve source.
After a committed edit, normal undo applies; the editor cannot report a false
rollback solely because a cancellation arrived late. Cancelled/replaced jobs
must fence late responses and cannot publish duplicated or redirected results.

Ordinary editing remains responsive to native input while provider or harness
work is active. The job/deadline/cancellation implementation uses actual native
owners. Pi process/protocol work belongs to its adapter; another feature's
commands or a harness's default tools do not enter AI authority.

## Dependencies and file observations

The directory-opening ticket is a required predecessor of the complete AI
experience: it establishes explicit roots and file-parent fallback. AI still
needs its own choice for a deeply nested individually opened file and any
read/transmission override. There is no implicit ancestor search outside a grant.

Re-evaluate available native jobs and structural APIs after their owning work
is delivered. Existing configuration, command maps and checked edits are reused;
AI adds a validated data dispatch layer rather than pretending those owners
already support user-defined commands. LSP remains a separate optional draft.

Direct API operation does not require Pi. Pi operation does not require native
API configuration. The optional adapter owns effective harness tool/resource
admission; a directory supplied as cwd is not an enforced filesystem boundary.
The directory prerequisite and checked editor snapshot apply to both paths.

## Data and diagnostic boundaries

Provider secrets never enter command prompts, context results, reusable command
files or visible traces. Diagnostics identify native failures without echoing
credential-bearing settings. Contextual source, filenames, search matches and
model text cannot change provider destination, target identity or tool grants.
The data policy must explicitly cover locally readable private/generated/
ignored/binary/large content before implementation defaults are frozen.

Any optional request/result retention needs an explicit policy and bounded
location/lifetime. Saved definitions are not saved conversations. No replay,
telemetry store or request archive is selected by this model.

## Decisions still required

- Definition/backend-selection storage, schema version and activation.
- Native provider/credential configuration independently of Pi settings/authentication.
- Stable name/binding resolution and collision handling with compiled commands.
- Missing-selection and output acceptance/presentation contracts.
- Canonical roots, links, ignores, explicit grants and transmission consent.
- Exact discovery schemas/search semantics, resource accounting and finite limits.
- Native API compatibility/streaming and separate Pi process/RPC/context contracts.
- Structural context admission and any optional LSP usage.
- Stale-target policy, cancellation fencing, result retention and idempotent apply.

These records preserve the proposed product intent; they do not choose a
protocol implementation, add a dependency, grant filesystem execution or
create an approved implementation slice map.
