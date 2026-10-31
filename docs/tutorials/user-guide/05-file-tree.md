# Browse Files

From your notes session, open the neighboring `todo.txt` through the file tree.

1. Press `C-x t t` to show the tree on the left.
2. Its immediate entries are already shown. Use Up/Down to select `todo.txt`.
3. Press Enter to open it. Keyboard focus returns to the editing pane.

You can also click `todo.txt` to select and open it. Clicking a directory
expands or collapses it. For selection-only clicks, set
`mouse.tree_activate_on_click = false` in startup configuration, then use Enter
to open the selected entry.

Look for these file rows; other entries are omitted:

```text
   notes.txt
   todo.txt
```

The tree starts at the parent directory of the file used to launch rev.
Launching without a file uses the current directory. File visits follow the
file's parent unless you chose a directory explicitly or configured cwd rooting.
An explicit directory root stays selected when you visit files elsewhere.

To return to the tree, press `C-x t f`. Right expands a directory; Left
collapses it or moves to its parent. `C-g` returns to editing while leaving
the tree visible. `C-x t t` hides it.

You can also open a file directly with `C-x C-f`. Type part of its name and
use Tab to fill the highlighted suggestion. The first match is highlighted,
even when several names share a prefix. Up/Down chooses another suggestion;
Tab fills it and Enter opens it.
Tab continues through a completed directory inside the prompt. Enter accepts
the directory itself, closes the prompt and opens its tree.

With `todo.txt` highlighted, typing `to` and pressing Tab fills the prompt even
when `tomorrow.txt` also matches:

```text
Find file: todo.txt
```

Run `rev .` to browse the current directory, or pass another directory path.
Its tree opens with immediate entries shown and child directories collapsed.
Hiding and showing it later preserves your selection and deliberate collapse.

To combine browsing with editing, start with `rev .`, use `C-x C-f` and Tab to
complete a file, then press Enter. Split with `C-x 3` and click another file in
the tree: it opens in the editing pane you last selected. Each pane wraps at its
own width. `C-g` cancels a Find File prompt without opening the draft path;
resizing preserves the documents and their source positions.

The [file tree reference](../../reference/file-tree.md) covers file creation,
rename, deletion and refresh. [Focused prompts](../../reference/prompts.md)
describes path completion in detail.

Next: [Set Your Preferences](09-preferences.md).

[Previous](04-search-and-commands.md) · [User Guide](index.md)
