# Language Server Installation

Status: Approved
Date: 2026-10-27
Reviewed: 2026-11-02
Approved: product owner, 2026-11-02; implementation and necessary preparation authorized
Format: OpenSpec capability specification
Source: product-owner discussion and explicit implementation approval
Delivery boundary: one autonomous Looper delivery through dev
Client owner: [Language Server Support](editor-language-server-support.md)
Model: [Language Server Installation Model](editor-language-server-installation-model.md)
Storage reference: [Editor Configuration](editor-configuration.md)
Plan: [Installation Delivery](../plan/rev-language-server-installation.md)
Tracker: [Installation Tracker](../tracker/rev-language-server-installation.md)
Readiness: approved for implementation; acceptance evidence is not yet available

## Purpose

Make optional language-server installation and discovery simple while keeping
rev small. The Rust binary contains the client and user-facing management flow;
server payloads and language runtimes are external.

## Confirmed priorities

| Priority | Languages |
| --- | --- |
| Primary | Rust |
| Secondary | Go, TypeScript/JavaScript, HTML/CSS, Markdown, Org mode, Ruby, Python, Odin |
| Easy additional integration | Elixir, Gleam, OCaml and other compatible LSP servers |

This is a priority list, not a language allowlist. Any installed server that
meets the native client's admitted stdio LSP contract can be registered for any
valid language identity and file pattern. A bundled or external Tree-sitter
grammar is not a prerequisite. Catalog recipes provide installation shortcuts;
they do not define which languages rev can use through LSP.

The priority list does not imply equal completion quality or recipe readiness.
The initial managed catalog below is an engineering selection under the owner's
delegation to complete preparation. Org keeps native completion and generic
external-server registration initially; this delivery does not claim a verified
managed Org recipe. Org server implementations exist, but their installation
and compatibility need separate evidence before a catalog entry can be offered.

## Management experience

`lsp-setup` is the ordinary entry point from M-x or modal `:`. It offers an
installed executable, a managed recipe, or a custom server. It asks for a new
language identity and patterns only when existing associations cannot supply
them. Choosing Go inherits `*.go`; choosing a different Go server preserves
that association. Paths and arguments are separate values, never a shell line.

The shared command registry also exposes `lsp-servers`, `lsp-select-server`,
`lsp-install-server`, `lsp-update-server`, `lsp-remove-server`,
`lsp-cancel-installation` and `lsp-reload-settings`. They reuse native vertical
selectors, prompts and result buffers. There is no new command parser or
mandatory key prefix. Names describe planned commands until their slices land.

`lsp-servers` shows language, configured server, selected source, managed or
external origin, version when known, readiness and actionable prerequisites.
Opening the catalog or starting rev does not acquire packages or check remote
versions. Installation and updates require explicit invocation. Successful
setup persists and applies its admitted LSP/association changes in the current
session; enabling LSP remains the separate existing `lsp-mode` operation.

## Initial managed catalog

Managed acquisition is initially verified on Linux x86-64. Other platforms
retain generic existing-server registration; a recipe must report unsupported
before acquisition when no admitted asset/runtime matches. Upstream support for
another platform is not evidence that rev tested that platform.

| Recipe | Languages | Acquisition and launch contract |
| --- | --- | --- |
| `rust-analyzer` | Rust | Resolve an official release, download the matching GNU/Linux gzip asset, verify the release SHA-256, unpack one executable. Reuse an existing executable when chosen. Report missing Rust sources/toolchain separately; do not install a compiler implicitly. |
| `gopls` | Go | Resolve an exact module version and use `go install` with an owned `GOBIN` and caches. Set `GOTOOLCHAIN=local`; report an insufficient Go compiler without downloading another. |
| `typescript-language-server` | TypeScript, JavaScript | Install exact package versions in an owned npm prefix with a lockfile and scripts disabled. Use the upstream-compatible TypeScript 6 major, not an unconstrained compiler upgrade; launch with `--stdio`. |
| `vscode-langservers-extracted` | HTML, CSS, JSON | One owned npm installation supplies three named stdio launch profiles. Resolve and retain exact package/lock versions. |
| `marksman` | Markdown | Download the official native release asset and verify its release SHA-256. Use the server's stdio entry point. |
| `ruby-lsp` | Ruby | Install the selected gem/version into an owned gem home with its dependency resolution. Scope `GEM_HOME`/`GEM_PATH` to the launched server; preserve the user's Ruby installation. |
| `python-lsp-server` | Python | Create an owned virtual environment at its final versioned path, install the exact base package without optional extras, and use its `pylsp` entry point. Do not move the environment after creation. |
| `ols` | Odin | Resolve an immutable upstream Git revision, build with the installed Odin toolchain through the upstream build script, and retain `builtin/` beside the server or its explicit environment path. Disable snippet-only completion features that the admitted client cannot apply. |

