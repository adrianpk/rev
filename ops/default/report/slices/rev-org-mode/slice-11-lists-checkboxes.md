# Slice 11: Edit Org lists and progress cookies

Status: reviewing
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-11-lists-checkboxes`
PR: [#112](https://forge.adrianpk.com/adrianpk/rev/pulls/112)

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
The first outermost tree command shifts the whole list by one column; nested
promotion adopts following siblings and updates the former parent's progress.
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
Linear checkbox regions select actual checkbox anchors, including starts inside
bullet prefixes; non-checkbox lines cannot determine the selected new state.

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

Boundary coverage exposed a missing unclosed-entry guard in the checkbox path;
it now shares checked source admission with structural list operations. Native
reference probes confirmed parsed ordinary-drawer and existing description
checkbox contexts, first outermost whole-list indentation and nested promotion
ownership. Those outcomes have explicit regression fixtures.
An existing description checkbox continues as a plain description item. The
regression retains that emitted source and prevents counting a nonexistent new
checkbox in the heading's cookie.

Decimal resets and increments use bounded decimal text, retaining counters wider
than a machine integer. Counter rendering observes the existing native byte
budget; source declarations remain unchanged.

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
- `tests/org_editing/list_edges.rs`: precise contexts, malformed/read-only/bounded
  rejection, timer descriptions, region anchors and shared-view replay.
- `scripts/configuration_ui/org_lists.py`: native checkbox/radio/counter/list
  controls within the existing owned RGB/NO_COLOR outline journey.
- `src/input/keymap/defaults.rs`, `docs/reference/config.toml`: contextual keys.
- `docs/reference/org-mode.md`: actual source, ownership and control behavior.

## Validation

Development checks passed:

- `cargo check`
- `cargo test --test org_editing lists::`: four tests passed.
- `cargo test --test org_editing checkboxes::`: six tests passed.
- `cargo test --test org_editing list_edges::`: nine tests passed.
- `cargo test --test commands org_headings::list_keys`: one test passed.
- `cargo test --test commands org_`: six tests passed.
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned RGB/NO_COLOR sessions passed, zero skipped; both original outline and
  added list assertions retained, with exact source bytes and terminal/fd/child
  cleanup checked.

Focused source-quality checks passed for changed Rust owners and tests.
The first required controller batch caught two Clippy collapsible-condition
findings in progress-state guards. They were converted to let chains without
changing the retained behavior assertions.
The required catalog regression also identified missing explicit M-x binding
rows, now documented without changing the catalog assertion. Unicode EOF
continuation coverage reproduced an interior-byte tail lookup; scalar-qualified
tail access fixes it while retaining exact source and full undo/redo assertions.
After the canonical PR-reference follow-up is pushed, the controller records
these required commands against the clean HEAD, in order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test editing --test commands`
- `python3 scripts/check-source.py`
- `git diff --check`

A separate focused controller batch records `cargo test --test configuration
--test windows`, `python3 scripts/check-config-reference.py`, the owned terminal
command above and `git diff --check` at the same HEAD. Window regressions cover
the shared source-anchor mapping changed by counter-aware reordering. Receipt IDs
and final outcomes are retained in controller evidence and Acta.

## Risks and Follow-ups

Incomplete observations reject list operations within the documented native
analysis and derived-model bounds. None recorded beyond those limits.
