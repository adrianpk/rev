# AI Commands

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.2
Format: OpenSpec capability proposal
Source: product-owner AI Commands proto-specification; prerequisite, optional Pi and independent native-API clarifications.
Model: [AI Commands Model](editor-ai-commands-model.md)
Baseline: [rev Editor](rev-editor.md)
Controls: [Editor Configuration](editor-configuration.md)
Required directory context: [Directory Opening](../ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md)
Optional enrichment: [Language Server Support](editor-language-server-support.md)
Optional harness backend: [Pi Integration](editor-ai-pi-integration.md)
Delivery boundary: future AI delivery; no implementation or slicing authorized.
Readiness: product direction captured; design decisions and required prerequisites remain open.

## Purpose

Add optional built-in AI commands for discrete editing operations: explain
selected text, insert a requested implementation, transform a declared target,
and preserve useful prompts as reusable named commands with optional bindings.
The user defines the task and target. rev owns contextual access, presentation
and checked edit publication. The model contributes information or content.

Command interaction and edit application are native editor behavior. Execution
has two independent paths: a direct provider API implemented by rev, and an
optional external headless harness, initially Pi. Direct API execution does
not require an agent harness. Neither path requires a chat interface, workspace
registration or executable editor extension interface. Iteration is limited
to admitted read-only context discovery for one explicit command. Ordinary
editing remains independent of AI configuration.

This proposal specifies the AI capability. It does not redefine the whole
editor or claim that directory opening, LSP or new grammar support already ship.

## Confirmed direction

- Provide an immediate prompt and reusable declarative AI command definitions.
- Initial built-ins are explain and implement; final public names are open.
- Input targets are selection, point or current buffer. Read context is a
  distinct grant; information from another file is not permission to edit it.
- Keep native direct API support: rev owns the endpoint, credential binding,
  model transport and context construction. The declared OpenAI-compatible
  protocol subset remains a design decision.
- Offer a separate Pi headless backend. Pi owns its provider authentication,
  context discovery, model exchanges and any admitted session compaction.
  A key used by Pi is Pi authentication, not rev's native API implementation.
- Discovery may list, search and read within an enforced boundary, with finite
  calls, IO, context and time. Both paths retain the command's read-only scope;
  the Pi adapter must enforce it rather than rely on its working directory.
- Tree-sitter may improve context where the grammar and revision support it.
  Optional active LSP may enrich it later; neither is a prerequisite for every
  command, and LSP is never required for the native filesystem path.
- Apply generated changes only to the invocation's declared target through
  ordinary checked buffer edits and undo. No autonomous multi-file editing.
- Keep credentials outside prompt/context content. Source text is data, not
  authority to broaden access or dispatch other editor commands.
- Initial output is explanation or generated/transformed text. Preview,
  acceptance, persistence schema and exact shortcuts remain explicitly open.

## Execution and authentication ownership

| Execution path | Provider credentials and model calls | Context construction |
| --- | --- | --- |
| Native direct API | rev's native provider configuration and transport. | rev assembles input and operates its own bounded discovery loop. Pi is not involved. |
| Pi headless | Pi's configured provider and authentication, which may use a supported subscription or an API key. | Pi discovers relevant files and manages its own model context within the admitted invocation boundary. |

Backend selection and authentication are separate concepts. Installing Pi,
logging into it or using an API key through it does not implement or configure
rev's direct API path. Configuring rev's native API does not require or
implicitly configure Pi. Neither path silently falls back to the other.

For both paths, rev supplies the user instruction, target identity and the
needed editor observations: selection, cursor and admitted unsaved text. A
harness cannot reconstruct unsaved buffer state from a directory. rev retains
result presentation, target validation and canonical edit application.

## Existing-state alignment and dependencies

The complete AI slice set is subordinate to its real prerequisites. It must
not be activated or declared ready by assuming they already exist. Shared
foundations may be reused when actually delivered; their implementation is not
silently added to another delivery.

