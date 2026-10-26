# Rectangles

`C-x SPC` or `M-x rectangle-mark-mode` activates rectangle selection at point.
If a mark is already active, it keeps that endpoint. Move point with the normal
motion commands to select the inclusive logical rows between point and mark
and the display columns between them. The selected pane highlights existing
selected cells, including during scrolling and resize.

`C-x r M-w` or `M-x copy-rectangle-as-kill` retains those rows without editing.
Reported Alt-w and the compatible Escape-w sequence both work after `C-x r`.
Copy requires active rectangle mode and a nonzero column span. A rejected copy
keeps the previously retained rectangle. Read-only text can be copied.

## Cell boundaries

- Columns follow the same four-cell tab stops and Unicode width as drawing.
- An edge touching a wide grapheme includes the complete cluster, including
  combining marks and joined emoji. A copied row can therefore be wider than
  the requested display span.
- A partially selected tab contributes its selected existing cells as spaces.
  A fully selected tab retains its original tab byte. Source tabs stay intact.
- Short or empty lines contribute only existing selected text, without trailing
  padding. Row fragments exclude LF/CRLF separators and preserve literal text,
  including visible control characters rather than their drawn notation.

The session retains one rectangle, separate from the linear kill ring. Copy
changes neither source text, cursor, mark, modification state nor undo history.
The next successful rectangle copy replaces the retained rows and requested
width. Ordinary linear copying and yanking use their existing ring.

Toggle rectangle mode again or press `C-g` to deactivate selection while keeping
the mark. A normal `C-SPC` starts linear selection. Successful text edits deactivate
selection; a refused edit preserves it. Shared buffers keep one mark and mode,
while the selected pane's own point determines the visible rectangle.

## Killing rows

`C-x r k` or `M-x kill-rectangle` removes and retains the selected rectangle.
It uses the same cell boundaries as copy. A partially selected tab leaves its
unselected cells as spaces; untouched tabs retain their bytes. Short and empty
rows receive no padding, and LF/CRLF separators stay intact.

Every changed row publishes together as one undo/redo step. The initiating
cursor moves to the top-left source row and column, following any retained tab
prefix spaces. Other shared cursors and marks map through all row changes.
One undo restores the original text, mark, initiating cursor and clean state
when the operation began clean. Undo does not discard the retained rectangle.

Read-only text, inactive/zero-width selection and prompt-owned input refuse
the kill without replacing the previous rectangle or changing any source row.

## Inserting retained rows

`C-x r y` or `M-x yank-rectangle` inserts the retained rows at point across
successive logical lines. It uses the selected view's display column for each
row. If that column touches the interior of a wide glyph or tab on a later
row, insertion snaps to that glyph's source start and preserves its text.

A short target line receives only the prefix spaces needed for a nonempty row.
An empty retained row advances the block without adding spaces to an existing
line. Missing target rows are created with the buffer's prevailing LF/CRLF
separator; a new empty row has no column padding. Source fragments keep their
actual unequal lengths, including wide-cluster expansion, without right padding.
Complete retained tabs stay tabs and follow the destination's four-cell stops.

All inserted rows, prefix padding and added separators form one undo/redo step.
The initiating cursor ends after the last row's actual insertion; shared views
and marks map through the complete change. Undo restores the original text,
mark, initiating cursor and clean state. The retained rectangle stays available.
Read-only or prompt-owned input refuses insertion without editing. An absent
block reports `No rectangle to yank`. The linear kill ring remains independent.
