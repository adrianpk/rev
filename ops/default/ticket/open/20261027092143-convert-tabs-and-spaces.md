---
id: TKT-20261027092143
title: Add explicit tabs-to-spaces and spaces-to-tabs commands
status: open
kind: task
severity: unclassified
priority: unclassified
scope: domain
tags: whitespace, indentation
source: chat
reported_at: 2026-10-27T09:21:43Z
commits: []
---

# Add explicit tabs-to-spaces and spaces-to-tabs commands

## Requested outcome

Provide explicit commands that convert existing tab characters to spaces and
spaces to tab characters under a documented tab-stop policy.

This is a future editing capability, separate from how the Tab key advances
and from configuring indentation style or tab display width. Loading settings
or displaying a tab must not perform this conversion implicitly.

## Acceptance direction

- Provide both conversion directions through the normal Rust command owner.
- Use the applicable tab stops and account for the current display column;
  replacing every fixed group of spaces blindly is insufficient.
- Preserve text outside the chosen conversion scope, including Unicode
  graphemes, line separators, BOM and final-newline state.
- Publish a conversion as one coherent edit with normal undo/redo, dirty-state
  tracking and shared-view position updates. A no-op conversion stays a no-op.
- Respect read-only buffers and retain ordinary cancellation and error behavior.

## Selected conversion contract

[Explicit Whitespace Conversion](../../spec/editor-whitespace-conversion.md)
selects marked linear regions, all eligible selected whitespace and preserved
visual columns through native tabify/untabify. No usable region causes feedback,
not implicit whole-buffer conversion. Existing region admission remains in force.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.
