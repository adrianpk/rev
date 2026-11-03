---
id: TKT-20261102211113
title: Joint acceptance and guidance
status: solved
kind: task
severity: unclassified
priority: normal
scope: docs
tags: ai, pi, files
source: chat
reported_at: 2026-11-02T21:11:13Z
ready_at: 2026-11-02T21:11:13Z
started_at: 2026-11-03T01:44:04Z
branch: feat/rev-ai-pi-08-acceptance
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/236
blocked_at: 2026-11-03T02:00:38Z
resumed_at: 2026-11-03T08:49:48Z
reviewed_at: 2026-11-03T08:54:31Z
closed_at: 2026-11-03T08:54:31Z
resolution: fixed
commits: [47928e8b376385049bf249ff76fd6e8299a6447d, b8fadd21cef5f9b8c34f3737965cc0ca908f289a, 297a8314f9d1f89104146f7cfdc724193bcb63ed, dbdb11b78c76f3d39c7cb3df6542b064f90a83ba]
---

# Joint acceptance and guidance

## Outcome

Deliver representative native and PTY journeys for both startup profiles, real kernel file notifications and the installed Pi RPC/tool/session lifecycle, with measured idle/RSS/startup limits.

## Inputs and current state

The optional LSP/configuration/native-editing owners are delivered.
Slices 1 through 7 deliver Pi and general external-file reload; joint acceptance and guided workflows remain.
Use [Pi Integration](../../spec/editor-ai-pi-integration.md), its
[model](../../spec/editor-ai-pi-integration-model.md), and
[External File Changes](../../spec/editor-external-file-changes.md) with its
[model](../../spec/editor-external-file-changes-model.md).

## Acceptance

- Deliver representative native and PTY journeys for both startup profiles, real kernel file notifications and the installed Pi RPC/tool/session lifecycle, with measured idle/RSS/startup limits.
- Exercise one bounded real-provider operation using available existing Pi authentication; report unavailable authentication variants distinctly and never invent live evidence from fixtures.
- Document ordinary Pi and reload workflows, complete references/configuration/command catalogs, canonical Markdown and Spore guidance and Unreleased; finish ticket transitions before the unchanged integrated gate and controller closure.
- Own EFC01-EFC08, PI01-PI08; selected shared AI behavior; preserve source, credentials and the admitted operation boundary.
- Keep application code in its existing owners and ordinary editing independent of optional Pi.

## Delivery

Owning [plan](../../plan/rev-ai-pi.md) and [tracker](../../tracker/rev-ai-pi.md):
Slice 8. Begin only after Slice 7 integrates and earlier contracts remain intact.
Use the exact branch/PR/report identity from the plan; Looper owns integration.

## Planned validation

The Slice 8 report records the exact configured checks and actual controlled
versus real-provider observations. Its seven-command receipt validates the
canonical merge head; the integrated delivery-set gate remains pending.

## Implementation evidence

Both-profile offline native/PTY installed-Pi journeys and six focused history
cases passed. The User Guide adds reload and explicit proposal editing; catalogs
pass 429 commands, 231 settings, 70 pages and 434 local links/anchors.

The single bounded real-provider attempt failed provider completion with the
existing configured default and Pi-owned OAuth. Its guard observed one model
turn and zero tool calls. That failure was retained as incident 1. The operator
then authorized one additional bounded operation after renewed OAuth. It passed in 2873 ms with
one turn, zero tools, the existing configured model and unchanged toy source.
No fallback or further request was used; the original failure remains recorded.

## Resolution

The resumed in_progress work completed the authorized live operation. Its result
and the canonical merge were then verified through reviewing before this solved
transition. PR #236 merged at `dbdb11b78c76f3d39c7cb3df6542b064f90a83ba`;
receipt `RJLI2OQ42YTAU7K373I7PFAIKG` passed all seven Slice 8 checks and 61 suite
cases, plus the nested save case. The [Slice 8 report](../../report/slices/rev-ai-pi/slice-8-acceptance.md)
preserves both provider observations and actual native/PTY/resource evidence.
The controller owns the pending integrated gate and delivery-set closure.
