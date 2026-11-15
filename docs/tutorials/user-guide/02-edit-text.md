# Move and Edit Text

Continue in `notes.txt` from the previous chapter. You will copy one line and
undo the insertion.

## Move to a line

Use the arrow keys to move through the text. `C-a` moves to the beginning of
the current visual row; `C-e` moves to its end. Long source lines wrap at word
boundaries to fit the pane. Up/Down and `C-n`/`C-p` follow those visible rows.

Use `M-m` to reach the first non-whitespace character of the current logical
line. It skips spaces and tabs without changing text or an active selection.
On a blank line it moves to the end; a count does not select another line.

Try `C-u 5 C-n` to request five ordinary rows. In this short note, movement
stops at the end; a wrapped continuation also counts as a row. `C-u` starts an
argument, `5` chooses the distance and `C-n` consumes it once. The echo area
shows the pending value. `C-g` cancels it before a command runs.

`C-u 2 C-p` requests two rows upward; `C-u - 2 C-n` requests the same direction
through a negative count. Run `M-x beginning-of-buffer` to return to the first
line, then use `C-a` to reach the beginning of `Pack the notebook.`.

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
Press `C-M-_` to redo the copy, then `C-x u` to undo it again. A count such as
`C-u 2 C-M-_` reapplies two available redo groups. `C-r` still searches backward.

Try `C-u 3 !` to insert `!!!`. One `C-x u` removes all three characters:
a counted insertion is one undo step. The argument has already been consumed,
so the next ordinary character inserts once.

## Try automatic delimiters

At the end of your note, press Enter and type `(`. Rev adds `)` and leaves the
cursor between them. Press Backspace to remove both empty delimiters, then press
`C-x u` once: the pair and its inside cursor return together. Type `note`, then
`)`. The result is `(note)` with the cursor after the existing closer; no second
closer appears. Backspace inside a nonempty pair deletes ordinary content.

Press `C-a`, then `C-SPC` and `C-e` to select this whole line. Type `[`.
The result is `[(note)]`, with the cursor after `[`. One `C-x u` restores `(note)`
and its active selection. Run `M-x pair-wrap`, enter `[` and press Enter to wrap
explicitly. With the cursor just after `[`, run `M-x pair-unwrap` to remove that
outer pair and leave `(note)` unchanged inside.

On a new empty line, `C-u 3 (` inserts three nested pairs in one undo step.
`M-x auto-pair-mode` toggles automatic editing for the current buffer and all its
panes. Explicit wrap/unwrap still work when automatic editing is off. Quotes
follow the language and context, while pasted delimiters remain literal.
See [Configure automatic delimiter editing](../../how-to/configure-delimiter-editing.md)
for independent controls and language overrides, and the
[reference](../../reference/delimiter-editing.md) for conservative boundaries.

On a line containing `(note) [next]`, place point before the first opening
parenthesis. Press C-M-n to move after `(note)`, then C-M-p to return before it.
C-u 2 C-M-n crosses both groups. A negative count reverses direction and zero
leaves point unchanged. These motions also work on read-only source and with
automatic pairing disabled. M-x forward-pair/backward-pair invoke the same
operations. For a jump onto the opposite confirmed delimiter itself, use the
unbound M-x matching-delimiter command.

Try `(note [next])` to navigate nesting. From before the outer parenthesis,
C-M-d enters it and another C-M-d enters the brackets. C-M-u returns to the
bracket opener; another C-M-u returns to the outer opener. C-u 2 C-M-d enters
both levels at once. Use M-x up-pair/down-pair when the terminal intercepts a
chord. A negative count leaves forward or enters backward; refusal preserves
point and the active mark.

Save with `C-x C-s`. You can also select text by dragging with the primary
mouse button, or paste text through your terminal. For more tasks, see
[Use command prefix arguments](../../how-to/use-prefix-arguments.md); the
[complete policy](../../reference/commands.md#command-prefix-arguments)
explains counts whose meaning differs from repeated insertion.

The [binding reference](../../reference/bindings.md) lists word, page and
whole-buffer movement. [Rectangles](../../reference/rectangles.md) describes
column-based selection when you need it.

Next: [Use Buffers and Panes](03-buffers-and-panes.md).

[Previous](01-first-file.md) · [User Guide](index.md)
