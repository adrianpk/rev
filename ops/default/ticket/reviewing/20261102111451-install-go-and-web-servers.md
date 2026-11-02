---
id: TKT-20261102111451
title: Go and web installation
status: reviewing
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
reviewed_at: 2026-11-02T14:50:44Z
pr: 225
commits: [659399f44008122423f8de04eaa40caccc078ef0, 50aac5222ff076ba4001bfbeb9b8c7ce674de183, d5a5cea182cb1f6b5869b4482ad3e9ac6feb6a2f]
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

## Validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
All three tasks are implemented; focused acquisition, native and source checks
passed. Actual managed gopls v0.24.0 provided unsaved semantic completion;
TypeScript Language Server/TypeScript 6.0.3 and web package 4.10.0 passed native
transport probes on Node v26.8.1/npm 12.0.2. See the
[Slice 4 report](../../report/slices/rev-language-server-installation/slice-4-go-web.md)
for exact commands, provenance and limitations. All four configured commands
passed at `d5a5cea182cb1f6b5869b4482ad3e9ac6feb6a2f` with receipt
`HKEDMZBY74EBCBV43XPUOV3CKK` (116 deterministic cases). Canonical PR: #225.
The final PR-reference head requires its own controller receipt before integration.
