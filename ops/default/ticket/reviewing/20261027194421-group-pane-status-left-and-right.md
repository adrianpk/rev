---
id: TKT-20261027194421
title: Group pane status information on the left and right
status: reviewing
ready_at: 2026-10-27T22:46:43Z
started_at: 2026-10-27T22:46:43Z
reviewed_at: 2026-10-27T23:00:09Z
branch: feat/rev-input-09-status-layout
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/89
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: status-bar, layout
source: chat
reported_at: 2026-10-27T19:44:21Z
commits:
  - 5aa3d1ed4d6a46bc8825d29454c4eaa0681cb99c
  - e85c4ea4dcd2496bbd4f37aae4aac2e7f98e82b4
  - 6e16780043d3181e43085c0777f2f49e4922b20e
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


## Implementation evidence

Primary Slice 9 is prepared in canonical PR [#89](https://forge.adrianpk.com/adrianpk/rev/pulls/89).
The [Slice 9 report](../../report/slices/rev-input-navigation/slice-9-status-layout.md)
records native group/degradation/state checks, two RGB/NO_COLOR status PTYs, four
historical acceptance journeys and source/reference checks. The controller
records final exact-head evidence after this reference commit. Integration and
solved closure remain pending canonical merge.
