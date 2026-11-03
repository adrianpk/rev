# Pi Integration

Status: Completed
Completed: 2026-11-03
Date: 2026-10-28
Reviewed: 2026-11-02
Version: 0.2
Approved: product owner, 2026-11-02; proceed with explained RPC/native-result scope and established autonomous delivery
Format: OpenSpec capability specification
Source: product-owner optional headless-harness and independent native-API clarifications
Parent contract: [AI Commands](editor-ai-commands.md)
Model: [Pi Integration Model](editor-ai-pi-integration-model.md)
Directory foundation: [Directory Opening](../ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md)
File observations: [External File Changes](editor-external-file-changes.md)
Plan: [Pi and Reload Delivery](../plan/rev-ai-pi.md)
Tracker: [Pi and Reload Tracker](../tracker/rev-ai-pi.md)
Delivery boundary: first AI delivery through Pi and general external reload in one approved Looper set through dev
Readiness: implemented and validated through the completed Pi/reload delivery; native API remains deferred

## Purpose

Let rev execute its AI editing commands through an optional external headless
Pi process. rev supplies the instruction, relevant editor state and an admitted
directory context. Pi handles its provider authentication, file discovery,
model exchanges and context management. rev retains presentation and checked
application to the original editing target.

The owner selected Pi before the native direct API implementation. The first
delivery includes the shared command/target/result behavior needed by Pi; it
does not wait for rev to build its own provider transport or discovery loop.
The native API remains a separate later capability. A Pi
provider authenticated with an API key still uses the Pi backend. Subscription
reuse follows the authentication supported by Pi and its selected provider;
rev does not turn a subscription credential into a native API key.

## Reference and scope

