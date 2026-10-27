---
id: TKT-20261027103711
title: Apply existing layout tree and syntax preferences
status: solved
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:11Z
ready_at: 2026-10-27T10:37:11Z
commits: [e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13, 167a49e33369d48cf5fcb009019ccd9303dde736, 9ec177b137cc84d566f961fac8422ad741d55fe6, c946be7f0ee451f2c554b457d5443451eff57797]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Apply existing layout tree and syntax preferences

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG18-CFG20.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 10: Define panel geometry contracts; pass its recorded focused checks and report its observable outcome.
- Slice 11: Compose editing panels; pass its recorded focused checks and report its observable outcome.
- Slice 12: Compose tree and auxiliary panels; pass its recorded focused checks and report its observable outcome.
- Slice 15: Apply presentation and syntax preferences; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 10: [#59](https://forge.adrianpk.com/adrianpk/rev/pulls/59); integrated `e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13`. [Report](../../report/slices/rev-configuration-ui/slice-10-panel-contracts.md) records the delivered behavior and exact-head checks.
- Slice 11: [#60](https://forge.adrianpk.com/adrianpk/rev/pulls/60); integrated `167a49e33369d48cf5fcb009019ccd9303dde736`. [Report](../../report/slices/rev-configuration-ui/slice-11-editor-panels.md) records the delivered behavior and exact-head checks.
- Slice 12: [#61](https://forge.adrianpk.com/adrianpk/rev/pulls/61); integrated `9ec177b137cc84d566f961fac8422ad741d55fe6`. [Report](../../report/slices/rev-configuration-ui/slice-12-tree-panels.md) records the delivered behavior and exact-head checks.
- Slice 15: [#64](https://forge.adrianpk.com/adrianpk/rev/pulls/64); integrated `c946be7f0ee451f2c554b457d5443451eff57797`. [Report](../../report/slices/rev-configuration-ui/slice-15-presentation-preferences.md) records the delivered behavior and exact-head checks.