| Dependency | Status and required relationship |
| --- | --- |
| File/directory opening and root ownership | Required for the complete proposed AI experience. The linked directory ticket is pending. Explicit directory choice and file-parent fallback must have actual native identity/lifetime rules before AI derives its context boundary. |
| Canonical buffers, source positions, edits, read-only and undo | Existing owners to reuse and verify. AI results must pass the same checks and retain invocation identity. |
| Native commands, context bindings and startup configuration | Existing foundations to extend deliberately. The current compiled-command whitelist is not already a saved-AI-command registry. A validated data-driven AI dispatcher must not become an executable plugin layer. |
| Cancellable native jobs and IO | Reuse the actual native job/lifetime owners available at implementation time. Verify the delivered job interfaces without depending on unrelated Org behavior. Reusing an owner does not inherit its other tools or authority. |
| Tree-sitter context access | Available parsing is grammar-specific; the captured editor has a Rust syntax owner. Public checked structural context extraction may need work in the AI set. Do not promise arbitrary-language symbols or an existing global semantic index. |
| LSP | Separate draft capability. Optional enrichment only; not a mandatory predecessor of AI. |
| Provider/profile and AI design decisions | Must be resolved before implementation planning can rely on them. No selected HTTP API, SDK, library or new runtime dependency is authorized here. |
| Optional Pi adapter | Separate integration contract for process/RPC ownership, delegated context and external authentication. It is required only when Pi execution is selected; native API support remains independent. |

The linked directory ticket supplies an opening/root foundation. It does not
settle AI's read/transmission authority for a deeply nested individually opened
file. Root policy, canonical links and explicit overrides must be resolved here.
No workspace or project registry is introduced to fill that gap.

## ADDED Requirements

### Requirement: AI01 Optional native command capability

AI SHALL be an optional built-in capability invoked by explicit editor actions.
Missing configuration, an unavailable backend or an AI failure SHALL leave
ordinary editing available. Startup and idle editing SHALL NOT independently
initiate model requests, harness processes, analysis or discovery. Native API
execution SHALL NOT require Codex, Pi or another agent harness. The optional
Pi path SHALL depend on Pi only when explicitly selected.

#### Scenario: Edit without a provider

- **GIVEN** no usable AI provider is configured
- **WHEN** the user edits, saves, navigates or switches ordinary buffers
- **THEN** those operations retain normal behavior
- **AND** an explicitly invoked AI command can report unavailability without changing source.

### Requirement: AI02 Native API configuration and credential separation

The native API path SHALL identify an endpoint, credential binding, default
model and admitted parameters through its own compatibility profile. It SHALL
perform provider requests through rev's native transport without calling Pi
or exporting credentials from a harness. The editor SHALL distinguish
unsupported protocol/capabilities from successful execution. Native credential
resolution and transport SHALL remain separate from prompts, filesystem
context, command definitions, Pi authentication and displayed diagnostics.
Exact native credential storage and protocol fields remain design decisions.

#### Scenario: Keep a provider credential outside context

- **GIVEN** a native API credential and a direct request with source context
- **WHEN** the editor prepares and sends the provider request
- **THEN** authentication is used only by its transport owner
- **AND** the credential is absent from prompts, discovery results and result buffers.

### Requirement: AI03 Immediate prompt execution

A user SHALL be able to invoke a discrete AI operation with a prompt and a
selected input/output/context policy without first defining a saved command.
The prompt SHALL not become persistent merely because it was executed. The
same native request and edit boundaries SHALL apply to immediate and saved
commands.

#### Scenario: Transform a selected table

- **GIVEN** a Markdown table is the declared selection target
- **WHEN** the user requests a totals row preserving its formatting
- **THEN** the editor supplies that selection and instruction under the chosen policy
- **AND** any accepted replacement uses the ordinary checked editing mechanism.

### Requirement: AI04 Reusable declarative commands

A reusable AI definition SHALL contain a stable name, instruction/template,
input scope, context policy and output behavior, plus optional keybinding and
model override. It SHALL be inspectable declarative data executed by the
native AI engine. Defining one SHALL NOT install executable handlers, scripts
or a plugin. Validated definitions SHALL integrate with the ordinary command
interface; naming collisions, storage and activation policy remain open.

#### Scenario: Reuse a table transformation

- **GIVEN** the user saved table-total with a binding and selection scope
- **WHEN** they invoke it on another table
- **THEN** the saved instruction receives the new invocation's declared selection
- **AND** the command does not reuse stale source or require retyping the prompt.

### Requirement: AI05 Captured editing targets

An invocation SHALL establish its target buffer and selection, point or buffer
scope from native editor state. That target SHALL not be independently selected
or expanded by the model. Missing required selection SHALL NOT silently widen
the operation to the whole buffer; the exact missing-input interaction remains
open. Target identity and validity SHALL be rechecked before publication.

#### Scenario: Keep the original insertion target

- **GIVEN** an implementation request was invoked at a checked point
- **WHEN** generation completes after the user has changed focus
- **THEN** focus alone does not redirect the insertion to another buffer
- **AND** target admission follows the current native identity/revision policy.

### Requirement: AI06 Separate input and read authority

