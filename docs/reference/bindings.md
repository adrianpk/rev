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
| Tab | `insert-tab` | Insert indentation; default spaces advance to the next four-column stop. |
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

## Supplied layout and scrolling preferences

Startup configuration and library sessions supplied with an admitted snapshot use `scroll.page_overlap`
to compute page distance from the current pane's content height. Usable panes
advance at least one row even when overlap exceeds that height; suspended pages
retain point. The default overlap remains two rows.

`windows.split_ratio` prefers the first child's share of the available extent
after its one-cell separator. Both axes and native panel composition use the
same ratio, clamped to each subtree's complete recursive minima. Small resizes
suspend the entire editing projection and retain view identities and history.
The default remains equal halves. Executable startup uses the admitted user
configuration snapshot; source edits take effect after restarting.

## Configuration keymap contracts

`keys.editor`, `keys.minibuffer` and `keys.tree` inherit independent complete
maps. Omitted entries keep their built-in command. An exact normalized override
replaces that chord, including an earlier spelling from another included layer.
Removing/commenting an override restores the inherited entry. Two aliases for
one chord in the same source reject, even when their target names agree.
Empty strings, booleans, sentinel values and removal operations cannot unbind.

The candidate is validated before publication. Unknown or unavailable targets,
invalid syntax, normalized duplicates and command/prefix collisions reject the
whole candidate. For example, `C-x` cannot become an immediate command while
`C-x C-s` remains a binding. Diagnostics identify the context, source key and
involved sequences; source key/value locations are included when available.
`C-g` is reserved for `keyboard-quit` in every context. A longer sequence that
contains it is unreachable because it cancels pending prefixes, and rejects.

These contracts expose validated maps through `Snapshot::keymaps()`. Library
input/session construction supplied with a snapshot dispatches `keys.editor`,
`keys.minibuffer` and `keys.tree` through their existing Rust action owners.
The focused prompt temporarily owns the minibuffer map even when the tree
retains workspace focus. No keymap edits the underlying document from a prompt.

### Sequence vocabulary

A sequence has one through eight normalized chords separated by ASCII whitespace.
Each chord is one printable scalar or a named key. Names are case-insensitive:
`RET`/`Return`/`Enter`, `TAB`, `SPC`/`Space`, `DEL`/`Backspace`, `Delete`,
`ESC`/`Escape`, `NULL`, `Left`, `Right`, `Up`, `Down`, `Home`, `End`, `PageUp`,
`PageDown`, `BackTab`. `Delete` removes forward; `DEL` means Backspace.

`C-` adds Control and `M-`/`A-` adds Alt/Meta; modifiers can combine but cannot
repeat. `S-` applies to named keys. For text, use the actual uppercase or shifted
scalar, such as `A` or `%`, rather than `S-a`. Control letters fold to lowercase.
`S-TAB` normalizes to `BackTab`. Named modified keys still require the terminal
to report them; configuration cannot create an intercepted or absent event.

Escape plus a plain non-Escape chord normalizes to its Meta form: `ESC j` and
`M-j` are one binding. A bare Escape is reserved for the existing finite Meta
wait/expiry operation. `ESC C-j` cannot promise a distinct physical sequence.
The [terminal input contract](terminal-input.md) describes timing and reports.

The conservative terminal vocabulary also unifies these aliases, including when
an enhanced protocol can report some physical keys separately:

| Spellings | Normalized key |
| --- | --- |
| `C-i` | `TAB` |
| `C-j`, `C-m` | `RET` |
| `C-h`, `C-?`, `C-8` | `Backspace` |
| `C-[`, `C-3` | Escape Meta prefix |
| `NULL`, `C-@`, `C-2` | `C-SPC` |
| `C-4` | Control-backslash |
| `C-5` | `C-]` |
| `C-6` | `C-^` |
| `C-/`, `C-7` | `C-_` |

Alt remains present on Control aliases: `C-M-h` and `M-DEL` are one chord.

### Editor defaults and availability

The first table on this page lists every editor default, with the aliases above.
Editor targets are the fixed implemented `Command` catalog used by M-x. A user
alias does not install a new handler, shell expression or executable body.
Prompt-only actions below are unavailable in the editor and M-x catalog.


A supplied editor mapping such as `keys.editor.'M-j' = 'next-line'` invokes the
same handler as M-x `next-line`. Reassigning `C-f` replaces that chord while its
unmodified Right alias remains inherited. The input owner copies the admitted
map; it does not reread configuration during typing or drawing.

Custom prefixes consume their complete bounded sequence and display the pending
normalized chords. A mismatch consumes the sequence and reports an error; no
suffix leaks into document text. C-g, paste, prompt acquisition/acceptance and
workspace ownership changes clear pending chords. Resize alone keeps an editing
prefix. Escape expiry clears the whole pending sequence under the existing
finite Meta policy. Unmapped printable input retains ordinary insertion.

