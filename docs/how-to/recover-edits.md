# Recover edits after a failed operation

Keep the live session while resolving an error. The echo distinguishes an
unperformed operation from a published filesystem change whose completion or
refresh failed. See [Failure recovery](../reference/recovery.md) for those outcomes.

## A save did not publish

1. Read the operation and path in the echo. For a missing or inaccessible parent,
   create/fix the intended directory outside the editor, or choose another
   existing writable parent with `C-x C-w`.
2. Request a fresh save with `C-x C-s` or `C-x C-w`. Previous canceled path or
   close requests do not carry authority into this command.
3. If the destination changed, inspect the external file before answering the new
   overwrite question. `n` or `C-g` retains both the live edits and external data.
4. Verify `Saved` and a clean `--` status before relying on the save for exit.

## A save published but completion is unresolved

1. Inspect the reported target on disk. The file may already contain the new
   bytes; the modified indicator also retains an unresolved-publication warning.
2. Resolve the reported synchronization/observation problem and request a new
   save explicitly. Any changed or unobserved destination needs fresh consent.
3. Use `C-g` to cancel a required close/exit flow while investigating. A later
   independent save does not automatically resume the canceled flow.

## A tree action failed or removed an open path

1. Read whether the echo reports `Created`, `Renamed`, `Deleted` or a possibly
   partial deletion. Do not assume that a later refresh error rolled disk back.
2. Focus the tree with `C-x t f` and refresh with `C-x t r` after fixing access or
   path problems. Loaded branches retain a usable selection.
3. Switch to the retained buffer with `C-x b`. A removed visited path keeps its
   text and history. Save explicitly to recreate its file in an existing parent,
   or use `C-x C-w` to preserve it elsewhere.

## Input, syntax or geometry interrupted the task

Use `C-g` to cancel the focused prompt, search or decision. Resize retains text,
history and pending input; enlarge the terminal to restore the normal panes.
Syntax failure leaves editable text and saving available; a later accepted
revision can recover highlighting. `C-x u` remains the terminal-reportable undo
alternative when a terminal cannot distinguish punctuation Control chords.

Use [Focused prompts](../reference/prompts.md), [File tree](../reference/file-tree.md)
and [Scope and defaults](../reference/defaults.md) for the retained-state rules.
