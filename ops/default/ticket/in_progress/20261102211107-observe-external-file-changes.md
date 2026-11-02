---
id: TKT-20261102211107
title: External change observation
status: in_progress
kind: task
severity: unclassified
priority: normal
scope: ops
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:07Z
ready_at: 2026-11-02T21:11:07Z
started_at: 2026-11-02T21:45:24Z
branch: feat/rev-ai-pi-02-observation
commits: []
---

# External change observation

## Outcome

Use the already locked rustix filesystem/event APIs for bounded parent-directory notifications and native session wake integration; no recursive project watcher or periodic content reads.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Use the already locked rustix filesystem/event APIs for bounded parent-directory notifications and native session wake integration; no recursive project watcher or periodic content reads.
- Apply reload/ask policies, coalesced per-buffer notices, dirty conflict protection and explicit reload; retain subscriptions through atomic rename, deletion/recreation and shared/hidden buffers.
- Verify native notifications, overflow/registration limits, cleanup, idle behavior and canonical invalidation of completion, bookmarks, Org projections and LSP state.
- Own EFC01-EFC08; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 2. Begin only after Slice 1 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.
