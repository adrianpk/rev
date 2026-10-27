---
id: TKT-20261027190521
title: Stop prolonged movement after repeated vertical arrow input
status: reviewing
kind: bug
severity: unclassified
priority: unclassified
scope: ui
tags: navigation, scrolling, input
source: chat
reported_at: 2026-10-27T19:05:21Z
ready_at: 2026-10-27T20:20:54Z
started_at: 2026-10-27T20:20:54Z
branch: fix/rev-input-01-navigation-responsiveness
reviewed_at: 2026-10-27T20:28:01Z
commits: [db32befb12db284ed56958d719ac3843c3cb71e5, 6ae33afb14c6e0d6973d2bf9238970708e1b91cf, ef908a5081e64d8030062405e95e6293ff95e7c3]
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/81
---

# Stop prolonged movement after repeated vertical arrow input

## Observed behavior

The user reports excessive inertia when navigating with Up or Down. Once the
cursor reaches an edge of the visible document area, vertical movement and
scrolling keep going for an extended period, in either direction.

The user is unsure whether this is a recent regression. No reproduction,
affected build identity or root cause has been verified yet.

## Requested outcome

Keep vertical navigation responsive during key repeat. Releasing the arrow
key must stop cursor movement and viewport scrolling without a perceptible
tail of pending movements.

## Acceptance direction

- Verify both Up and Down when movement starts within the viewport and then
  crosses its top or bottom edge.
- Verify sustained key repeat and release in ordinary and large documents;
  scrolling must stop promptly when input ends, without continued drift.
- Retain accurate single-press movement and sustained navigation while the
  key remains held. Verify reversing direction remains responsive.
- Preserve buffer-start/end bounds, cursor visibility and document bytes,
  dirty state and undo history during navigation.
- Record a real-terminal reproduction and a regression check at the boundary
  responsible for the defect. Measure input-to-visible-update and release-to-stop
  behavior before choosing the correction and its acceptance bound.

## Investigation boundary

Check event consumption, terminal key-repeat delivery, rendering cost and
viewport updates. An accumulated input backlog is a hypothesis, not an
established diagnosis. Do not prescribe event dropping or change system-wide
keyboard repeat settings as part of ticket capture.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.

## Implementation evidence

The [Slice 1 report](../../report/slices/rev-input-navigation/slice-1-navigation-responsiveness.md)
records the baseline and corrected timing measurements. Native event consumption
retains ordered handlers and draws after at most 32 events or 4 ms of event work.
The focused navigation PTY suite covers ordinary/large documents, Up/Down,
release, reversal, bounds, editing/undo, Escape, cancellation and unchanged bytes.
The ticket remains reviewing until canonical integration is verified.
