---
id: TKT-20261027085102
title: Introduce a minimal panel presentation abstraction
status: ready
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: panels, layout, rendering
source: chat
reported_at: 2026-10-27T08:51:02Z
ready_at: 2026-10-27T10:37:08Z
commits: []
---

# Introduce a minimal panel presentation abstraction

## Observed behavior

The README screenshot exposes ASCII pipe separators between the file tree
and editing panes. Inspection confirms a real window model: `Layout` is a
binary split tree, `Geometry` assigns pane rectangles, and each leaf references
an independent editing view.

The missing layer is a common visual-panel contract. `src/display/compose.rs`
paints pane rows and repeated `|` separators directly; `src/display/tree.rs`
prepends the tree text and its separator and adjusts display offsets itself.
This request concerns that presentation structure, not only the separator glyph.

## Expected outcome

Define and use a small panel presentation abstraction suitable for rev's
existing editor panes and auxiliary UI. Give content bounds, borders or
separators, clipping, status placement, and active-state presentation one
coherent composition contract.

Odra's `ui` surfaces and panel composition are the architectural reference.
Inspect its Extensible UI Foundation Reference and `src/ui/contracts.odin`,
while keeping rev's implementation proportional to its built-in terminal UI.

## Acceptance criteria

- Editing panes and the file tree use explicit panel geometry and composition
  rather than independent string-prefix and separator-placement conventions.
- Content and border cells have distinct reserved bounds. Nested horizontal
  and vertical splits compose without broken boundaries or incorrect offsets.
- Existing buffers, views, split ownership and input contexts remain the
  semantic owners; the presentation layer does not duplicate their state.
- Cursor, syntax, selection, search and status positions use the computed
  content bounds, including Unicode text and different tab display policies.
- Active and inactive presentation consumes semantic palette roles. Resize,
  tiny geometry and hiding the tree preserve text, points and focus behavior.
- The contract can host the separately requested vertical M-x area without
  requiring its command-selection behavior to be implemented in this ticket.
- Validate geometry and composition at native boundaries and one representative
  terminal journey with nested panes, tree visibility, focus and resize.


## Related work

- [Vertical M-x completion](20261027084156-vertical-command-completion.md).


## Approved delivery

Ready for the single [configuration and interaction plan](../../plan/rev-configuration-ui.md).
[Editor Interaction](../../spec/editor-interaction.md) fixes UI01, including
geometry, navigation, matching, path acceptance and bounds. Assigned slices:
10, 11, 12, 13, 14. Use their recorded branches, task commits and
reports; no separate ticket branch or independent controller is required.
