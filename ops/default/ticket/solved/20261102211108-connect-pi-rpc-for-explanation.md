---
id: TKT-20261102211108
title: Pi RPC explanation
status: solved
kind: task
severity: unclassified
priority: normal
scope: api
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:08Z
ready_at: 2026-11-02T21:11:08Z
started_at: 2026-11-02T22:33:17Z
branch: feat/rev-ai-pi-03-rpc
pr: 231
closed_at: 2026-11-02T23:02:57Z
resolution: fixed
reviewed_at: 2026-11-02T22:59:13Z
commits: [8725da90411ada30b14fb5371c9be4dd06f7d8f7, 9cc5443b89a5bb11f89e3b5cb769b099f2af3049, 0e81da925428984974e1f23251d1972566020a69, 18dfaa6cc4c0ef0875b9330dc27bc325733e911f]
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

## Implementation evidence

Canonical PR [#231](https://forge.adrianpk.com/adrianpk/rev/pulls/231) delivers
checked lazy Pi RPC and input-only selection explanation through native commands
and read-only buffers. Source ownership, ongoing editing, prompt cancellation,
protocol bounds and owned process shutdown are covered by the Slice 3 report.
Real offline Pi 1.3.0 get_state/shutdown probes passed without provider generation.
Final clean-head evidence follows the committed PR references and is retained by
the controller and canonical PR validation. The report introduction is
`18dfaa6cc4c0ef0875b9330dc27bc325733e911f`.

## Closure

PR #231 is integrated at `eb2b45803856e7707ffc122ad697ef02d84a9fc7`.
Receipt `N77GD6JFCVJQGKPXJ7QCEPDOZA` passed all configured Slice 3 checks
and 155 tests. The linked report retains exact focused and offline observations.
