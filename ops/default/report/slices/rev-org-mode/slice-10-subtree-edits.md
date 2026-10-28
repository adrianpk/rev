# Slice 10: Move and promote coherent Org subtrees

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-10-subtree-edits`
PR: [#111](https://forge.adrianpk.com/adrianpk/rev/pulls/111)

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

Stored marks, peer points and viewport anchors follow the original source.
Undo/redo map each peer's current position, restore the initiating point and
retain clean/modified state. Level edits map marks through all preceding heading
changes and retire obsolete folds in every affected pane.

Active linear regions must cover whole adjacent sibling subtrees. Partial,
rectangular, read-only, over-depth and malformed-parent targets reject without
changing source, points, marks or history. Unrelated malformed source stays
editable through operations on valid siblings.

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

The EOF peer regression exposed a boundary collision in the initial position
mapping. The permutation now exchanges both the former EOF and the preceding
line-end anchor, making forward and inverse mappings agree through replay.

## Contracts Added or Changed

- ORG04: complete subtree level changes and sibling moves in one transaction.
- ORG02: source fidelity, exact admission and recoverable rejection.

## Files of Interest

- `src/org/visibility/subtree/`: ownership, hierarchy and sibling validation.
- `src/buffer/edit/reorder.rs`: checked fragment permutation and inverse mapping.
- `src/session/outline/subtree.rs`: canonical prepare/publish integration.
- `tests/org_editing/subtrees.rs`: levels, attached content and EOF round trips.
- `tests/org_editing/subtree_positions.rs`: stored marks, independent source
  anchors, current-peer replay and EOF insertion-boundary regression.
- `tests/org_editing/subtree_bounds.rs`: region admission, malformed parents,
  read-only/depth limits and replay at every grapheme boundary.
- `tests/commands/org_headings.rs`, `scripts/configuration_ui/org_outline.py`:
  context-qualified keys and owned RGB/NO_COLOR terminal journeys.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: activated controls.
- `docs/reference/org-mode.md`: subtree controls, ownership and rejection rules.

## Validation

Development checks passed:

- `cargo test --test org_editing subtrees::`: three tests passed.
- `cargo test --test org_editing subtree_positions::`: three tests passed.
- `cargo test --test org_editing subtree_bounds::`: four tests passed.
- `cargo test --test commands org_headings::subtree_keys`: one test passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned sessions passed, zero skipped; source bytes, clean-state restoration,
  terminal restoration and child/fd cleanup checked.

Focused source-quality checks passed for the changed edit and subtree owners.
The subtree coverage checks whole sibling regions in both directions, negative
admission and undo/redo at every grapheme boundary of Unicode/CRLF/EOF source.

After the canonical PR-reference follow-up is pushed, the controller records
these required commands against that clean HEAD, in order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test editing --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

A separate focused controller batch records `cargo test --test configuration
--test commands`, `python3 scripts/check-config-reference.py`, the owned terminal
command above and `git diff --check` at the same HEAD. Receipt IDs and final
outcomes are retained in controller evidence and Acta.

## Risks and Follow-ups

Incomplete observations reject structural edits within the existing native
analysis limits. None recorded beyond those documented limits.
