# Native Bookmarks Model

Status: draft
Date: 2026-10-27
Reviewed: 2026-10-31
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

## Lifecycle and resolution

Creation captures document/location evidence through the native owner.
Navigation resolves and validates that evidence before selecting a document
and point. Removal ends the bookmark's identity rather than turning its old
list position into another bookmark. A view-local marker is disposable.

Edits, undo/redo, file rename/close/reopen and deleted locations require an
explicit policy. Candidate resolution outcomes include valid, unavailable and
stale; the final states and recovery behavior remain open. A stale observation
must never silently address unrelated current text. Define these policies with
examples before treating anchors as stable across revisions.

## Queries and ownership

The admitted command set may need location lookup, enumeration and ordered
navigation. Their scope, uniqueness and ordering remain open alongside named
versus unnamed bookmarks. Do not add a database index or project registry
before choosing those semantics. Derived marker lookup uses current logical
line identity and gutter visibility; it does not own canonical bookmark state.

## Storage decision

The bookmark owner holds canonical semantics. View-local marker projections
are derived and cannot mutate bookmark state. The existing transient mark and
selection keep their own editing identities.

No persistence schema, index, XDG filename or migration is selected. Add those
contracts only if persistence enters the accepted bookmark scope; retain
session-only behavior as an explicit alternative until that decision is made.

If persistence is selected, extend this model with field ordering, versioned
serialization, actual XDG ownership, uniqueness, load/recovery and migration
rules. If session-only is selected, no storage or migration machinery is needed.
Neither policy is selected by this formalization.
