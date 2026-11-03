# Run a shell command or text filter

Use installed local tools such as `sort` or `tr` from an editing buffer.
The command prompt uses shell syntax. Source text is supplied only by the two
region commands, as literal stdin data followed by EOF.

## Show command output

Run `M-x shell-command` or modal `:shell-command`, enter `printf 'hello\n'`
and press Return. Non-modal `M-!` opens the same prompt. Keep editing while it
runs, then use `shell-command-output` to visit its read-only plain-text stdout.
The command uses the source file's containing directory captured before the
prompt. For an unnamed buffer it uses the explicitly opened directory, then the
editor launch directory.

## Transform a selection

Select a nonempty linear range. For the whole buffer in non-modal editing,
use `M-< C-SPC M->`; modal Normal can use `ggVG`.

Run `shell-filter-region` (`C-u M-|` in non-modal editing), enter `sort` or
`tr a-z A-Z`, then press Return. Complete successful stdout replaces only the
captured range in one native edit. Undo once with `C-x u` or modal Normal `u`.
Empty stdout deletes the region. The visiting file is not saved automatically.

Use `shell-command-on-region` (`M-|`) to inspect a transformation without
editing. Use `shell-command-insert` (`C-u M-!`) to insert stdout at the captured
point with no document input. Empty insertion leaves history unchanged.

## Cancel or inspect a failure

`C-g` cancels an open shell prompt. To stop an active job, run
`shell-command-cancel` or press `C-g` in its source after any inner prompt,
search or popup has closed. A second job waits for explicit retry after cleanup;
it is not queued. Normal quit consent remains in effect.

Use `shell-command-output` for the last retained stdout or failure diagnostics;
`C-x b` can select `*Shell Diagnostics*` directly when status/stderr is separate.
A changed or closed output identity is not recreated by that command.

A stale, closed, reloaded, rebound or newly read-only target rejects automatic
publication. Inspect the retained output and run a fresh command against your
current source. Nonzero status, invalid UTF-8/NUL, aggregate output overflow,
timeout and cancellation do not publish a partial edit.

Configure the startup [shell preferences](../reference/defaults.md#shell-command-preferences)
when you need another installed shell or finite limits. See the
[User Guide exercise](../tutorials/user-guide/15-shell-commands.md) for a complete
sort-and-undo workflow and the [reference](../reference/shell-commands.md) for
all exact bounds and interaction rules.
