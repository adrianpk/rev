# Return to Projects and Workspaces

Keep a directory's editing session by name, then build a separate session over
two directories. This exercise uses the default non-modal keys and ordinary text
files. It needs no Git repository, language server or configuration changes.
In modal editing, use `:` from Normal state for the same named commands; return
from tree focus with C-g first.

## Start with ordinary files and directories

In a terminal, create an unused practice directory and two files:

```sh
mkdir -p rev-context-practice/notes rev-context-practice/reference
printf 'First note.\nSecond note.\n' > rev-context-practice/notes/notes.txt
printf 'Reference material.\n' > rev-context-practice/reference/reading.txt
cd rev-context-practice
rev notes/notes.txt
```

The file opens directly as standalone editing. Run `M-x dashboard-open`. The
startup screen keeps that file available behind it. Its five actions are:

```text
New Buffer
Open File
Open Dir
Open Project
Open Workspace
```

Use arrows and Enter to choose Open Dir, then accept the displayed `notes`
directory in the native path prompt. The prompt starts at the visiting file's
parent; Enter accepts that offered directory. The tree opens there. Modified or
unnamed work must first pass the ordinary save/discard/cancel decisions. This is temporary
directory work. Opening a directory does not register it or restore saved files.
C-g cancels a prompt to the screen; C-g on the screen returns to retained editing.

Press C-g if the tree has focus. Use `C-x C-f` to visit `notes.txt`, then move to
the second line. You can also start directly with `rev notes` from the practice
parent directory, or `rev .` from inside `notes`.

## Keep the directory as a Project

1. Run `M-x project-promote` and replace the proposed name with `Practice Notes`.
2. Press Enter. The current buffers, positions and unsaved text stay live. This
   now has a Project registration; the command does not save source text.
3. Run `M-x context-close`. Resolve any source decisions. The named file session
   is checkpointed, and the startup screen appears.
4. Choose Open Project, type `Practice Notes`, use Tab to fill the selected
   label and Enter to open it. `notes.txt` returns at its saved source position.

The same picker is available through `project-switch-project` or `project-list`.
Selecting the active Project simply returns to its current work. Leaving it or
quitting cleanly captures its ordered file buffers, active file and positions.
Unnamed contents and undo history do not become saved session text.

To see the difference between path and name, run `M-x open-directory` and accept
the displayed `notes` directory. It opens temporary browsing without replaying the
Project session. Use `project-switch-project` to restore that session explicitly.

## Build an independent Workspace

1. Run `M-x workspace-create` and name it `Practice Desk`. It opens empty.
2. Run `M-x workspace-add-project` and select `Practice Notes`. Only its directory
   is added; the Project and its saved session remain independent.
3. Run `M-x workspace-add-directory` and choose the sibling `reference` directory.
   Replace the offered path with its full path if needed; path prompts resolve
   relative entries from their displayed base.
4. Focus the tree with `C-x t f`. Each root is a collapsed peer. Expand one with
   Right, select a file and press Enter. Open `reading.txt` from the other root
   the same way. Each file opens in the retained editing pane.
5. Run `M-x context-close`, then choose Open Workspace and select `Practice Desk`.
   Its own open files and positions return. Use Open Project afterward to check
   that `Practice Notes` still has its separate file session.

Use `workspace-open` or `workspace-list` to return directly by name. A Workspace
can remain empty, and files explicitly opened outside its roots are saved as
loose files. Root membership does not grant tools or AI access across directories.

For cleanup, use [Manage saved contexts](../../how-to/manage-saved-contexts.md).
For conversion, duplication and removing membership, see
[Compose a Workspace](../../how-to/compose-workspace.md).
Missing files are diagnosed without creating replacements; the
[context reference](../../reference/contexts.md) gives the exact restoration rules.
[Editing contexts](../../explanation/editing-contexts.md) explains their ownership.

Next: [Set Your Preferences](09-preferences.md).

[Previous](05-file-tree.md) · [User Guide](index.md)
