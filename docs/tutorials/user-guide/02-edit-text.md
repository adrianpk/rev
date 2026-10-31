# Move and Edit Text

Continue in `notes.txt` from the previous chapter. You will copy one line and
undo the insertion.

## Move to a line

Use the arrow keys to move through the text. `C-a` moves to the beginning of
the current visual row; `C-e` moves to its end. Long source lines wrap at word
boundaries to fit the pane. Up/Down and `C-n`/`C-p` follow those visible rows.

Move to the beginning of `Pack the notebook.` with Up and `C-a`.

In a narrow pane, one longer source line can look like this:

```text
1 │ Pack the notebook and
  │ bring a pencil.
```

The blank number cell marks a continuation, not another source line. Wrapping
adds no newline to the file. `C-k` kills to the visual row's end through the
ordinary kill ring; `C-y` yanks the killed text and `C-x u` undoes the edit.
Line numbers and `M-x goto-line` still refer to source lines.

## Select and copy

Press `C-SPC` to set the mark, then `C-e` to select the line's text.
The region between the mark and cursor is highlighted.

Press `M-w` to copy it. Move to the end of the second line and press Enter,
then `C-y` to insert the copied text. The buffer now contains:

```text
Pack the notebook.
Bring a pencil.
Pack the notebook.
```

The copied text stays in rev's internal kill ring. It does not use the system
clipboard. To move a selected region instead of copying it, use `C-w`, then
`C-y` at its destination.

## Undo a change

Press `C-x u`. The inserted copy disappears; the new line you created before
it remains. Each accepted paste is one undo step.

Save with `C-x C-s`. You can also select text by dragging with the primary
mouse button, or paste text through your terminal.

The [binding reference](../../reference/bindings.md) lists word, page and
whole-buffer movement. [Rectangles](../../reference/rectangles.md) describes
column-based selection when you need it.

Next: [Use Buffers and Panes](03-buffers-and-panes.md).

[Previous](01-first-file.md) · [User Guide](index.md)
