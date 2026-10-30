# Browse Files

From your notes session, open the neighboring `todo.txt` through the file tree.

1. Press `C-x t t` to show the tree on the left.
2. If the root is collapsed, press Right to expand it.
3. Use Up/Down to select `todo.txt`.
4. Press Enter to open it. Keyboard focus returns to the editing pane.

The tree starts at the parent directory of the file used to launch rev.
Launching without a file uses the current directory. Opening another file
later does not change that root.

To return to the tree, press `C-x t f`. Right expands a directory; Left
collapses it or moves to its parent. `C-g` returns to editing while leaving
the tree visible. `C-x t t` hides it.

You can also open a file directly with `C-x C-f`. Type part of its name and
use Tab to complete a unique match or shared prefix. To choose a particular
suggestion, move to it with Up/Down, then use Tab to complete or Enter to open.
Selecting a directory continues browsing inside the prompt.

The current version launches with file paths; `rev .` does not open a directory.
Use the tree or file prompt to browse it instead.

The [file tree reference](../../reference/file-tree.md) covers file creation,
rename, deletion and refresh. [Focused prompts](../../reference/prompts.md)
describes path completion in detail.

Next: [Write Org Notes](06-org-notes.md).

[Previous](04-search-and-commands.md) · [User Guide](index.md)
