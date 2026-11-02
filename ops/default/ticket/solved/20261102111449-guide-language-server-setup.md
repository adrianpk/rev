---
id: TKT-20261102111449
title: Catalog and guided setup
status: solved
kind: task
severity: unclassified
priority: normal
scope: ui
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:49Z
ready_at: 2026-11-02T11:14:49Z
started_at: 2026-11-02T12:00:58Z
branch: feat/rev-language-server-installation-02-catalog
reviewed_at: 2026-11-02T12:18:47Z
pr: 223
commits: [fa45a2e6040ebefdf5a019c199ee1702eeedde1c, 92655f9611e6573982b462edf44ecb534663ea61, 0a736a305f15bd4c1e641b2b445b186b587ac5e2]
---

# Catalog and guided setup

## Outcome

Add the finite recipe metadata catalog and lsp-setup/lsp-servers/lsp-select-server interactions, including custom languages and existing executables.

## Current behavior and inputs

The optional stdio LSP client and exclusive semantic/native completion policy
are delivered. Generic registration and scoped reload are delivered through PR #222.
Catalog guidance and managed installation are not yet delivered.
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

## Validation

Catalog filesystem observations, both-profile native selectors, cancellation,
settings/association inheritance and bounded read-only result checks passed.
See the [Slice 2 report](../../report/slices/rev-language-server-installation/slice-2-catalog.md).
All four configured commands passed at `fa45a2e6040ebefdf5a019c199ee1702eeedde1c` with receipt
`LNZASM2O7S5S2G2FFW3IP35PUR`. Canonical PR: #223. The final PR-reference
head requires its own controller receipt before integration.

Final controller receipt `LLIYHTI5G7DR77NSFEFN34MBQ7` passed all configured
commands at `2b44e552c8d9db9ed4aeb95636c0fdc5dd2bc5a0`, merged by PR #223.
