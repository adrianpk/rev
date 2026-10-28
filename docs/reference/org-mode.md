# Native Org outline

An admitted `.org` file or `M-x org-mode` uses native Org mode. Source starts
fully visible. Visibility controls change only the selected editing view; they also
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

`org-insert-heading` reports unavailability in lists, tables, drawers and
literal bodies. M-RET selects list continuation in an admitted item; other
unsupported deeper contexts retain their source ownership. C-g can cancel an observation request
before insertion, just as for visibility commands.

Heading motion also works on read-only source and never creates history.
Next/previous motion skips folded headings even outside the viewport. Same-level
motion stops at a parent boundary; parent motion visits the nearest structural
parent, including skipped numerical levels. These targeted moves use ordinary
hierarchy reveal. A missing target reports unavailability and keeps the view.

## Subtree editing

| Key | Command | Behavior |
| --- | --- | --- |
| M-Left in heading context | `org-metaleft` | Remove one star from the heading and every descendant. |
| M-Right in heading context | `org-metaright` | Add one star to the heading and every descendant. |
| M-Up in heading context | `org-metaup` | Move the complete subtree before its previous sibling. |
| M-Down in heading context | `org-metadown` | Move the complete subtree after its next sibling. |

The explicit M-x names `org-promote-subtree`, `org-demote-subtree`,
`org-move-subtree-up` and `org-move-subtree-down` retain heading-only ownership.
Meta-arrow names select actual list context before these heading handlers.
Point can be on the heading or in its ordinary
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

## Lists and checkbox progress

| Key | Command | Behavior in an admitted list |
| --- | --- | --- |
| M-RET | `org-meta-return` | Split or continue an item, retaining its marker/description form. |
| M-Left / M-Right | `org-metaleft` / `org-metaright` | Outdent/indent the item and continuation text, leaving nested items in place. |
| M-S-Left / M-S-Right | `org-outdent-item-tree` / `org-indent-item-tree` | Change indentation with complete nested source. |
| M-Up / M-Down | `org-metaup` / `org-metadown` | Exchange complete adjacent items and renumber ordered siblings. |
| M-S-RET | `org-insert-item-checkbox` | Insert a new unchecked checkbox item. |
| C-c C-c | `org-ctrl-c-ctrl-c` | Toggle an existing checkbox and verify current list numbering. |
| C-c C-x C-b | `org-toggle-checkbox` | Toggle the current checkbox, selected checkboxes, or the current heading section. |
| C-c C-x C-r | `org-toggle-radio-button` | Select a checkbox and clear same-level radio peers. |
| C-c # | `org-update-statistics-cookies` | Refresh existing item/heading fraction and percentage cookies. |

M-x also provides `org-insert-item`, `org-indent-item`, `org-outdent-item`,
`org-move-item-up`, `org-move-item-down`, `org-renumber-list` and
`org-toggle-checkbox-presence`. Presence adds/removes checkboxes explicitly;
factory description items reject adding them. Existing checkbox text in a
description item remains actionable. Ordinary drawers may contain real lists;
literal example/source blocks and table cells never become list targets.

Ordered markers retain period/parenthesis spelling and explicit `[@N]` resets.
Decimal counters do not overflow a machine integer; generated counters share the
finite native source byte budget. Description continuation creates an empty
term, including existing relative-timer description spelling, without arming a
timer or inventing elapsed time. New checkbox items start unchecked unless their
existing nested children determine a derived state.

Ordinary outdent rejects detaching children. Use the tree command for that
operation. The first outermost tree command shifts the whole list by one column;
ordinary first-item indentation and nested first-item demotion require a real
preceding item. Promoting a nested item adopts the following source siblings.
Structural list commands require an unselected item. Checkbox operations accept
linear regions in either direction and use the first selected checkbox's new
state; rectangles reject. Read-only, incomplete and unclosed current-entry
observations reject before publication.

Checkbox parents reflect direct children as `[ ]`, `[-]` or `[X]`. Existing `[/]`
and `[%]` cookies update as fractions or integer percentages. A heading normally
counts its direct checkboxes, or direct TODO/DONE child headings when no checkbox
count applies. Local `COOKIE_DATA: todo` or `COOKIE_DATA: checkbox` selects that
source explicitly. Local `ORDERED: t` blocks newly completing a checkbox before
an unfinished predecessor. `#+ATTR_ORG: :radio t` applies radio choice to its
list; hierarchy still derives parent states from child checkboxes.

A list edit and its parent/cookie changes undo as one operation. Untouched source,
BOM, line endings and EOF spelling remain intact. Moves retain source anchors in
shared panes, including outside a list whose counter widths changed. At most
8192 item/list/heading records and 4096 statistics cookies participate in this
bounded derived model. Other entries' cookie spelling stays untouched.

The [Org list reference](https://orgmode.org/manual/Plain-Lists.html) and
[checkbox reference](https://orgmode.org/manual/Checkboxes.html) describe the
corresponding source forms and familiar interactions.

## Properties and source owners

`C-c C-x p` and `M-x org-set-property` prompt for a name and local value.
New drawers follow the heading and its planning line. Before the first heading,
they precede non-comment content. Existing key spelling, indentation, unknown
properties and surrounding source remain intact. Names are case-insensitive;
new names use uppercase. Append `+` to add a value without replacing its base.

`org-delete-property` removes only the named local base; a name ending in `+`
removes its local additions. It preserves ancestors, file declarations and the
other form. Removing the final field retains the empty drawer. `org-inspect-property`
shows the effective value and source line for each local, inherited or file
contribution. Derived fields such as ITEM, TODO, PRIORITY, TAGS and FILE are
identified as derived and cannot be stored through these commands.

`org-edit-property-owner` explicitly edits an effective source contribution.
It identifies the source line before asking for a value. When there are several
contributions, select their 1-based number first. This action can edit an ancestor
or a file directive; ordinary local set/delete never silently changes that owner.
One accepted edit forms one undo group and maps shared views through canonical
source. C-g cancels either prompt without changing the document.

Factory property inheritance is off. Set `org.properties.inherit = ["PROJECT",
"CUSTOM"]` to inherit only those stored names, including file defaults. A local
base wins over the nearest ancestor, then the file drawer and `#+PROPERTY:`
declarations. Local `NAME+` lines extend the selected inherited base. A `nil`
value stops the search and represents an explicitly undefined value. File
`#+PROPERTY: NAME value` declarations apply in source order; a later base resets
its earlier file additions. A file drawer base overrides file declarations.
Recognized keywords change data semantics only. They cannot alter inheritance
policy, editor configuration or process authority, and keywords inside literal
or nested forms do not become file property defaults.

Names are 1..128 UTF-8 bytes without whitespace, colons or interior `+`.
Inheritance accepts at most 128 unique stored names; an empty array replaces an
included array and disables inheritance. Property prompts use the existing
minibuffer input limit. The observation retains at most 4096 source entries and
8192 stored/declaration records. Incomplete observations, ambiguous duplicate
local bases, malformed metadata, literal/deeper contexts and read-only mutation
reject before publication. Inspection does not insert inherited text. Existing
ordinary and property drawers fold independently through the drawer commands.
