---
id: TKT-20261102111453
title: Update cancellation and removal
status: in_progress
kind: task
severity: unclassified
priority: normal
scope: persistence
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:53Z
ready_at: 2026-11-02T11:14:53Z
started_at: 2026-11-02T15:43:51Z
branch: feat/rev-language-server-installation-06-lifecycle
commits: []
---

# Update cancellation and removal

## Outcome

Add explicit update/remove/cancel commands; prepare and verify new versions before atomically retargeting still-owned profiles.

## Current behavior and inputs

The optional stdio LSP client, guided setup and initial managed recipe catalog
are delivered. Preserve the exclusive semantic/native completion policy.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Add explicit update/remove/cancel commands; prepare and verify new versions before atomically retargeting still-owned profiles.
- Handle shared web-profile references, source conflicts, incomplete jobs and bounded cache cleanup without deleting external or running payloads.
- Verify process-group stop/reap, editor-exit cleanup, concurrent instances, failed replacement and preservation of ordinary editing.
- Own requirements INS02, INS03, INS04, INS05, INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 6; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 5 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
No execution result is claimed by this ready ticket.
