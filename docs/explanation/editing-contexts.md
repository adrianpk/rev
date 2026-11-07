# Editing contexts

A context groups the buffers you are working with now. Registration adds a name
and a saved file session; tree visibility only changes presentation. This lets
ordinary file visits stay temporary while named work can be resumed deliberately.

| Context | What it owns | What survives leaving |
| --- | --- | --- |
| Standalone | Ordinary files and unnamed buffers | Source files saved normally; no registered file session. |
| Temporary directory | Live buffers and one explicitly chosen directory | Source files; no registration or saved context session. |
| Project | One named directory and an independent file session | Registration, ordered file references and source positions. |
| Workspace | Ordered independent roots and its own file session | Registration, root membership and source positions. |

Opening a registered Project's path is still an ordinary directory operation.
Selecting its name expresses a different intent: resume its saved work. This
separation avoids accidentally replacing a Project session just to browse its
directory. Promoting the current temporary directory makes that choice explicit
while preserving its live buffers and unsaved text.

A Workspace collects paths rather than owning Project objects. Adding a Project
copies its directory path and optional provenance; it does not import that
Project's open files. Forgetting or relocating the Project therefore cannot
silently change other Workspaces. Conversion replaces a Project registration;
duplication keeps it and begins an independent Workspace session. Neither copies
source files, so ordinary save and external-change rules still apply.

A saved session remembers where to reopen files, not their unsaved contents.
Native save/discard/cancel decisions resolve live sources before departure.
Missing files and unavailable roots remain diagnosed references so temporary
unavailability does not erase work. Pane splits, tree expansions and undo history
are live editor state rather than this basic file snapshot.

The startup screen is a view over a retained context, not another document.
Opening a picker can be canceled without losing that context. Successful
replacement retires the old context through its existing source and job owners.
At process startup, rev shows the five actions; named sessions restore only when
explicitly selected. There is no automatic last-application-session restoration.

A root also gives existing per-source services a directory boundary. A Workspace
does not combine its roots into an AI grant or start tools automatically.
Bookmarks, command usage and the kill ring keep their existing global ownership.

Try [Return to Projects and Workspaces](../tutorials/user-guide/20-projects-and-workspaces.md).
Use the [context reference](../reference/contexts.md) for commands and behavior,
and [Context state](../reference/context-state.md) for the durable format.
