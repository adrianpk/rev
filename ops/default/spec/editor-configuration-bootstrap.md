# Configuration Bootstrap

Status: accepted
Approved: product owner, 2026-10-27; finalize the selected seven-topic scope and start one combined Looper cycle.
Date: 2026-10-27
Format: OpenSpec change
Direction: product owner requested automatic creation of absent user configuration
Baseline: [Editor Configuration](editor-configuration.md), CFG01, CFG05, CFG14 and CFG15
Existing model: [Editor Configuration Model](editor-configuration-model.md)
Ticket: [Create missing user configuration](../ticket/in_progress/20261027193636-create-missing-user-configuration.md)
Delivery boundary: selected combined input/navigation/presentation delivery; no implementation started

## Purpose

Make the complete editable user configuration available automatically on the
first normal editor startup. Preserve existing settings and the editor's
self-contained binary, XDG resolution and recoverable startup behavior.

This proposal explicitly replaces the current prohibition on startup creation.
The completed configuration/UI delivery and its validation record remain
historical evidence. Existing loading/validation helpers remain read-only;
provisioning belongs to the application startup path.

## MODIFIED Requirements

### Requirement: CFG01 XDG locations and ownership

`rev` SHALL resolve the following locations through their XDG variables, using
the defaults shown when a variable is unset or empty. Relative variable values
SHALL be ignored as invalid. The configuration filename SHALL be `config.toml`.

| Responsibility | Variable | Default project directory |
| --- | --- | --- |
| User settings | `XDG_CONFIG_HOME` | `~/.config/rev/` |
| Durable user data and possible installed resources | `XDG_DATA_HOME` | `~/.local/share/rev/` |
| Restart-persistent operational state | `XDG_STATE_HOME` | `~/.local/state/rev/` |
| Regenerable nonessential data | `XDG_CACHE_HOME` | `~/.cache/rev/` |

Normal editor startup SHALL provision the resolved user configuration when
no destination entry exists, creating its configuration parents only as needed.
The file SHALL use the packaged complete reference template under CFG23/CFG24.
No existing destination entry SHALL be rewritten or replaced by provisioning.
Data, state and cache directories SHALL NOT be created merely to start the
editor. User documents SHALL NOT be redirected into these directories. The
existing distinction between configuration, data, state and cache is retained.

#### Scenario: Custom configuration home

- **GIVEN** an absolute `XDG_CONFIG_HOME` and no other location override
- **WHEN** `rev` locates user settings
- **THEN** it reads `rev/config.toml` relative to that directory
- **AND** data, state, and cache retain their independently resolved defaults.

#### Scenario: Missing user configuration

- **GIVEN** no user configuration file exists
- **WHEN** the editor starts
- **THEN** it creates the complete default-equivalent configuration at the
  resolved user path and uses the ordinary validated startup settings
- **AND** no unrelated data, state or cache directory is created.

### Requirement: CFG14 Startup errors keep usable defaults

A missing implicit user root SHALL trigger the bounded provisioning attempt
under CFG01/CFG24. If that attempt cannot complete, startup SHALL retain usable
complete built-ins and report the provisioning failure without blocking editing.
A present malformed, unreadable, or semantically invalid candidate SHALL be
reported with useful source, key, and location context where available.
Startup SHALL keep the editor usable with complete built-ins and a visible
configuration diagnostic; the rejected candidate SHALL have no partial effect.

Provisioning SHALL be limited to an absent destination entry. The loader
SHALL NOT repair, rewrite, or overwrite existing user configuration as a side
effect of reading it. Secrets or unrelated environment contents SHALL NOT
be copied into diagnostics. An explicitly requested broken include is not an
absent optional root file.

#### Scenario: Bad color in an otherwise valid file

- **GIVEN** a configuration changes editing settings but contains an invalid color
- **WHEN** startup validates the whole candidate
- **THEN** the error identifies the bad key/source
- **AND** no mixture of accepted and rejected file settings becomes effective.

#### Scenario: Configuration cannot be created

