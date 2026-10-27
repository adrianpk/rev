---
id: TKT-20261027103713
title: Configure existing commands by input context
status: ready
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:13Z
ready_at: 2026-10-27T10:37:13Z
commits: []
---

# Configure existing commands by input context

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG06, CFG12-CFG13, modified R22.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 17: Validate context keymaps; pass its recorded focused checks and report its observable outcome.
- Slice 18: Dispatch configurable editor bindings; pass its recorded focused checks and report its observable outcome.
- Slice 19: Dispatch transient and tree bindings; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.
