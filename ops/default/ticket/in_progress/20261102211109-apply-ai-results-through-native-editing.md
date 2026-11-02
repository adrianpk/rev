---
id: TKT-20261102211109
title: Checked AI editing
status: in_progress
kind: task
severity: unclassified
priority: normal
scope: ui
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:09Z
ready_at: 2026-11-02T21:11:09Z
started_at: 2026-11-02T23:14:42Z
branch: feat/rev-ai-pi-04-results
commits: []
---

# Checked AI editing

## Outcome

Add ai-prompt and ai-implement with explicit point/selection capture and the bounded version-1 generated-text contract; display proposals through the native transient-buffer owner.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Add ai-prompt and ai-implement with explicit point/selection capture and the bounded version-1 generated-text contract; display proposals through the native transient-buffer owner.
- Deliver explicit ai-apply-result and ai-discard-result with source revision/read-only admission, one native undo step and no implicit source save or model-selected target.
- Fence partial, stale, cancelled, duplicated and oversized results, including local edits, external reloads and focus changes during generation.
- Own PI03, PI06, PI07; AI03, AI05-AI09, AI14-AI17; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 4. Begin only after Slice 3 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.
