# Use command prefix arguments

Supply an argument before the command that should use it. With default
non-modal bindings, `C-u 5 C-n` moves five ordinary rows, stopping at the buffer
boundary. Wrapped continuations count as rows. The echo area shows the pending
value; `C-g` cancels it. An argument is consumed once.

## Move or insert a count

- `C-u 2 C-p` moves two rows up. `C-u - 2 C-n` reverses next-line instead.
- `C-u 3 !` inserts `!!!`; one `C-x u` removes the complete counted insertion.
- `C-u 2 M-x goto-line` followed by Enter selects logical line two directly.
  M-x query edits and candidate navigation keep their normal meaning.

A plain `C-u` supplies four; another multiplies that plain prefix by four.
Meta digits start decimal entry and `M--` starts a negative argument. To insert
a digit with a count, end numeric entry first: `C-u 6 4 C-u 1` inserts sixty-four
copies of `1`.

## Select an earlier kill

Copy or kill three separate pieces of text. Copying separate regions makes
separate ring entries; consecutive directional kills can combine into one.
At the destination, press `C-u 3 C-y` to insert the third newest entry once.

A plain `C-u C-y` selects the newest entry and leaves point before it and mark
after it. `C-u 4 C-y` selects the fourth newest entry; the numeric value four
has a different meaning from a plain prefix here. After a yank, `C-u - 1 M-y`
rotates toward a newer entry. Zero rotation leaves the current insertion alone.

## Replay existing undo groups

Press `C-u 3 C-x u` to undo up to three existing groups. Use
`C-u 3 M-x undo-redo` to replay them forward. Each counted source edit is one
group, so this does not undo one character per internal insertion step.

At a history boundary, completed groups remain applied. `C-g` cancels remaining
work; it does not roll back groups already replayed. Negative history counts
refuse. Zero leaves the source unchanged.

## Keep the active input owner

Modal Normal/Visual `C-u` retains half-page movement. In those states, invoke
`prefix-argument` by name when you need an editor prefix. Modal Insert/Replace
inherit editor prefix entry. Do not combine a native modal count with an editor
prefix; ambiguous combinations refuse. Escape cancels a pending modal prefix.

Existing `C-u M-!` insertion and `C-u M-|` filtering still use one shell job.
A prefix before named `shell-command` or `shell-command-on-region` chooses the
same existing variant. Raw terminal input, prompts and completion navigation
retain their owners. Other commands run once unless the
[complete command policy](../reference/commands.md#command-prefix-arguments)
gives their argument a specific meaning.

The reference also covers signed deletion/kills, logical line counts, buffer
fractions, recentering, loaded tree rows, checked limits and cancellation.
