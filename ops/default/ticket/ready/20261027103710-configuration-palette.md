---
id: TKT-20261027103710
title: Apply the configured Tokyo Night palette
status: ready
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:10Z
ready_at: 2026-10-27T10:37:10Z
commits: []
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
