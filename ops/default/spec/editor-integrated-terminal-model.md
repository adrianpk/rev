# Integrated Terminal Model

Status: Approved
Date: 2026-10-28
Reviewed: 2026-11-03
Approved: product owner, 2026-11-03; promoted with the terminal specification
Version: 0.4
Specification: [Integrated Terminal Sessions](editor-integrated-terminal.md)
Delivery: [Plan](../plan/rev-integrated-terminal.md), [tracker](../tracker/rev-integrated-terminal.md)
Existing editor model: [rev Editor Model](rev-editor-model.md)
Separate owner: [Shell Commands Model](editor-shell-commands-model.md) owns one-shot stdin/stdout jobs; those jobs do not create terminal sessions or placements

## Ownership

One terminal session owns one process/PTY lifetime and alacritty_terminal state.
A stable terminal-backed buffer identifies that session in editor navigation.
A view/attachment presents it in the lower panel or main editing area. Moving
an attachment does not move, copy or recreate the process or its terminal state.

Terminal-backed buffers are not canonical editable file ropes. Their live
screen and bounded scrollback derive from process output and terminal state;
ordinary source save/splice/undo operations require explicit type admission.
The existing source-buffer owner remains unchanged.

## Session-local semantic records

| Record | Fields and contract |
| --- | --- |
| Terminal session | Stable identity, process/PTY ownership, creation context, lifecycle state, exit observation and finite resource account. Independent of visibility. |
| Terminal-backed buffer | Stable buffer/session relation, user-visible name and live/exited state. Participates in navigation without becoming a writable source file. |
| Emulator state | alacritty_terminal Term/grid/modes/cursor/style, alternate screen and bounded retained output, owned under one session lifetime. rev adapts its state and events to native rendering and host-effect policy. |
| Placement | Lower panel or selected main-area buffer attachment, focus and valid geometry. Changing this record preserves session/buffer identity. |
| Resize observation | Session, authoritative live attachment, dimensions and generation. Child receives admitted current geometry through the PTY owner. |
| Input/output work | Owning session, bounded bytes/events, ordering and lifecycle evidence. Input cannot be redirected by a stale focus/session observation. |
| Retained projection | Session/screen generation, view/geometry, text/cell runs and optional copy selection. Historical output is not source text or current process authority. |

Ordinary panes may mirror the same terminal-backed buffer. One focused live
attachment owns resize authority; other views retain independent viewport/copy
state over the same session. With no live view, retain the last valid geometry.
Reattachment publishes a checked resize before geometry-sensitive input.

Use the crate's existing PTY/event facilities where they satisfy rev's lifetime
and bounds. The delivery pins alacritty_terminal =0.26.0 with default features
disabled, and requires rev-owned bounded IO instead of adopting its unbounded
EventLoop queues. No additional direct crate is selected. No Zig engine,
desktop Alacritty process/configuration or window/font backend enters this model.

## Preferences and defaults

Default creation uses the lower panel. Direct creation in a main-area buffer
is a separate explicit operation. The configured user shell is the interactive
default; explicit argv/environment remain structured data. rev's directory
context supplies the initial location with file-parent/launch-directory
fallbacks. The approved plan defines the initial typed TOML keys, panel
dimensions and finite resource bounds. The shipped sample is checked against
the live schema; Slice 7 records actual storage, queue, CPU/RSS and cleanup
observations. The user reference lists all seven startup fields.

There is one instance-global default session and one remembered lower-panel
terminal/size. The lower panel displays one selected session at a time;
selecting another retains the previous session and its process. Multiple
independent sessions are part of the first delivery, selected through ordinary
buffer navigation. Creation is on demand: editor startup and buffer listing
do not launch children, and selecting or moving an existing session does not
duplicate its process. terminal-open reuses a live default; terminal-new and
terminal-new-buffer create independent sessions. Opening after default exit
creates a fresh default without discarding its exited buffer. Buffer names are
terminal, terminal-2 and so on, with explicit identity-preserving rename.

Moving preserves actual child working state. Metadata about the current shell
directory must come from an admitted source, not guessed prompt text. Future
directory-opening ownership should supply any chosen editor-context default
without introducing workspace registration or changing a running child's cwd.

## Lifecycle and failure

Creation must prepare process, terminal state, buffer identity and a valid
attachment coherently. A failed candidate leaves no unowned child. Session
states are Running, Draining, Exited, Closing, Closed and Failed. Prepared and
Starting describe the pre-publication creation transaction, not published live
registry states. Draining retains final IO/cleanup after leader exit. Hidden or
detached remains presentation state, not a replacement process state.

Relocation prepares the new attachment/geometry before replacing the old one.
Failure retains the old valid placement. Hide/detach is not process termination.
Killing a live terminal-backed buffer/session requires explicit termination
confirmation. Declining preserves it. Confirmed closure owns signaling, waiting
and resource release; unrelated processes remain outside ownership. Editor
exit with live sessions asks once to terminate them or cancel exit. Session
processes do not implicitly survive an editor restart. Normal child exit
retains bounded output and truthful status outside the child output stream
until explicit closure.

Child control sequences are interpreted under the emulator contract and lowered
into editor-owned rendering. Output does not acquire authority to overwrite
other panes, invoke host commands or change AI tool grants. Live keyboard and
pointer routing must distinguish child interaction from host command/copy mode.

Live input, protected host command/recovery routes and literal-next handling
follow the paired specification's explicit TRM06/TRM11 contract. Copy mode owns stable retained-text navigation while
the child continues. Native user paste follows child bracketed-paste state;
child-requested clipboard effects are denied and hyperlinks require explicit
activation. The crate's own optional vi-navigation helpers do not select or
replace rev's startup-only non-modal/modal profile.

## Implementation mapping and readiness

`buffer::Resource::Text` retains canonical source text and file/history authority;
`Resource::Terminal(SessionId)` identifies a terminal resource with inert source
storage. `term::Sessions` owns process/PTY/emulator state, bounded IO and cleanup.
Session attachments own panel/main-area placement and copy viewports. Input
uses this type/focus distinction before source-only commands or providers.

Confirmed close is asynchronous: a Closing session retains buffer/capacity
until its owned worker returns successful cleanup. Failed cleanup keeps its
process ownership and diagnostic. Discovery/signals/wait/reap stay off the UI,
with the unchanged 500 ms grace and two-second cleanup deadline. Live commands
have no runtime cap. Interrupted zero-time exit observation defers to the next
bounded service turn rather than converting a live child into a failure.

No live session, scrollback, process or attachment is persisted across editor
restart. Only the existing immutable startup configuration is durable. The
baseline [editor model](rev-editor-model.md) preserves source/file/history
invariants and delegates these transient records to this model.

Slices 1-7 are integrated through canonical PRs #248-#254. Their reports retain
exact-candidate implementation and measured acceptance evidence; the paired
specification maps TRM01-TRM11. Slice 8 reconciles guidance/catalogs and the
baseline. Final combined acceptance and delivery closure remain with the
controller's unchanged gate, so these records do not mark delivery Completed.
