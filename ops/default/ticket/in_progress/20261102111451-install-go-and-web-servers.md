---
id: TKT-20261102111451
title: Go and web installation
status: in_progress
kind: task
severity: unclassified
priority: normal
scope: ops
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:51Z
ready_at: 2026-11-02T11:14:51Z
started_at: 2026-11-02T14:16:56Z
branch: feat/rev-language-server-installation-04-go-web
commits: []
---

# Go and web installation

## Outcome

Add isolated gopls acquisition with exact module provenance and local-only compiler selection.

## Current behavior and inputs

The optional stdio LSP client, guided management and managed Rust installation
are delivered. Preserve the exclusive semantic/native completion policy.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Add isolated gopls acquisition with exact module provenance and local-only compiler selection.
- Add exact npm-prefix TypeScript/JavaScript and HTML/CSS/JSON recipes with script execution disabled, package locks and separate launch profiles.
- Verify prerequisites, package normalization, inherited associations and one semantic/native source; record actual supported package versions and real Go server evidence.
- Own requirements INS01, INS02, INS03, INS04, INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 4; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 3 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
No execution result is claimed by this ready ticket.
