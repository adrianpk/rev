---
id: TKT-20261102211112
title: Reusable AI commands
status: ready
kind: task
severity: unclassified
priority: normal
scope: persistence
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:12Z
ready_at: 2026-11-02T21:11:12Z
commits: []
---

# Reusable AI commands

## Outcome

Admit bounded literal ai.commands definitions and dynamic ai-name resolution in the shared M-x/colon catalog, preserving typed input/context/output combinations and name collision rules.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Admit bounded literal ai.commands definitions and dynamic ai-name resolution in the shared M-x/colon catalog, preserving typed input/context/output combinations and name collision rules.
- Implement ai-save-command through the existing format-preserving conflict-checked root publisher; activate the validated definition while retaining ordinary startup keybinding semantics.
- Verify include/source conflicts, user comments, profile keymaps, narrow input context after broad directory history and reuse with a fresh native target and optional Pi model.
- Own PI01, PI06, PI08; AI03, AI04, AI14, AI18; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 7. Begin only after Slice 6 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.
