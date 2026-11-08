# Record and Replay Editing

Add the same marker to three lines by recording one short editing interaction.
Use the default non-modal profile and a small practice file:

```sh
rev macro-practice.txt
```

Enter these lines, press Enter after `blue` to leave an empty line, then run
M-x beginning-of-buffer:

```text
red
green
blue
```

## Record one line

Press F3. The status shows Def: edits now happen normally and become a recording.

1. Press C-a to reach the line beginning.
2. Type `- `, including the space.
3. Press C-n, then C-a, to reach the next line beginning.
4. Press F4 to finish recording without replaying it.

The file now begins with `- red`. The cursor is at the start of `green`.

## Replay and undo

Press F4 once to mark `green`, then again to mark `blue`:

```text
- red
- green
- blue
```

Each replay uses the same native commands at the current cursor. It does not
select another line for you beyond the movement you recorded. Keep recordings
short and try the first replay where you can inspect its result.

Press C-x u once. Only the last replay is undone, so `blue` loses its marker.
Press C-g to stop a recording or playback; edits already made remain real and
can be undone normally. Save the practice file with C-x C-s.

If your terminal intercepts function keys, C-x ( starts recording, C-x ) finishes,
and C-x e runs the current macro. These are ordinary configurable command keys.
The [task guide](../../how-to/apply-keyboard-macros.md) shows counted replay,
regions, counters and queries, including a recording that restores modal Normal.

The last recording stays in this editor session. Give an independent snapshot a
name and [save it explicitly](../../how-to/save-keyboard-macros.md) to reuse it
after restarting. Source-file saving does not save macro definitions.
See [macro ownership](../../explanation/keyboard-macro-ownership.md) for why.

Next: [Browse Files](05-file-tree.md).

[Previous](04-search-and-commands.md) · [User Guide](index.md)
