# Native Org visibility

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
