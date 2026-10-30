# File and Directory Navigation Model

Status: accepted
Date: 2026-10-31
Approved: product owner, 2026-10-31; selected five-topic delivery, Emacs interaction criterion and explicit Looper start request
Plan: [Wrap and Navigation](../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../tracker/rev-wrap-navigation.md)
Specification: [File and Directory Navigation](editor-file-navigation.md)
Existing owners: [rev Editor Model](rev-editor-model.md), [Editor Configuration Model](editor-configuration-model.md)

## Ownership and semantic records

These concepts describe responsibilities, not mandatory Rust structs or a new
workspace model. Canonical documents retain text, file identity and history.
The session owns chosen browsing context; views own focus and tree projection.

| Concept | Evidence and lifetime | Invariant |
| --- | --- | --- |
| Opening request | CLI or find-file origin, typed path spelling, base directory and explicit intent. | Resolve the resource kind before selecting a file or directory operation; completion is not opening. |
| Resolved target | Existing path/alias identity, file/directory kind and current admission observation. | Existing checked identity rules control buffer reuse; a directory does not become document text. |
| Tree root choice | Resolved directory, explicit choice or automatic file-parent origin, and selection generation. | An explicit root is not narrowed by visiting a nested file; explicit directory wins, then configured cwd, then admitted file-parent/cwd fallback. |
| Tree listing | Root/directory identity, finite observed entries and success/failure state. | Failed observation is not an empty success; descendants stay lazy. |
| Tree navigation | Initialized-root evidence, selected entry identity, expansion set and viewport. | First expansion runs once per new root; hiding preserves deliberate state. |
| Path completion session | Prompt/draft identity, query cursor, candidate identities, current selection and listing observation. | Tab acts on the selected candidate; a row number alone cannot identify a refreshed candidate. |
| Tree pointer hit | Current layout/viewport/listing observation and rendered entry identity. | Revalidate before selection or any admitted activation; blank and stale hits cannot redirect an action. |

## Transitions and failure

An opening request resolves and prepares a candidate before changing the
current root or document. A failed candidate preserves the previous state.
Directory opening commits the chosen root and browsing visibility together.
File opening follows existing canonical file admission and view selection.

First root presentation observes immediate children through the existing tree
owner. Later hide/show does not reset that initialization or deliberate
collapse state. Directory completion changes a prompt draft/listing only;
it does not commit a root choice. Pointer selection updates the tree's selected
identity without manufacturing a document edit.

Primary click selects and activates by default; the startup preference
mouse.tree_activate_on_click = false selects only. File visits preserve an
explicit root even outside it. Existing checked path/alias handling remains
authoritative. Rendering does not become a second path-resolution or IO owner.

## Persistence boundary

Requests, candidate lists, hits and root/navigation state are session-local.
This proposal adds no disk record, project file, persisted directory history,
schema migration or implicit AI access grant. Existing startup preferences keep
their own loader and storage contract; the new mouse.tree_activate_on_click
boolean defaults true and uses the existing loader, validation and complete
sample catalog.
