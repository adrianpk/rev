# Soft Wrap Model

Status: draft
Date: 2026-10-31
Specification: [Word-Aware Soft Wrapping](editor-soft-wrap.md)
Existing owners: [rev Editor Model](rev-editor-model.md), [Editor Gutter Model](editor-gutter-model.md), [Org Mode Model](editor-org-mode-model.md)

## Canonical and derived state

The canonical rope, source-line map and text revision remain the only text
authority. A wrap fragment is a display observation, never another source line.
Views of the same buffer may have different geometry and derived fragments.

| Concept | Fields and evidence | Contract |
| --- | --- | --- |
| Effective wrap policy | Validated startup preference, admitted view override and generation. | Defaults and inheritance are unresolved; runtime changes do not write settings or text. |
| Layout identity | Buffer/revision, view, text width, tab policy, folds/projection and settings generations. | A geometry or source change invalidates dependent fragments and pointer observations. |
| Wrapped fragment | Logical line identity, checked source span, display-row position and cell mapping. | UTF-8 byte positions, graphemes and terminal cells are distinct coordinate domains. |
| Continuation presentation | Indentation, optional marker cells and gutter association. | Derived cells are not source text, selectable bytes or additional document lines. |
| Viewport anchor | Checked source location plus the admitted fragment/scroll affinity. | Reflow preserves semantic point/selection; no raw obsolete screen row becomes canonical. |
| Layout work/cache | Current observations, bounded entries and work accounting. | Disposal loses no text; repeated navigation cannot cause unbounded retained layout growth. |

## Transitions

Startup policy resolves through the existing configuration owner. An admitted
runtime choice, resize, edit or fold change invalidates affected layout. The
view obtains current fragments for rendering and hit testing. Both operations
use the same mapping; a stale mapping is rejected or recomputed before an edit.

Toggling wrapping changes only derived state. Source edits still use ordinary
checked ranges and history. Undo/redo can invalidate layout but never undo a
presentation toggle as though it inserted/deleted newline bytes.

## Dependencies and open representation

Shared projection coordinates, gutter geometry, Org recognition/folding and
pointer mapping already have owners. Extend those owners rather than create
private wrapped selections, line-number indexes or alternative text stores.
Word-break rules, token fallback, fragment cache strategy, navigation affinity
and finite limits remain to be decided with the behavior contract.

No layout is persisted. This proposal introduces no cache file, database,
migration or project-local configuration. Any eventual startup setting belongs
to the existing TOML catalog; exact keys and factory values are not selected.
