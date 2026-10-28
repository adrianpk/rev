# Slice 9: Insert and navigate Org headings

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-09-outline-navigation`
PR: pending

## Purpose

Insert headings and navigate native outline targets through checked source edits
and independent views.

## Delivered Behavior

Next/previous visible-heading commands skip folded descendants. Same-level
commands stop at parent boundaries; parent motion uses the admitted hierarchy.
Missing targets report unavailability without approximate movement. Literal
block content never becomes an admitted heading target.

`org-insert-heading` inserts at the current level, splitting ordinary title/body
text at point, converting a paragraph start, or inserting above a heading at its
start. Existing source sections remain byte-for-byte intact. One checked edit
records the initiating point and reconciles shared views for undo/redo.

## Implementation Notes

The existing temporary observation worker prepares the stamped heading model
and merged exclusion ranges for deeper source forms. Requests reuse its single
owned slot and require the original view, source revision and editing focus.
No input/render path parses source. The existing 4096 heading/drawer bound,
finite structural-node budget and 256-level native profile remain in force.

Source targets convert checked byte offsets to canonical scalar positions.
View projection filters only visible-heading motion; targeted hierarchy motion
uses ordinary reveal. Edits use the canonical buffer prepare/publish boundary.

## Contracts Added or Changed

- ORG04: heading insertion and exact next/previous/sibling/parent targets.
- ORG17: compiled names, Org context keys and complete reference agreement.

## Files of Interest

- `src/org/visibility/navigation.rs`, `insertion.rs`: source targets and edit plans.
- `src/session/outline.rs`, `outline/operation.rs`: owned request and publication.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: activated keys.
- `tests/org_editing/headings.rs`: motion, insertion and observation admission.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_editing headings::`: three tests passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`

Focused source-quality checks passed for the changed outline owners. The literal
fixture uses Org comma escaping; the original skip assertion is retained.
Required exact-head controller evidence follows the completed tasks and pushed
canonical PR-reference follow-up.

## Risks and Follow-ups

Contextual M-RET and broader native boundary coverage remain in T9.2 and T9.3.
