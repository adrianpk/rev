---
id: TKT-20261102211110
title: Guarded directory context
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: api
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:10Z
ready_at: 2026-11-02T21:11:10Z
started_at: 2026-11-02T23:42:32Z
branch: feat/rev-ai-pi-05-discovery
pr: 233
reviewed_at: 2026-11-03T00:04:47Z
commits: [bee7889f191cd961de5f275b24ca7cc26d375aa7, 984889b23b2c0885ccf8c67accd14b5af9d03467]
---

# Guarded directory context

## Outcome

Package a small internal Pi JavaScript resource with rev and load it explicitly through the admitted Pi extension interface; expose only bounded read/list/search operations without a shell or user plugin installation.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Pi and general external-file reload are not yet implemented.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Package a small internal Pi JavaScript resource with rev and load it explicitly through the admitted Pi extension interface; expose only bounded read/list/search operations without a shell or user plugin installation.
- Enforce canonical-root and opened-handle path checks, per-operation budgets and fresh unsaved-source precedence while Pi retains all relevance selection and model/context iteration.
- Validate traversal, symlink/root replacement races, missing/binary/large sources, disabled external resources and actual Pi tool admission; native API transport/discovery remains deferred.
- Own PI02-PI05, PI07; AI06, AI09, AI11-AI13, AI17, AI20; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 5. Begin only after Slice 4 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

Run the slice's exact ordered configured checks and its relevant acceptance.
Record real versus controlled observations and the tested candidate in the
report. No test result, runtime implementation or PR is claimed by this ticket.

## Implementation evidence

The implementation commit `bee7889f191cd961de5f275b24ca7cc26d375aa7` delivers
the internal resource, root/handle/tool admission and native snapshot precedence.
Six focused native discovery cases and 27 actual offline Pi checks passed; the
local controlled model drives real Pi tool execution without an external provider
request. The canonical Slice 5 report records boundaries and commands. Final
clean-head validation follows the PR-reference commit through the controller.

Canonical PR: [#233](https://forge.adrianpk.com/adrianpk/rev/pulls/233).
Report introduction and actual-runtime T5.3 evidence:
`984889b23b2c0885ccf8c67accd14b5af9d03467`.
