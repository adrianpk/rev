---
id: TKT-20261027194421
title: Group pane status information on the left and right
status: open
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: status-bar, layout
source: chat
reported_at: 2026-10-27T19:44:21Z
commits: []
---

# Group pane status information on the left and right

## Requested outcome

Replace the current concatenated status text with two aligned groups, following
the product owner's preferred distribution while retaining rev's simple style.
The left group contains buffer identity, modified/read-only state and position.
The right group contains the existing syntax/language label.

Selected ordinary presentation:

```text
config.rs -- RW -- L37 C3                                      Rust
```

The existing `--`/`**` marker retains its clean/modified meaning. The additional
separator separates write policy from position; spacing does not replace state.
Existing active-pane indication remains available.

## Specification

[Pane Status Layout](../../spec/editor-status-layout.md) defines the R05
extension, per-view ownership and narrow-width behavior. The selected simple
presentation belongs to this delivery.

## Acceptance direction

- Align against each editing pane's own status rectangle, not terminal width.
- Retain truthful file/buffer, dirty, access, position and syntax labels.
- Verify shared buffers with independent points and distinct active styling.
- Separate groups with flexible space and prevent overlap in narrow splits.
- Use terminal-cell widths and safe clipping for Unicode labels.
- Preserve the single status row, current palette roles and prompt boundaries.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.