Pi 1.3.0's installed CLI and bundled RPC, CLI Integration, Providers and How Pi
Works documents were inspected. Its [public documentation](https://pi.dev/)
describes RPC for process integrations, API-key/OAuth authentication and context
management. Pi 1.3.0 is the reference contract. Admit 1.3.x only after checking
the required protocol behavior; do not claim that an untested patch is already
verified. Another minor/major version requires an explicit compatibility
review. Existing CLI availability is not integration acceptance evidence.

Pi's working directory is a context/discovery starting point, not a sandbox.
Its enabled tools run with the process's operating-system permissions. The
adapter therefore needs an enforceable read-only tool/path/resource contract.
It cannot assume that headless operation or a prompt instruction removes
Pi's default write, shell or extension capabilities.

## ADDED Requirements

### Requirement: PI01 Independent optional execution path

Pi SHALL be the first delivered execution path of the optional AI command system. It
SHALL NOT be used to implement, proxy or validate native API execution. Native
API credentials SHALL NOT be required for a usable Pi invocation. Pi's own
credential method SHALL not change its backend identity. Missing Pi or failed
Pi execution SHALL NOT cause automatic native API failover or interfere with
ordinary editing.

#### Scenario: Start with Pi before native API implementation

- **GIVEN** usable Pi authentication and no native API implementation or key
- **WHEN** the user invokes a supported AI command
- **THEN** rev uses the admitted Pi adapter
- **AND** no direct provider transport or native discovery engine is required.

#### Scenario: Distinguish a Pi API key

- **GIVEN** Pi execution and an API key configured through Pi's own mechanisms
- **WHEN** a command runs through the adapter
- **THEN** Pi performs the provider exchange
- **AND** that run is recorded and validated as Pi execution rather than native API support.

### Requirement: PI02 Owned process and admitted protocol

rev SHALL control a version-compatible Pi child through a declared headless
process protocol. Use the 1.3.0 RPC contract over stdin/stdout, with LF-delimited
JSON records and request IDs. This is an external child process, not an RPC
network service or an embedded TypeScript SDK. Launch SHALL use explicit
executable/arguments/directory settings without shell command interpolation.
Record framing, request correlation, stream/backpressure limits, diagnostics,
completion detection and shutdown SHALL have finite native ownership. A prompt
acceptance response SHALL NOT be confused with successful completed generation.
The adapter SHALL not take over an unrelated interactive Pi session.
Read stdout continuously, keep stderr separate, and preserve backpressure.
Wait for `agent_settled`, then validate the final assistant result and failure
state. `response.success`, `message_end` and `agent_end` alone are insufficient.

#### Scenario: Wait for a complete result

- **GIVEN** an owned compatible Pi operation
- **WHEN** Pi accepts the prompt and subsequently streams work
- **THEN** rev keeps the invocation pending until the admitted completion outcome
- **AND** acceptance, intermediate tool activity or a partial stream is not published as a finished edit.

### Requirement: PI03 Delegated context and editor snapshots

The adapter SHALL start the owned child in the checked directory and supply the
explicit user instruction. Pi SHALL own admitted file discovery, iterative model exchanges,
history reconstruction and any permitted compaction. rev SHALL provide needed
selection, cursor, target identity/revision and unsaved-buffer observations;
directory access alone SHALL NOT stand in for current editor state. The Pi
path SHALL not require rev to duplicate its native API discovery/context loop.
Unsaved observations SHALL not be written to source files merely to expose them.
Use the editor's explicit opened directory when present; otherwise use the
visiting file's containing directory, or the launch directory for an unnamed
buffer. Never silently walk upward to a repository root. `ai-set-directory`
can explicitly select another session-local root without creating a workspace.

The initial context policies are `input` and `directory`. `input` supplies only
the declared editor observations in a fresh ephemeral Pi session, with no
filesystem discovery or earlier conversation history. `directory` permits
read/list/search within the canonical root and reuses only a compatible owned
session. A catalog, grammar, LSP connection or file index is not required.

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
Inherit Pi's existing authentication environment/store without copying secret
values into editor settings. A missing login reports that authentication must
be completed in Pi, then retried; the initial adapter does not export tokens,
launch authentication browsers or accept credential-bearing extension dialogs.

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

The initial launch profile disables automatic extensions, MCP, skills, prompt
templates and context-file discovery. Only the adapter's admitted read/list/
search tools and explicit native snapshots are exposed. Supplying a minimal
guarded tool adapter is an internal integration detail, not an editor plugin
installation or a second model/discovery engine. Check canonical roots and
escaping links before IO and preserve path observations through each read.
Credential files and Pi's internal state are not discoverable source context.

Current target bytes and admitted dirty-buffer snapshots override stale disk
versions for those paths. Report disk-only search observations as disk data;
do not present them as proof about unsaved source. Resource discovery is never
silently enabled to repair an unsupported request.

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

Use a transient read-only result buffer in the selected pane, retaining the
source buffer and its return location. Edit results require explicit
`ai-apply-result`; `ai-discard-result` leaves source unchanged. Explanation
results have no apply action. An accepted result is one native undo step and
does not implicitly save the visited file.

The final model response contract is one JSON object containing `version: 1`
and `text`, without surrounding Markdown fences or additional fields. The
target, operation kind and invocation identity come from rev's captured state,
never fields supplied by the model. Preserve text bytes; do not heuristically
strip fences or trim code. Invalid, partial or oversized output remains an
error/read-only observation, never an automatically applied edit.

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

Allow one active invocation and no hidden command queue. C-g, and modal Escape
where cancellation owns input, use `clear_queue` followed by `abort`; wait for
the admitted idle boundary. On timeout, close only the owned child with bounded
stop/reap and fence its stream. Closing a running result view alone must not
be confused with canceling another operation. A rejected or stale result can
be inspected, but cannot change target or restart work implicitly.
Report failed or uncertain cleanup truthfully instead of claiming that an
unobserved child exit completed within the intended grace period.

#### Scenario: Cancel an owned operation

- **GIVEN** a Pi invocation is discovering context or generating text
- **WHEN** the user cancels before native publication
- **THEN** the adapter cancels or fences its owned work under the admitted lifecycle
- **AND** source remains unchanged and later events cannot publish an edit.

### Requirement: PI08 Session and root isolation

Any reused Pi process/session SHALL remain bound to its admitted directory,
settings and invocation ownership. A directory change or a different document
context SHALL not silently reuse unrelated conversation state. Directory-policy
operations reuse a private session keyed by canonical root and compatible
model/configuration/grant. Input-only operations use fresh ephemeral sessions.
Start the process lazily, preserve compatible directory history according to
the configured persistence policy, and close after its idle deadline or editor
exit. An owned saved session can resume later without leaving a daemon running.
Never select the user's unrelated last session through implicit `--continue`.
The user interaction SHALL remain discrete editor commands without requiring
a persistent chat interface. Both command surfaces use the same backend path.

Pi owns reconstruction and automatic compaction. Retained tool results are
historical observations, not a live filesystem index. Supply fresh editor
state on every invocation and re-read relevant disk sources when versions
change. `ai-reset-session` begins a fresh owned history. A root, model/provider
or grant change crosses an isolation boundary; cancel/settle active work before
switching instead of mixing histories from different grants.

#### Scenario: Change the editor directory

- **GIVEN** retained Pi context belongs to an earlier selected directory
- **WHEN** a new command is invoked under another directory context
- **THEN** the adapter applies its declared session-boundary policy
- **AND** earlier source context is not silently included as if it belonged to the new invocation.

## First-delivery commands and definitions

These names are delivered through the native command registry:

| Command | Behavior |
| --- | --- |
| `ai-explain` | Require a selection, permit directory discovery and present a read-only explanation. |
| `ai-implement` | Ask for an instruction, capture point and offer generated text for insertion. |
| `ai-prompt` | Ask for an instruction; selection means replacement, otherwise point insertion. Show the chosen target before execution. |
| `ai-save-command` | Save the current successful instruction as a named declarative command; do not save generated text as its prompt. |
| `ai-apply-result` / `ai-discard-result` | Admit an edit against the original revision, or discard the proposal. |
| `ai-cancel` | Cancel the current owned invocation. |
| `ai-set-directory` | Select an explicit session-local context root. |
| `ai-select-model` | Choose a Pi-reported available model without exposing credentials; cross the session boundary before a changed provider/model. |
| `ai-reset-session` / `ai-disconnect` | Start fresh history, or stop the owned idle connection while preserving admitted session storage. |

No default shortcut namespace or chat interface is introduced. All commands
use the common M-x/colon registry. No-selection explain reports the missing
selection instead of silently explaining or replacing the entire buffer.
Whole-buffer targets are explicit saved-definition choices.

Store definitions under `ai.commands.<name>` in the existing TOML root. Their
public command name is `ai-<name>`. Reject duplicate or compiled-name collisions
before admission. Definitions contain prompt, input, context and output, plus
an optional model. They contain no executable code or custom tools. A newly
saved definition becomes callable after complete scoped validation; bindings
stored in the ordinary keymaps take effect under the existing startup policy.
The model defines exact values, persistence and finite limits.

## Scope and preparation decisions

RPC is the selected editor transport, with lazy owned process/session reuse.
One-shot CLI execution is a valid Pi capability but is not a second required
adapter in this first delivery. Direct API transport, native model context
construction and a fallback between execution paths are deferred.

The original single-target application contract is retained. The owner's
external-reload agreement defines general file behavior; it does not itself
authorize Pi to write sources, run shell commands or perform multi-file edits.
Direct filesystem application is a separate possible expansion, not an open
dependency that prevents implementing this bounded first delivery.

Selected defaults, storage, message/input/result bounds, cancellation deadlines
and session lifetimes are in the paired model. They are engineering choices
under the approved delivery scope, not individually named owner decisions. The
plan records the successful Pi 1.3.0 offline guarded-read admission probe and
requires complete guard/lifecycle and process-cost evidence in implementation.
No crate, SDK or extra
runtime beyond the separately installed Pi is selected by this document.

## Acceptance and delivery

Validate this adapter without requiring native API support. Cover a Pi-managed
subscription where supported and a Pi-managed API key, missing authentication,
ordinary editing with Pi absent, delegated context and unsaved snapshots,
root/tool restrictions, partial/failed/cancelled output and native undo/read-only
admission. Fixtures alone do not prove real-provider subscription acceptance.
External authenticated checks require their own authorized credentials and
must not be performed merely because this specification exists.

Include related declarations in another file, an unbundled language with no
LSP, a dirty target, incompatible history, input-only after directory discovery,
process restart with retained history, compaction, and both startup interaction
profiles. Prove actual LF framing, backpressure, queue cancellation and the
settled/failure boundary. Use realistic payloads and a real admitted Pi probe;
protocol fixtures alone are not evidence of provider/model correctness.

The [external-file specification](editor-external-file-changes.md) independently
covers external writers. When planned together, verify that a disk reload
invalidates any pending AI proposal through the same source revision checks.

Primary references reviewed 2026-11-02:

- [CLI integration](https://pi.dev/docs/latest/cli-integration)
- [RPC protocol](https://pi.dev/docs/latest/rpc)
- [RPC commands](https://pi.dev/docs/latest/rpc-commands)
- [Sessions and context](https://pi.dev/docs/latest/sessions)
- [Context construction](https://pi.dev/docs/latest/how-pi-works)

This is a subordinate optional AI integration contract, not a new autonomous
development workflow or an executable extension layer for rev. This complete
specification is approved for the linked delivery. Its exact plan/tracker,
gate and authority own implementation and controller startup; this approval
does not cover direct filesystem writes, account changes or native API delivery.

## Delivery evidence

PI01-PI08 completed through canonical PRs #229-#236 in the
[Pi/reload delivery](../plan/rev-ai-pi.md). The unchanged controller full gate
passed at `ecd02abcbd1f08530d8fad1395672dc6d00ee085`, receipt
`HBFDUZX54JK3RJUBGSQYIJANGO`, on 2026-11-03. The
[Slice 8 closure](../report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
records exact commands, 578 selected top-level tests, both-profile native/PTY
acceptance and separate real-provider evidence. Existing requirement, scenario,
configuration and resource contracts remain intact.

The real provider observation used Pi-owned OAuth and the existing configured
model. Its original failed attempt and one explicitly authorized successful
continuation remain recorded. API-key variants were not exercised live. The
broader native API specification/model remain draft; direct transport and native
context construction stay deferred.