Only the chosen recipe requires its external tools: curl/gzip/SHA-256 tools for
native releases, Go, Node/npm, Ruby/gem, Python/venv/pip, or Git/Odin/Bash for
the corresponding package. These are not rev build or ordinary-use
prerequisites. No global package installation, elevated privileges, runtime
upgrade, executable auto-download on editing, or automatic update is implied.

Resolve floating upstream selectors once per explicit operation, then retain
concrete versions, source references and verification records. TLS and trusted
package registries establish acquisition provenance; a checksum from the same
release channel establishes integrity, not independent publisher identity.
Never invent a missing upstream digest. Reject an unverifiable native release
instead of marking it ready. A custom existing executable remains available.

## ADDED Requirements

### Requirement: INS01 Simple optional installation

The user SHALL be able to install a supported server through a simple guided
operation and receive a usable native-client configuration. Existing compatible
servers SHALL be reusable. Missing language runtimes/toolchains and incomplete
installations SHALL be identified with an actionable result rather than a
false success. Setup SHALL reuse known file associations and persist through the
existing user TOML root rather than require duplicate per-language setup.

#### Scenario: Install the preferred Rust server

- **GIVEN** the selected Rust recipe and its admitted environment requirements
- **WHEN** the user requests installation
- **THEN** rev obtains or reuses the required server and verifies it is launchable
- **AND** the result clearly distinguishes ready, missing prerequisite and failure.

### Requirement: INS02 Keep payload and lifecycle optional

Server payloads SHALL remain outside the rev executable. A managed installation
SHALL use the appropriate existing XDG data ownership; configuration and cache
SHALL retain their established roles. Installing a server SHALL NOT by itself
activate continuous analysis when LSP is disabled.

#### Scenario: Install without activating analysis

- **GIVEN** LSP disabled and a supported server installed successfully
- **WHEN** the user returns to editing
- **THEN** the server remains available on disk without being started by rev
- **AND** ordinary editing continues with the admitted native fallback.

### Requirement: INS03 Defined recipes and custom connections

The supported catalog SHALL provide concrete server/launch requirements for
its declared platforms and languages. A user SHALL be able to connect another
compatible installed server through native settings without an editor addon.
A partial failure SHALL NOT destroy a previously usable managed installation.

#### Scenario: A replacement installation fails

- **GIVEN** a usable managed server and an attempted replacement
- **WHEN** acquisition, validation or publication fails
- **THEN** the prior usable installation remains available
- **AND** the management UI reports the actual incomplete or failed operation.

#### Scenario: Register a language without a bundled grammar

- **GIVEN** a compatible installed server for Elixir and no Elixir grammar
- **WHEN** the user registers its executable, language identity and `*.ex` pattern
- **THEN** rev can attach it, synchronize the document and request semantic completion
- **AND** no editor rebuild, catalog entry or grammar installation is required.

### Requirement: INS04 One selected semantic source with native fallback

rev SHALL select one configured server per language and root using an explicit
language preference when present. In its absence, preserve the existing
deterministic server-name selection. A native-only preference SHALL bypass LSP
for that language. The manager SHALL NOT introduce separate implicit project,
managed and PATH precedence: the selected profile's command is authoritative;
PATH resolution remains the process launcher's ordinary rule.

Each completion request SHALL use the admitted semantic result or the existing
native fallback, never concatenate both lists. Changing servers SHALL preserve
the grammar and native structural behavior. Native fallback for a language
without a grammar may have no structural candidates; this does not invalidate
its semantic server. The manager does not expand the client's LSP feature set.

#### Scenario: Replace the Go server

- **GIVEN** native Go structure and a selected Go language server
- **WHEN** another configured Go server is selected
- **THEN** new requests use only the new semantic source when available
- **AND** old responses cannot publish after the selection change
- **AND** empty, failed or timed-out semantic requests follow existing fallback.

### Requirement: INS05 Safe persistence and scoped session application

Use the sole existing user configuration root, its include precedence and
whole-candidate admission. Store associations in `files.associations`, server
profiles in `lsp.servers` and choices in `lsp.preferred`. Do not discover or
write project-local configuration. Preserve comments, unrelated fields,
includes and the selected root's symlink target.

