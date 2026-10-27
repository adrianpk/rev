---
id: TKT-20261027103710
title: Apply the configured Tokyo Night palette
status: solved
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:10Z
ready_at: 2026-10-27T10:37:10Z
commits: [8921273e463e2d8cdb5999aede301c984893a7b6, 3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Apply the configured Tokyo Night palette

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG04, CFG18.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 13: Resolve palette and syntax colors; pass its recorded focused checks and report its observable outcome.
- Slice 14: Render UI palette and restore terminal; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 13: [#62](https://forge.adrianpk.com/adrianpk/rev/pulls/62); integrated `8921273e463e2d8cdb5999aede301c984893a7b6`. [Report](../../report/slices/rev-configuration-ui/slice-13-palette-and-syntax.md) records the delivered behavior and exact-head checks.
- Slice 14: [#63](https://forge.adrianpk.com/adrianpk/rev/pulls/63); integrated `3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d`. [Report](../../report/slices/rev-configuration-ui/slice-14-ui-palette.md) records the delivered behavior and exact-head checks.