An invocation SHALL carry separate editing-target and context-read grants.
Context policy may admit other files without granting write access. No prompt,
file contents, model message or discovered path SHALL promote that grant into
file creation/deletion, unrelated edits, shell execution, tests or repository
operations. A shared native job owner SHALL not expose another feature's tools.

#### Scenario: Read another declaration without editing its file

- **GIVEN** only the current buffer target is writable by the command
- **WHEN** discovery reads a permitted declaration in another file
- **THEN** that file contributes read context only
- **AND** generated content cannot publish an edit to it.

### Requirement: AI07 Read-only explanation

The explain operation SHALL use a declared selection as its normal primary
input and may acquire admitted additional context. Its result SHALL be an
inspectable explanation, without changing the selected text or other source.
The presentation mechanism and no-selection behavior remain open.

#### Scenario: Explain code using a related type

- **GIVEN** selected code refers to a type in a permitted contextual file
- **WHEN** the user invokes explain and discovery obtains that type
- **THEN** the result can describe the selected code using the obtained context
- **AND** no source file or buffer is modified.

### Requirement: AI08 Point-oriented implementation

The implement operation SHALL accept an explicit instruction and captured
point in the current buffer. It may read surrounding/admitted external context
to generate a concrete implementation. The result SHALL be code for the
declared insertion target, not an autonomous ticket workflow or multi-file edit.

#### Scenario: Implement an interface declared elsewhere

- **GIVEN** a checked insertion point and an interface in the allowed context
- **WHEN** the user requests an implementation and discovery reads the declaration
- **THEN** generated code is offered for that invocation's buffer target
- **AND** unrelated files remain unchanged.

### Requirement: AI09 Explicit context selection

A command SHALL be able to operate with explicitly identified regions/files
without requiring discovered context. Supplied regions SHALL reflect checked
editor observations; buffer and disk versions SHALL not be misrepresented as
one another. Explicit and discovered context SHALL remain distinguishable in
native request state.

#### Scenario: Execute a self-contained transformation

- **GIVEN** an explicit-selection command needs no additional files
- **WHEN** the user invokes it
- **THEN** the supplied context is limited to the admitted selection/instruction
- **AND** no discovery capability is silently enabled.

### Requirement: AI10 Native bounded discovery

In native API execution, discovered context SHALL use rev-owned read-only
capabilities equivalent to bounded file listing, content/symbol search and
range/file reading. rev SHALL construct context and manage iterative provider
requests under the invocation's grants and limits. Tool inputs and results
SHALL be validated by rev; a requested operation is not authority to execute
it. No arbitrary shell/tool bridge or harness dependency is included. Pi
execution delegates this loop under AI20 rather than duplicating it in rev.

#### Scenario: Discover a relevant implementation selectively

- **GIVEN** a native API command with discovery enabled in an allowed directory
- **WHEN** the model lists, searches and reads relevant source in several exchanges
- **THEN** native checks and budgets apply to every exchange
- **AND** no build, test, write or Git command becomes available.

### Requirement: AI11 Filesystem and transmission boundaries

Every context observation SHALL be contained by an explicit native boundary,
including checked path/link resolution. Directory opening supplies a natural
candidate root; individual-file ancestry and overrides remain undecided.
Failure to obtain context SHALL NOT silently ascend or escape the allowed
boundary. Local accessibility SHALL not itself authorize remote transmission.

#### Scenario: Reject a linked path outside the read grant

- **GIVEN** an allowed directory contains a link to an outside file
- **WHEN** discovery requests that file without an applicable explicit grant
- **THEN** the editor rejects the observation or requests the defined user decision
- **AND** no outside content is sent and no broader root is silently selected.

### Requirement: AI12 Selective discovery and finite resources

Discovery SHALL account for ignored/generated content, binaries, large files,
recursive structures and links. File observations, search results, read bytes,
context size, model exchanges and elapsed work SHALL have finite documented
limits before implementation. The native system SHALL report truncation and
budget exhaustion; sending an entire directory is not the default strategy.
Exact rules and thresholds remain open.

#### Scenario: Exhaust a discovery budget

- **GIVEN** useful context has not been found within the admitted budget
- **WHEN** the next observation or exchange would exceed it
- **THEN** discovery stops with a truthful incomplete/insufficient-context outcome
- **AND** limits and authority are not expanded to manufacture success.

### Requirement: AI13 Optional structural and LSP context

Native context construction MAY use available Tree-sitter node/declaration
information when it is valid for the captured source revision and grammar.
No command SHALL require structural analysis for all input. Optional active
LSP may enrich discovery under a later-defined policy; an LSP-independent
filesystem/text path SHALL remain available. Missing capability SHALL not be
represented as verified symbol or type evidence.

