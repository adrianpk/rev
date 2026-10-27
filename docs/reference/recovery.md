# Failure recovery

Recoverable errors retain accepted in-memory text and report the operation and
path when applicable. Error text describes actual filesystem effects; it does
not promise rollback after publication.

| Result | Retained state | Next action |
| --- | --- | --- |
| Invalid or unreadable startup configuration | Usable editor with complete built-ins and a source/key diagnostic; no partial configured leaves apply. | Correct the source and start a new session; running sessions retain their snapshot. |
| Failed file read or invalid UTF-8/binary input | Current buffers, history and shared views. No partial file becomes a buffer. | Correct the file/path or cancel the pending interaction. |
| Save not published | Current text, history, association and unsaved state. The save has not replaced the destination. | Resolve the reported error and request a fresh save. |
| Save published, completion unresolved | The replacement exists and the actual target remains associated. A warning keeps the buffer modified; required close/quit saving is not satisfied. | Inspect the target and explicitly retry saving; changed or unobserved disk state needs fresh consent. |
| Tree creation directory changed during its prompt | No entry is created by that request, including in the replacement directory. | Refresh, select the intended directory and start creation again. |
| Tree operation published, refresh/observation failed | Published entries and reconciled visited paths remain effective. Partial deletion retains every open buffer and records actual missing paths. | Resolve the reported access/path problem, then refresh. |
| Syntax parse/query failed or exceeded its result bound | Accepted text, marks, shared positions and history remain. Styling falls back without blocking saving or editing. | Continue editing; a later accepted revision can recover parsing, and a smaller visible query can still return valid styling. |

Saving observes the target before temporary output and rechecks it after
staging, before replacement. A detected external change aborts that attempt and
cleans up its owned temporary. These conservative metadata checks do not prevent
all changes in the final interval before replacement.

A rejected compound edit changes no row or shared position. Read-only changes
and altered peer positions are rechecked at publication; rejection preserves
marks, saved state, history and the redo future. C-g cancels transient input;
normal region deactivation still follows the relevant interaction's contract.

## Terminal restoration

The terminal owner attempts bracketed-paste disable, cursor show, attribute
reset, alternate-screen exit and raw-mode release in that order. An earlier
output failure does not skip later obligations, including raw-mode release.
A combined error names every failed step and retains the first error kind.
Unfinished obligations remain owned and are retried on Drop. A retry can repeat
output commands already delivered after a buffered write failure; errors report
attempted restoration rather than claiming that every step succeeded.

See [Focused prompts](prompts.md) for decision/cancel behavior,
[File tree](file-tree.md) for path changes, and [Syntax highlighting](syntax.md)
for revision-qualified styling.
