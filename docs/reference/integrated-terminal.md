# Integrated Terminals

Use the [terminal chapter](../tutorials/user-guide/16-integrated-terminal.md)
for a guided shell/tool session, or the [task guide](../how-to/use-integrated-terminal.md)
for placement, copy and shutdown procedures. This reference defines the exact
interactive terminal commands and limits. [One-shot shell commands](shell-commands.md)
receive finite stdin and collect plain text for a checked edit; interactive
sessions run a shell or tool under a PTY and retain terminal state.

## Opening a shell

Use `M-x terminal-open`, or `:terminal-open` from modal source editing, to open
the instance's default shell in a lower panel. Repeating the command selects
the same live session. Creation uses the selected directory context, then the
current file's parent or launch directory. Startup `[terminal]` preferences
select the executable and literal argument array; no shell starts automatically.

## Live input and mouse

The live panel owns printable keys and ordinary child input in either profile.
`C-g` returns focus to the editor. `M-x` opens host commands while the panel is
focused; a prompt's `C-g` cancels that prompt before returning editor focus.
Printable colon and Escape belong to the live child. `C-x` retains the host prefix.
`C-q`, or `terminal-send-next-key`, forwards exactly one following normalized
key to the child, including `C-g`, `M-x` or `C-x`. Key releases do not consume
that request. Return, Tab, deletion, arrows, Home/End, paging and F1-F20 use
classic VT encodings; modified movement uses native modifier parameters. Child
application-cursor, reported application-keypad and newline modes are respected.
`C-c C-t` enters a frozen copy view of retained output. `C-c C-c` sends one
child interrupt, and `C-q C-c` sends it literally; `C-c C-g` cancels the prefix.
An unclaimed second key sends the deferred C-c and that key together.

The child cursor uses its requested block, bar or underline shape and blink
state; host prompts and source editing restore their own cursor presentation.

Child-enabled pointer reporting uses classic, UTF-8 or SGR coordinates with
button, drag, wheel and requested motion events. Shift retains local cell
selection and bounded history scrolling. Without child reporting, pointer
selection and wheel scrolling are local to that attachment; retained-history
views also keep local control. Releasing outside a captured terminal clips to
its admitted cells. Source drags retain their original source owner.
Visible terminal surfaces acquire native mouse reports even when source mouse
mode is disabled; only a focused child requesting motion acquires all-motion
tracking. Hiding or leaving the terminal restores native tracking ownership.

## Links and child effects

Control-click activates an OSC 8 hyperlink only after URI validation. Admitted
schemes are HTTP, HTTPS, mailto and local file URIs; remote file authorities and
executable schemes are rejected. Links require explicit `org.tools.viewer`
argv, using the existing `{target}` substitution or one appended target. The
viewer receives the URI as literal data in the terminal's creation directory,
with the existing viewer time/output limits. One activation can be owned at a
time. Missing configuration and viewer failure remain visible. Child clipboard
requests, host commands, title effects and unsupported graphics remain inert.

## Retained output and paste

`terminal-copy-mode` captures a private read-only projection for the selected
attachment. Native point, region, search and selection copying inspect that
snapshot while the child continues producing output. Copying retains the view
and selection. Return or `terminal-copy-mode-done` resumes live input without
sending a child Return. C-g cancels a search first, then clears a selection, then
leaves copy mode; stronger host prompts retain their cancellation owner.

Each projection preserves physical terminal rows, cell spacing, wide characters
and combining text, with a 1 MiB UTF-8 text cap. At most 32 projections are retained
under an 8 MiB editor storage charge, separately from emulator history. Admission
failure keeps existing views and the live child. Local pointer inspection also
freezes output; returning its wheel position to the tail, or accepted live input,
resumes following. Explicit copy mode retains keyboard ownership until exited.

`terminal-yank` (C-y in live terminal input) pastes the shared kill ring's
newest entry; `terminal-yank-pop` (M-y) rotates it only while that exact paste
remains wholly queued. Any write, later input, protocol reply, unrelated command
or changed kill ring ends replacement authority. Rev never guesses how to erase
text a child has consumed. Literal-next still sends these keys to the child.
Explicit frontend paste uses the same atomic queue and active bracketed-paste
mode. Text exceeding the configured input limit, including framing, is rejected
before queue mutation. Escape and control characters other than line breaks and
tabs are rejected. Child clipboard requests remain denied.

`terminal-clear` sends C-l to the child; `terminal-clear-scrollback` removes
retained emulator history and copy projections for that session, preserves the
live screen and sends no child input. Copy line start/end operate on the frozen
text. `terminal-reset-cursor-point` aligns with the live cursor only while its
current text is still represented by that copy; otherwise it reports the need
to refresh. `terminal-send-return`, tab, escape, backspace, delete, up/down,
left/right, prior/next (paging), and space use the same mode-aware live VT key
encoder. `terminal-undo` sends C-_ to the child without changing source history.
Input commands, including literal-next, require exiting explicit copy mode.

