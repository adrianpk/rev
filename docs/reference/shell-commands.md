# Shell commands

Shell commands execute one explicitly accepted command string with an installed
external shell. One invocation is owned at a time. Results are native plain-text
buffers, with checked optional source publication.

## Commands and input

| Command | Standard input | Result |
| --- | --- | --- |
| `shell-command` | Immediate EOF; no document text. | Retain stdout. |
| `shell-command-on-region` | Captured nonempty linear selection, then EOF. | Retain stdout without editing. |
| `shell-command-insert` | Immediate EOF; no document text. | Insert complete successful stdout at the captured point. |
| `shell-filter-region` | Captured nonempty linear selection, then EOF. | Replace that exact range with complete successful stdout. |
| `shell-command-cancel` | None. | Request cancellation and stop/reap the owned process group. |
| `shell-command-output` | None. | Visit the last retained stdout or diagnostic identity. |

All commands use the shared M-x and modal colon selectors and support configured
bindings. The non-modal defaults are `M-!` (command), `M-|` (region display),
`C-u M-!` (insert) and `C-u M-|` (filter). These are exact keymap sequences;
there is no general numeric-argument engine. Org's other local C-u routes remain
available. Modal Normal/Visual retain their C-u motion.

The prompt accepts a nonempty, non-whitespace UTF-8 string of at most 4096 bytes,
without NUL. Ordinary native single-line prompt editing rejects control input.
Return admits one execution; cancellation changes neither source nor history.
M-p/M-n navigate at most 32 admitted strings held in session memory and restore
the current draft. History recall does not execute a command.

A region must be nonempty and linear. It is never replaced by an implicit
whole-document input. Linear modal Visual ranges survive colon entry; rectangles
and blocks reject. Region bytes come from the captured unsaved canonical text,
with the actual newlines and Unicode content preserved.

## Program, arguments and directory

Resolve the program from nonempty `shell.program`, otherwise nonempty `SHELL`
captured at editor startup, otherwise `/bin/sh`. A bare program name uses ordinary
executable lookup. Append the accepted command as one final argument after
`shell.args`, whose default is `["-c"]`. No login/interactive flag is added.

Capture the visiting file's parent directory before the prompt. For unnamed
buffers, use the explicitly opened directory, then the editor launch directory.
A later focus or tree change does not redirect that invocation. The child inherits
the editor process environment. Configuration and file opening never launch a job,
and command execution never implicitly saves the source.

## Exact output and publication

Collect stdout and stderr separately under one aggregate byte bound. Completion
requires exit and complete stream observation. Strict UTF-8 without NUL is
required for native result presentation; bytes are not trimmed, newline-normalized,
formatted or interpreted as terminal control. ANSI-looking text remains text.

A filter or insertion requires complete successful stdout and the original
buffer/revision/range, file association and writable policy. Edits, undo, closure,
reopening, reload, pending external observation/discard consent and changed disk
evidence revoke automatic publication. Valid stdout remains inspectable.
Publication uses one native edit/undo step and maps current pane points. Empty
replacement deletes its range; empty insertion makes no history entry.

Completion retains `*Shell Output*` without selecting it or replacing a newer
interaction's message. Nonzero status and stderr/rejection details remain in
`*Shell Diagnostics*`. Use the output command or native buffer switching to inspect
them. Invalid text, cancellation, timeout and overflow provide diagnostics without
an applicable partial result. Output handles cannot resurrect closed buffers or
overwrite buffers whose native identity/ownership changed.

## Limits and lifecycle

These five preferences are global and startup-only. They participate in ordinary
include precedence, source provenance and whole-candidate fallback. See the
[complete sample](config.toml) and [configuration catalog](defaults.md#shell-command-preferences).

| Preference | Default | Admission |
| --- | --- | --- |
| `shell.program` | `""` | Empty selects the startup fallback; otherwise at most 4096 UTF-8 bytes, without NUL. |
| `shell.args` | `["-c"]` | 1–32 strings, at most 4096 bytes each and 32768 total, without NUL; whole-array replacement. |
| `shell.timeout_seconds` | `600` | Integer 1–600. |
| `shell.max_input_bytes` | `8388608` | Integer 1–16777216; checked before copying/staging selection bytes. |
| `shell.max_output_bytes` | `8388608` | Integer 1–16777216; aggregate stdout plus stderr. |

The job remains active while stopping; another request neither queues nor replaces
it. Native input, rendering, editing and file observation continue. Contextual
source C-g or the named cancel command stops the job after inner prompt/search/
popup or modal Escape ownership. Quitting stops it only after ordinary save/quit
consent succeeds; cancelled quit retains it.

Timeout, cancellation, output failure, leader exit and shutdown close streams,
stop owned descendants and reap the leader. Background shell children are part
of that invocation's cleanup rather than retained sessions. No shell worker or
periodic shell input/render deadline remains after completion. Active status
shows running/stopping where it fits; narrow panes preserve ordinary status.

For a first successful transformation, follow
[Run Shell Commands](../tutorials/user-guide/15-shell-commands.md). For a specific
task, see [Run a shell command or text filter](../how-to/run-shell-command.md).
