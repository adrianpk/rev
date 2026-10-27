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
branch: feat/rev-input-06-relative-gutter
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/86
commits: [3762cf7f1f26a5a8244f5ce904214d57ef5d645a, e3364ba61da2cb8fb8fbd8ef295f7079dc87b8ca, 3e83ab48b1d45bbc1b2f1d10630bca2b9dd9586c, 469b7f0a07fea889a24c94cb1f54e8316f84747b, 35c6006d2634764a37059590946d78b67a03cc44, a90d4e9a12a3a7b6314a20dd058bde9dba04e2aa, 6a1d15d697d76622bb3b397b1c3b50def73bced4, 9f491efaf72c48d9770ccfe4060ddd17bfa68c12, cf4ebcd56703127cfd5cb8c24f1fe91e77c2126e]
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
Slice 4 integrated into dev at `db5006cb038a644b4786d015f80d83afeb33daaf`; canonical evidence `AVX4W4B2M4OCHU3UXSQUUIYFON` and focused evidence `PZDKOF6JKENREIFMCNYTWMY3KP` passed. The ticket remains in progress through its remaining assigned primary slices 5-8.

## Slice 5 evidence

The [absolute rendering report](../../report/slices/rev-input-navigation/slice-5-absolute-gutter.md)
records real logical labels, final text/cursor/overlay bounds, semantic roles,
ASCII output and native/owned-terminal fidelity. Canonical PR: [#85](https://forge.adrianpk.com/adrianpk/rev/pulls/85).
Slice 5 integrated into dev at `d4707f0381b0a20b4c0558724c321710b3ef8f20`; canonical evidence `IEKFELNPRVBKZWGWRGSE5HA5SC` and focused evidence `6NOINAR4Q7ZBHZM5UCKOBJGA7J` passed. The ticket remains in progress through primary slices 6-8.

## Slice 6 evidence

The [relative rendering report](../../report/slices/rev-input-navigation/slice-6-relative-gutter.md)
records view-local logical distances, independent current-line forms, stable
width, composed shared panes and native file/buffer/edit/history lifetime.
Canonical PR: [#86](https://forge.adrianpk.com/adrianpk/rev/pulls/86).
Slice 6 integrated into dev at `3e1f4b9651fed9306949ca2679c16859a5ed916d`; canonical evidence `F47M2RWK5WRIAP3PNKT3GJTDUI` and focused evidence `X5QA4V5IQIY5WXOUYBMXEZ2WOP` passed. The ticket remains in progress through primary slices 7-8.
