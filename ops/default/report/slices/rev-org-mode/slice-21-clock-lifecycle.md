# Slice 21: switch and recover Org task clocks

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-21-clock-lifecycle`
PR: pending

## Purpose

Deliver checked session clock transitions and explicit recovery under the
[approved plan](../../../plan/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks)
and [tracker](../../../tracker/rev-org-mode.md#slice-21-switch-and-recover-org-task-clocks).

## Delivered Behavior

T21.1 switches task clocks using one captured instant: close the previous interval
and open the selected heading after validating both source buffers. Selecting the
same active heading leaves its record unchanged. Completing a clocked task closes
its interval together with the keyword, planning and selected history/note edits.
`org.clock.out_when_done` defaults true. Cancelling a note leaves source and clock
unchanged; accepting it uses the captured completion time.

## Implementation Notes

The existing buffer publication owner now admits source, points, marks and
history capacity separately before synchronous publication. Clock operations
prepare at most two buffers and all actual shared views, including a hidden
participant with no initiating view. No participant publishes until every source,
point, viewport map and history allocation is admitted. Canonical publication
still owns text/history; no duplicate editable document is introduced.

Clock completion coalesces a drawer insertion with its adjacent CLOCK replacement.
Zero-minute removal retains a drawer when task history is inserted into it.
Source identity remains revision-qualified; stale source rejects before either
close or open publishes. Files remain subject to ordinary explicit saves.

## Contracts Added or Changed

- ORG19: one active session clock, checked switching and coherent completion.
- ORG18: bounded source preparation and ordinary edit/history ownership.

## Files of Interest

- `src/session/outline/clocks/`: switching, completion and synchronous publication.
- `src/buffer/edit.rs`, `src/window/points.rs`: shared publication admission.
- `src/session/outline/tasks/repeat.rs`: captured completion through note entry.
- `tests/org_editing/clock_switching.rs`: native switching and completion boundaries.

## Validation

T21.1 development checks passed:

- `cargo check --quiet`.
- `cargo test --test org_editing clock`: 16 selected tests, zero skips.
- `cargo test --test configuration org_clocks`: two policy cases, zero skips.
- `cargo test --test editing --test windows`: 96 edit and 41 shared-window tests,
  zero skips; approximately 15 seconds warm, selected because publication admission changed.
- `cargo clippy --lib --test org_editing --test configuration -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 118 schema/catalog entries,
  46 RGB roles, 169 default bindings and 14779 packaged template bytes agree.
- Changed-source quality passed at the original safe-Rust, parameter, nesting
  and complexity limits.

A policy-off fixture initially expected the Org clock provider's timestamp while
using the ordinary task logger. Its task clock is now explicitly injected too;
production timestamp and policy behavior were unchanged.

## Risks and Follow-ups

T21.2 and T21.3 remain in progress. The delivery-set full gate has not run.
