# Native Bookmarks Model

Status: draft
Date: 2026-10-27
Specification: [Native Bookmarks](editor-bookmarks.md)
Text/view ownership: [rev Editor Model](rev-editor-model.md)

## Candidate semantic fields

| Field or relation | Meaning | Decision state |
| --- | --- | --- |
| Bookmark identity | Stable reference for navigation and derived markers. | Representation and uniqueness scope open. |
| Document/buffer association | Owner of the intended location. | File, unsaved buffer and rename policy open. |
| Checked location evidence | Anchor/range/revision policy used to resolve the location. | Edit gravity, stale/delete and undo behavior open. |
| Optional name/category | User-facing identity if named bookmarks are selected. | Not yet selected. |
| Lifecycle/persistence | Session survival, removal and possible storage. | Session-only versus persistent unresolved. |

The bookmark owner holds canonical semantics. View-local marker projections
are derived and cannot mutate bookmark state. The existing transient mark and
selection keep their own editing identities.

No persistence schema, index, XDG filename or migration is selected. Add those
contracts only if persistence enters the accepted bookmark scope; retain
session-only behavior as an explicit alternative until that decision is made.
