# Compose a Workspace

Use a Workspace to keep one editing session over independent directory roots.
No Git repository, common parent or Project registration is required. Invoke
these names through M-x or modal colon from source editing.

## Add directories

1. Run `workspace-create`, name the empty Workspace and press Enter. To use an
   existing Workspace instead, select it through `workspace-open`.
2. Run `workspace-add-directory` and accept an existing directory. Repeat for
   another directory. Each becomes a collapsed peer root in insertion order.
3. To reuse a registered directory, run `workspace-add-project` and choose the
   Project. Its registration and saved files stay independent.
4. Expand roots in the tree and visit files. Leave with `context-close`, then
   reopen by name to check the Workspace's saved file session.

Equivalent canonical or symlink paths cannot appear twice in one Workspace.
Ancestor and descendant roots may overlap; opening the same file through either
route reuses its live buffer. Missing roots stay listed for later repair.

## Remove a root

Run `workspace-remove-root`, select the identified root and resolve any source
decisions. Files still covered by another root remain open. Files losing their
only covering root close through save/discard/cancel. Loose files remain open.
Cancel keeps both membership and live work. Removing the final root leaves a
valid empty Workspace. No source directory or Project registration is deleted.

## Convert or duplicate a Project

Choose the operation according to the desired registration:

| Desired result | Command | Source session |
| --- | --- | --- |
| Replace the Project with a Workspace | `project-promote-to-workspace` | Active live file positions or inactive saved session. Active dirty text stays live. |
| Keep both as independent sessions | `project-duplicate-as-workspace` | Active file positions or inactive saved session, with source decisions before destination publication. |

1. Run the chosen command and select a Project.
2. Enter a new Workspace name.
3. For duplication, resolve modified/unnamed buffers in the departing context.
   C-g cancels and creates no Workspace. Save keeps its ordinary disk effects;
   discard makes the destination read current disk contents.
4. Select the Workspace through `workspace-open`, then add other roots normally.

Conversion publishes the new Workspace and removes the Project in one catalog
replacement. Inactive conversion leaves current editing alone. Duplication opens
the new Workspace and leaves the Project's selected snapshot intact. Both use
fresh Workspace/root identities. Source files are shared, never copied: separate
editing sessions do not give separate file contents or bypass external-change
protection.

Use [Manage saved contexts](manage-saved-contexts.md) to rename, reconnect or
forget artifacts. See the [context reference](../reference/contexts.md) for
membership, overlap, cancellation and restoration details.
