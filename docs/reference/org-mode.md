# Native Org outline

An admitted `.org` file or `M-x org-mode` uses native Org mode. Source starts
fully visible. These controls change only the selected editing view; they also
work on read-only text. Folding never changes saved bytes, modification state,
regions or undo history.

## Keys and named commands

| Key | Command | Behavior |
| --- | --- | --- |
| TAB | `org-cycle` | On a heading, cycle through heading-only, direct child headings and complete subtree. In a drawer, toggle its body. |
| S-TAB | `org-shifttab` | Cycle the whole view through overview, contents and show-all. |

Overview shows top-level headings. Contents shows all heading lines and hides
their bodies. Show-all restores all source, including drawer bodies. Drawer
folding keeps both boundary lines visible. A complete subtree also opens drawers
inside it.

M-x provides the same handlers and these direct choices:

| Commands | Target |
| --- | --- |
| `org-overview`, `org-content`, `org-show-all` | The selected view. |
| `org-hide-subtree`, `org-show-children`, `org-show-subtree` | The heading at point or containing point. |
| `org-hide-drawer`, `org-show-drawer`, `org-toggle-drawer` | The drawer containing point. |

TAB requires a heading line or admitted drawer. Ordinary body text, literal block
text and unavailable structural targets report that no target is available.
They do not insert text or fold another heading. Plain-text, capture and source
editing retain their own TAB binding; a focused prompt retains completion.
The [binding reference](bindings.md) and [configuration sample](config.toml)
list the context maps.

## Navigation and independent views

Splitting a pane copies its current visibility. Later choices affect only the
selected pane. Resize preserves those choices. When explicit folding would hide
point, point moves to the visible heading or drawer opener that owns the hidden
body.

Vertical, page and wheel motion skip hidden rows. A targeted jump or search
reveals the target entry and its ancestor path while retaining unrelated folded
entries. C-g during search restores the starting point, scroll and visibility.
Logical line numbers and source regions continue to refer to canonical text.
Edits, undo and redo release obsolete visibility choices in affected views.

## Pending observations

A command can briefly report a pending structural observation; editing remains
available. C-g cancels its owned request. Moving to another view, changing source
or opening another interaction prevents that request from folding a later state.
Visibility works with syntax coloring disabled and with NO_COLOR.

An incomplete observation rejects folding and keeps source readable. At most
4096 heading/drawer records participate, within the ordinary native Org source
observation limits. `org-show-all` remains available without an observation.

## Heading insertion and motion

| Key | Command | Behavior |
| --- | --- | --- |
| M-RET | `org-meta-return` | Insert a heading in ordinary heading/paragraph context. |
| C-c C-n | `org-next-visible-heading` | Visit the next visible heading. |
| C-c C-p | `org-previous-visible-heading` | Visit the previous visible heading. |
| C-c C-f | `org-forward-heading-same-level` | Visit the next sibling, skipping descendants. |
| C-c C-b | `org-backward-heading-same-level` | Visit the previous sibling, skipping descendants. |
| C-c C-u | `org-up-heading-safe` | Visit the parent heading. |

M-x exposes every name above and `org-insert-heading`, which uses the same
insertion handler. A new heading has the current heading's level, or level one
before the first heading. At the start of a heading, insertion creates a heading
above it. In a title or paragraph, insertion splits at point; at a paragraph
start it prefixes the existing line. Following section text remains after the
new heading. Drawers, blocks, descendants and blank separators retain their
original bytes and order. BOM, LF/CRLF and final-newline presence are preserved.
Point stays on the new title. Insertion is one ordinary undo/redo operation.

Lists, tables, drawers, literal bodies and other admitted deeper forms report
that heading insertion is unavailable in that context. M-RET does not fall
through to editing a containing heading. C-g can cancel an observation request
before insertion, just as for visibility commands.

Heading motion also works on read-only source and never creates history.
Next/previous motion skips folded headings even outside the viewport. Same-level
motion stops at a parent boundary; parent motion visits the nearest structural
parent, including skipped numerical levels. These targeted moves use ordinary
hierarchy reveal. A missing target reports unavailability and keeps the view.

## Subtree editing

| Key | Command | Behavior |
| --- | --- | --- |
| M-Left | `org-promote-subtree` | Remove one star from the heading and every descendant. |
| M-Right | `org-demote-subtree` | Add one star to the heading and every descendant. |
| M-Up | `org-move-subtree-up` | Move the complete subtree before its previous sibling. |
| M-Down | `org-move-subtree-down` | Move the complete subtree after its next sibling. |

M-x invokes these same handlers. Point can be on the heading or in its ordinary
section text. A list, table, drawer, literal block or other deeper context keeps
its own ownership and reports that the subtree edit is unavailable there.

Demotion requires a preceding sibling to become the new parent. Promotion rejects
level one. Movement stops at parent boundaries. Missing parent levels, unclosed
owned/parent structures, incomplete observations and levels beyond 256 reject the
whole operation. Malformed source outside the affected ownership remains intact.
Read-only buffers reject every subtree mutation.

A linear region may cover one or more complete adjacent sibling subtrees with
the same parent and level. Both endpoints must match those ownership boundaries.
Partial, empty or rectangular regions reject; descendants are never detached.

Only heading stars change during promotion/demotion. Movement retains drawers,
blocks, table metadata, unknown directives and blank separators with their
subtree. At an unterminated EOF, the existing separator run remains between the
reordered subtrees; no newline bytes are invented. BOM, LF/CRLF and final-newline
presence remain intact.

Stored marks, peer points and viewport anchors follow canonical source fragments.
A boundary at a heading start follows that heading; an unterminated line end
follows that line. One undo/redo reverses the complete operation and restores the
initiating point, while peers map from their current positions. As with ordinary
edits, publication releases active selection and obsolete folds in affected views.
