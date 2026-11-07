# Manage saved contexts

Use these procedures for existing Projects or Workspaces. Invoke command names
through M-x, or `:` from modal Normal editing. M-x also works from tree focus;
C-g returns from the tree before a modal colon command. These commands select an
artifact by typed identity, so it need not be the active context.

## Rename a registration

1. Run `project-rename` or `workspace-rename`.
2. Select the artifact by its name and path.
3. Edit the name and press Enter. C-g cancels.

The picker shows the new name. Roots, saved files and live text stay intact.
Names are trimmed and case-sensitive; a duplicate name of the same type keeps
the name prompt open. A Project and a Workspace may share a name.

## Reconnect a moved directory

Move the directory with your ordinary filesystem tools first, then:

1. Run `project-set-root`, or `workspace-set-root-path` for one Workspace root.
2. Choose the artifact. For a Workspace, also select the root by ordinal/path.
3. Enter its existing replacement directory. Resolve source decisions if active.
4. Reopen the artifact and check its available files and positions.

The command updates stored paths without moving or copying source files. IDs
remain stable. Root-relative saved files follow the replacement directory; loose
absolute files keep their paths. Other Projects and Workspaces keep their own
paths, including roots originally added from this Project. Repeat relocation
for those independent registrations when needed.

Missing files remain saved entries with diagnostics. A missing Project root
blocks opening until you reconnect or forget it. A partially available Workspace
can open its available roots; unavailable members remain listed. C-g cancels a
prepared change. A storage conflict or failure retains the prior registration
and usable live work; it does not complete relocation.

## Forget a registration

1. Run `project-forget` or `workspace-forget` and select the artifact.
2. Read the identified artifact, then type `yes` to confirm. C-g cancels.
3. If forgetting the active Workspace, resolve source/clock/job departure.

The registration and its saved session disappear from that picker. Source files
and directories remain on disk. Forgetting an active Project keeps its buffers,
including dirty text, as temporary directory work. Forgetting an active Workspace
returns to the startup screen over standalone work, with no replacement root.
Forgetting an inactive artifact leaves current editing alone. Separately
registered Projects and copied Workspace roots remain independent.

For the practice exercise, forget `Practice Desk` and `Practice Notes` by name;
the practice source files remain available through ordinary file opening.

[Lifecycle reference](../reference/contexts.md#lifecycle-operations) gives exact
conversion and source-decision behavior. [Context state](../reference/context-state.md)
defines storage limits, conflicts and preserved invalid input.
