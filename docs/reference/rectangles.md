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
