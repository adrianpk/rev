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

Press `M-x` and type `goto`. The prompt and matching command appear together
in a sufficiently wide terminal:

```text
M-x goto
goto-line  [M-g M-g]  Go to a one-based logical line number.
```

Press Tab to complete `goto-line`, then Enter to run it. At `Goto line:`,
enter `2`:

```text
Goto line: 2
```

Press Enter. The cursor moves to the second line.

The shortcut beside the name belongs to the context where you opened the list.
With the factory bindings, `M-g M-g` also runs `goto-line`. Configured shortcuts
and local input owners can change the displayed hint. When several bindings
work, rev shows the one with the fewest normalized chords, then the first in
lexical order; your other valid bindings still work.

A command with no shortcut leaves that field empty and stays available through M-x.
Every built-in command has a description in the third column, including commands
without shortcuts. User-defined AI commands without descriptions leave that
column blank. A narrow list hides a hint if the complete name, two-cell gap and complete
bracketed shortcut cannot fit. Widening the terminal restores it. Annotations do not participate
in matching: Tab inserts only the command name, and Enter invokes that name once.
See [Effective keybinding hints](../../reference/commands.md#effective-keybinding-hints)
for context and display rules.

Open `M-x` again without typing. The latest named command leads the list; after
this exercise its first row is `goto-line`:

```text
M-x
goto-line        [M-g M-g]  Go to a one-based logical line number.
ai-apply-result             Apply the selected proposal once to its original checked target.
ai-cancel                   Cancel the active invocation or AI-owned input/result.
...
```

Your list can differ if you have used other commands. Up to four distinct recent
commands come first, followed by up to four other frequent commands and then
the alphabetical remainder. Typing a query still gives better text matches
priority. The modal profile's `:` selector uses the same ordering.

M-x remembers the latest 256 named invocations across editor starts. Direct
keyboard shortcuts do not enter that window. M-p/M-n recall distinct command
names from the current session only; this recall is separate from saved ranking.
Opening or canceling the list adds no usage. If history cannot be read or saved,
a `Command usage: ...` message reports the problem and commands remain usable.
See [Command usage state](../../reference/commands.md#command-usage-state)
for storage and recovery details.

M-x shows matching commands vertically. Up/Down selects a candidate; Tab
completes it and Enter runs it. `C-g` leaves without running a command.
When a keyboard shortcut is awkward to send from your terminal, the command
name provides another route. Query replace, for example, is `query-replace`.

Search is literal and does not wrap around the file. Lowercase queries ignore
case; a query containing uppercase characters matches case exactly.
See [Search and replacement](../../reference/search.md) and
[Commands](../../reference/commands.md) for the complete rules.

## Keep a named location

At a place you want to revisit, press `C-x r m`, type `checkpoint` and press
Enter. In `notes.txt`, the prompt and feedback look like this:

```text
Set bookmark (default notes.txt): checkpoint
Bookmark set
```

A blue square `■` before the line number marks the bookmarked source line.
The marker stays visible when line numbers are hidden. After moving elsewhere,
`C-x r b`, `checkpoint`, Enter returns there, even from another file.
Bookmarks follow live edits and persist on ordinary quit. They do not save
source text; continue using `C-x C-s` for that.

`C-x r l` lists names, and `M-x bookmark-markers-mode` toggles their markers
for the current pane without removing names. [Set Your Preferences](09-preferences.md#choose-a-bookmark-symbol)
shows the optional symbol for a Nerd Fonts terminal. To remove the test bookmark, run `M-x bookmark-delete`,
enter `checkpoint` and press Enter. The name is removed; the file is unchanged.
The [bookmark guide](../../how-to/use-bookmarks.md)
explains replacement, deletion, checked reopening and storage recovery.

Next: [Browse Files](05-file-tree.md).

[Previous](03-buffers-and-panes.md) · [User Guide](index.md)
