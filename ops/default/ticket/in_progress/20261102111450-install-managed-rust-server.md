---
id: TKT-20261102111450
title: Managed Rust installation
status: in_progress
kind: task
severity: unclassified
priority: normal
scope: ops
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:50Z
ready_at: 2026-11-02T11:14:50Z
in_progress_at: 2026-11-02T12:29:53Z
branch: feat/rev-language-server-installation-03-managed-rust
commits: []
---

# Managed Rust installation

## Outcome

Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.

## Current behavior and inputs

The optional stdio LSP client, guided existing-server setup and exclusive
semantic/native completion policy are delivered. Managed acquisition is active
implementation in this ticket.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.
- Acquire an exact official Rust server release with integrity verification and finite structured helper processes; initialize/shutdown in an owned empty root before publication.
- Publish ready inventory before conflict-checked selection; cover missing prerequisites, corrupt downloads, invalid registry, failed probe and installation while LSP is disabled.
- Own requirements INS01, INS02, INS03, INS05, INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 3; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 2 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
T3.1 storage and worker development checks passed; see the tracker for exact
commands and coverage. Acquisition, protocol verification and native management
interaction remain pending. No completed installation or PR readiness is claimed.