## Reported prompts and error locations

`terminal-previous-prompt` and `terminal-next-prompt` navigate actual retained
OSC 133 A reports in a copy projection. Prompt-looking text supplies no evidence.
The engine retains at most 1024 reports, follows ordinary full-screen scrolling
and drops unproven positions after resizing or complex grid transformations.
Each copy keeps its captured reports while live output continues.
`terminal-next-error` recognizes retained `path:line[:column]: error:` locations
with positive decimal positions. It opens an existing regular file using the
terminal's creation directory for relative paths. Missing files, malformed or
out-of-range locations and unavailable evidence retain the current source.
It does not infer a shell's later working directory.
`terminal-open-link` activates the validated URI at the live cursor or frozen
copy point through the configured viewer. Copy snapshots retain at most 1024
link spans charged within 64 KiB; absent evidence never falls back to new live
output. Control-click also uses captured evidence during copy inspection.

## Placement and selection

Use `terminal-toggle` to hide or restore the panel without restarting its shell.
The panel remembers its height and clamps it to preserve a usable source pane.

`list-buffers` and `switch-to-buffer` include the terminal's stable buffer name.
Selecting it restores its current placement. Use `terminal-new` for an
independent panel session or `terminal-new-buffer` for direct main-area entry.
Move the same process with `terminal-move-to-buffer` and
`terminal-move-to-panel`; a rejected geometry keeps the old attachment.
`terminal-list` shows retained sessions, and `terminal-rename` accepts a unique
1-256 byte name without control characters. Names start with `terminal`, then
`terminal-2` under ordinary buffer name conflict handling.

`terminal-open-here` uses the current source file's containing directory. For
a terminal resource, it uses the admitted creation directory; it does not guess
a shell's later working-directory changes. `terminal-open-other-pane` selects
the terminal in the next pane, preparing a right split when there is only one.
Splitting a terminal buffer presents the same child in another view; focused
live input owns resize, and each attachment retains its own viewport state.
Small screens suspend presentation until geometry becomes usable again.

Terminal buffers hold emulator state rather
than editable source text, so file saves, text edits, undo, bookmarks, completion,
LSP, Org and AI source actions require returning to a source buffer. Killing a
live terminal buffer asks before terminating its owned process group.

## Command inventory

Every name below is available through M-x and the shared command catalog.
Modal colon entry requires a compatible source or copy context: a colon typed
in live terminal input belongs to the child.

| Command | Effect |
| --- | --- |
| `terminal-open` | Open or select the instance default interactive terminal in the lower panel. |
| `terminal-toggle` | Hide or restore the lower terminal panel without restarting the child. |
| `terminal-new-buffer` | Create an independent terminal directly in the selected editing pane. |
| `terminal-move-to-buffer` | Move the current terminal into the selected editing pane. |
| `terminal-move-to-panel` | Dock the same session in the lower panel. |
| `terminal-new` | Create an independent lower-panel session. |
| `terminal-list` | Select or inspect retained terminal sessions. |
| `terminal-rename` | Give the selected terminal buffer a unique name without changing its identity. |
| `terminal-open-here` | Create a session from the current resource's directory context. |
| `terminal-open-other-pane` | Select a terminal in the next pane, creating a right split when needed. |
| `terminal-send-next-key` | Send one following normalized key literally to the child, bypassing host recovery. |
| `terminal-copy-mode` | Freeze retained output for native navigation, selection and search. |
| `terminal-copy-mode-done` | Resume live input without sending Return to the child. |
| `terminal-yank` | Paste the newest retained text using active bracket framing. |
| `terminal-yank-pop` | Rotate only while the preceding paste remains wholly queued. |
| `terminal-clear` | Send C-l through ordinary child input. |
| `terminal-clear-scrollback` | Clear retained history and copy projections, preserving live screen/input. |
| `terminal-beginning-of-line` | Move to the frozen copy line start. |
| `terminal-end-of-line` | Move to the frozen copy line end. |
| `terminal-reset-cursor-point` | Align copy point with a still-represented live cursor. |
| `terminal-undo` | Send C-_ to the child without source undo. |
| `terminal-send-return` | Send Return with the live mode-aware VT encoder. |
| `terminal-send-tab` | Send Tab with the live mode-aware VT encoder. |
| `terminal-send-escape` | Send Escape with the live mode-aware VT encoder. |
| `terminal-send-backspace` | Send Backspace with the live mode-aware VT encoder. |
| `terminal-send-delete` | Send Delete with the live mode-aware VT encoder. |
| `terminal-send-up` | Send Up with the live mode-aware VT encoder. |
| `terminal-send-down` | Send Down with the live mode-aware VT encoder. |
| `terminal-send-left` | Send Left with the live mode-aware VT encoder. |
| `terminal-send-right` | Send Right with the live mode-aware VT encoder. |
| `terminal-send-prior` | Send PageUp with the live mode-aware VT encoder. |
| `terminal-send-next` | Send PageDown with the live mode-aware VT encoder. |
| `terminal-send-space` | Send Space with the live mode-aware VT encoder. |
| `terminal-next-prompt` | Navigate the next captured OSC 133 prompt. |
| `terminal-previous-prompt` | Navigate the previous captured OSC 133 prompt. |
| `terminal-next-error` | Visit an actually recognized retained error location. |
| `terminal-open-link` | Explicitly activate the validated link at live or frozen point. |

