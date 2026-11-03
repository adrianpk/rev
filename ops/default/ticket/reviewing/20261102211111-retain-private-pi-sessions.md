---
id: TKT-20261102211111
title: Private Pi sessions
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: persistence
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:11Z
ready_at: 2026-11-02T21:11:11Z
started_at: 2026-11-03T00:15:21Z
branch: feat/rev-ai-pi-06-sessions
pr: 234
reviewed_at: 2026-11-03T00:58:15Z
commits: [ad97bb9e569c5858eb0f3e33a3ef36fd1618d581, 7ef7b03396602642ca99a408101e867d2b017c78]
---

# Private Pi sessions

## Outcome

Persist bounded schema-1 private session bindings in XDG state while Pi owns its JSONL history; reuse only matching root/model/grant observations.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Persist bounded schema-1 private session bindings in XDG state while Pi owns its JSONL history; reuse only matching root/model/grant observations.
- Implement ai-set-directory, ai-select-model, ai-reset-session and ai-disconnect, with owned idle shutdown, safe restart and fresh ephemeral input-only sessions.
- Cover history isolation, changed roots/providers, compaction and session limits, corrupt records, concurrent instances and actual child exit without exporting credentials.
- Own PI01-PI04, PI07, PI08; AI16-AI20; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 6. Begin only after Slice 5 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.

## Implementation evidence

Private binding storage and native lifecycle/model/directory controls are
implemented, with nine focused native session cases and 27 actual offline Pi
checks passing. Real Pi with a local controlled provider restores prior history
and exits its owned process without external provider requests. The canonical
Slice 6 report records commands and bounds. Final clean-head validation follows
the PR-reference commits through the controller.

Canonical PR: [#234](https://forge.adrianpk.com/adrianpk/rev/pulls/234).
Report introduction: `31b7d41f182361cb9c7de84f8a5a66b5d881f881`.
