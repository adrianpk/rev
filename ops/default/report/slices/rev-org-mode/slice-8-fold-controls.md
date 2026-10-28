# Slice 8: Deliver native Org visibility controls

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-08-fold-controls`
PR: pending

## Purpose

Expose native outline and drawer visibility through shared key and named handlers.

## Delivered Behavior

TAB cycles the heading at point through heading-only, direct child headings and
complete subtree. On an admitted drawer it toggles only that drawer's body.
S-TAB cycles this view through overview, contents and show-all. Eleven compiled
commands expose those cycles and direct heading/drawer choices. Initial source
visibility is show-all. Plain, capture and prompt TAB meanings remain distinct.

Native structural observations exclude literal block text from outline targets.
A missing exact TAB target reports unavailability without folding an ancestor or
editing source. Drawer boundaries remain visible. Explicit folding of a point's
body moves that view to its checked visible owner; no display row aliases source.

## Implementation Notes

The existing single temporary observation worker builds a stamped fold model
from structural observations. At most 4096 heading/drawer records participate;
incomplete observations reject visibility operations without changing source.
The command path does not parse source. A single pending request captures its
view and source and is admitted only while they remain current. C-g cancels that
intent and owned work, including requests made with syntax coloring disabled.

View choices retain immutable shared model data and independent requested states.
State and interval projection remain view metadata, outside canonical text/history.

Targeted motion and search reveal only the ancestor path and target entry;
sibling folds and peer views keep their requested choices. Search cancellation
restores the saved complete view state, and resize preserves live choices.
Visible-row movement skips folded source. Native edits/undo/redo retire obsolete
choices instead of requalifying stale byte offsets. Read-only controls remain
usable. Requests whose source, view or prompt owner changes cannot publish late.

## Contracts Added or Changed

- ORG03: local/global cycles and exact drawer visibility through native views.
- ORG17: compiled M-x identities, context keys, complete sample and catalog.

## Files of Interest

- `src/command/org.rs`, `src/input/keymap/defaults.rs`: compiled identities and keys.
- `src/org/visibility/`: bounded structural fold model and state transitions.
- `src/view/visibility.rs`: checked view-only publication.
- `src/session/visibility.rs`, `src/session/org.rs`: request and observation lifetime.
- `tests/org_editing/visibility.rs`, `tests/commands/org_visibility.rs`: native controls.
- `tests/org_editing/visibility_reveal.rs`: hierarchy, saved state, read-only,
  named equivalence, incomplete observations and retired-request behavior.
- `docs/reference/config.toml`, `commands.md`, `bindings.md`: active control catalog.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_editing visibility::`: four passed.
- `cargo test --test commands org_visibility::`: one passed.
- `cargo test --test org_editing visibility_reveal::`: six passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`

Focused source-quality checks passed for changed visibility owners.
Required exact-head controller validation follows the remaining tasks and pushed
canonical PR-reference follow-up.

## Risks and Follow-ups

The owned outline terminal journey remains in T8.3. Existing edits release obsolete view choices.
