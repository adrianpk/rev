# Use Integrated Terminals

Use an interactive terminal for a shell, a long-running command or a full-screen
tool such as lazygit. For selected text passed to `sort`/`tr` and a checked
one-step edit, use a [shell command or filter](run-shell-command.md).

## Choose a placement

1. Invoke `terminal-open` from M-x, or modal colon in source editing, for the
   default lower-panel shell. Repeating it selects the same live default.
2. Invoke `terminal-new-buffer` for a new independent main-area terminal.
3. With an existing session selected, invoke `terminal-move-to-buffer` or
   `terminal-move-to-panel` to move it without restarting the child.

For lazygit, open a terminal in a local repository, run `lazygit`, then use
M-x for either move command while its interface is still active. Its cwd,
process and tool state remain. A rejected small-screen geometry retains the
old placement. Increase the host terminal size and try the desired move again.
Use `terminal-open-here` for a new session in the current resource's containing
or creation directory; it does not guess a running shell's later cwd.

## Switch sessions and return to source

Create another shell with `terminal-new`. Use `terminal-list` or ordinary
C-x b selection to revisit any terminal. C-x b immediately lists source and
terminal buffers; type to filter, use arrows to select and Return to visit.
Panel terminals appear in the same list. Rename the selected buffer
with `terminal-rename`; names must be unique, 1–256 UTF-8 bytes and contain no
control characters. Hide the panel with `terminal-toggle` and restore it later.

C-g returns to source editing. M-x and the C-x prefix remain host controls.
A stronger prompt consumes its own C-g first. Printable colon and Escape belong
to a live child even in the modal startup profile. Use C-q followed by a key to
send one otherwise protected host key literally. For example, C-q C-g sends
BEL to the child; C-c C-c sends one child interrupt. Source save/undo, bookmarks,
Org, completion, language-server and Pi operations require a source buffer.

## Copy retained output

1. Press C-c C-t in live input to freeze a read-only output projection.
2. In non-modal copy mode, use M-< to visit its beginning, C-SPC to set the mark,
   ordinary motions to extend the range, and M-w to copy. In modal copy mode,
   use Normal/Visual motions, selection and `y` copy. Native search is available.
3. Press Return to resume live input. This consumes Return locally.
4. Use C-y to paste the shared kill-ring entry into the child. M-y can rotate
   that paste only while it remains wholly queued. Once the child has consumed
   any part, rev reports that replacement is no longer available.

The live child keeps producing output while you inspect the snapshot. C-g
cancels an inner search, then clears a selection, then leaves copy mode. Shift
keeps pointer selection/scrolling local even when a tool requests mouse input.
`terminal-clear-scrollback` discards retained history and copy projections while
preserving the live screen; `terminal-clear` sends C-l to the child.

## Close owned jobs

Quit a tool with its own control to return to its shell. An actual shell exit
automatically closes its terminal buffer after owned cleanup. A main-area
terminal returns to its previous source; the selected panel terminal is removed.
This also applies to nonzero shell exit codes. If a live buffer is closed with
C-x k, answer the termination question;
no/cancel retains it. Hiding a pane or switching buffers does not ask or signal.

When quitting the editor, resolve source save decisions, then answer the single
live-session confirmation. Approved cleanup continues asynchronously while
pending/failure state remains visible. Cancelling exit after approval does not
reverse already authorized cleanup. A cleanup failure cancels exit rather than
claiming success; source buffers remain available. No session persists across
an editor restart.

## Configure startup and handle limits

Use the [terminal settings](../reference/integrated-terminal.md#startup-settings)
to select one executable, literal argv, preferred panel height and bounded
session/history/queue sizes. Restart rev to use changed startup preferences.
`terminal-open` starts a new default after the previous default exited.
Exited sessions release their buffers and capacity after successful cleanup.
A full input queue rejects the complete new input or paste without accepting a
partial sequence. Close an unneeded retained session or reduce output history
when admission is exhausted; never assume a rejected creation started a child.

See the [exact reference](../reference/integrated-terminal.md) for key reports,
prompt/error navigation, explicit link activation, resource limits and lifecycle.
