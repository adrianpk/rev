---
id: TKT-20261102111452
title: Remaining optional recipes
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: ops
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:52Z
ready_at: 2026-11-02T11:14:52Z
started_at: 2026-11-02T15:00:15Z
branch: feat/rev-language-server-installation-05-catalog-recipes
reviewed_at: 2026-11-02T15:36:44Z
commits: [5ebe8d85a6a82b2ce44f290b42770c387439b4ee, 829a4211fdc61fd36498ec4bb52b1e99304d321d]
---

# Remaining optional recipes

## Outcome

Add verified Marksman native acquisition, isolated Ruby gems and a fixed-path minimal Python virtual environment.

## Current behavior and inputs

The optional stdio LSP client, guided setup and managed Rust/Go/web recipes
are delivered. Preserve the exclusive semantic/native completion policy.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Add verified Marksman native acquisition, isolated Ruby gems and a fixed-path minimal Python virtual environment.
- Add immutable-revision OLS acquisition/build with the installed toolchain and required builtin paths; preserve generic custom setup for Org and all other languages.
- Exercise recipe-specific tool/version failures and available real-server smoke probes; distinguish installed evidence from fixtures and keep all helpers optional.
- Own requirements INS01, INS02, INS03, INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 5; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 4 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
Focused optional acquisition, failure and native workflow checks passed.
Live Marksman, Ruby and base Python transport probes passed. Live OLS correctly
remained incomplete because the installed Odin lacks `core:odin/tool-format`.
See the [Slice 5 report](../../report/slices/rev-language-server-installation/slice-5-catalog-recipes.md)
for commands, provenance and limitations. Controller validation is pending.