## Startup settings

The following seven settings belong to `[terminal]`. They are read at editor
startup and do not start a shell by themselves. `terminal.args` is literal argv;
no `-c` or login flag is added implicitly. The child inherits the editor's
process environment. These preferences are separate from `[shell]` text-filter
preferences. Invalid configuration follows the ordinary all-or-default policy.

| Setting | Type/range | Default | Contract |
| --- | --- | --- | --- |
| `terminal.program` | String | `""` | Empty selects the nonempty startup SHELL value, then /bin/sh. Otherwise one executable, at most 4096 UTF-8 bytes without NUL. |
| `terminal.args` | Array of strings | `[]` | Whole-array replacement, including an empty array; at most 32 strings and 16384 combined UTF-8 bytes without NUL. |
| `terminal.panel_height` | Integer 3..=200 | `10` | Preferred terminal content rows, clamped to valid geometry that preserves usable source panes. |
| `terminal.max_sessions` | Integer 1..=32 | `8` | Owned live and exited sessions; explicit closure releases capacity. Cell admission can reject a session before this count is reached. |
| `terminal.scrollback_lines` | Integer 0..=20000 | `2000` | Requested retained rows, additionally constrained by cell and extra-storage admission. |
| `terminal.max_pending_input_bytes` | Integer 4096..=1048576 | `262144` | Complete input sequences are accepted atomically into this bound. |
| `terminal.max_pending_output_bytes` | Integer 4096..=1048576 | `262144` | Maximum pre-parser output; direct bounded reads apply PTY backpressure without an accumulating output queue. |

For example, select an interactive Bash explicitly:

```toml
[terminal]
program = "/bin/bash"
args = ["--noprofile", "--norc", "-i"]
panel_height = 15
```

The [complete sample](config.toml) remains the packaged default configuration.
No terminal session, process, scrollback or current tool state is restored after
restarting rev.

## Lifecycle and ownership

| State | Meaning |
| --- | --- |
| Running | The editor owns the live child; hidden views do not change this state. |
| Draining | Leader exit was observed; final output and owned cleanup are still pending. |
| Exited | The actual exit status and bounded final output remain inspectable. |
| Closing | Explicitly approved cleanup is running; the buffer and capacity remain owned. |
| Failed | A diagnostic is retained; failure does not count as successful cleanup. |
| Closed | Cleanup completed and the registry can release this session. |

Closing a live terminal with `kill-buffer` asks whether to terminate it.
Declining or cancelling retains the session. Closing an exited buffer releases
its retained output and capacity. Merely hiding the panel, changing buffers or
removing a view does not terminate a child. Opening the default after its exit
creates a fresh shell and retains the exited buffer until explicit closure.

Quitting first resolves source saves/discards, then asks once about live
terminal sessions. Declining keeps the editor and sessions. Approved cleanup
runs off the UI: owned session groups receive hangup, a 500 ms grace period,
then kill if needed, within the two-second cleanup deadline. This includes
owned background groups; unrelated processes are never targeted. Pending or
failed cleanup remains visible and cannot be reported as completed. A changed
source cancels exit; cancelling exit after cleanup was approved does not undo
signals already authorized. A host I/O failure still attempts owned cleanup and
terminal restoration. Live commands and loops have no runtime timeout.

## Resource limits and rendering

Admission is bounded by 1048576 reserved cells per session and 2097152 across
the editor, including both screens, retained rows and row caches. Each session
has a separate 8 MiB extras bound; control strings stop at 4096 bytes and
combining text at 64 scalars per cell. One service turn handles at most 65536
bytes across sessions with fair rotation. Reads use bounded scratch storage
and PTY backpressure; accepted arbitrary bytes are not dropped to meet a limit.
A rejected creation, resize, move or paste preserves its previously admitted
state and reports the failure.

The host terminal owns fonts, glyph appearance and ligatures. rev renders
admitted terminal cells/styles within its own panes. Unsupported child effects
do not become host commands or source edits. Physical keys depend on
[distinguishable terminal reports](terminal-input.md); in live mode a separately
decoded Escape goes to the child, while an explicit reported Alt-x opens host
commands. The source editor's Escape/Meta deadline is not a live child prefix.

See the [terminal acceptance procedure](../how-to/terminal-journeys.md#run-integrated-terminal-acceptance)
for the bounded real-shell/lazygit and resource observations.
