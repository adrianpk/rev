# Slice 9: Insert and navigate Org headings

Status: reviewing
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

Native M-RET and `org-meta-return` share that handler. Lists, tables, metadata,
drawers, literal bodies and admitted unclosed delimiters reject insertion in
their exact context, including EOF. Attached source, blank separators, BOM,
LF/CRLF and final-newline presence remain intact; point stays on the new title.

Read-only buffers support heading motion and reject insertion without source or
history changes. Inactive mode, focused prompts and canceled prefixes cannot
invoke contextual edits. Native heading keys and named commands share compiled
handlers. Shared-view undo restores the original initiator while mapping peers
from their current source positions. Tiny/suspended views retain logical targets.

The existing owned outline PTY journey now exercises heading chords, native
M-RET and undo in RGB and NO_COLOR. It retains every earlier visibility/search,
exact-file snapshot and terminal-restoration/child-reaping assertion.

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
- `tests/org_editing/heading_context.rs`: native/named equivalence, attached
  source, byte fidelity and precise-context rejection.
- `docs/reference/org-mode.md`: active keys and insertion/motion behavior.
- `tests/org_editing/heading_edges.rs`, `tests/commands/org_headings.rs`:
  permissions, context ownership, shared history and native input boundaries.
- `scripts/configuration_ui/org_outline.py`: native terminal heading journey.

## Validation

Implementation checks passed:

- `cargo check --lib`
- `cargo test --test org_editing headings::`: three tests passed.
- `cargo test --test org_editing heading_context::`: three tests passed.
- `cargo test --test commands org_visibility::`
- `cargo test --test org_editing --test commands heading`: twelve tests passed.
- `cargo test --test commands catalog_reference`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned sessions passed, zero skipped.

Focused source-quality checks passed for the changed outline owners. The literal
fixture uses Org comma escaping; the original skip assertion is retained.
The new PTY dirty-status predicate was corrected to the established field order;
the new-title point, exact source rows, undo and cleanup assertions remain intact.

Required exact-head controller evidence follows the pushed canonical PR-reference
follow-up, in this order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test commands --test editing`
- `python3 scripts/check-source.py`
- `git diff --check`

Configuration/reference checks and the owned outline PTY journey are recorded in
a separate focused batch at the same candidate. Exact receipts remain in the
controller ledger; verified integration closes this report after merge.

## Risks and Follow-ups

Incomplete observations reject structural operations. Pending requests require
their original source, view and focus before publication.
