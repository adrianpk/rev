# Browse Files

From your notes session, open the neighboring `todo.txt` through the file tree.

1. Press `C-x t t` to show the tree on the left.
2. Its immediate entries are already shown. Use Up/Down to select `todo.txt`.
3. Press Enter to open it. Keyboard focus returns to the editing pane.

The `todo.txt` row stays highlighted after it opens, including when it was
already displayed. Switch to `notes.txt` with `C-x b`: the highlight follows
the active file. In split panes, selecting another editing pane follows that
pane's file. The matching row scrolls into view if it is already loaded.

Press `C-x t f` and browse another entry without opening it. That entry gets
the highlight while the tree has focus. Press Tab or `C-g` to return to editing;
the highlight returns to the active file, and your browsing position is retained
for the next tree visit. A collapsed folder stays collapsed. An unnamed buffer
or a file outside the chosen tree has no active-file highlight.

To keep your manual selection while switching editing buffers, edit the setting
in the existing `[tree]` table and restart:

```toml
[tree]
follow_active_buffer = false
```

Set it to true and restart to restore following. The default is true when the
setting is absent. Both choices keep persistent selection feedback without
changing your text or undo history.

You can also click `todo.txt` to select and open it. Clicking a directory
expands or collapses it. For selection-only clicks, set
`mouse.tree_activate_on_click = false` in startup configuration, then use Enter
to open the selected entry.

The default ordinary-font presentation looks like this when your directory is
named `notes`; other entries are omitted:

```text
▾ ◆ notes/
    ↑ ..
    ≡ notes.txt
    ≡ todo.txt
```

`◆` identifies the root and `≡` identifies documents. Directories have a `/`
kind symbol and a separate `▸` disclosure; C-f changes it to `▾` and shows
the children. Source files use `λ`, configuration files `=`, other files `·`
and symlinks `@`. The symbols remain useful without color.

The tree starts at the parent directory of the file used to launch rev.
After New Buffer, manually showing the tree uses the launch directory. File
visits follow the file's parent unless you chose a directory explicitly or configured cwd rooting.

With the tree focused, press `^` to show its parent directory. You can also
select the `..` row and press Enter, or click it with default mouse activation.
A selection-only click waits for Enter. The directory you just left stays
selected and scrolls into view; press Enter to expand it
again. Repeat `^` to move up another level. `M-x tree-up-directory` does the
same in both profiles. Explicit directory, Project, Workspace and configured
cwd roots stay fixed. At the filesystem root there is no parent. If the parent
cannot be listed or your previous directory cannot be shown, the tree stays
where it was and explains the reason. Going up preserves your text, split panes
and editing context; later file visits still follow their parent directories.
An explicit directory root stays selected when you visit files elsewhere.

To return to the tree, press `C-x t f`. C-f expands a directory; C-b
collapses it or moves to its parent. `C-g` returns to editing while leaving
the tree visible. `C-x t t` hides it.

The tree automatically grows for expanded names, including entries below the
visible rows. Folding a directory or selecting a short name keeps that width
stable. The default minimum is 24 columns, and the panel uses at most one third
of the terminal so your editing area remains usable.

With tree focus, press `>` to widen or `<` to narrow it by four columns. That
choice stays in effect while you browse, refresh or change directories. From
an editing buffer, use `C-x t >` or `C-x t <`; ordinary `<` and `>` still type
text. Press `C-x t =` to reset to your configured width policy. Automatic reset
fits the currently expanded names, so it can return to more than 24 columns.

Run `M-x tree-auto-width-mode` (or `:tree-auto-width-mode` in the modal profile)
to toggle automatic fitting. Disabling keeps the current desired width;
enabling fits the current tree again. These controls leave your configuration
file unchanged. To start with fixed width, merge this into your existing
`[tree]` table and restart:

```toml
[tree]
auto_width = false
width = 24
```

Set `auto_width = true` for automatic fitting; `width` then sets its normal
minimum. A narrow terminal temporarily clamps or hides the panel, and the
chosen desired width returns when space is available again.

