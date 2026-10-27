# Language Server Support

Status: draft
Date: 2026-10-27
Format: OpenSpec capability proposal
Source: product-owner discussion; captured for later refinement
Delivery boundary: outside rev-configuration-ui; no implementation or slicing authorized
Completion owner: [Native Autocompletion](editor-autocompletion.md)
Installation owner: [Language Server Installation](editor-language-server-installation.md)
Controls: [Editor Configuration](editor-configuration.md)

## Purpose

Add optional native LSP client support for semantic completion. Keep the client
in rev's Rust binary and language servers outside it, with no editor addon or
public programmable-provider layer required.

## Confirmed direction

- LSP is optional and must not make ordinary editing depend on a server.
- Enabled, operational LSP takes precedence over Tree-sitter completion.
- Disabling or losing a server leaves Tree-sitter fallback available within
  its stated grammar/query coverage.
- Completion is the selected first LSP capability. Diagnostics and other IDE
  features do not enter this scope merely because a server supports them.
- Rust is the primary language. The installation draft owns the secondary
  language list and easy connection of additional servers.

## ADDED Requirements

### Requirement: LSP01 Optional resource ownership

Rev SHALL own the language-server processes it starts and keep their work
optional. When LSP is disabled, rev SHALL NOT start servers or associated
analysis work. Server absence or failure SHALL preserve normal editing and
allow the native completion owner to select its fallback.

#### Scenario: Edit with LSP disabled

- **GIVEN** a supported project and LSP disabled by the user
- **WHEN** rev starts and the user edits source files
- **THEN** no language-server process is started by rev
- **AND** editing and admitted Tree-sitter completion remain usable.

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

#### Scenario: Select an additional installed server

- **GIVEN** an external LSP server compatible with the supported client contract
- **WHEN** the user supplies its admitted executable and launch settings
- **THEN** rev can connect through its native client without a language addon
- **AND** the declared language is not assumed to have a bundled grammar.

## Open decisions

- Factory activation policy and activation granularity: buffer, language and
  project; sharing and shutdown rules for server processes.
- Transport, protocol baseline, capabilities and required server-request handling.
- Timeouts, retries, cancellation, message/result/queue bounds and restart policy.
- Unicode mapping, incremental synchronization and completion edit forms.
- Exact TOML/command catalog and testing against real servers plus failure fixtures.

These decisions belong in this draft before implementation. No generic plugin
system, bundled-server strategy or new persisted settings shape is selected.
