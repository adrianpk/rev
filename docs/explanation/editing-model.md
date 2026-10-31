# Buffers, panes and saved files

A file is stored on disk. A buffer is the text you are editing in memory.
A pane is a view of a buffer, with its own cursor and scroll position.

These distinctions explain why opening another file does not close the first,
and why closing a pane does not discard its text. The buffer can remain open
without being displayed. Two panes can show the same buffer, and editing either
one changes the shared text.

The tree is a separate filesystem view. It is not part of the editing-pane
cycle. An explicitly opened directory remains its root across file visits.
Otherwise the default follows the visiting file's parent, falling back to the
launch directory without a file; `tree.root = "cwd"` retains the launch directory.

## Saving and history

Ordinary editing changes the buffer first. Save writes its current text to
disk; it does not clear undo history. Undo can return to a saved state, at
which point the buffer is clean again. Undo does not reverse a completed
filesystem write until you explicitly save the changed text.

Opening the same file again in a session reuses its live buffer. This preserves
unsaved text and history instead of replacing them with another disk read.
When an operation detects an external file change, the save decision gives
you a chance to retain both versions before overwriting anything.

Closing a modified buffer or quitting asks what to do with its text. Canceling
keeps the session alive. A new process reads saved files with fresh undo history;
rev does not restore a previous session automatically.

See [Use Buffers and Panes](../tutorials/user-guide/03-buffers-and-panes.md)
for a short exercise and [Failure recovery](../reference/recovery.md) for
save and filesystem outcomes.