#### Scenario: Discover context without LSP or a supported grammar

- **GIVEN** no active language server or usable structural observation
- **WHEN** an AI command requires additional context
- **THEN** it can use its admitted native file/text discovery path
- **AND** it does not require starting a server or invent structural evidence.

### Requirement: AI14 Editor-owned atomic application

Generated insertion/replacement SHALL use the ordinary native edit owner,
source validation, read-only checks and undo history. Only the declared target
may change. rev SHALL revalidate result/target admission and prevent duplicate
publication of one invocation. Exact preview, acceptance and stale-source
interaction remain open; no policy may silently mutate an unrelated range.

#### Scenario: Undo an accepted replacement

- **GIVEN** a usable result has been admitted for the declared selection
- **WHEN** it is applied and the user undoes it
- **THEN** the original selected text is restored through normal history
- **AND** unrelated source remains unchanged.

#### Scenario: Detect a stale target

- **GIVEN** the target's source has changed while the request ran
- **WHEN** a result becomes ready
- **THEN** the native owner revalidates it or invokes the defined stale-result decision
- **AND** it does not apply blindly to old coordinates.

#### Scenario: Reject a read-only target

- **GIVEN** generated content targets a read-only document
- **WHEN** application is attempted
- **THEN** ordinary read-only admission rejects the edit
- **AND** the result can be retained without changing source.

### Requirement: AI15 Native result interaction

Explanations and generated text SHALL be inspectable through editor-native
interaction. An invocation SHALL not require an ongoing chat conversation.
A non-editing result SHALL not be confused with accepted source, and streamed
or provisional output SHALL not become an unintended partial edit. Exact result
views, acceptance controls and non-text output contracts remain open.

#### Scenario: Inspect an explanation without a conversation

- **GIVEN** one explain invocation has produced a result
- **WHEN** the user inspects and leaves it
- **THEN** ordinary editing remains available
- **AND** no persistent chat session or source edit is required.

### Requirement: AI16 Cancellation failure and ordinary editing

Provider or harness IO and bounded discovery SHALL not make ordinary editing
depend on a successful AI operation. Failure, timeout, unusable output and cancellation
before edit publication SHALL preserve original source. Late responses SHALL
not revive a cancelled invocation or publish an extra edit. Already committed
edits remain governed by normal undo, not a false cancellation rollback claim.

#### Scenario: Cancel before publication

- **GIVEN** an AI request is discovering context or generating a result
- **WHEN** the user cancels before its edit is published
- **THEN** owned work is released or fenced under its native lifetime policy
- **AND** source remains unchanged and late output cannot mutate it.

### Requirement: AI17 Context data and credential isolation

Files and discovered text SHALL be contextual data, not instructions granting
new capabilities, changing the target, choosing another provider destination
or invoking editor commands. Credentials and unrelated sensitive configuration
SHALL stay outside model context and visible traces. The selected data policy
must distinguish local read permission from permission to send content to a
configured service; exact inclusion/ignore/consent rules remain open.

#### Scenario: Treat a source instruction as data

- **GIVEN** a contextual file asks the model to read outside the root and run a shell
- **WHEN** it is supplied during a bounded read-only command
- **THEN** that text cannot change native grants or available operations
- **AND** requests exceeding those grants are rejected independently of model output.

### Requirement: AI18 Definitions and invocation lifetimes

Saved definitions SHALL persist as reusable data independently of ephemeral
request context and results. Repeated use SHALL create fresh invocation/target
observations. Stable names/prompts do not promise identical model results.
A command may contain a declared model override interpreted by the selected
backend without gaining a credential, endpoint or authority override. Model
names valid in one backend SHALL NOT silently select the other backend.
Request/result retention and registration/reload rules remain open and must
align with the native configuration model.

#### Scenario: Run a saved command in a later session

- **GIVEN** an admitted saved command and usable selected-backend configuration
- **WHEN** the user invokes it after restarting on different text
- **THEN** its stored behavior uses fresh source and target state
- **AND** an earlier conversation or result is not required.

### Requirement: AI19 Independent execution backends

Native API and Pi execution SHALL be separately configurable, selectable and
verifiable paths over the same editor commands. Selecting one SHALL not require
credentials, software or discovery machinery belonging only to the other.
A Pi request using an API key SHALL remain a Pi request and SHALL NOT count as
native API support. Failure SHALL not silently switch backend, authentication
or provider. Native API implementation and acceptance SHALL remain required
independently of the optional harness integration.

#### Scenario: Use native API without Pi

