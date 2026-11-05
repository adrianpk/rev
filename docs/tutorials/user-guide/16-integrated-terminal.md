# Use an Integrated Terminal

Open an interactive shell, give lazygit the main editing area, then return to
the same shell without losing its state. Use an existing local Git repository
with Git and lazygit installed for the tool steps. The ordinary shell steps
work without lazygit.

## Open a shell beside your notes

From that repository, open a practice file:

```sh
rev terminal-notes.txt
```

Write and save one line using the controls from the earlier chapters. In
non-modal editing, run `M-x terminal-open`. In modal Normal, use
`:terminal-open` and Return. A shell appears below your source:

```text
+-----------------------------+
| terminal-notes.txt           |
+-----------------------------+
| terminal: interactive shell |
+-----------------------------+
```

Type `pwd` and press Return. The shell prints its directory and accepts another
command. Your typed text belongs to the shell; it is not inserted in the file.
Press C-g to return to source editing, then invoke `terminal-open` again. It
selects the same live default shell. Startup and ordinary buffer listing never
create a terminal automatically.

## Give a running tool more space

At the shell prompt, type:

```sh
lazygit
```

Use lazygit's own controls, such as `?` for help and Escape to leave help.
While the live terminal owns focus, M-x opens rev's command selector. Run
`terminal-move-to-buffer` to fill the selected editing pane with that same tool:

```text
+-----------------------------+
| terminal: same lazygit       |
| more rows for files/help    |
+-----------------------------+
```

The process and working state stay intact; this is a placement change. Run
`M-x terminal-move-to-panel` to dock it again. `terminal-new-buffer` is available
when you want a new independent session directly in the main area.

## Keep two shells available

Run `M-x terminal-new` to create another panel session. The first remains owned
and can keep running while hidden. Use `terminal-list` to select one, or C-x b
to see and filter all buffers, including panel terminals. Names start with
`terminal`, then `terminal-2`; use
`terminal-rename` for a recognizable name such as `build`. The lower panel
shows one selected session at a time.

`terminal-toggle` hides or restores that panel. C-g changes focus. Neither
operation closes a shell. For source save, undo, completion, Org, language-server
or Pi actions, first return to a source buffer. A terminal is a typed resource,
so source commands cannot edit its process output as document text.

## Inspect output and finish

Return to the lazygit session, close its help if necessary and use its `q`
control to return to the shell. Run:

```sh
printf 'hello from the shell\n'
```

Press C-c C-t to enter frozen output copy mode. Navigate, select and copy with
your profile's ordinary controls while the child keeps running. Return leaves
copy mode without sending a Return to the child. C-y pastes the shared retained
text into live input, with the child's bracketed-paste mode when enabled.
The [task guide](../../how-to/use-integrated-terminal.md#copy-retained-output)
has concrete selection steps.

At the shell prompt, enter `exit`. Its terminal buffer closes automatically.
The main area returns to its previous source; a selected lower-panel terminal
disappears from the panel. Closing a live terminal with C-x k and Return asks
before terminating its owned jobs. Quitting rev
resolves source saves first and asks once about live sessions; declining keeps
them. These sessions last for this editor process and are not restored after
restart.

You have completed the everyday editing path. The independent
[Git path](git/index.md) teaches repository work. Continue with the
[terminal task guide](../../how-to/use-integrated-terminal.md) or the
[exact terminal reference](../../reference/integrated-terminal.md).

[Previous](15-shell-commands.md) · [User Guide](index.md)
