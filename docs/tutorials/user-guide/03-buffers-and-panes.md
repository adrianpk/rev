# Use Buffers and Panes

Keep `notes.txt` open. You will open another file without closing it, then
put both files on screen.

## Open another file

Press `C-x C-f`, type `todo.txt` and press Enter. Type `Check the forecast.`
and save it with `C-x C-s`.

Both files now have live buffers: rev's in-memory copies of their text.
Switching files does not close or save the other buffer.

Press `C-x b`, type `notes.txt` and press Enter to return to your notes.
`C-x C-b` offers a buffer list instead: move with Up/Down and press Enter.

## Show both files

Press `C-x 3` to split the editing area into left and right panes. Both panes
initially show the same buffer.

Press `C-x o` to select the other pane. Use `C-x b`, enter `todo.txt` and press
Enter. You can now see the notes and reminder together.

The two panes contain the following text. This sketch labels each pane with
its filename and omits gutters and status details:

```text
notes.txt              │ todo.txt
Pack the notebook.     │ Check the forecast.
Bring a pencil.        │
```

Edits belong to the buffer. If both panes show the same buffer, a change in
one appears in the other, while each pane keeps its own cursor and scroll.

## Return to one pane

Press `C-x 1` to keep the selected pane. Both buffers remain open.
Return to `notes.txt` with `C-x b`.

For a top/bottom split, use `C-x 2`. To close a buffer, use `C-x k` and accept
its name; modified text prompts you to save, discard or cancel.

See [Buffers, panes and saved files](../../explanation/editing-model.md) for
the distinction between a view, live text and a file on disk.

Next: [Search and Run Commands](04-search-and-commands.md).

[Previous](02-edit-text.md) · [User Guide](index.md)
