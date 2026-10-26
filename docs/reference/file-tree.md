# File tree

`M-x toggle-file-tree` or `C-x t t` shows or hides the left pane. It starts
hidden. Showing it focuses the selected collapsed root, marked with `>` and
reverse video. The footer shows `Tree *` while the tree owns workspace input.
The editor's selected window, text, point, mark and undo history remain live.

The initial file's resolved parent is the startup root. Without an initial
file, the root is the current directory when the tree first opens. The retained
root survives hiding, resizing and later file visits. Root presentation reads
no Git metadata and does not traverse directories.

## Keyboard focus

- `C-x t f` or `M-x focus-file-tree` shows and focuses the pane.
- Tab or `C-g` in the tree returns to the retained editing window, leaving the
  pane visible. Editing keys and bracketed paste cannot change document text
  while the tree owns workspace focus.
- M-x uses its normal focused command prompt. Canceling or entering an unknown
  name returns to the tree. Named tree commands use the same handlers as keys.
- `other-window` cycles only editing leaves. The tree never enters that order;
  return to the editor before cycling. Guarded session exit remains available.

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
