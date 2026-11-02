---
id: TKT-20261102111449
title: Catalog and guided setup
status: ready
kind: task
severity: unclassified
priority: normal
scope: ui
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:49Z
ready_at: 2026-11-02T11:14:49Z
commits: []
---

# Catalog and guided setup

## Outcome

Add the finite recipe metadata catalog and lsp-setup/lsp-servers/lsp-select-server interactions, including custom languages and existing executables.

## Current behavior and inputs

The optional stdio LSP client and exclusive semantic/native completion policy
are delivered. Guided management and managed installation are not yet delivered.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Add the finite recipe metadata catalog and lsp-setup/lsp-servers/lsp-select-server interactions, including custom languages and existing executables.
- Inherit known associations and selected profile settings; ask for patterns only when needed, and display native-only/external/managed readiness without network discovery.
- Cover selector cancellation, both interaction profiles, unknown language identities and registry/catalog independence.
- Own requirements INS01, INS03, INS04; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 2; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 1 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
No execution result is claimed by this ready ticket.
