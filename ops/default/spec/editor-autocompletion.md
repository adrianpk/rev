# Native Autocompletion

Status: draft
Date: 2026-10-27
Reviewed: 2026-10-31
Format: OpenSpec capability proposal
Source: product-owner discussion; captured for later refinement
Delivery boundary: future capability; no implementation or slicing authorized
Baseline: [rev Editor](rev-editor.md)
Related: [Language Server Support](editor-language-server-support.md)
Model: [Native Autocompletion Model](editor-autocompletion-model.md)
Controls: [Editor Configuration](editor-configuration.md)
Readiness: native interaction and source precedence confirmed; triggers, extraction scope and limits remain open

## Purpose

Provide useful autocompletion as a built-in Rust feature while preserving rev's
small CPU/memory footprint and operation without additional editor packages.

## Confirmed direction

- Completion UI, filtering, selection and edit application are part of the binary.
- An enabled, operational LSP is the preferred semantic completion source.
- When LSP is disabled, absent or unavailable, use Tree-sitter context/symbols.
- Ctags and competing local indexing engines are excluded.
- All user-facing concerns have TOML preferences, compiled M-x controls, or
  both as appropriate. Useful factory defaults require no configuration file.

## ADDED Requirements

### Requirement: CMP01 Native and shared interaction

Completion presentation and interaction SHALL be compiled Rust functionality,
usable without an editor addon, package or scripting layer. Local and LSP
candidates SHALL use the same user-facing selection/accept/cancel workflow.

#### Scenario: Complete without an editor package

- **GIVEN** rev, an admitted grammar and a file containing recognized symbols
- **WHEN** the user invokes built-in completion without an LSP
- **THEN** Tree-sitter-derived candidates use the native completion interface
- **AND** no editor package or language-server process is required.

### Requirement: CMP02 Explicit source precedence

An enabled, operational LSP SHALL be preferred. When it is disabled, absent or
unavailable, Tree-sitter SHALL supply the fallback within its admitted language
coverage. Ctags SHALL NOT be introduced. A valid empty LSP result is distinct
from an unavailable server; any fallback policy for that case remains open.

#### Scenario: Disable an optional server

- **GIVEN** a supported document and an enabled LSP connection
- **WHEN** the user disables LSP
- **THEN** completion uses the native Tree-sitter path
- **AND** the editor remains usable without that server's analysis work.

### Requirement: CMP03 Tree-sitter context and bounded indexing

Symbol extraction SHALL use included grammars and explicit per-language queries.
Rev SHALL own any project symbol index, its bounded/cancellable updates and its
reconciliation with unsaved buffers. Cursor movement SHALL NOT trigger a full
project scan. Missing grammar/query coverage SHALL be reported truthfully and
SHALL NOT imply the presence of another local analysis engine.

#### Scenario: Unsaved symbols override an older observation

- **GIVEN** an indexed file whose open buffer has modified declarations
- **WHEN** native completion observes that buffer
- **THEN** its candidates reflect the current admitted buffer revision
- **AND** stale disk observations are not presented as current local state.

### Requirement: CMP04 Canonical edits and usable controls

Accepting a completion SHALL use the canonical edit/history owner as one
coherent undo operation. Candidate ranges SHALL be validated against the
applicable text revision and cursor context. Canceling SHALL preserve text.
User preferences and compiled M-x commands SHALL expose appropriate controls
without changing the startup-only TOML loading contract.

#### Scenario: Cancel a completion session

- **GIVEN** a selected candidate and unsaved document text
- **WHEN** the user cancels the completion session
- **THEN** focus returns to editing and the document is unchanged
- **AND** no completion edit or undo transaction is created.

## Open decisions

- Trigger policy: manual invocation, automatic triggers, delay and defaults.
- Initial grammar/query coverage, extraction kinds and context ranking.
- Project boundaries, indexing schedule, limits and whether a persistent cache
  is useful. The model distinguishes disposable observations from any future
  persisted index; no stored format is selected.
- Behavior for valid empty LSP results, late replies, candidate list bounds and
  unsupported completion forms such as snippets.
- Exact settings, command names and validation/performance fixtures.

## Dependencies and planning readiness

Reuse native input, checked edits/history, configuration and admitted
Tree-sitter grammars. Decide whether completion is buffer-local or uses a
directory-bounded index; directory-root ownership is required only for the
latter scope. The future LSP client supplies an optional semantic source and
must be integrated with this UI/fallback contract. Managed server installation
is not required to use an already installed server.

Close the listed trigger, coverage, source/result, settings and resource
decisions before planning. A native-only stage does not satisfy the full
source-switching acceptance contract. No generic programmable provider surface
or additional local indexing engine is implied.

## Validation direction

Cover current/dirty-buffer symbols, Unicode positions, index invalidation,
source switching, cancellation, atomic undo, real terminal interaction and
bounded large-project/idle work. No implementation evidence is claimed.
