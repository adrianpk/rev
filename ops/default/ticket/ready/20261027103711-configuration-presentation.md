---
id: TKT-20261027103711
title: Apply existing layout tree and syntax preferences
status: ready
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:11Z
ready_at: 2026-10-27T10:37:11Z
commits: []
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