Before publication, validate the complete proposed configuration and compare
the captured source/root identities against current disk state. Refuse a
concurrent edit or changed include rather than overwrite it. Serialize manager
mutations across rev processes. Publish a complete temporary sibling atomically;
post-publication durability uncertainty must be reported truthfully, not called
a rollback. Invalid startup fallback is not permission to overwrite invalid
source configuration.

Apply only admitted LSP settings and file associations to the live session.
Preserve the current global LSP enablement, per-buffer exclusions, text, undo,
point, selection and immutable modal/non-modal profile. Affected connections
retire through the existing client owner; unaffected roots remain usable.
`lsp-reload-settings` follows the same restricted application contract.

#### Scenario: Save while the user edits configuration

- **GIVEN** setup prepared from a valid user configuration graph
- **WHEN** the root or an include changes before publication
- **THEN** persistence reports the conflict and preserves the user's new bytes
- **AND** a verified payload may remain installed but unselected
- **AND** the running configuration is not partially replaced.

### Requirement: INS06 Owned, bounded installation lifecycle

Acquisition, build, verification and cleanup SHALL run outside the input/render
loop. One active mutation job per data root is admitted. Every subprocess uses
structured arguments, owned working directories and bounded output/deadlines.
Recipe tools execute with user permissions, not in a security sandbox.

Use versioned external payloads and the model's schema-versioned registry.
Verify a candidate through a bounded initialize/shutdown exchange against an
owned empty root, without sending user buffers. Only a ready record can be
selected. Installation does not enable persistent analysis. Updates prepare a
new version before retargeting profiles; failures preserve the previous usable
version and settings. Remove only installer-owned, unreferenced, stopped data.
Never delete an external executable or a shared language runtime.

C-g or modal Escape cancels a focused management interaction. An explicit
cancel command can cancel its job while editing remains active. Cancellation
and confirmed editor exit terminate/reap owned process groups without signaling
unrelated processes. Retain truthful incomplete records after interruption.
Corrupt or unknown-version registry data blocks destructive management, not
ordinary editing or external-server use.

#### Scenario: Cancel an update while editing

- **GIVEN** a selected ready installation and a replacement job
- **WHEN** the user cancels the job and continues typing
- **THEN** the job stops within its bounded shutdown policy
- **AND** the active installation and user's text remain intact
- **AND** no helper continues as an unowned background process.

## Bounds and acceptance

The model defines finite defaults and admitted ranges. Exercise real filesystem
publication, source conflicts, symlink preservation, cross-instance locking,
missing tools, interrupted acquisition, invalid digests, failed probes,
unsupported platform, update rollback, shared-profile references and cleanup.
Use controlled local acquisition fixtures for repeatable failures; distinguish
these from actual upstream installation evidence.

Final acceptance includes real Rust and Go server use, one managed acquisition,
both startup interaction profiles, preserved native fallback, and an existing
server for a non-bundled language. A small protocol fixture can establish
grammar-independent routing but cannot prove a particular external server's
semantic quality. Inspect all recipe prerequisite/provenance paths; report
which secondary servers were actually installed and initialized.

Document the ordinary setup path with short commands and UI excerpts. Keep
full fields, recipes and limitations in reference documentation. Markdown is
the source of the matching Spore guide. Complete configuration sample/default
coverage is part of delivery. No new CLI installer surface is required.

## Dependencies and reference review

The delivered LSP client, configuration/XDG owners, native command/prompt/job
surfaces and source precedence are prerequisites. The paired model specifies
the narrow persistence delta. Language/grammar installation remains independent.
No user machine installation is performed merely by approving this document.

Primary sources reviewed on 2026-11-02; concrete resolved versions must be
recorded when implementation actually installs a server:

- [rust-analyzer installation](https://rust-analyzer.github.io/book/installation.html)
- [gopls installation](https://go.dev/gopls/)
- [TypeScript Language Server](https://github.com/typescript-language-server/typescript-language-server)
- [Web language servers](https://github.com/hrsh7th/vscode-langservers-extracted)
- [Marksman](https://github.com/artempyanykh/marksman)
- [Ruby LSP editors](https://shopify.github.io/ruby-lsp/editors.html)
- [Python LSP Server](https://github.com/python-lsp/python-lsp-server)
- [Odin Language Server](https://github.com/DanielGavin/ols)
- [Orgwise](https://github.com/PoiScript/orgwise)
- [TOML edit preservation](https://docs.rs/toml_edit/latest/toml_edit/)
