---
id: TKT-20261027193500
title: Deliver the approved native line-number gutter
status: in_progress
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: gutter, line-numbers
source: chat
reported_at: 2026-10-27T19:35:00Z
ready_at: 2026-10-27T21:28:15Z
started_at: 2026-10-27T21:28:15Z
branch: refactor/rev-input-04-gutter-policy
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/84
commits: [3762cf7f1f26a5a8244f5ce904214d57ef5d645a, e3364ba61da2cb8fb8fbd8ef295f7079dc87b8ca, 3e83ab48b1d45bbc1b2f1d10630bca2b9dd9586c]
---

# Deliver the approved native line-number gutter

## Requested outcome

Include the accepted gutter capability in the next combined Looper delivery,
together with the current input, navigation and configuration tickets.

The approved [specification](../../spec/editor-gutter.md) and
[model](../../spec/editor-gutter-model.md) own its behavior, controls, defaults,
geometry, performance bounds and validation obligations. Preserve their
accepted scope: absolute/relative logical line numbers implemented in Rust.

## Acceptance direction

- Deliver all accepted GUT requirements and the companion control catalog.
- Verify independent shared views, left/right placement, narrow geometry,
  Unicode/tabs, overlays, runtime commands and startup settings.
- Verify bounded visible-row work and event-driven idle behavior.
- Integrate gutter geometry before validating pointer hit mapping and text
  selection in the combined delivery. Gutter labels must never become text.
- Do not infer bookmark or LSP implementation from future references.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.

## Slice 4 evidence

The [native policy report](../../report/slices/rev-input-navigation/slice-4-gutter-policy.md)
records independent view policy, split/rebind/close lifetime and checked integer/
cell geometry. Twelve native groups cover real session paths, source/view
invalidation and boundary partitions. Canonical PR: [#84](https://forge.adrianpk.com/adrianpk/rev/pulls/84).
The ticket remains in progress through its remaining assigned primary slices 5-8.
