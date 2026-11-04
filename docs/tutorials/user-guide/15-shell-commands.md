# Run Shell Commands

Sort an unsaved list with a local command, undo the whole transformation and
inspect command output without leaving rev. This exercise uses the installed
`sh`, `sort` and `tr` programs.

## Prepare a small list

Create a practice file and open it:

```sh
printf 'pear\napple\n' > fruit.txt
rev fruit.txt
```

Add `banana` on its own line without saving. Your buffer now contains the new
line while the file on disk still contains only pear and apple.

In non-modal editing, move to the beginning with `M-<`, set the mark with
`C-SPC`, then move to the end with `M->`. This selects the complete buffer.
In modal Normal, use `ggVG` for a linewise Visual selection.

## Sort the selected text

Press `C-u M-|` in non-modal editing. In modal Visual, type
`:shell-filter-region` and press Return. At `Shell command:`, enter:

```sh
sort
```

Press Return. When it completes, the selected text becomes:

```text
apple
banana
pear
```

The unsaved banana line reaches sort through stdin; rev does not save or read
an old copy of fruit.txt. Only the selected range is replaced. `C-x u` in
non-modal editing, or `u` in modal Normal, undoes the entire filter once.
The file stays unchanged until you explicitly save.

Try the same selection with `tr a-z A-Z` to change the list to uppercase.
Undo once to restore it. Selected text is input data; the text you accept at
the shell prompt is the command interpreted by the shell.

## Inspect output before changing text

Use `M-|` or the shared `shell-command-on-region` command with `sort` to send
the selected text and retain stdout without replacing the list. Run
`M-x shell-command-output`, or `:shell-command-output`, to visit the read-only
plain-text output. Use `C-x b`, enter `fruit.txt` and press Return to return.

For a command that needs no selected text, press `M-!`, or invoke
`shell-command` by name, and enter:

```sh
printf 'hello\n'
```

The shell receives immediate stdin EOF. Reopen its result with
`shell-command-output`. To insert stdout at your captured point instead, use
`C-u M-!` or `shell-command-insert`. For example, `printf 'ready\n'` inserts
one line and one undo removes it.

All six [shell commands](../../reference/shell-commands.md) are available through
M-x and modal colon. The four dedicated sequences are non-modal defaults;
modal Normal/Visual keep their existing C-u motion. Linear Visual ranges remain
selected across colon entry; block selections are rejected.

## Cancel and keep working

At the shell prompt, `C-g` cancels without executing or changing your selection.
Modal prompt Escape also cancels. `M-p` and `M-n` recall admitted commands and
restore the current draft; at most 32 strings stay in this editor session.

Continue editing while a command runs. Use `shell-command-cancel`, or press
`C-g` in the originating source, to stop it. Another prompt, search or completion
popup consumes its own cancellation first. Running/stopping status is visible
when the pane has room. A second invocation reports the active job until cleanup
finishes. Quitting uses ordinary save/discard decisions; cancelling quit keeps
the job running.

A filter or insertion applies only if its original source is still writable
and unchanged, with the same visiting-file association and no conflicting disk
observation/reload. If you edit, undo, reload or close the source while it runs,
valid output remains inspectable but cannot automatically change that source.
Nonzero exit, invalid output, timeout and cancellation also leave source intact.
Inspect `*Shell Diagnostics*` with `C-x b` for status, stderr or rejection details.

Next: [Use an Integrated Terminal](16-integrated-terminal.md) for a
persistent shell or a full-screen tool. These finite text filters do not provide
interactive stdin or terminal emulation. See the
[shell task guide](../../how-to/run-shell-command.md) or the
[exact shell reference](../../reference/shell-commands.md).

[Previous](14-ai-editing.md) · [User Guide](index.md)