- **GIVEN** no configuration file and an unwritable configuration location
- **WHEN** normal editor startup attempts provisioning
- **THEN** editing continues with complete factory defaults and a useful diagnostic
- **AND** startup does not claim that the file was created successfully.

### Requirement: CFG15 Startup-only application

The editor SHALL read and validate configuration at startup. The resulting
effective configuration SHALL remain fixed for that session; changing source
files SHALL NOT trigger manual or automatic reload in this delivery.

Startup application SHALL be complete and coherent. The sole filesystem
creation exception is provisioning the absent user configuration under
CFG01/CFG24 before ordinary loading. Location resolution, parsing, includes,
validation and applying settings SHALL NOT independently create files or
directories. Startup application SHALL NOT silently save user documents,
execute a bound command or treat preferences as document edits. Runtime reload
and automatic change watching remain future capabilities.

#### Scenario: Configuration edits wait for the next startup

- **GIVEN** an editor session uses valid settings and contains unsaved text
- **WHEN** the configuration file changes on disk
- **THEN** the running session retains its settings and unsaved text
- **AND** a later startup reads and validates the changed configuration.

## ADDED Requirements

### Requirement: CFG23 Packaged exhaustive default-equivalent template

The binary SHALL contain the complete supported configuration template. It
SHALL NOT require a repository checkout, separately installed sample or network
access to provision it. Every active value SHALL equal the corresponding
compiled default; optional includes, language overrides and unused examples
SHALL remain commented. The complete palette SHALL appear explicitly.

The generated file SHALL describe the supported configurable universe with
English comments and the existing documented bounds/options. Fields SHALL be
added only alongside implemented capabilities. The combined delivery's gutter
and pointer fields SHALL enter this template when those capabilities ship;
future unimplemented features SHALL NOT appear as usable settings.

Catalog checks SHALL compare the packaged template, reference sample and typed
defaults so a second independently maintained default set cannot drift.

#### Scenario: First startup without configuration

- **GIVEN** a standalone binary, no user configuration and no reference files
- **WHEN** the editor provisions its user configuration
- **THEN** the complete generated TOML validates through the ordinary schema
- **AND** its effective settings equal complete factory defaults.

### Requirement: CFG24 Preserve existing entries and publish a complete file

Provisioning SHALL never replace, truncate or repair an existing destination
entry, including an empty/invalid file, directory or symbolic link. Such a path
SHALL retain the normal read/diagnostic behavior of the existing loader.

First-start publication SHALL expose a complete file without overwriting a
configuration created concurrently. An interrupted or failed write SHALL NOT
publish a truncated configuration as success. Temporary resources SHALL have
bounded ownership and cleanup; no periodic writer or idle work is introduced.

After another process wins publication, startup SHALL load the existing winner
through normal validation. It SHALL NOT overwrite that file with the template.
Only missing configuration parents needed for this operation may be created.

#### Scenario: Preserve an existing customized file

- **GIVEN** a user configuration already exists with custom values or invalid text
- **WHEN** the editor starts
- **THEN** provisioning leaves its bytes and directory entry unchanged
- **AND** the ordinary loader determines effective settings or fallback.

#### Scenario: Two first starts race

- **GIVEN** two processes observe the same missing configuration
- **WHEN** one publishes a complete file before the other
- **THEN** the second process preserves and validates that published file
- **AND** neither process exposes a partial template or overwrites the winner.

## Validation obligations

Cover absent parents/file, custom XDG roots, unresolved roots, existing valid,
empty, invalid and unreadable files, symbolic links, non-file destinations,
permission errors, interrupted publication and concurrent startup. Verify full
template/schema/default equality and ordinary terminal startup and cleanup.
Pure path resolution and explicit configuration loading must retain their
read-only behavior. No runtime or gate result is claimed by this proposal.

## Review state

Automatic creation when absent, default-equivalent template publication and
recoverable failure handling are selected for the combined delivery. No new
settings schema or separate model is introduced. Runtime implementation and
final acceptance evidence remain required.
