# Native Autocompletion Model

Status: draft
Date: 2026-10-31
Reviewed: 2026-10-31
Specification: [Native Autocompletion](editor-autocompletion.md)
Semantic source: [Language Server Support](editor-language-server-support.md)
Existing owners: [rev Editor Model](rev-editor-model.md), [Editor Configuration Model](editor-configuration-model.md)

## Ownership

The native completion owner selects, presents and accepts candidates. Candidate
sources supply proposals; canonical buffers still own edits and history.
Tree-sitter supplies structural candidates within admitted grammar/query
coverage. An enabled, operational LSP supplies the preferred semantic source.
There is no ctags source or mandatory server process.

## Semantic records

| Concept | Evidence | Contract |
| --- | --- | --- |
| Completion session | Invocation, view/buffer identity, revision, point/prefix context and status. | Input/presentation lifetime is separate from source jobs and buffer ownership. |
| Source choice | Admitted language/grammar, LSP readiness, selected source and reason. | Unavailable server and valid empty result are distinct; the latter policy is unresolved. |
| Candidate | Stable session/source identity, display text, ordering/filter evidence and proposed replacement. | Display labels are not automatically insertion text; ranking remains an explicit design choice. |
| Proposed edit | Checked native range, replacement text and original buffer/revision/context evidence. | Encoding conversion and read-only/range admission precede publication; unsupported edit forms are explicit. |
| Structural observation | Buffer or disk identity, revision/observation, grammar/query coverage and extracted symbols. | Current unsaved buffers cannot be overwritten by an older disk observation. |
| Optional project observation | Chosen root, bounded file set, source generations and completeness. | Indexing policy is unresolved; no scan on every cursor movement or mandatory global index. |
| Source work | Request/job identity, cancellation state and finite input/result/work budget. | Late replies do not revive a cancelled or incompatible completion session. |

## Lifecycle and queries

An admitted trigger captures point and revision, resolves the eligible source
and requests bounded candidates. The native UI owns filtering, selection,
acceptance and cancellation. Candidate arrival is not permission to edit.
Acceptance revalidates the source target and applies an admitted edit as one
ordinary undo operation. Cancellation/failure leaves canonical text untouched.

Queries needed by the capability are prefix/context candidate lookup,
revision/grammar invalidation and source-readiness selection. Their exact
ranking, extraction scope and implementation remain open. Changing buffer,
revision or source availability must invalidate or reconcile stale observations
under a documented policy, not reuse unrelated candidates by row number.

## Storage and readiness

Completion sessions, candidate lists and source observations are disposable.
No persisted symbol index is selected. If persistent caching is later accepted,
specify its source identities, freshness, format, cleanup and migration policy
before delivery; it must not become canonical document state.

Resolve the specification's trigger policy, extraction coverage, root/index
scope, edit forms, empty-result fallback, list/work limits and controls before
planning. Exact TOML fields, protocol payloads and native types remain open.
