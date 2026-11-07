# Tree Selection Policy

Status: Approved
Date: 2026-11-06
Updated: 2026-11-07
Inspection baseline: `978deacc49f6f5f948cea2070919ce91031f67a2`
Specification: [File Tree Selection and Active Buffer](editor-tree-selection.md)

## Configuration contract

| Field | Type | Default | Lifetime |
| --- | --- | --- | --- |
| `tree.follow_active_buffer` | boolean | `true` | Existing admitted startup snapshot |

The enabled default and configurability are owner decisions. The field name
and the specification's complete focus/reveal policy were approved on 2026-11-07. Add the field
to the native catalog, built-in defaults, complete sample, overrides and source
provenance. Invalid values follow normal candidate rejection; no partial
configuration or project-local discovery is introduced.

## Transient ownership

| Value | Owner and invariant |
| --- | --- |
| Browsing selection | Existing tree row/path selection; retained while browsing unopened entries |
| Editing origin | Active editing view, or last editing view when another input owner has focus |
| Active-file identity | Existing canonical buffer/path identity; absent for an unnamed buffer |
| Matching route | Existing root membership/path identity in the loaded preorder; optional when the file has no reliable listing match |
| Visible indication | At most one row, derived from the originating input owner, startup following policy and the matching route |

Do not persist paths, selections, pane IDs or an extra follow-state file.
Changing the indication does not open a file or alter source history. Keep
native navigation's root identity and directory-alias rules authoritative.
Retire stale listing-row associations on refresh; never retain a previous
row index as proof that a different entry is still active.

## Indication policy

| Input owner and preference | Highlight source |
| --- | --- |
| Focused tree, either preference | Manual browsing selection |
| Editing, following enabled | Eligible active-file route; absent if unmatched |
| Editing, following disabled | Retained manual browsing selection |
| Command or argument prompt | Same role as its captured tree/editing origin |
| Hidden tree | No rendered indication; preserve identities for later presentation |

The target of an explicit editing-buffer transition supersedes its former
file identity, including transitions to unnamed, result or terminal buffers.
A transient minibuffer never becomes a source-file identity. No matching route
is represented by a parent directory or a fallback selected index.

## Route and lifetime rules

- Preserve canonical source identity and existing alias evidence. Labels and
  basenames never participate in identity matching. Resolve nothing through
  filesystem I/O added solely for following.
- For overlapping Workspace roots, keep a matching retained route when possible;
  otherwise choose the first loaded match in declared root order. Keep one
  canonical buffer and one highlighted route, not one per matching membership.
- Recompute after successful file activation, pane changes, rename/Save As,
  buffer replacement, context changes and relevant listing publication. Reject
  stale routes after refresh or root removal. Point movement in the same file
  does not cause repeated matching, scrolling or filesystem work.
- A follow transition may reveal a loaded offscreen entry by scrolling the tree.
  It never expands a directory, changes filtering, substitutes a root, opens a
  hidden pane or alters an editing source/view position.
- Preserve manual browsing and collapse choices. Unmatched active files leave
  their indication absent while the browsing cursor stays available for tree focus.

Use the existing palette.tree_selected and palette.tree_selected_text roles,
existing reverse video under NO_COLOR and existing tree focus/cursor ownership.
No persisted UI selection, context-session field, state file, command, binding,
glyph catalog or palette field is added by this delivery.
