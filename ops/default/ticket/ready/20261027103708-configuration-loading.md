---
id: TKT-20261027103708
title: Load declarative XDG TOML configuration
status: ready
kind: task
severity: unclassified
priority: normal
scope: ops
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:08Z
ready_at: 2026-10-27T10:37:08Z
commits: []
---

# Load declarative XDG TOML configuration

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG01-CFG03, CFG07-CFG08, CFG14-CFG15, CFG21.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 1: Model typed configuration; pass its recorded focused checks and report its observable outcome.
- Slice 2: Validate TOML and diagnostics; pass its recorded focused checks and report its observable outcome.
- Slice 3: Load bounded include graphs; pass its recorded focused checks and report its observable outcome.
- Slice 4: Resolve layered settings; pass its recorded focused checks and report its observable outcome.
- Slice 20: Activate startup configuration; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.
