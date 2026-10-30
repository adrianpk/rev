# Edit Your First File

This chapter leaves you with a saved text file you can reopen.
Use an unused filename for the exercise, such as `notes.txt`.

## Open and write

In your terminal, run:

```sh
rev notes.txt
```

An empty editing buffer opens. Type:

```text
Pack the notebook.
Bring a pencil.
```

Press Enter between the lines. The modified marker `**` means the buffer has
changes to save. The file does not exist on disk until you save it.

At the left of the status line, you will see:

```text
> notes.txt ** RW
```

## Save

Press `C-x C-s`: hold Control and press x, then hold Control and press s.
The message area reports `Saved`, and `**` changes to `--`.

That part of the status line now reads:

```text
> notes.txt -- RW
```

Throughout this guide, `C-` means Control and `M-` means Alt/Meta.
Keys separated by spaces are pressed in sequence. `M-x`, for example, means
Alt-x. Enter accepts a prompt; `C-g` cancels it.

## Leave and reopen

Press `C-x C-c` to quit. Back in your terminal, run:

```sh
rev notes.txt
```

Both lines are still there. Leave the file open for the next chapter.

When you quit with unsaved changes, `y` saves, `n` leaves those changes unsaved
and `C-g` cancels quitting. A clean buffer needs no decision.

For a short recipe outside this exercise, see
[Edit and save files](../../how-to/edit-files.md).

Next: [Move and Edit Text](02-edit-text.md).

[User Guide](index.md)
