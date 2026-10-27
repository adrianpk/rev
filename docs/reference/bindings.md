# Bindings

`C-` means Control; `M-` means reported Alt/Meta or a compatible Escape prefix.
Spaces separate successive chords: `C-x C-s` means Control-x, then Control-s.
Ordinary printable text inserts directly. These editing bindings apply while
an editing window owns focus; tree, prompt and inventory focus have their own
keys. See [Terminal input](terminal-input.md) for reports and physical-terminal
limits, and [Focused prompts](prompts.md) for argument and decision keys.

Every command below is also available as `M-x` followed by its exact name.

| Keys | Command | Result |
| --- | --- | --- |
| `M-x` | `execute-extended-command` | Find and invoke a command. |
| `C-g` | `keyboard-quit` | Cancel the current interaction or deactivate the region. |
| `C-x C-c` | `save-buffers-kill-terminal` | Guarded session exit. |
| `C-x C-f` | `find-file` | Open or reuse a visited file. |
| `C-x C-s` | `save-buffer` | Save to the current destination. |
| `C-x C-w` | `write-file` | Request another destination. |
| `C-x b` | `switch-to-buffer` | Select a live buffer by name. |
| `C-x C-b` | `list-buffers` | Select from the live buffer inventory. |
| `C-x k` | `kill-buffer` | Close a buffer with modified-text protection. |
| `C-x 2` | `split-window-below` | Split into upper/lower views. |
| `C-x 3` | `split-window-right` | Split into left/right views. |
| `C-x o` | `other-window` | Cycle editing views. |
| `C-x 0` | `delete-window` | Close the selected view, retaining its buffer. |
| `C-x 1` | `delete-other-windows` | Keep only the selected view. |
| Return | `newline` | Insert the buffer's preferred line separator. |
| Tab | `insert-tab` | Insert a literal tab. |
| Backspace | `backward-delete-char` | Delete the preceding grapheme. |
| Delete, `C-d` | `delete-char` | Delete the following grapheme. |
| Right, `C-f` | `forward-char` | Move over one grapheme. |
| Left, `C-b` | `backward-char` | Move back one grapheme. |
| `M-f` | `forward-word` | Move to the next word end. |
| `M-b` | `backward-word` | Move to the preceding word start. |
| Home, `C-a` | `move-beginning-of-line` | Move to logical line start. |
| End, `C-e` | `move-end-of-line` | Move to logical line end. |
| `M-<` | `beginning-of-buffer` | Move to buffer start. |
| `M->` | `end-of-buffer` | Move to buffer end. |
| Down, `C-n` | `next-line` | Move down, retaining the intended display column. |
| Up, `C-p` | `previous-line` | Move up, retaining the intended display column. |
| Page Down, `C-v` | `scroll-up-command` | Advance one page. |
| Page Up, `M-v` | `scroll-down-command` | Move back one page. |
| `C-l` | `recenter-top-bottom` | Cycle point's line through center/top/bottom. |
| `C-x u`, reported `C-/` or `C-_` | `undo` | Undo a buffer-local group. |
| `M-x undo-redo` | `undo-redo` | Reapply the next redo group. |
| `C-s` | `isearch-forward` | Incremental literal forward search. |
| `C-r` | `isearch-backward` | Incremental literal backward search. |
| `M-%` | `query-replace` | Confirm each literal replacement. |
| `C-SPC` | `set-mark-command` | Activate a linear region at point. |
| `C-x C-x` | `exchange-point-and-mark` | Exchange point and mark. |
| `C-k` | `kill-line` | Kill to logical line end, then its separator. |
| `M-d` | `kill-word` | Kill toward the next word end. |
| `M-Backspace` / `M-DEL` | `backward-kill-word` | Kill toward the preceding word start. |
| `C-w` | `kill-region` | Remove the active linear region into the ring. |
| `M-w` | `kill-ring-save` | Copy the active linear region. |
| `C-y` | `yank` | Insert the newest linear ring entry. |
| `M-y` | `yank-pop` | Rotate the current yank's exact insertion range. |
| `C-x SPC` | `rectangle-mark-mode` | Toggle display-column selection. |
| `C-x r M-w` | `copy-rectangle-as-kill` | Retain selected rectangle rows. |
| `C-x r k` | `kill-rectangle` | Remove and retain the rectangle as one edit. |
| `C-x r y` | `yank-rectangle` | Insert retained rows with required padding. |
| `C-x t t` | `toggle-file-tree` | Show or hide the left tree. |
| `C-x t f` | `focus-file-tree` | Focus the visible tree. |
| Up, `C-p` in tree | `tree-previous-line` | Select the preceding visible entry. |
| Down, `C-n` in tree | `tree-next-line` | Select the next visible entry. |
| Right, `C-f` in tree | `tree-expand` | Lazily expand a directory. |
| Left, `C-b` in tree | `tree-collapse` | Collapse or select the parent. |
| `M-x tree-toggle-directory` | `tree-toggle-directory` | Toggle a directory; leaves report an error. |
| `C-x t r` | `tree-refresh` | Reload already opened tree branches. |
| Return in tree, `C-x t o` | `tree-open` | Open a file or toggle a directory. |
| `C-x t n` | `tree-create-file` | Create an empty file at the selected directory. |
| `C-x t d` | `tree-create-directory` | Create a directory at the selected directory. |
| `C-x t R` | `tree-rename` | Rename one entry and reconcile visited paths. |
| `C-x t D` | `tree-delete` | Confirm removal, including separate recursive consent. |

Tab or `C-g` returns from tree focus to editing. The tree is separate from
editing-window cycling. In the buffer inventory, use arrows, Home/End, Return
or `C-g`. In a decision, use a fresh `y` or `n`; query-replace also accepts `q`.
Paste and Return do not grant affirmative consent.

See [Commands](commands.md) for matching, completion and command history,
[Rectangles](rectangles.md) for block edges and [Scope and defaults](defaults.md)
for retained text, history and file behavior.
