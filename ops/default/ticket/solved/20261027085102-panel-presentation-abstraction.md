---
id: TKT-20261027085102
title: Introduce a minimal panel presentation abstraction
status: solved
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: panels, layout, rendering
source: chat
reported_at: 2026-10-27T08:51:02Z
ready_at: 2026-10-27T10:37:08Z
commits: [e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13, 167a49e33369d48cf5fcb009019ccd9303dde736, 9ec177b137cc84d566f961fac8422ad741d55fe6, 8921273e463e2d8cdb5999aede301c984893a7b6, 3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
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

## Verified delivery evidence

- Slice 10: [#59](https://forge.adrianpk.com/adrianpk/rev/pulls/59); integrated `e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13`. [Report](../../report/slices/rev-configuration-ui/slice-10-panel-contracts.md) records the delivered behavior and exact-head checks.
- Slice 11: [#60](https://forge.adrianpk.com/adrianpk/rev/pulls/60); integrated `167a49e33369d48cf5fcb009019ccd9303dde736`. [Report](../../report/slices/rev-configuration-ui/slice-11-editor-panels.md) records the delivered behavior and exact-head checks.
- Slice 12: [#61](https://forge.adrianpk.com/adrianpk/rev/pulls/61); integrated `9ec177b137cc84d566f961fac8422ad741d55fe6`. [Report](../../report/slices/rev-configuration-ui/slice-12-tree-panels.md) records the delivered behavior and exact-head checks.
- Slice 13: [#62](https://forge.adrianpk.com/adrianpk/rev/pulls/62); integrated `8921273e463e2d8cdb5999aede301c984893a7b6`. [Report](../../report/slices/rev-configuration-ui/slice-13-palette-and-syntax.md) records the delivered behavior and exact-head checks.
- Slice 14: [#63](https://forge.adrianpk.com/adrianpk/rev/pulls/63); integrated `3ba6bb0ecd825a4f3e75081ec5b056821fa6ff5d`. [Report](../../report/slices/rev-configuration-ui/slice-14-ui-palette.md) records the delivered behavior and exact-head checks.
