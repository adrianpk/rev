# Soft Wrap Model

Status: accepted
Date: 2026-10-31
Approved: product owner, 2026-10-31; selected five-topic delivery, Emacs interaction criterion and explicit Looper start request
Plan: [Wrap and Navigation](../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../tracker/rev-wrap-navigation.md)
Specification: [Word-Aware Soft Wrapping](editor-soft-wrap.md)
Existing owners: [rev Editor Model](rev-editor-model.md), [Editor Gutter Model](editor-gutter-model.md), [Org Mode Model](editor-org-mode-model.md)

## Canonical and derived state

The canonical rope, source-line map and text revision remain the only text
authority. A wrap fragment is a display observation, never another source line.
Views of the same buffer may have different geometry and derived fragments.

| Concept | Fields and evidence | Contract |
| --- | --- | --- |
| Effective wrap policy | Word-aware wrapping enabled at pane text width by default, validated startup preference, buffer-local runtime override and generation. | editor.soft_wrap defaults true; visual-line-mode toggles one buffer for all its views until close. New buffers inherit startup settings. Runtime changes do not write settings or text. |
| Layout identity | Buffer/revision, view, text width, tab policy, folds/projection and settings generations. | A geometry or source change invalidates dependent fragments and pointer observations. |
| Wrapped fragment | Logical line identity, checked source span, display-row position and cell mapping. | UTF-8 byte positions, graphemes and terminal cells are distinct coordinate domains. |
| Continuation presentation | No added prefix/indentation, blank continuation-number cells and logical gutter association. | Derived cells are not source text, selectable bytes or additional document lines. |
| Viewport anchor | Checked source location plus the admitted fragment/scroll affinity. | Reflow preserves semantic point/selection; no raw obsolete screen row becomes canonical. |
| Visual-line command context | Current view wrap policy, source revision, fragment mapping, canonical point and command identity. | Up/Down and C-n/C-p traverse visual rows; C-a/C-e/C-k follow Emacs Visual Line behavior when wrapped. Line numbers and goto-line retain logical source identity. |
| Layout work/cache | Current observations, bounded entries and work accounting. | Disposal loses no text; repeated navigation cannot cause unbounded retained layout growth. |

## Transitions

Startup policy resolves through the existing configuration owner. An admitted
runtime choice, resize, edit or fold change invalidates affected layout. The
view obtains current fragments for rendering and hit testing. Both operations
use the same mapping; a stale mapping is rejected or recomputed before an edit.

Toggling wrapping changes only derived state. Source edits still use ordinary
checked ranges and history. Undo/redo can invalidate layout but never undo a
presentation toggle as though it inserted/deleted newline bytes.

Visual-line commands resolve their target through the current checked mapping.
C-k performs an ordinary source deletion and kill-ring/history transaction;
unlike a wrap toggle, it is a real edit and remains undoable. Disabling wrapping
restores the existing logical-line command behavior. Contextual prompts and
other input owners do not inherit source-view motions merely because a wrapped
document is visible.

## Dependencies and implementation representation

Shared projection coordinates, gutter geometry, Org recognition/folding and
pointer mapping already have owners. Extend those owners rather than create
private wrapped selections, line-number indexes or alternative text stores.
The specification fixes whitespace-preferred/grapheme-safe breaks, buffer-local
override lifetime, source-based rectangles and a maximum of 8192 cached
fragments per view. The implementation chooses bounded resumable work and
eviction details, recording finite ceilings and measurements in its reports.

No layout is persisted. This proposal introduces no cache file, database,
migration or project-local configuration. Any eventual startup setting belongs
to the existing TOML catalog. Enabled word-aware wrapping at pane text width
is the selected factory policy under editor.soft_wrap. No per-language table,
fixed-column mode or persistent layout record is introduced.
