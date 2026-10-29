# Slice 30: edit canonical Org source from the agenda

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-30-agenda-actions`
PR: pending

## Purpose

Deliver checked source actions under the [approved plan](../../../plan/rev-org-mode.md#slice-30-edit-canonical-org-source-from-the-agenda)
and [tracker](../../../tracker/rev-org-mode.md#slice-30-edit-canonical-org-source-from-the-agenda).

## Delivered Behavior

T30.1 routes task, schedule/deadline, tag, priority, effort and clock operations
from actual calendar/query entries through ordinary source handlers. Exact
buffer revisions or bounded disk text/metadata reads qualify targets before
canonical buffers are admitted. Read-only and stale targets reject visibly.

## Implementation Notes

The session temporarily addresses the real canonical source while its saved
agenda view participates in the ordinary checked point/viewport maps. Shared
views and queued source anchors map through the same transactions. There are
no synthetic source identities or title-based mutation guesses. Logging/date
prompts remain cancellable. Clock-out/cancel must prove selected task ownership.
Changes remain dirty canonical text and keep ordinary single-operation undo;
source actions do not save files implicitly. The existing owned job handles
source reads and structural analysis, with bounded preparation and cancellation.

## Contracts Added or Changed

- ORG09: checked canonical source actions and retained agenda/origin state.
- ORG23: explicit per-target action evidence.

## Files of Interest

- `src/session/agenda_actions/`: source qualification and ordinary handler bridge.
- `src/window/source_action.rs`: mapped saved view and source anchors.
- `src/org/agenda/actions.rs`: qualified targets and truthful action outcomes.
- `tests/org_agenda/source_actions.rs`: source bytes, cancellation and shared views.

## Validation

- `cargo test --test org_agenda source_action -- --nocapture`: seven cases passed,
  zero failures or ignored tests, including disk BOM/CRLF, undo/redo, stale and
  read-only rejection, task-note cancellation, metadata, clock ownership and peers.
- `cargo clippy --lib --test org_agenda -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter, nesting and complexity
  limits after extracting checked navigation publication into its own owner.
- Final exact-head controller checks remain pending the other slice tasks.

## Risks and Follow-ups

T30.2 and T30.3 remain active. The delivery-set full gate has not run.
