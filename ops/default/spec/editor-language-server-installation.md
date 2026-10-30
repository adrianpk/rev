# Language Server Installation

Status: draft
Date: 2026-10-27
Reviewed: 2026-10-31
Format: OpenSpec capability proposal
Source: product-owner discussion; captured for later refinement
Delivery boundary: future capability; no implementation or slicing authorized
Client owner: [Language Server Support](editor-language-server-support.md)
Model: [Language Server Installation Model](editor-language-server-installation-model.md)
Storage reference: [Editor Configuration](editor-configuration.md)
Readiness: language priorities and external payload ownership confirmed; catalog, acquisition and trust policies remain open

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

This is a product priority list, not a claim of equal server availability,
completion quality or completed installation recipes. Select and validate the
actual server/version/runtime combination for each supported entry.

## ADDED Requirements

### Requirement: INS01 Simple optional installation

The user SHALL be able to install a supported server through a simple guided
operation and receive a usable native-client configuration. Existing compatible
servers SHALL be reusable. Missing language runtimes/toolchains and incomplete
installations SHALL be identified with an actionable result rather than a
false success. The exact management commands remain to be selected.

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

The supported catalog SHALL provide reproducible server/launch requirements for
its declared platforms and languages. A user SHALL be able to connect another
compatible installed server through native settings without an editor addon.
A partial failure SHALL NOT destroy a previously usable managed installation.

#### Scenario: A replacement installation fails

- **GIVEN** a usable managed server and an attempted replacement
- **WHEN** acquisition, validation or publication fails
- **THEN** the prior usable installation remains available
- **AND** the management UI reports the actual incomplete or failed operation.

## Open decisions

- Preferred server and validated platform/runtime/version combinations per language.
- Recipes, downloads versus native package tools, verification and update policy.
- Discovery/override precedence for managed, project and PATH-installed servers.
- Managed directory layout, installation records, cancellation and rollback.
- Exact M-x and optional CLI operations; handling and diagnosis of prerequisites.
- Org mode server suitability and precise completion coverage require evaluation.

The model records candidate ownership without freezing a manifest or catalog
format. This draft does not authorize downloads, installations or updates now.

## Dependencies and validation direction

Use the LSP client's admitted launch/configuration contract and existing XDG
configuration/data/cache ownership. Installation is separate from activation:
the client remains usable with unmanaged compatible servers. The listed
language priorities do not establish equal recipe availability or substitute
for checking each chosen server's current prerequisites and distribution terms.

Before planning, select the actual catalog and supported platforms, acquisition
and verification policy, failure/cancellation behavior and managed/external
precedence. Cover existing-server reuse, missing runtimes, unavailable network,
incomplete payloads, failed replacement, cancellation and verified launchability
without implicitly enabling continuous analysis. No machine installation is
performed by this formalization.
