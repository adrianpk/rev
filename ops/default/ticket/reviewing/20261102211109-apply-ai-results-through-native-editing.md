---
id: TKT-20261102211109
title: Checked AI editing
status: reviewing
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
pr: 232
reviewed_at: 2026-11-02T23:32:17Z
commits: [b9dbf29d6b89f49d1735c15e3104470fba9de1c2, 4a4fb82f913f1e2a0bf376ffc3932324c34f920c]
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

## Implementation evidence

Canonical PR [#232](https://forge.adrianpk.com/adrianpk/rev/pulls/232) delivers
captured instruction targets and explicit native proposal apply/discard. Ten new
focused integration cases and two private live-admission cases passed, including
native undo/shared views, exact bytes, stale/readonly/association/closed-source
guards and real external reload. The canonical report introduction is
`4a4fb82f913f1e2a0bf376ffc3932324c34f920c`. Final clean-head validation follows
these committed PR references and is retained by the controller and canonical PR.
