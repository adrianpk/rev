# Slice 11: Edit Org lists and progress cookies

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-11-lists-checkboxes`
PR: pending

## Purpose

Edit native Org lists and checkbox progress through canonical checked history.

## Delivered Behavior

M-RET and `org-insert-item` continue or split real unordered, ordered and
description items. Ordered siblings renumber with retained `[@N]` resets and
their original period/parenthesis terminator. Checkbox continuation starts
unchecked. Description continuation leaves point in the new empty term.

Meta-arrow commands select the actual list context before heading behavior.
Ordinary indentation leaves nested children at their current level; ordinary
outdent rejects detaching children. M-S-Left/Right move the whole item tree.
Sibling moves carry continuation paragraphs, blocks and nested items, preserving
BOM, CRLF/LF and unterminated EOF spelling. Explicit list and subtree command
identities remain available through M-x.

Ordinary checkbox changes derive unchecked, intermediate and checked parents
from direct children. Item/heading fraction and percentage cookies update in
the same transaction. Radio selection clears only same-level peers; both the
named action and a local `ATTR_ORG :radio t` declaration work. A local `ORDERED`
property diagnoses an unfinished predecessor before mutation.

Checkbox presence and insertion have explicit working commands. Checkbox
regions use one selected state. Statistics headings count direct factory
TODO/DONE children when no checkbox count applies or `COOKIE_DATA: todo` is
selected. Checkbox changes preserve other entries' source and cookie spelling.

## Implementation Notes

The existing temporary observation worker derives bounded item ownership and
header offsets from native structure. No input/render parser, dependency,
permanent worker or independent editable document is introduced.

List plans requalify source and use ordinary prepare/publish checks. Ordered
moves construct both their text and point mapping from a checked partition of
at most six original source fragments plus counter replacements. This retains
unchanged source anchors when counter widths change. History replays the same
mapping in reverse; changed counter prefixes retain normal replacement affinity.

The worker retains at most 8192 item/list/heading records and 4096 statistics
cookies in addition to existing source observations. Incomplete observations
cannot authorize list mutation. Progress derives from bounded parent links,
with no timer, external job or executable directive authority.

## Contracts Added or Changed

- ORG04: contextual list continuation, indentation and complete item moves.
- ORG06: checkbox hierarchy, radio choices, ordered admission and atomic cookies.
- ORG30: native source controls retain canonical source coordinates.

## Files of Interest

- `src/org/list/`: bounded ownership, numbering and checked edit plans.
- `src/session/outline/list.rs`: normal canonical transaction publication.
- `src/buffer/edit/reorder/modified.rs`: checked movement with counter changes.
- `tests/org_editing/lists.rs`: list operations, attached source and replay.
- `tests/org_editing/checkboxes.rs`: hierarchy, radio, ordered checks, explicit
  presence/insertion and list/task statistics with atomic history.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: contextual keys.

## Validation

Development checks passed:

- `cargo check`
- `cargo test --test org_editing lists::`: four tests passed.
- `cargo test --test org_editing checkboxes::`: six tests passed.
- `cargo test --test commands org_`: six tests passed.
- `python3 scripts/check-config-reference.py`

Focused source-quality checks passed for changed Rust owners and tests.
Exact-head required and focused controller evidence follows the completed slice
and pushed canonical PR-reference follow-up.

## Risks and Follow-ups

T11.3 adds boundary, timer-item/literal-context and representative owned-terminal
coverage.
