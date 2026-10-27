---
id: TKT-20261027103709
title: Apply buffer and language editing preferences
status: ready
kind: task
severity: unclassified
priority: normal
scope: domain
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:09Z
ready_at: 2026-10-27T10:37:09Z
commits: []
---

# Apply buffer and language editing preferences

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG09-CFG11, CFG16, modified R17/R20.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 5: Identify buffer languages; pass its recorded focused checks and report its observable outcome.
- Slice 6: Project buffer tab policies; pass its recorded focused checks and report its observable outcome.
- Slice 7: Apply indentation and new-file defaults; pass its recorded focused checks and report its observable outcome.
- Slice 8: Capture Emacs-style rectangles; pass its recorded focused checks and report its observable outcome.
- Slice 9: Keep rectangle and path transitions coherent; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.
