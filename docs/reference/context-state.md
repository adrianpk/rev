# Context state

Projects and Workspaces use one application-owned TOML aggregate. The executable
uses `$XDG_STATE_HOME/rev/contexts.toml`, falling back to
`$HOME/.local/state/rev/contexts.toml` under the ordinary
[state-root rules](defaults.md#startup-configuration). No metadata file is needed
in a source directory. Use the [context commands](contexts.md#commands) to manage
registrations; this format reference is not a state-file editing procedure.

## Version 1 format

The aggregate contains `schema_version = 1`, a catalog `revision`, and `projects`,
`workspaces` and `sessions` collections. Empty collections encode as arrays;
nonempty collections use TOML arrays of tables. Each Project or Workspace selects
exactly one owned session by `session_revision`.

| Record | Fields |
| --- | --- |
| Project | `id`, `revision`, `name`, `roots_revision`, `session_revision`, `root`. |
| Workspace | `id`, `revision`, `name`, `roots_revision`, `session_revision`, ordered `roots`. Empty membership is valid. |
| Root | `id`, `label`, `location`, optional `origin_project_id`. Provenance is not an authoritative link and need not still identify a registration. |
| Session | `schema_version = 1`, `owner`, `revision`, `roots_revision`, ordered `files`, optional `active_file`. |
| Owner | `kind = "project"` or `"workspace"`, plus the corresponding `id`. |
| File | Optional `root`, `location`, `cursor`, optional `viewport`. |
| Source position | One-based logical `line` and Unicode-scalar `column`. |
| Viewport | Source-position `anchor` and nonnegative `horizontal_cells`. |
| Location | Exactly one of UTF-8 `path` or reversible native-byte `path_bytes`. |

Artifact and root IDs are opaque, typed 32-character lowercase hexadecimal
identities. Names and paths are mutable metadata. Revisions are positive integers
through `9223372036854775807`, checked without wrapping. Root/path membership and
its selected session must have matching `roots_revision`. Renaming does not
change membership or the selected session.

A file with `root` has a root-relative location, with no parent traversal. Without
`root`, its location is an absolute loose-file path. Root locations are absolute.
Native path spelling/case follows filesystem rules; it is not globally lowercased
or restricted to UTF-8. Alias detection uses canonical existing filesystem identity.
`active_file` is a zero-based index into the saved file order, omitted when absent.
Cursor columns count Unicode scalars, including combining characters, rather than
UTF-8 bytes, graphemes or display cells. Restoration clamps changed source positions
to valid line ends. The viewport stores a source anchor and horizontal cell offset,
not a rendered row or pane layout.

This illustrative aggregate contains one Project and one file. IDs and paths are
examples; rev allocates identities and publishes state itself:

```toml
schema_version = 1
revision = 1
workspaces = []

[[projects]]
id = "11111111111111111111111111111111"
revision = 1
name = "Notes"
roots_revision = 1
session_revision = 1
root = { id = "22222222222222222222222222222222", label = "notes", location = { path = "/home/example/notes" } }

[[sessions]]
schema_version = 1
owner = { kind = "project", id = "11111111111111111111111111111111" }
revision = 1
roots_revision = 1
files = [{ root = "22222222222222222222222222222222", location = { path = "notes.txt" }, cursor = { line = 2, column = 1 }, viewport = { anchor = { line = 1, column = 1 }, horizontal_cells = 0 } }]
active_file = 0
```

Snapshots contain file references and positions, not source contents, credentials,
terminal state, process handles, unnamed text, undo history, pane splits or tree
expansions. Unavailable files/root references remain in the saved snapshot. A
Workspace chooses the most specific covering root, with stored order breaking ties.
Loose absolute references retain their paths when roots relocate.

## Capacity and staged work

These are fixed admission limits, not configuration settings. Counts and encoded
bytes apply together; all per-record maxima need not fit in one catalog.

| Boundary | Limit |
| --- | --- |
| Projects | 256 |
| Workspaces | 128 |
| Roots per Workspace | 64 |
| Files per session, including retained unavailable entries | 512 |
| Display name | 256 UTF-8 bytes after trimming |
| Native path | 4096 native bytes, separately from expanded TOML encoding |
| Encoded session | 2 MiB, including its TOML representation |
| Complete encoded catalog | 8 MiB |
| Active context / prepared replacement / owned state IO job | At most one of each |
| Restore ingress per cooperative pass | At most 16 files |

Overflow rejects the mutation without truncating the prior snapshot. Preparation
and publication stay separate; cancellation releases staged work. File ingress
uses native loaders and background state work yields to input/rendering. Context
work retains no polling once it settles. Tree discovery remains independently
bounded and lazy under the [file-tree contract](file-tree.md#traversal-and-refresh).

## Publication and failures

A mutation checks captured catalog/record/session revisions again under one
nonblocking lock attempt. It writes a complete temporary aggregate, syncs it and
atomically replaces the catalog. Conversion adds the Workspace and consumes the
Project at this one commit point. Root membership and its matching session are
published together. There is no second snapshot-file commit or orphan collection.

| Condition | Outcome |
| --- | --- |
| Busy lock or stale writer | Conflict; no silent replacement of newer registrations or sessions. |
| Failure before atomic replacement | Prior catalog and usable live sources remain; transition is incomplete. |
| Warning after replacement | New catalog is adopted; final directory durability/observation is uncertain. A second registration is not needed. |
| Invalid TOML, unsupported schema or broken ownership | Original bytes are retained; mutation is rejected with a scoped diagnostic. |
| Damaged selected session payload | That session cannot restore, relocate, convert or duplicate; valid unrelated registrations remain usable. Rename/forget can manage its registration. |
| Missing file or unavailable Workspace root | Restore available files, diagnose unavailable entries and preserve their metadata. No empty replacement file or cwd root is invented. |
| Missing Project root | Opening is blocked; registration remains available for explicit relocation or forgetting. |
| Cancel before publication | Keep current context and registration; previously approved ordinary saves keep their disk effects. |

Ordinary file/directory opening and tree display do not read or create this
catalog. Listing is read-only, and an empty listing creates no state directory.
Embedded `Session::new`/`empty` callers opt in with
`Session::set_context_storage`; they do not discover user state implicitly.

For practical recovery, see [Manage saved contexts](../how-to/manage-saved-contexts.md).
For capture timing and source decisions, see [File sessions](contexts.md#file-sessions).