Repetition follows the resolved action: single-chord editing/motion actions keep
native repeat behavior, while prefix completions and session/prompt transitions
require fresh presses. Releases do not dispatch. Every Enter alias requires a
fresh prompt acceptance. Literal paste bypasses bindings, including printable
characters assigned to commands, and publishes through the canonical edit owner.
Read-only refusal, saved-state identity and undo use the same owners as M-x.
Executable startup reads user configuration once; source edits require a restart.

### Minibuffer defaults and availability

These defaults describe the existing stage owners. A target is admitted only
where its stage supports it; it never edits the underlying document. Decisions
retain fresh y/n consent (query-replace also accepts q), and Return/paste cannot
affirm a decision. Inventory keeps its existing selection owner.

| Keys | Target | Existing stage behavior |
| --- | --- | --- |
| `C-g` | `keyboard-quit` | Cancel any stage. |
| `RET` | `minibuffer-accept` | Accept text/search, invoke a command, or select inventory; no affirmative decision default. |
| `TAB` | `minibuffer-complete` | Complete commands, find-file paths or buffer names; write/save shows path guidance; literal search/replacement inserts a tab. |
| Backspace | `backward-delete-char` | Delete the preceding text grapheme. |
| Delete, `C-d` | `delete-char` | Delete the following text grapheme. |
| Left, `C-b` | `backward-char` | Move through text graphemes. |
| Right, `C-f` | `forward-char` | Move through text graphemes. |
| Home, `C-a` | `move-beginning-of-line` | Start of prompt text; inventory uses Home alone for first item. |
| End, `C-e` | `move-end-of-line` | End of prompt text; inventory uses End alone for last item. |
| Up, `C-p` | `minibuffer-previous` | Previous command or find-file candidate; inventory uses Up alone for previous item. |
| Down, `C-n` | `minibuffer-next` | Next command or find-file candidate; inventory uses Down alone for next item. |
| `M-p` | `minibuffer-history-previous` | Recall an older accepted command in M-x. |
| `M-n` | `minibuffer-history-next` | Recall a newer command or restore the M-x draft. |
| `C-s` | `isearch-forward` | Repeat forward in incremental search. |
| `C-r` | `isearch-backward` | Repeat backward in incremental search. |

The six `minibuffer-*` targets name existing Rust prompt actions in the common
binding catalog. They are context-only actions. Minibuffer configuration admits
exactly these targets plus the text-motion, deletion, search and cancel command
identities in this table. An unrelated command such as `save-buffer` rejects.

A supplied alias such as `keys.minibuffer.'M-a' = 'minibuffer-accept'` accepts
through the active owner: M-x records its command history, find-file consumes
its path argument, and search accepts its current result. Completion aliases
retain the table's stage behavior, including literal tabs in search/replacement.
Custom text-motion/deletion aliases use the same grapheme cursor and prompt
limit. Explicit previous/next aliases select the corresponding command candidate
or inventory item; history aliases recall commands only in M-x; inherited keys keep the stage distinctions in the table.

Decision stages retain fixed fresh y/n answers (query-replace also accepts q).
Mapped acceptance, completion and editing cannot supply consent. Additional
`keyboard-quit` aliases may be sequences; their mismatched suffix is consumed,
so a y suffix cannot affirm a pending decision. Reserved C-g always cancels.
Accept aliases, completed prefixes and transitions require fresh presses;
releases and held acceptance keys cannot submit a newly acquired argument.

### Tree defaults and availability

| Keys | Target |
| --- | --- |
| `C-g` | `keyboard-quit` |
| `TAB` | `insert-tab` (return to editing) |
| `M-x` | `execute-extended-command` |
| Up, `C-p` | `tree-previous-line` |
| Down, `C-n` | `tree-next-line` |
| Right, `C-f` | `tree-expand` |
| Left, `C-b` | `tree-collapse` |
| `RET`, `C-x t o` | `tree-open` |
| `C-x C-c` | `save-buffers-kill-terminal` |
| `C-x t t` | `toggle-file-tree` |
| `C-x t f` | `focus-file-tree` |
| `C-x t r` | `tree-refresh` |
| `C-x t D` | `tree-delete` |
| `C-x t R` | `tree-rename` |
| `C-x t n` | `tree-create-file` |
| `C-x t d` | `tree-create-directory` |

Tree configuration admits the tree command catalog, cancel, exit, M-x and
return-to-editing, plus the existing navigation aliases `previous-line`,
`next-line`, `backward-char`, `forward-char` and `newline`. File mutation commands
retain their prompts and explicit confirmation; a binding cannot supply consent.

Supplied tree bindings dispatch only while the tree owns workspace focus and no
prompt is active. Exact reassignment preserves independent inherited aliases:
changing `C-n` does not change Down. Navigation aliases can repeat; open, refresh,
mutation and workspace transitions require fresh presses. Pending prefixes are
cleared by prompt acquisition, cancellation and workspace changes. Tree text
and paste never enter the visiting buffer while tree focus is retained.