- **GIVEN** usable native API settings and no Pi installation
- **WHEN** the user invokes a command through native API execution
- **THEN** rev performs its own context construction and provider exchange
- **AND** no harness process or harness authentication is needed.

#### Scenario: Use Pi without native API credentials

- **GIVEN** selected Pi execution with usable Pi authentication and no native API key
- **WHEN** the user invokes an AI command
- **THEN** execution uses the Pi adapter and its admitted context policy
- **AND** absence of native API credentials does not invalidate that invocation.

#### Scenario: Keep a Pi API key under Pi ownership

- **GIVEN** Pi's selected provider authenticates through an API key
- **WHEN** rev executes a command through Pi
- **THEN** Pi owns that provider exchange and credential resolution
- **AND** this does not configure, exercise or validate rev's native API path.

### Requirement: AI20 Delegated harness context

For Pi execution, rev SHALL pass the instruction, checked directory context,
input policy and relevant editor observations to the adapter. Pi SHALL own
admitted filesystem discovery, iterative model context and any permitted
history/compaction. rev SHALL NOT recreate its native API discovery loop on
top of Pi. Read-only access, transmission, work limits and result-application
constraints SHALL still apply through the integration contract. A working
directory alone SHALL NOT be treated as confinement or knowledge of unsaved text.

#### Scenario: Delegate discovery while preserving unsaved input

- **GIVEN** selected Pi execution, an admitted directory and a modified source buffer
- **WHEN** the command requires related files
- **THEN** Pi manages discovery using the admitted editor snapshot and directory context
- **AND** rev preserves its target identity and owns any accepted buffer edit.

## Acceptance examples from the supplied proposal

| Original scenario | Required coverage |
| --- | --- |
| A: ad hoc table transformation | AI03, AI05, AI09, AI14 |
| B: persistent table-total | AI04, AI18 |
| C: explain requiring external context | AI06, AI07, AI10-AI13 |
| D: implement using an external interface | AI05, AI06, AI08, AI10-AI14 |
| E: no LSP | AI13 |
| F: unavailable selected backend | AI01, AI02, AI16, AI19 |

These are future acceptance obligations, not executable tests or delivered
capabilities. Native discovery can report the provenance/completeness of what
it actually observed; it cannot certify that arbitrary model prose is correct.
Missing source is never synthesized as a native discovery fact.

## Open design decisions

None of these choices is settled by writing this draft.

1. File-only discovery boundary: containing directory, selected ancestor or a
   deliberate override; relationship to explicit directory/root ownership.
2. Discovery interface: exact native operations, input/result schemas, search
   semantics and structural queries; relationship to the chosen wire profile.
3. Context activation: definition defaults, per-invocation choices and explicit
   disclosure/consent when locally accessible content would be transmitted.
4. Missing selection: reject, ask for a region or offer another explicit target;
   never an implicit whole-buffer replacement.
5. Result presentation: temporary buffer, existing pane or another native view.
6. Edit acceptance: preview/explicit accept, immediate application or a typed
   policy; handling stale targets and retaining rejected results.
7. Persistence: definition format, location, versioning, reload/activation and
   compatibility with startup TOML. TOML is the existing convention, not a
   silently selected AI schema.
8. Public names/keybindings: builtin names, saved-name collisions and any AI
   prefix; no unrelated shortcut reassignment or plugin namespace is assumed.
9. Backend selection and profiles: configuration/default selection, native
   OpenAI-compatible API subset and native credentials; separate Pi launch,
   model/authentication reference and RPC compatibility. No automatic failover.
10. Resource/ignore policy: finite discovery/search/read/context/call/time
    ceilings, cancellation, generated/binary/large files and path/link handling.
11. Structural enrichment: checked Tree-sitter extraction and optional LSP
    usage, including automatic use versus an invocation choice.
12. Output contract: plain insertion/replacement/explanation, usable-result
    validation, presentation/acceptance and any future richer result form.

## Boundaries and later planning

AI context iteration is bounded read-only work, owned by rev for direct API
execution and delegated to Pi for harness execution. Neither path grants
build/test/shell execution, Git operations or autonomous multi-file changes.
Prompt definitions remain data, not a package marketplace, scripting language
or loadable handler API. The editor gains no mandatory workspace or chat model.
There is no continuous background AI analysis.

This draft/model pair is kept with deferred work and depends on the directory
opening foundation plus its other actual prerequisites. Once decisions and
prerequisites are reviewable, the capability can receive its own approved slice
set and tracker. This capture does not authorize implementation, provider
requests or alteration of completed delivery records. Versioning the draft does
not approve its open decisions or create an implementation plan.
