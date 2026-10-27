---
id: TKT-20261027092143
title: Add explicit tabs-to-spaces and spaces-to-tabs commands
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: domain
tags: whitespace, indentation
source: chat
reported_at: 2026-10-27T09:21:43Z
ready_at: 2026-10-27T20:51:03Z
started_at: 2026-10-27T20:51:03Z
reviewed_at: 2026-10-27T21:20:51Z
branch: feat/rev-input-03-whitespace-conversion
commits: [e015dcfb6104872696571df7f0e361209774c2f7, 28be1f4d52fd44c50ab92c9d586a4fe54b648d60]
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

## Implementation evidence

Slice 3 implements both commands on the existing explicit linear region using
true tab-stop columns, one transaction and ordinary shared-view history. Native
and owned PTY checks cover exact bytes, Unicode/line-format fidelity, undo/redo,
read-only/no-op admission and cancellation. See the [canonical report](../../report/slices/rev-input-navigation/slice-3-whitespace-conversion.md).
Canonical PR references and final clean-HEAD evidence are pending handoff.
