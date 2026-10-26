# File tree

`M-x toggle-file-tree` or `C-x t t` shows or hides the left pane. It starts
hidden. Showing it focuses the retained selection, initially the collapsed root.
The focused selection uses reverse video. The footer shows `Tree *` while the tree owns workspace input.
The editor's selected window, text, point, mark and undo history remain live.

The initial file's resolved parent is the startup root. Without an initial
file, the root is the current directory when the tree first opens. The retained
root survives hiding, resizing and later file visits. Root presentation reads
no Git metadata. Drawing and visible movement perform no filesystem reads.

## Keyboard focus

- `C-x t f` or `M-x focus-file-tree` shows and focuses the pane.
- Tab or `C-g` in the tree returns to the retained editing window, leaving the
  pane visible. Editing keys and bracketed paste cannot change document text
  while the tree owns workspace focus.
- M-x uses its normal focused command prompt. Canceling or entering an unknown
  name returns to the tree. Named tree commands use the same handlers as keys.
- `other-window` cycles only editing leaves. The tree never enters that order;
  return to the editor before cycling. Guarded session exit remains available.

## Traversal and refresh

Up/Down (or C-p/C-n) move through visible entries without wrapping. Right
(or C-f) expands a directory; Left (or C-b) collapses an expanded directory
or selects the visible parent. Enter toggles a directory. Symlinks remain
leaves, including links to directories; Enter on a leaf reports that condition.
These actions preserve editor text, point and history.

The corresponding names are `tree-previous-line`, `tree-next-line`,
`tree-expand`, `tree-collapse` and `tree-toggle-directory`. Tree navigation
requires tree focus. M-x names and bound keys use the same handlers.

Directory markers are `>` for collapsed and `v` for expanded. Symlinks use
`@`. Children sort directories first, then by the native filename order;
comparison is case-sensitive and independent of locale. Dot entries are shown.
Each expansion reads only that directory's direct children. Collapsing removes
its loaded descendants; expanding again observes fresh children. There is no
recursive scan or implicit symlink traversal.

`C-x t r` or `M-x tree-refresh` reloads the root and previously expanded
branches. New directories stay collapsed. Selection follows its retained path;
if that entry disappears, the nearest visible ancestor becomes selected.
Read/inspection errors identify their operation and path in the echo area.
An expansion failure keeps the existing list. Refresh collapses a failing
branch while retaining successful observations elsewhere; a missing root
remains selected and collapsed so refresh can recover when it returns.
The pane scrolls to keep the selected entry above its footer, including after
resize. Refresh and expansion run only on explicit commands and require work
proportional to the direct entries of the requested open branches.

## Geometry

The pane uses at most 24 columns and at most one third of the frame, plus a
single divider column. Editing panes divide the remaining width; status,
selection, search and cursor coordinates use that editing area. The session
keeps a full-width echo/prompt row.

Below 32 columns, or without usable text/status geometry, the pane hides and
releases tree focus. The show preference and root remain. Widening restores
presentation with editor focus. Explicitly hiding the tree restores full
editing width. Unicode names and control text follow the normal safe drawing
and clipping policy.
