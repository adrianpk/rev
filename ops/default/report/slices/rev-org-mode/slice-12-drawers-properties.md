# Slice 12: edit Org drawers and inherited properties

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-12-drawers-properties`
PR: pending

## Purpose

Deliver checked property edits and inspection with explicit source ownership.

## Delivered Behavior

Local property set/delete, additive storage and inspection use native prompts.
Selective `org.properties.inherit` defaults to empty and resolves nearest bases,
additions, nil barriers and file declarations without inserting inherited text.
Inspection reports source-owner provenance; explicit owner editing targets those
exact stored value ranges.
New drawers follow heading/planning metadata. Existing drawer folding retains
its independent view behavior and ordinary/property distinction.

## Implementation Notes

The existing temporary observation worker collects revision-bound property
owners and source ranges. Plans publish through the ordinary checked edit path.
Prompt stages recheck the originating view and source; C-g cancels without edits.
The derived model admits at most 4096 entries and 8192 property records; incomplete
observations reject property operations.

## Contracts Added or Changed

- ORG05: distinct drawers, local/additive stored properties and source owners.
- ORG17: compiled commands and the Org-local C-c C-x p binding.

## Files of Interest

- `src/org/property/`: native source ownership and edit plans.
- `src/session/outline/property.rs`: qualified prompt and publication lifecycle.
- `tests/org_editing/properties.rs`: source spelling, drawer placement and undo.

## Validation

- `cargo test --test org_editing properties::`: five tests passed.
- `cargo test --test configuration org_properties::`: two policy tests passed.
- `python3 scripts/check-config-reference.py`: schema/sample/template agree.
- Owned clean Emacs observations confirmed selective file defaults, additive
  ordering and file-base resets; no editor runtime dependency was introduced.
- Focused source-quality checks passed for the new owners and tests.

## Risks and Follow-ups

T12.3 boundary and owner-edit validation remains active.
