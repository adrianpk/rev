---
id: TKT-20261102211112
title: Reusable AI commands
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: persistence
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:12Z
ready_at: 2026-11-02T21:11:12Z
started_at: 2026-11-03T01:06:17Z
branch: feat/rev-ai-pi-07-definitions
pr: 235
reviewed_at: 2026-11-03T01:21:25Z
commits: [8f349aca77a480575a4738c4b5a43d05922e809a, 9509b8d606a1933dc641e0fccd0c0ac3583a2b84]
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

## Implementation evidence

Bounded literal definitions resolve through the shared command/keymap owners.
Successful instructions save through the conflict-checked root publisher and
activate only the chosen definition. Seven focused cases, strict library lint,
all-test compilation and documentation catalogs passed. Native tests use an
owned controlled RPC peer and real isolated settings publication; no external
provider requests are claimed. The Slice 7 report records commands and bounds.
Final clean-head validation follows the PR-reference commits through Looper.

Canonical PR: [#235](https://forge.adrianpk.com/adrianpk/rev/pulls/235).
Report introduction: `0c074ee5a9ee191ffed3280b3d1bf44966877c1d`.
