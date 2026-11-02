---
id: TKT-20261102211106
title: Checked file reload
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: domain
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:06Z
ready_at: 2026-11-02T21:11:06Z
started_at: 2026-11-02T21:20:38Z
branch: feat/rev-ai-pi-01-reload
pr: 229
reviewed_at: 2026-11-02T21:32:00Z
commits: [256c17b2b360bb72b62ed42fa79581afcb7d1898, 1dbe66439e2273fda440996e08bbd399751925af, 7232bd69a5f58aebda0407b9df6bea5fd7fbb7b4]
---

# Checked file reload

## Outcome

Add the proposed file-policy settings and explicit revert-buffer command through the native file/buffer owners, with complete startup schema/default/sample coverage.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Add the proposed file-policy settings and explicit revert-buffer command through the native file/buffer owners, with complete startup schema/default/sample coverage.
- Capture and validate fresh disk bytes asynchronously; preserve unsaved text through revision-bound discard consent and publish one native undoable reload with checked clean-state evidence.
- Cover invalid/missing/readonly sources, disk and typing races, undo/redo, all shared views and unchanged-byte observations without a second text or history owner.
- Own EFC01, EFC03-EFC07; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 1. Prerequisites are the delivered file/buffer/configuration owners.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.

## Implementation evidence

Startup policy/default/schema/sample/reference coverage and the native
revert-buffer owner are implemented. The command uses bounded worker IO,
revision/destination-bound discard consent and one native undoable publication.
Focused validation passed: 11 external reload tests, configuration startup/include
checks, all-target compilation and documentation catalogs. The Slice 1 report and
canonical PR/controller receipt retain the candidate validation evidence.
