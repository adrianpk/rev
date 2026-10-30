# Slice 63: reconcile native Org contracts and references

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `docs/rev-org-63-baseline-reconciliation`
PR: pending

## Purpose

Reconcile ORG01, ORG17 and ORG31 with the delivered editor/configuration/view
contracts and native references. The [plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) retain the approved slice boundary.

## Delivered Behavior

The editor baseline admits native Org and explicit, consented operation adapters
while preserving canonical text, coordinate domains, ordinary history, safe saves
and independent views. Configuration and panel models now link the actual native
control owners. References expose clock recovery/report/timer sections and the
previously unindexed capture, columns, source-block and feed pages.

## Implementation Notes

Earlier completed baseline metadata and validated candidates remain unchanged.
Org source/model additions do not claim final delivery acceptance. Clock and
cross-document draft language now points to the delivered policies instead of
unresolved implementation choices. All source/runtime behavior remains governed
by the existing native owners.

## Contracts Added or Changed

- ORG01: reconcile native activation with the base runtime boundary.
- ORG17: connect shipped controls, source contexts and exact startup references.
- ORG31: separate historical validation from current Org evidence.

## Files of Interest

- `ops/default/spec/rev-editor.md` and `rev-editor-model.md`: native Org amendments.
- `ops/default/spec/editor-configuration.md` and model: shared startup owner.
- `ops/default/spec/editor-interaction.md`: compatible retained projections.
- `docs/reference/org-mode.md`, `org-controls.md`, `bindings.md`, `index.md`: complete reference navigation.

## Validation

Implementation checks are in progress. Required commands retain the approved
order. No full delivery gate is run by this slice.

## Risks and Follow-ups

The owning controller still must integrate the remaining approved slice and run
the unchanged final gate before delivery closure.
