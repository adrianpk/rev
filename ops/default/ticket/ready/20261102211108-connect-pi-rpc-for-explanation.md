---
id: TKT-20261102211108
title: Pi RPC explanation
status: ready
kind: task
severity: unclassified
priority: normal
scope: api
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:08Z
ready_at: 2026-11-02T21:11:08Z
commits: []
---

# Pi RPC explanation

## Outcome

Admit typed AI/Pi configuration and own one lazy version-checked RPC child with bounded LF framing, independent streams, continuous consumption and explicit operation generations.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Admit typed AI/Pi configuration and own one lazy version-checked RPC child with bounded LF framing, independent streams, continuous consumption and explicit operation generations.
- Deliver ai-explain for a captured selection through a fresh input-only session and a native read-only result buffer; keep the limited intermediate context visible until directory discovery lands.
- Handle prompt acceptance versus settled completion, errors, unavailable authentication, C-g/modal Escape cancellation and process shutdown while ordinary editing remains responsive.
- Own PI01-PI04, PI07; AI01, AI05, AI07, AI16; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 3. Begin only after Slice 2 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.
