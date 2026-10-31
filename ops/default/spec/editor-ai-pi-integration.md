# Pi Integration

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.1
Format: OpenSpec capability proposal
Source: product-owner optional headless-harness and independent native-API clarifications
Parent contract: [AI Commands](editor-ai-commands.md)
Model: [Pi Integration Model](editor-ai-pi-integration-model.md)
Directory foundation: [Directory Opening](../ticket/reviewing/20261028094803-open-directory-targets-from-cli-and-find-file.md)
Delivery boundary: future optional AI backend; no implementation or slicing authorized
Readiness: execution/authentication separation is confirmed; protocol admission and listed details remain open

## Purpose

Let rev execute its AI editing commands through an optional external headless
Pi process. rev supplies the instruction, relevant editor state and an admitted
directory context. Pi handles its provider authentication, file discovery,
model exchanges and context management. rev retains presentation and checked
application to the original editing target.

This is distinct from rev's required native direct API implementation. A Pi
provider authenticated with an API key still uses the Pi backend. Subscription
reuse follows the authentication supported by Pi and its selected provider;
rev does not turn a subscription credential into a native API key.

## Reference and scope

Pi 1.3.0's installed CLI and bundled RPC, CLI Integration, Providers and How Pi
Works documents were inspected. Its [public documentation](https://pi.dev/)
describes RPC for process integrations, API-key/OAuth authentication and context
management. The supported version range and exact wire contract must still be
fixed before implementation; existing CLI availability is not integration
acceptance evidence.

Pi's working directory is a context/discovery starting point, not a sandbox.
Its enabled tools run with the process's operating-system permissions. The
adapter therefore needs an enforceable read-only tool/path/resource contract.
It cannot assume that headless operation or a prompt instruction removes
Pi's default write, shell or extension capabilities.

## ADDED Requirements

### Requirement: PI01 Independent optional execution path

Pi SHALL be an optional external backend of the parent AI command system. It
SHALL NOT be used to implement, proxy or validate native API execution. Native
API credentials SHALL NOT be required for a usable Pi invocation. Pi's own
credential method SHALL not change its backend identity. Missing Pi or failed
Pi execution SHALL NOT cause automatic native API failover or interfere with
ordinary editing.

#### Scenario: Keep native API independent

- **GIVEN** usable native API settings and no Pi installation or login
- **WHEN** native API execution is selected
- **THEN** rev owns the provider request and context construction
- **AND** no Pi process, SDK or credential export is involved.

#### Scenario: Distinguish a Pi API key

- **GIVEN** Pi execution and an API key configured through Pi's own mechanisms
- **WHEN** a command runs through the adapter
- **THEN** Pi performs the provider exchange
- **AND** that run is recorded and validated as Pi execution rather than native API support.

### Requirement: PI02 Owned process and admitted protocol

rev SHALL control a version-compatible Pi child through a declared headless
process protocol. RPC over stdin/stdout is the selected integration direction;
the exact supported contract remains to be pinned. Launch SHALL use explicit
executable/arguments/directory settings without shell command interpolation.
Record framing, request correlation, stream/backpressure limits, diagnostics,
completion detection and shutdown SHALL have finite native ownership. A prompt
acceptance response SHALL NOT be confused with successful completed generation.
The adapter SHALL not take over an unrelated interactive Pi session.

#### Scenario: Wait for a complete result

- **GIVEN** an owned compatible Pi operation
- **WHEN** Pi accepts the prompt and subsequently streams work
- **THEN** rev keeps the invocation pending until the admitted completion outcome
- **AND** acceptance, intermediate tool activity or a partial stream is not published as a finished edit.

### Requirement: PI03 Delegated context and editor snapshots

The adapter SHALL supply a checked directory context and the explicit user
instruction. Pi SHALL own admitted file discovery, iterative model exchanges,
history reconstruction and any permitted compaction. rev SHALL provide needed
selection, cursor, target identity/revision and unsaved-buffer observations;
directory access alone SHALL NOT stand in for current editor state. The Pi
path SHALL not require rev to duplicate its native API discovery/context loop.
Unsaved observations SHALL not be written to source files merely to expose them.

#### Scenario: Discover surrounding source

- **GIVEN** a Pi command with an admitted directory and discovery policy
- **WHEN** related declarations are needed
- **THEN** Pi locates and incorporates admitted source using its own context machinery
- **AND** rev does not run a second model discovery loop over the same operation.

#### Scenario: Preserve unsaved source identity

- **GIVEN** the target buffer differs from its associated disk file
- **WHEN** rev sends the invocation to Pi
- **THEN** the editor snapshot is identified separately from disk observations
- **AND** no implicit save or claim that stale disk text is current occurs.

### Requirement: PI04 Pi-owned authentication

Provider authentication and credential refresh for Pi SHALL remain under Pi's
own mechanisms. rev SHALL NOT import, export, parse into its native credential
store or serialize Pi's provider secrets into prompts and results. Native API
settings SHALL not be passed as a substitute Pi login. A provider/model choice
for Pi SHALL be resolved within Pi's admitted configuration. Authentication
failure SHALL produce an actionable Pi-backend outcome without changing paths.
The child environment and any external authentication interaction require an
explicit integration policy; shared secret values do not merge backend owners.

#### Scenario: Reuse Pi authentication

- **GIVEN** usable Pi-managed authentication and no configured native API credential
- **WHEN** a Pi command is invoked
- **THEN** Pi resolves its own provider access
- **AND** rev neither requests a native key nor copies Pi's token into native API settings.

### Requirement: PI05 Enforced discovery authority

The effective Pi tools, filesystem access and loaded resources SHALL preserve
the parent command's read-only discovery and transmission grants. Ordinary
model execution SHALL not acquire shell/build/test/Git/write tools, unrelated
file modification or arbitrary native command dispatch. Cwd and textual
instructions SHALL NOT be treated as enforcement. Symlinks, absolute paths,
project/global instructions, extensions, skills and MCP/resource discovery
SHALL follow an admitted policy. If the selected process/tool configuration
cannot enforce the required boundary, the adapter SHALL reject it truthfully.

#### Scenario: Refuse an unenforceable tool configuration

- **GIVEN** a Pi configuration that exposes a forbidden write or shell capability
- **WHEN** the adapter checks command admission
- **THEN** that configuration is not accepted for the bounded editor operation
- **AND** rev does not claim it is safe solely because Pi started in the chosen directory.

### Requirement: PI06 Native result application

Pi results SHALL be proposals associated with the original invocation and
target. Explanations SHALL remain read-only output. Accepted generated text
SHALL use rev's canonical range/revision, read-only, undo and publication
checks. Pi SHALL NOT autonomously edit source files as an alternative result
channel. Model overrides, late output or another focused buffer SHALL not
redirect the target or select the native API backend.

#### Scenario: Revalidate a generated insertion

- **GIVEN** a Pi result whose target changed during execution
- **WHEN** native application is attempted
- **THEN** rev performs its ordinary stale-target admission
- **AND** Pi output is not blindly inserted at an old or newly focused position.

### Requirement: PI07 Cancellation failure and resource bounds

Process IO, context discovery, model iterations, retained messages and result
sizes SHALL have documented limits and cancellation ownership. Failure,
timeout, incompatible protocol, child exit or cancellation before publication
SHALL preserve editor source. Late events SHALL not revive cancelled work.
Ordinary editing SHALL remain responsive, and cleanup SHALL affect only the
adapter-owned operation/process. No unrelated Pi process SHALL be terminated.

#### Scenario: Cancel an owned operation

- **GIVEN** a Pi invocation is discovering context or generating text
- **WHEN** the user cancels before native publication
- **THEN** the adapter cancels or fences its owned work under the admitted lifecycle
- **AND** source remains unchanged and later events cannot publish an edit.

### Requirement: PI08 Session and root isolation

Any reused Pi process/session SHALL remain bound to its admitted directory,
settings and invocation ownership. A directory change or a different document
context SHALL not silently reuse unrelated conversation state. Fresh versus
reused sessions, persistence, retention and compaction policy remain open.
The user interaction SHALL remain discrete editor commands without requiring
a persistent chat interface. Both command surfaces use the same backend path.

#### Scenario: Change the editor directory

- **GIVEN** retained Pi context belongs to an earlier selected directory
- **WHEN** a new command is invoked under another directory context
- **THEN** the adapter applies its declared session-boundary policy
- **AND** earlier source context is not silently included as if it belonged to the new invocation.

## Open design details

1. Supported Pi versions, RPC/event lifecycle and capability admission.
2. Exact launch/environment configuration, backend selection scope and model
   references, kept separate from native API credentials and settings.
3. Effective read-only tools, filesystem/path enforcement and loaded-resource
   policy. No implementation mechanism or extra dependency is chosen here.
4. Editor snapshot encoding and precedence over disk observations, including
   explicit files and relevant unsaved buffers.
5. One-operation versus reused process/session, directory transitions,
   compaction, persistence and retention locations/lifetimes.
6. Finite IO/context/tool/model/time limits, cancellation and completion/result
   contract; integration of any authentication or other external UI requests.

## Acceptance and delivery

Validate this adapter separately from native API support. Cover a Pi-managed
subscription where supported and a Pi-managed API key, missing authentication,
native operation with Pi absent, delegated context and unsaved snapshots,
root/tool restrictions, partial/failed/cancelled output and native undo/read-only
admission. Fixtures alone do not prove real-provider subscription acceptance.
External authenticated checks require their own authorized credentials and
must not be performed merely because this draft exists.

This is a subordinate optional AI integration contract, not a new autonomous
development workflow or an executable extension layer for rev. All linked
documents remain drafts. Versioning them does not authorize implementation,
slicing, installation, login or controller launch.
