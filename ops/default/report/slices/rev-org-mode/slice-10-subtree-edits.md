# Slice 10: Move and promote coherent Org subtrees

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-10-subtree-edits`
PR: pending

## Purpose

Change complete native Org subtree levels and sibling order through one checked
history operation, preserving owned source and canonical positions.

## Delivered Behavior

M-Left/M-Right promote/demote complete subtrees with every descendant. Demotion
requires a preceding sibling as the new parent. M-Up/M-Down exchange complete
owned source with an adjacent sibling. The four compiled M-x commands use the
same handlers. Metadata, literal blocks, tables, affiliated directives and blank
separators move with their owner. Level changes touch only real heading stars.

At unterminated EOF, movement retains the existing separator run between the
reordered subtrees. BOM, separator bytes and final-newline state are preserved.
Missing siblings, incomplete observations, missing parent levels and unclosed
owned/parent structures reject before publication.

## Implementation Notes

The existing bounded structural worker supplies current source ownership and
merged malformed ranges. The existing one-request lifetime and source/view/focus
checks admit all operations. No syntax is parsed by input/render paths.

The edit owner validates a partition of two or three original fragments before
building one private candidate. Its permutation maps points, stored marks and
viewport anchors; history uses the inverse for current peer positions. Ordinary
edits retain their existing mapping. Publication uses the same source/read-only,
point/mark and history checks as other canonical edits. Existing history and
native analysis limits remain unchanged.

## Contracts Added or Changed

- ORG04: complete subtree level changes and sibling moves in one transaction.
- ORG02: source fidelity, exact admission and recoverable rejection.

## Files of Interest

- `src/org/visibility/subtree/`: ownership, hierarchy and sibling validation.
- `src/buffer/edit/reorder.rs`: checked fragment permutation and inverse mapping.
- `src/session/outline/subtree.rs`: canonical prepare/publish integration.
- `tests/org_editing/subtrees.rs`: levels, attached content and EOF round trips.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: activated controls.

## Validation

Development checks passed:

- `cargo test --test org_editing subtrees::`: three tests passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`

Focused source-quality checks passed for the changed edit and subtree owners.
Required exact-head controller evidence follows the remaining preservation and
boundary coverage and the pushed canonical PR-reference follow-up.

## Risks and Follow-ups

Independent points/marks and regional boundary coverage remain in T10.2/T10.3.