With tree focus, j/k select the next/previous row. These aliases work in both
editing profiles and leave ordinary source input unchanged.

When a name still extends beyond the width cap, use h/l or Left/Right in the tree to
pan all rows one column at a time. The indentation and icons move with the names,
so deep entries stay aligned. The arrows preserve selection and do not expand a
directory; use C-f/C-b for expansion/collapse and Enter to open or toggle.
Pan right to see a name's ending and left to return to its beginning. Position
survives browsing, focus and hide/show. A wider pane or shorter content clamps
only excess position. Changing roots or pressing `C-x t =` returns to the start.
The named commands are `tree-scroll-left` and `tree-scroll-right`; find them with
M-x or the modal selector, including their effective shortcuts.

You can also open a file directly with `C-x C-f`. Type part of its name and
use Tab to fill the highlighted suggestion. The first match is highlighted,
even when several names share a prefix. Up/Down chooses another suggestion;
Tab fills it and Enter opens it.
Tab continues through a completed directory inside the prompt. Enter accepts
the directory itself and prepares a fresh temporary editing context. Save or
discard existing modified buffers when asked; `C-g` cancels the replacement and
retains them. The new context opens the directory's tree.

With `todo.txt` highlighted, typing `to` and pressing Tab fills the prompt even
when `tomorrow.txt` also matches:

```text
Find file: todo.txt
```

Run `rev .` to browse the current directory, or pass another directory path.
Its tree opens with immediate entries shown and child directories collapsed.
Hiding and showing it later preserves your selection and deliberate collapse.
Use `M-x open-directory` for a directory-only prompt. These temporary directories
do not register a named Project or overwrite a Project's saved session.

`M-x generate-new-buffer` adds an unnamed buffer to your current work. To leave
all current buffers while keeping rev open, run `M-x context-close`. It uses the
same save/discard/cancel decisions as quitting and settles owned work before
returning to the startup screen over fresh standalone work. In Vim-style editing,
invoke the same names after `:`.

The next chapter turns a temporary directory into a named Project, then builds
an independent Workspace. See [Return to Projects and Workspaces](20-projects-and-workspaces.md).

To combine browsing with editing, start with `rev .`, use `C-x C-f` and Tab to
complete a file, then press Enter. Split with `C-x 3` and click another file in
the tree: it opens in the editing pane you last selected. Each pane wraps at its
own width. `C-g` cancels a Find File prompt without opening the draft path;
resizing preserves the documents and their source positions.

To try minimal labels, edit `decorated` in the existing `[tree]` table of your
startup configuration, then restart rev:

```toml
[tree]
decorated = false
```

Your file rows now look like this, with the same navigation and click behavior:

```text
v notes/
    ..
    notes.txt
    todo.txt
```

Set `decorated = true` and restart to restore symbols and entry colors. The
[preferences chapter](09-preferences.md) explains how to edit startup settings;
merge these excerpts into existing tables rather than adding duplicate tables.

If your terminal already uses a Nerd Fonts-compatible font, set `nerd_fonts`
to true in its existing `[gutter]` table and restart:

```toml
[gutter]
nerd_fonts = true
```

This changes both decorated tree icons and gutter bookmarks. Minimal tree
labels stay minimal. The default is false; rev does not install or detect
fonts. If icons appear as boxes, set it back to false for ordinary symbols.

The [file tree reference](../../reference/file-tree.md) covers file creation,
rename, deletion and refresh. [Focused prompts](../../reference/prompts.md)
describes path completion in detail.

Next: [Return to Projects and Workspaces](20-projects-and-workspaces.md).

[Previous](21-keyboard-macros.md) · [User Guide](index.md)

## Manage saved contexts

Use [Manage saved contexts](../../how-to/manage-saved-contexts.md) to rename,
reconnect or forget registrations. [Compose a Workspace](../../how-to/compose-workspace.md)
covers independent membership, conversion and duplication.
