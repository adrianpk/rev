# Search and Run Commands

Use `notes.txt` to find a word, replace it and visit a line by command name.

## Find text

Move to the beginning of the first line. Press `C-s` and type `notebook`.
rev highlights the match as you type. The search prompt reads:

```text
I-search: notebook
```

Press Enter to stay there.

During a search, `C-s` finds the next match and `C-r` searches backward.
`C-g` cancels the search and restores your starting position.

## Replace a word

Return to the beginning of the first line. Press `M-%` to start query replace.
On many keyboards this is Alt-Shift-5.

1. Enter `notebook`, then press Enter.
2. Enter `journal`, then press Enter.
3. Press `y` to replace the highlighted occurrence.

The first line now reads:

```text
Pack the journal.
```

If there are more matches, `n` skips one and `q` stops. `C-x u` undoes one
accepted replacement. Save the result with `C-x C-s`.

## Run a named command

Press `M-x` and type `goto`. The prompt and matching command appear together:

```text
M-x goto
goto-line
```

Press Tab to complete `goto-line`, then Enter to run it. At `Goto line:`,
enter `2`:

```text
Goto line: 2
```

Press Enter. The cursor moves to the second line.

M-x shows matching commands vertically. Up/Down selects a candidate; Tab
completes it and Enter runs it. `C-g` leaves without running a command.
When a keyboard shortcut is awkward to send from your terminal, the command
name provides another route. Query replace, for example, is `query-replace`.

Search is literal and does not wrap around the file. Lowercase queries ignore
case; a query containing uppercase characters matches case exactly.
See [Search and replacement](../../reference/search.md) and
[Commands](../../reference/commands.md) for the complete rules.

Next: [Browse Files](05-file-tree.md).

[Previous](03-buffers-and-panes.md) · [User Guide](index.md)
