# Bindings

`C-` means Control; `M-` means reported Alt/Meta or a compatible Escape prefix.
Spaces separate successive chords: `C-x C-s` means Control-x, then Control-s.
In the default non-modal profile, editing is Emacs-like and ordinary printable
text inserts directly. These editing bindings apply while an editing window
owns focus; tree, prompt and inventory focus have their own
keys. See [Terminal input](terminal-input.md) for reports and physical-terminal
limits, and [Focused prompts](prompts.md) for argument and decision keys.

Every command below is also available as `M-x` followed by its exact name.

| Keys | Command | Result |
| --- | --- | --- |
| `M-x` | `execute-extended-command` | Find and invoke a command. |
| `M-x lsp-mode` | `lsp-mode` | Toggle optional owned server transport for this session. |
| `M-x lsp-connect` | `lsp-connect` | Restore this buffer eligibility without enabling globally disabled LSP. |
| `M-x lsp-disconnect` | `lsp-disconnect` | Exclude this canonical buffer across all its views until restored or closed. |
| `M-x lsp-restart` | `lsp-restart` | Restart the eligible current root; preserve global mode and buffer exclusions. |
| `M-x lsp-status` | `lsp-status` | Show the current file transport state or recoverable prerequisite failure. |
| `M-x lsp-setup` | `lsp-setup` | Register an installed server, acquire a supported managed recipe, or select a compatible source. |
| `M-x lsp-reload-settings` | `lsp-reload-settings` | Reload only LSP settings and file associations, preserving runtime mode, exclusions and startup profile. |
| `M-x lsp-servers` | `lsp-servers` | Open a read-only snapshot of configured sources, local executable availability and managed recipe prerequisites. |
| `M-x lsp-select-server` | `lsp-select-server` | Persist a compatible configured source, ready managed installation or native-only choice. |
| `M-x lsp-install-server` | `lsp-install-server` | Acquire and verify a managed catalog recipe, or select a ready installation; preserve runtime LSP mode. |
| `M-x lsp-update-server` | `lsp-update-server` | Verify a replacement before retargeting selected managed profiles; retain the old payload. |
| `M-x lsp-remove-server` | `lsp-remove-server` | Remove a confirmed installer-owned payload only when unreferenced and stopped. |
| `M-x lsp-cancel-installation` | `lsp-cancel-installation` | Request active management cancellation while its worker retains cleanup ownership. |
| `M-x completion-mode` | `completion-mode` | Toggle native completion for this session. |
| `M-x completion-auto-mode` | `completion-auto-mode` | Toggle automatic requests while retaining explicit invocation. |
| `M-TAB` | `completion-at-point` | Show native suggestions near the source cursor. |
| Down, `C-n` in a completion popup | `completion-next` | Select the next candidate without moving source point. |
| Up, `C-p` in a completion popup | `completion-previous` | Select the previous candidate. |
| Tab or Return in a completion popup | `completion-accept` | Replace the admitted identifier as one undo operation. |
| Escape or `C-g` in a completion popup | `completion-cancel` | Dismiss the popup without editing source. |
| `C-g` | `keyboard-quit` | Cancel the current interaction or deactivate the region. |
| `C-x C-c` | `save-buffers-kill-terminal` | Guarded session exit. |
| `C-x C-f` | `find-file` | Open a file or directory; available in editing and tree focus. |
| `C-x C-s` | `save-buffer` | Save to the current destination. |
| `C-x C-w` | `write-file` | Request another destination. |
| `M-x revert-buffer`, `:revert-buffer` | `revert-buffer` | Reload a checked disk image; confirm before discarding unsaved text. |
| `M-x ai-explain`, `:ai-explain` | `ai-explain` | Explain the captured selection with guarded directory context and compatible private Pi history. |
| `M-x ai-prompt`, `:ai-prompt` | `ai-prompt` | Ask for an instruction; show selection replacement or point insertion before generation. |
| `M-x ai-implement`, `:ai-implement` | `ai-implement` | Ask for an instruction for the captured insertion point. |
| `M-x ai-apply-result`, `:ai-apply-result` | `ai-apply-result` | Apply the selected proposal once to its original checked target. |
| `M-x ai-discard-result`, `:ai-discard-result` | `ai-discard-result` | Discard the pending result without editing source. |
| `M-x ai-set-directory`, `:ai-set-directory` | `ai-set-directory` | Choose a session-local AI context root. |
| `M-x ai-select-model`, `:ai-select-model` | `ai-select-model` | Choose a Pi-reported provider/model for this editor session. |
| `M-x ai-reset-session`, `:ai-reset-session` | `ai-reset-session` | Start fresh owned history on the next explicit invocation. |
| `M-x ai-disconnect`, `:ai-disconnect` | `ai-disconnect` | Stop the owned child and retain private history/results. |
| `M-x ai-cancel`, `:ai-cancel` | `ai-cancel` | Cancel the active invocation or AI-owned input/result. |
| `M-x ai-save-command`, `:ai-save-command` | `ai-save-command` | Save the successful literal instruction as a reusable command. |
| `M-!` (non-modal), `M-x shell-command`, `:shell-command` | `shell-command` | Prompt for an explicit one-shot shell command with null stdin and bounded retained output. |
| `M-\|` (non-modal), `M-x shell-command-on-region`, `:shell-command-on-region` | `shell-command-on-region` | Send the captured nonempty linear region as bounded stdin data and retain complete output without changing source. |
| `C-u M-\|` (non-modal), `M-x shell-filter-region`, `:shell-filter-region` | `shell-filter-region` | Replace the captured nonempty linear region with complete successful stdout as one checked edit. |
| `C-u M-!` (non-modal), `M-x shell-command-insert`, `:shell-command-insert` | `shell-command-insert` | Insert complete successful stdout at the point captured before the prompt, with null stdin. |
| `M-x shell-command-cancel`, `:shell-command-cancel` | `shell-command-cancel` | Cancel the owned invocation; originating editing C-g also cancels after inner input owners. |
| `M-x shell-command-output`, `:shell-command-output` | `shell-command-output` | Visit the last retained read-only plain-text result or diagnostics. |
| Return in History | `git-show-range` | Compare the marked exact base and selected endpoint in their explicit order. |
| `a` in History | `git-sequence-mark` | Append an exact ordered revision without changing the range base. |
| `U` in History | `git-sequence-unmark` | Remove an ordered revision, retaining remaining order and the base. |
| `l r` in Status | `git-reflog` | Inspect bounded exact HEAD reflog entries. |
| `B b`, `B B` in Status | `git-blame` | Attribute the selected or visiting source at checked HEAD. |
| `h`, `l l` in Status | `git-log` | Open exact bounded ancestry; detached HEAD collects an explicit root. |
| Tab in History; `M-x git-show`, `:git-show` | `git-show` | Inspect full revision metadata, changed paths and exact parent comparison. |
| `f` in History | `git-log-next-page` | Continue exact ancestry from its pinned root. |
| `M-x git-commit-amend` | `git-commit-amend` | Prepare exact observed HEAD composition, then explicitly review the replacement commit. |
| `M-x git-commit-fixup` | `git-commit-fixup` | Resolve one exact target, compose its fixup body and review before submission. |
| `c` in Git Status | `git-commit` | Open native commit text directly; a subsequent c is literal message text. |
| `C-Enter`, `C-c C-c` in composer | `git-commit-submit` | Commit the checked index with the accepted message. |
| `C-g`, `C-c C-k` in composer | `git-commit-cancel` | Retain the session draft and return to Status before process start. |
| `C-x g`, `M-x git-status`, `:git-status` | `git-status` | Discover the current repository and open full-canvas native Status. |
| `q` in Git | `git-quit` | Dismiss the innermost menu or return to the retained editing context; owned work continues. |
| `k`, `x d` in Git Status or Diff | `git-discard` | Confirm one tracked unstaged whole path from its index; protect dirty visiting buffers. |
| `*`, `x m` in Git Status or Diff | `git-mark` | Mark one exact current path or supported hunk independently of navigation. |
| `-`, `x u` in Git Status or Diff | `git-unmark` | Remove the exact selected path or hunk mark. |
| `0`, `x c` in Git Status or Diff | `git-unmark-all` | Clear all retained path and hunk marks. |
| `s` in Git Status or Diff | `git-stage` | Stage the exact selected path, supported text hunk or complete compatible marked group; unsaved source text is retained. |
| `u` in Git Status or Diff | `git-unstage` | Unstage the exact selected staged path, supported text hunk or complete compatible marked group without changing working files. |
| `g` in Git | `git-refresh` | Request a replacement observation; retain confirmed Status on failure. |
| Tab in Git | `git-toggle-section` | Collapse or expand the selected Status group, path comparison or semantic hunk. |
| `?` in Git | `git-dispatch` | Show the available native Git actions. |
| `$` in Git, `M-x git-processes`, `:git-processes` | `git-processes` | Inspect active and recent owned operations without cancelling them. |
| Return in Status or Diff | `git-visit-file` | Visit the canonical current source, preserving an existing modified buffer. |
| `M-x git-process-show`, `:git-process-show`; Return in Processes | `git-process-show` | Inspect the selected operation; the Return binding dispatches through git-visit-file. |
| `M-x git-diff`, `:git-diff` | `git-diff` | Inspect the exact selected comparison as read-only detail. |
| `C-g` in Processes or process detail, `M-x git-process-cancel` | `git-process-cancel` | Cancel the selected live operation through its exact identity; menus and prompts keep first refusal. |
| `M-x terminal-open`, `:terminal-open` | `terminal-open` | Open or select the instance default interactive terminal in the lower panel. |
| `M-x terminal-toggle`, `:terminal-toggle` | `terminal-toggle` | Hide or restore the lower terminal panel without restarting the child. |
| `M-x terminal-new-buffer`, `:terminal-new-buffer` | `terminal-new-buffer` | Create an independent terminal directly in the selected editing pane. |
| `M-x terminal-move-to-buffer`, `:terminal-move-to-buffer` | `terminal-move-to-buffer` | Move the current terminal into the selected editing pane. |
| `M-x terminal-move-to-panel`, `:terminal-move-to-panel` | `terminal-move-to-panel` | Dock the same session in the lower panel. |
| `M-x terminal-new`, `:terminal-new` | `terminal-new` | Create an independent lower-panel session. |
| `M-x terminal-list`, `:terminal-list` | `terminal-list` | Select or inspect retained terminal sessions. |
| `M-x terminal-rename`, `:terminal-rename` | `terminal-rename` | Give the selected terminal buffer a unique name without changing its identity. |
| `M-x terminal-open-here`, `:terminal-open-here` | `terminal-open-here` | Create a session from the current resource's directory context. |
| `M-x terminal-open-other-pane`, `:terminal-open-other-pane` | `terminal-open-other-pane` | Select a terminal in the next pane, creating a right split when needed. |
| `C-q` in live terminal input, `M-x terminal-send-next-key` | `terminal-send-next-key` | Send one following normalized key literally to the child, bypassing host recovery. |
| `C-c C-t` in live terminal input, `M-x terminal-copy-mode` | `terminal-copy-mode` | Freeze retained output for native navigation, selection and search. |
| Return in terminal copy mode, `M-x terminal-copy-mode-done` | `terminal-copy-mode-done` | Resume live input without sending Return to the child. |
| C-y in live input, M-x terminal-yank | `terminal-yank` | Paste the newest retained text using active bracket framing. |
| M-y in live input, M-x terminal-yank-pop | `terminal-yank-pop` | Rotate only while the preceding paste remains wholly queued. |
| M-x terminal-clear | `terminal-clear` | Send C-l through ordinary child input. |
| M-x terminal-clear-scrollback | `terminal-clear-scrollback` | Clear retained history and copy projections, preserving live screen/input. |
| M-x terminal-beginning-of-line | `terminal-beginning-of-line` | Move to the frozen copy line start. |
| M-x terminal-end-of-line | `terminal-end-of-line` | Move to the frozen copy line end. |
| M-x terminal-reset-cursor-point | `terminal-reset-cursor-point` | Align copy point with a still-represented live cursor. |
| M-x terminal-undo | `terminal-undo` | Send C-_ to the child without source undo. |
| M-x terminal-send-return | `terminal-send-return` | Send Return with the live mode-aware VT encoder. |
| M-x terminal-send-tab | `terminal-send-tab` | Send Tab with the live mode-aware VT encoder. |
| M-x terminal-send-escape | `terminal-send-escape` | Send Escape with the live mode-aware VT encoder. |
| M-x terminal-send-backspace | `terminal-send-backspace` | Send Backspace with the live mode-aware VT encoder. |
| M-x terminal-send-delete | `terminal-send-delete` | Send Delete with the live mode-aware VT encoder. |
| M-x terminal-send-up | `terminal-send-up` | Send Up with the live mode-aware VT encoder. |
| M-x terminal-send-down | `terminal-send-down` | Send Down with the live mode-aware VT encoder. |
| M-x terminal-send-left | `terminal-send-left` | Send Left with the live mode-aware VT encoder. |
| M-x terminal-send-right | `terminal-send-right` | Send Right with the live mode-aware VT encoder. |
| M-x terminal-send-prior | `terminal-send-prior` | Send PageUp with the live mode-aware VT encoder. |
| M-x terminal-send-next | `terminal-send-next` | Send PageDown with the live mode-aware VT encoder. |
| M-x terminal-send-space | `terminal-send-space` | Send Space with the live mode-aware VT encoder. |
| M-x terminal-next-prompt | `terminal-next-prompt` | Navigate the next captured OSC 133 prompt. |
| M-x terminal-previous-prompt | `terminal-previous-prompt` | Navigate the previous captured OSC 133 prompt. |
| M-x terminal-next-error | `terminal-next-error` | Visit an actually recognized retained error location. |
| M-x terminal-open-link | `terminal-open-link` | Explicitly activate the validated link at live or frozen point. |
| `C-x b` | `switch-to-buffer` | Select a live buffer by name. |
| `C-x C-b` | `list-buffers` | Select from the live buffer inventory. |
| `C-x k` | `kill-buffer` | Close a buffer with modified-text protection. |
| `C-x 2` | `split-window-below` | Split into upper/lower views. |
| `C-x 3` | `split-window-right` | Split into left/right views. |
| `C-x o` | `other-window` | Cycle editing views. |
| `C-x 0` | `delete-window` | Close the selected view, retaining its buffer. |
| `C-x 1` | `delete-other-windows` | Keep only the selected view. |
| `M-x windmove-left` | `windmove-left` | Focus the nearest overlapping pane to the left. |
| `M-x windmove-down` | `windmove-down` | Focus the nearest overlapping pane below. |
| `M-x windmove-up` | `windmove-up` | Focus the nearest overlapping pane above. |
| `M-x windmove-right` | `windmove-right` | Focus the nearest overlapping pane to the right. |
| `M-x balance-windows` | `balance-windows` | Equalize existing pane divisions, preserving focus and startup split policy. |
| Return | `newline` | Insert the buffer's preferred line separator. |
| Tab | `insert-tab` | Insert indentation; default spaces advance to the next four-column stop. |
| Backspace | `backward-delete-char` | Delete the preceding grapheme. |
| Delete, `C-d` | `delete-char` | Delete the following grapheme. |
| Right, `C-f` | `forward-char` | Move over one grapheme. |
| Left, `C-b` | `backward-char` | Move back one grapheme. |
| `M-f` | `forward-word` | Move to the next word end. |
| `M-b` | `backward-word` | Move to the preceding word start. |
| Home, `C-a` | `move-beginning-of-line` | Move to visual line start when wrapped, otherwise logical line start. |
| End, `C-e` | `move-end-of-line` | Move to visual line end when wrapped, otherwise logical line end. |
| `M-<` | `beginning-of-buffer` | Move to buffer start. |
| `M->` | `end-of-buffer` | Move to buffer end. |
| `M-g g`, `M-g M-g` | `goto-line` | Prompt for a one-based logical line; M-g alone is a prefix. |
| Down, `C-n` | `next-line` | Move down one visual row when wrapped, otherwise one logical row; retain the intended display column. |
| Up, `C-p` | `previous-line` | Move up one visual row when wrapped, otherwise one logical row; retain the intended display column. |
| Page Down, `C-v` | `scroll-up-command` | Advance one page. |
| Page Up, `M-v` | `scroll-down-command` | Move back one page. |
| `C-l` | `recenter-top-bottom` | Cycle point's line through center/top/bottom. |
| `C-x u`, reported `C-/` or `C-_` | `undo` | Undo a buffer-local group. |
| `M-x undo-redo` | `undo-redo` | Reapply the next redo group. |
| `M-x tabify` | `tabify` | Compress the marked whitespace at effective tab stops when shorter. |
| `M-x untabify` | `untabify` | Expand marked literal tabs while preserving text columns. |
| `M-x mouse-mode` | `mouse-mode` | Toggle native pointer support for this session without writing settings. |
| `M-x visual-line-mode` | `visual-line-mode` | Toggle word-aware wrapping for this buffer and all its views without changing source or settings. |
| `M-x gutter-toggle-line-numbers` | `gutter-toggle-line-numbers` | Hide or restore numbers, retaining mode. |
| `M-x gutter-cycle-line-number-mode` | `gutter-cycle-line-number-mode` | Alternate absolute and relative, retaining visibility. |
| `M-x gutter-set-line-numbers-absolute` | `gutter-set-line-numbers-absolute` | Show absolute logical line numbers. |
| `M-x gutter-set-line-numbers-relative` | `gutter-set-line-numbers-relative` | Show distances from this view's logical point. |
| `M-x gutter-cycle-current-line` | `gutter-cycle-current-line` | Cycle absolute, zero and hidden current labels. |
| `M-x gutter-toggle-line-number-side` | `gutter-toggle-line-number-side` | Move this view's number field left or right. |
| `M-x gutter-inspect-numbering` | `gutter-inspect-numbering` | Report effective policy and startup/view override sources. |
| `M-x gutter-reset-numbering` | `gutter-reset-numbering` | Restore this view's immutable startup policy. |
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
| `C-x r m` | `bookmark-set` | Name or replace the current file position. |
| `C-x r M` | `bookmark-set-no-overwrite` | Name the current position without replacement. |
| `C-x r b` | `bookmark-jump` | Complete a name and visit its checked location. |
| `M-x bookmark-delete` | `bookmark-delete` | Complete a name and remove that bookmark. |
| `C-x r l` | `list-bookmarks` | Open the read-only native bookmark list. |
| `d` in bookmark list | `bookmark-bmenu-delete` | Mark the selected record for deletion and advance. |
| `u` in bookmark list | `bookmark-bmenu-unmark` | Unmark the selected record and advance. |
| `x` in bookmark list | `bookmark-bmenu-execute-deletions` | Execute checked marked deletions. |
| `RET` in bookmark list | `bookmark-bmenu-this-window` | Visit the selected stable record in this pane. |
| `M-x bookmark-next` | `bookmark-next` | Visit the next resolved current-file record, wrapping and stepping same-position names. |
| `M-x bookmark-previous` | `bookmark-previous` | Visit the previous resolved current-file record in source/name order. |
| `M-x bookmark-markers-mode` | `bookmark-markers-mode` | Toggle marker visibility for the selected editing view. |
| `M-x bookmark-save` | `bookmark-save` | Checkpoint the user collection without saving sources. |
| `M-x bookmark-reload` | `bookmark-reload` | Validate disk and confirm replacing unsaved bookmarks. |
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

Global Org entry keys include `C-c a` (agenda), `C-c c` (capture) and
`C-c l` (store link). See [Org bindings](org/bindings.md) for those keys,
all Org commands and the source, draft, agenda and calendar contexts.

## Modal startup profile

Select `editor.interaction_profile = "modal"` and restart for Vim-like editing.
Modal editing starts in Normal; unmatched text and bracketed paste do not insert
there. Non-modal editing remains the default.
[Choose Modal Editing](../tutorials/user-guide/11-modal-editing.md) walks through
profile selection, ordinary edits, shapes and shared command entry.
The following state commands require the modal startup profile:

| Input | Command | Result |
| --- | --- | --- |
| `i` in Normal | `modal-enter-insert` | Enter Insert at point. |
| `R` in Normal | `modal-enter-replace` | Enter persistent Replace with an underline cursor. |
| `.` in Normal | `modal-repeat-last-change` | Resolve and repeat the latest admitted semantic change; a count overrides its count. |
| `a` in Normal | `modal-enter-insert-after` | Enter Insert after the current grapheme, stopping at logical EOL. |
| `I` in Normal | `modal-enter-insert-line-first` | Enter Insert at the first non-whitespace boundary. |
| `A` in Normal | `modal-enter-insert-line-end` | Enter Insert at logical EOL. |
| `o` in Normal | `modal-open-line-below` | Open one logical line below and enter Insert. |
| `O` in Normal | `modal-open-line-above` | Open one logical line above and enter Insert. |
| `v` in Normal or Visual | `modal-enter-visual-character` | Select the current grapheme, convert shape or leave a repeated Character state. |
| `V` in Normal or Visual | `modal-enter-visual-line` | Select complete logical lines, convert shape or leave a repeated Line state. |
| `C-v` in Normal or Visual | `modal-enter-visual-block` | Select inclusive logical grapheme columns, convert shape or leave a repeated Block state. |
| `C-o` in Insert | `modal-one-normal-command` | Admit one complete Normal command, then resume compatible insertion. |
| Escape or `C-[` in modal editing | `modal-enter-normal` | Return to Normal, preserving the native point. |

Insert and Replace use native editing and compatible modified factory bindings. Normal
retains M-x, C-g, Meta commands and C-x/C-c command prefixes. State changes
alone create no history; opening a line creates one ordinary undoable edit.
Local prompts, completion, tree and Org maps keep their input priority.
In a modal component, Escape or `C-[` invokes the same cancellation as `C-g`
after bounded Alt/Meta recognition; a second Escape cancels immediately.

### Replace and semantic repetition

R overwrites corresponding complete graphemes on the current logical line.
At EOL it inserts before the existing LF/CRLF separator; it never consumes that
separator. Input that joins the preceding grapheme extends it without consuming
another source grapheme. Escape/C-[ returns to Normal at the native point.
Connected Insert/Replace input and a preceding change or line opening share one
ordinary undo/redo group. Modified movement or another command ends that group.

Dot retains one semantic entry, operator, direct edit, indentation or paste.
Entries i/a/I/A/o/O/R resolve their entry destination again. Operators resolve
their motion/object against current source. An explicit dot count replaces the
recorded count; it does not multiply it. Inserted payload is expanded within the
native 64-MiB materialization bound before publication. Dot stays Normal and
creates one ordinary native transaction.

A completed record is usable in another compatible editing pane or buffer.
It checks current focus, source point, target and permissions before mutation.
A failed repeat preserves the record. Undo/redo and copying text do not replace
it. A retained paste keeps its admitted linear, linewise or logical-block payload,
so a later copy cannot change the repeated insertion. Visual repetition retains
its grapheme count, line count or block height/width, anchored at current point.
For a Visual record, an explicit count replaces the linear/line count or block
width; height remains fixed for a block.

At most 1,024 text bytes are retained, including the complete paste payload.
Overflow never rejects an otherwise valid edit or exposes a truncated replay;
it makes that change unavailable to dot. The native history budget still owns
undo data. An empty insertion entry keeps the prior record. External source/view
changes retire unfinished entries. Destructive entry controls that cannot be
represented as the retained insertion, source-bound accepted-search ranges and
unrelated native command workflows remain ordinary edits and retire repetition.
Prompts, file consent and provider work are never replayed.

### Commands, panes and completion

In Normal and all Visual shapes, `:` opens the same vertical selector as `M-x`.
Use full canonical names such as `:split-window-right`, then answer any native
argument prompt. Matching, history, completion, availability and dispatch are
shared. Colon supplies no Ex aliases, ranges, pipelines or trailing arguments.
Escape or C-g cancels the selector. Cancellation retains a compatible Visual
shape and native anchor; changed source,
point, mark or selected view retires that selection instead of restoring stale
positions. Insert, Replace and argument prompts keep literal colon/slash text.

The Normal/Visual pane prefix is `C-w`: h/j/k/l focus left/down/up/right, s/v split
below/right, c closes the selected pane, o keeps only that pane and = balances
existing divisions. Directional focus requires orthogonal overlap and chooses the
nearest pane, using native screen order for equal distances. No neighbor is a
no-op. Balancing reserves separators, shares same-axis pane extents and preserves
topology, focus, source and the startup ratio for later splits.

Automatic source suggestions run only in Insert/Replace. Manual
`completion-at-point` remains available through the shared catalog. Popup arrows,
C-p/C-n, Tab/C-i and Return retain native actions and colored kinds; source syntax
colors remain visible. Checked primary edits and imports form one undo operation.
Escape/C-g dismisses the popup locally before a later editing Escape returns to
Normal. Incompatible state/view/source transitions retire list and resolve work;
a late reply cannot reopen the popup or publish an edit. Insert C-o waits through
an asynchronous completion workflow before resuming compatible insertion.

Tree, search, argument/decision prompts, inventories, bookmarks and local Org
maps retain priority over editing grammar. Org source drafts use the immutable
session profile with native local save and abort controls. Cancelling their
command selector leaves the outer draft active. Bracketed paste remains literal
text in entry states and never starts automatic completion.

### Normal navigation

These bindings resolve one checked native destination. The same named commands
are available through M-x in either startup profile. Emacs factory keys keep
their existing visual-line and native word behavior.

| Input | Command | Destination |
| --- | --- | --- |
| `h`, `Left` | `modal-backward-char` | Previous whole grapheme, including a whole LF/CRLF boundary. |
| `l`, `Right` | `modal-forward-char` | Next whole grapheme, including a whole LF/CRLF boundary. |
| `j`, `Down` | `modal-next-line` | Next logical line; retain desired terminal-cell column and skip hidden Org bodies. |
| `k`, `Up` | `modal-previous-line` | Previous logical line; retain desired terminal-cell column and skip hidden Org bodies. |
| `0` | `modal-line-start` | Logical line start. |
| `^` | `modal-line-first-text` | First non-whitespace grapheme of the logical line. |
| `$` | `modal-line-end` | Logical content-end boundary before LF/CRLF. A count selects the corresponding following logical line. |
| `w` | `modal-next-word-start` | Start of the next word or punctuation run, skipping whitespace. |
| `b` | `modal-previous-word-start` | Previous Unicode word start, skipping punctuation. |
| `e` | `modal-next-word-end` | Next Unicode word's exclusive end boundary, skipping punctuation. |
| `W` | `modal-next-big-word-start` | Next start of a whitespace-separated WORD. |
| `B` | `modal-previous-big-word-start` | Previous start of a whitespace-separated WORD. |
| `E` | `modal-next-big-word-end` | Exclusive end of the next whitespace-separated WORD. |
| `g e` | `modal-previous-word-end` | Exclusive end of the preceding Unicode word, excluding the run containing the point's predecessor. |
| `g E` | `modal-previous-big-word-end` | Exclusive end of the preceding WORD, excluding the run containing the point's predecessor. |
| `g g` | `modal-buffer-start` | Start of buffer; with a count, start of that one-based logical line. |
| `G` | `modal-buffer-end` | End of buffer; with a count, start of that one-based logical line. |
| `f` | `modal-find-forward` | Counted next occurrence of one Unicode scalar within the logical line; land on its grapheme start. |
| `F` | `modal-find-backward` | Counted previous occurrence of one Unicode scalar within the logical line; land on its grapheme start. |
| `t` | `modal-till-forward` | Grapheme immediately before the counted next occurrence. |
| `T` | `modal-till-backward` | Grapheme immediately after the counted previous occurrence. |
| `;` | `modal-repeat-find` | Repeat the last successful find in its original direction; till skips its prior match. |
| `,` | `modal-reverse-find` | Repeat the last successful find in the opposite direction; retain the original repeat direction. |
| `{` | `modal-previous-paragraph` | Previous empty/whitespace-only logical-line boundary, clamped to BOF. |
| `}` | `modal-next-paragraph` | Next empty/whitespace-only logical-line boundary, clamped to EOF. |
| `%` | `modal-matching-delimiter` | Match the first parenthesis, bracket or brace at/after point on this logical line; respect nested pairs of the same kind. Missing mates preserve point. |
| `PageDown`, `C-f` | `modal-page-forward` | Full-page forward through native geometry and page overlap. |
| `PageUp`, `C-b` | `modal-page-backward` | Full-page backward through native geometry and page overlap. |
| `C-d` | `modal-half-page-forward` | Half-page forward. |
| `C-u` | `modal-half-page-backward` | Half-page backward; a local Org prefix beginning with C-u owns that key in Org editing. |
| `H` | `modal-screen-top` | Top displayed text row; count inward from the top. |
| `M` | `modal-screen-middle` | Middle displayed text row, including wrapped fragments and folds. |
| `L` | `modal-screen-bottom` | Bottom displayed text row; count inward from the bottom. |
| `z t` | `modal-align-top` | Align point at the viewport top without moving it. |
| `z z` | `modal-align-middle` | Align point at the viewport middle without moving it. |
| `z b` | `modal-align-bottom` | Align point at the viewport bottom without moving it. |
| `/` | `modal-search-forward` | Open the shared forward literal search prompt. |
| `?` | `modal-search-backward` | Open the shared backward literal search prompt. |
| `n` | `modal-search-repeat` | Continue the confirmed search in its original direction. |
| `N` | `modal-search-reverse` | Continue the confirmed search in the opposite direction. |
| `*` | `modal-search-word-forward` | Search forward for the Unicode word at point with complete word boundaries. |
| `#` | `modal-search-word-backward` | Search backward for the Unicode word at point with complete word boundaries. |

The shared h/l, logical line, 0/^/$, w/b/e and gg/G endpoints follow the inspected
Odra behavior. In particular e is an insertion boundary after the word. The
WORD, ge/gE and counted forms above are additional rev requirements. `w` on
`one 🙂two` reaches the emoji; `e` on `one !!! two` reaches the boundary after
`one`, then after `two` on its next invocation.

A decimal count starts with 1–9; following zeroes extend it, while standalone
0 remains logical start. The ceiling is 4,294,967,295 and pending grammar is
limited to 16 strokes. Targets retain one Unicode scalar (at most four UTF-8
bytes, within the 96-byte target bound). Counts scan the finite source or
resolve a projected destination; they do not repeat UI commands. Overflow,
missing counted finds/searches and stale observations preserve point and text.
Idempotent start/first-text, delimiter, middle-row and alignment targets retain
their single-target meaning with a count. Counts on unrelated commands reject.
Explicit local digit mappings own the start of a command; digits following an admitted count extend that count. M-x cancels pending syntax and opens native command entry.

Escape/C-[, C-g, paste or a changed view/buffer/revision/point retire pending
syntax. A retained find or search continuation is qualified by its native view
and source revision. Search keeps literal smart-case matching, no wrapping and
its native endpoint policy: forward lands after the match, backward at its
start. C-g during search restores the complete origin view. A confirmed native
C-s/C-r search also supplies n/N; prompts retain their own key meanings.

### Normal operators and transfer

| Input | Command | Result |
| --- | --- | --- |
| `d` | `modal-delete` | Delete one checked target into the shared kill ring. |
| `c` | `modal-change` | Retain/delete the target, then enter Insert after successful publication. |
| `y` | `modal-copy` | Copy the target to the shared kill ring without editing or moving point. |
| `>` | `modal-indent` | Indent each intersecting logical line by one active indentation unit. |
| `<` | `modal-outdent` | Outdent each intersecting logical line by one active indentation unit. |
| `D` | `modal-delete-line-end` | Delete through the logical content-end boundary; a count selects following lines. |
| `C` | `modal-change-line-end` | Change through the logical content-end boundary, then enter Insert. |
| `Y` | `modal-copy-line-end` | Copy through the logical content-end boundary, retaining linear shape. |
| `x` | `modal-delete-forward` | Delete counted whole graphemes within this logical line, preserving retained text. |
| `X` | `modal-delete-backward` | Delete counted preceding whole graphemes within this logical line, preserving retained text. |
| `r` | `modal-replace-character` | Replace counted complete graphemes within this logical line with one printable Unicode scalar. |
| `p` | `modal-paste-after` | Paste retained text after the current grapheme, or after this logical line for linewise text. |
| `P` | `modal-paste-before` | Paste retained text at point, or before this logical line for linewise text. |
| `J` | `modal-join-lines` | Join at least two logical lines (count selects total lines), stripping leading indentation and inserting needed spaces. |

`u` and `C-r` invoke ordinary undo and redo. Prefix commands d/c/y/>/< require
modal Normal; completed semantic edit requests and standalone edit commands use
the same native range/edit owners. Pending operators never move point or edit.
Escape/C-[, C-g, M-x, paste or changed native source/view/point retire them.

A count before an operator multiplies a count before its target, using checked
u32 arithmetic. `2d3w` resolves six word starts and publishes one deletion/undo.
`dd`, `cc`, `yy`, `>>` and `<<` select counted complete logical lines. A line
change preserves an empty logical insertion slot; changing an empty valid
interior can enter Insert without an edit. Failed or read-only changes stay in
their prior state and preserve retained text. Copy remains usable read-only.

Character and word operator ranges use the native half-open interval between
origin and destination. Thus `dw` on `one two` removes `one `, while `de`
removes `one` and preserves its following separator. Shared f/F/t/T endpoints
also retain this Odra range policy: `dfX` on `abcXdef` retains `abc` and leaves
`Xdef`; the matched target is not implicitly added. Logical line/file/page,
visible-row and paragraph motions select complete intersecting source lines.
Missing counted finds or absent objects publish nothing. Alignment commands do
not select an operator range.

Text objects are `iw`/`aw`, `iW`/`aW`, `ip`/`ap`, and inner/around parentheses,
brackets, braces, single/double quotes and backticks. Both opening and closing
pair spellings are aliases. Word objects select Unicode word/punctuation runs;
WORD objects use whitespace-separated runs. Around word includes following
whitespace, falling back to preceding whitespace at the end. Counts include
following runs. Paragraphs select connected logical lines; around includes
adjacent blank lines. Pair counts select successive enclosing balanced pairs;
quote counts extend through successive quoted pairs on the logical line.
Backslash escaping is respected without introducing a language parser.

Operator / and ? use the existing search prompt while point remains at its
origin. Return applies the complete native search endpoint once; C-g cancels.
Confirmed n/N and */# targets reuse the shared literal matcher and its existing
smart-case, nonwrapping policy.

Linewise intent lives in the existing bounded kill ring. `yy`, linewise delete
and change retain complete logical text, including its actual terminators.
p/P normalize a linewise entry to the destination newline convention and add
only necessary separators at EOF. Linear paste retains the native yank endpoint
after inserted text; linewise paste places point at the inserted line start.
Native yank/yank-pop and modal paste share entries; no modal register is added.
Compatible consecutive kills of the same shape follow `kill_ring.coalesce`; a
changed shape starts a fresh entry.
Small x/X deletion and r replacement preserve the previous entry. Repeated
paste/replacement composes one prepared edit; count-expanded text is bounded to
64 MiB before allocation. An ordinary single transfer keeps native admission.
Indentation and multi-line joining are one prepared batch and one undo unit.

### Jump history and Insert continuation

| Input in Normal | Command | Result |
| --- | --- | --- |
| `C-o` | `modal-jump-backward` | Revisit an older checked location, with an optional count. |
| `C-i` or Tab | `modal-jump-forward` | Revisit a newer checked location, with an optional count. |

Each view retains at most 256 session-only locations. A split copies that bounded
history and traversal index, then each view branches independently. Closing a view
releases its handles. Explicit buffer/line, paragraph and page/screen destinations,
accepted literal searches and native file visits record qualified origins and
destinations. Ordinary h/j/k/l, word steps, search previews, drawing and jump
traversal do not add entries. Counts clamp by retained entries, without replaying
primitive commands. Starting backward traversal refreshes the newest current point;
older entries keep their rebased locations. A new admitted jump after moving back
removes that view's forward branch. Failed navigation/traversal changes no index.

Live locations follow native prepared edits, structural permutations and undo/redo.
Deleting an anchor or joining away its grapheme boundary makes it unavailable;
native undo can restore recorded outcomes. Traversal reuses an existing canonical
buffer, including its unsaved text and read-only state. A closed file can reopen
only when native file identity/content/permission observations, the exact retained
position and up to 64 source scalars on each side still agree. Missing, replaced,
changed, stale or non-file closed targets reject without guessing another position.
Canonical rename carries closed paths, and save/close retains the live file identity.
Paths have the native 4096-byte limit. Jump state never reads/writes bookmark names,
collections or persistence; a later linear/selection edit remains ordinary history.

In Insert, `C-o` enters Normal for one semantic command. Counts, pending operators,
find characters, command selection and argument/search prompts belong to that one
lifetime. Compatible completion returns to Insert at the checked native point,
including after an admitted edit. A different view/buffer or unrelated source
revision retires insertion authority; explicit Visual entry keeps its new state.
Queued input after deferred layout is interpreted after the command's state
transition. Native prompts/completion keep their own Tab/C-i action; non-modal bindings
and ordinary Insert Tab remain unchanged.

### Visual selections

Visual states use the buffer's shared mark and the selected view's native point.
Character includes the destination grapheme, including a whole LF/CRLF cluster;
EOF itself contributes no extra text. Line includes complete logical source
lines between both endpoints. Normal motions, counts, finds and shared literal
search extend these selections. `o` uses exchange-point-and-mark to move the
active end; repeated shape entry, Escape/C-[ or C-g cancels presentation.

`d`, `c` and `y` apply immediately to the selected character, line or block range.
Delete and copy return to Normal; change enters Insert after successful native
admission. All affected rows publish as one edit and one ordinary undo/redo step.
Read-only or rejected targets preserve selection, prior transfer and history.
Linear Visual `>`/`<` indents/outdents every intersecting logical line and returns
to Normal. Text object entry is deliberately limited to linear Visual `iw`, `aw`,
`ip` and `ap`; selecting an object sets Character endpoints around that object.

| Input in linear Visual | Command | Result |
| --- | --- | --- |
| `i` then `w` or `p` | `modal-select-inner-object` | Select a counted inner word or paragraph through native endpoints. |
| `a` then `w` or `p` | `modal-select-around-object` | Select a counted word or paragraph with its surrounding whitespace. |

Visual Block counts each source grapheme once: a tab, wide character or joined
cluster occupies one logical column. Both endpoint columns are included. Existing
short rows contribute only their actual text, and line separators are excluded.
Drawing projects the selected glyphs back to their actual display cells, including
the full tab stop. non-modal rectangles retain display-cell edges and partial-tab
padding. See [rectangle coordinate and transfer policies](rectangles.md).

Block transfer keeps native rectangle rows and logical-column intent in the
shared transfer entry; its linear text view joins those rows with LF. `p`/`P`
reinsert the block across logical rows, padding short destinations on the left
and preserving LF/CRLF when extending EOF. Counts repeat each retained row
horizontally within the existing 64-MiB expanded-text bound. Native yank-rectangle
can use the same retained block. A later linear kill/copy selects linear paste
again; native yank/yank-pop retain their linear span policy.

A different view of the same unchanged buffer can retain the shared selection.
Switching buffers or publishing a source revision retires Visual authority.
Pointer drag replaces the shared anchor while retaining the active shape;
folding, wrapping and resize affect projection without making a separate range.

For live terminal key ownership and copy controls, see the
[integrated terminal reference](integrated-terminal.md#live-input-and-mouse).

### Native refs bindings

| Keys | Command | Behavior |
| --- | --- | --- |
| `b`, `y` in Status | `git-refs` | Inspect exact native repository refs. |
| `n` in Refs | `git-branch-create` | Confirm a new local branch at the selected full commit. |
| `S` in Refs | `git-branch-switch` | Confirm a checked clean local branch switch. |
| `R` in Refs | `git-branch-rename` | Confirm the exact local branch and new name. |
| `D` in Refs | `git-branch-delete` | Confirm non-forced local deletion. |
| `t` in Refs | `git-tag-create` | Confirm a local lightweight tag or explicitly compose an annotated tag. |
| `T` in Refs | `git-tag-delete` | Confirm deletion of the exact local tag object. |

## Configuration keymap contracts

The [complete reference sample](config.toml) spells out every default target in
all supported contexts, including local overlays and the state maps. Copy only desired overrides to preserve inherited stage
distinctions; explicit mappings select their named action at the active owner.

`keys.editor` belongs to non-modal editing. Modal editing resolves `keys.vim_normal` or
`keys.vim_insert` or `keys.vim_replace` over independent native factories;
non-modal overrides do not leak into those states. Local Org editing/capture/source maps overlay the selected state.
`keys.vim_visual_character`, `keys.vim_visual_line` and `keys.vim_visual_block`
resolve independent state factories. M-x and C-g remain protected recovery in modal editing.
Escape/C-[ is a state transition while modal editing owns input, so a standalone
Escape cannot become a configurable command prefix there.

`keys.completion` supplies the active popup map; Escape remains literal cancel.
`keys.editor`, `keys.minibuffer`, `keys.tree` and `keys.git` inherit independent complete
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
`keys.minibuffer`, `keys.tree` and `keys.git` through their native Rust action owners.
The focused prompt temporarily owns the minibuffer map even when an Org role or
the tree retains workspace focus. Tree focus takes precedence over Org editing.
No keymap edits the underlying document from a prompt.

Org context inheritance and role availability are described in
[Org keymap contracts](org/bindings.md#configuration-keymap-contracts).

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

The editor inventory above and the [Org inventory](org/bindings.md) together
list every compiled command, including global Org entry keys, with the aliases above.
Editor targets are the compiled commands and admitted `ai-<name>` definitions used by M-x. A user
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
through the active owner: M-x records its command history, find-file opens the
selected or typed file, descends into directories or retains failed drafts, and
search accepts its current result. Completion aliases
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
| `C-x C-f` | `find-file` |
| `C-x g` | `git-status` |
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

Tree configuration admits the tree command catalog, Git Status, find-file, cancel, exit, M-x and
return-to-editing, plus the existing navigation aliases `previous-line`,
`next-line`, `backward-char`, `forward-char` and `newline`. File mutation commands
retain their prompts and explicit confirmation; a binding cannot supply consent.

Supplied tree bindings dispatch only while the tree owns workspace focus and no
prompt is active. Exact reassignment preserves independent inherited aliases:
changing `C-n` does not change Down. Navigation aliases can repeat; open, refresh,
mutation and workspace transitions require fresh presses. Pending prefixes are
cleared by prompt acquisition, cancellation and workspace changes. Tree text
and paste never enter the visiting buffer while tree focus is retained.

### Git commit defaults and availability

`keys.git_commit` inherits editor text editing in both startup profiles. Normal
composition begins with c in Status. Plain Return inserts a newline; submission
requires C-Enter (a distinguishable host event) or C-c C-c. C-g and C-c C-k
retain the cancelled session draft. Composer text is separate from source
buffers and filesystem publication.

| Keys | Target |
| --- | --- |
| `C-Enter`, `C-c C-c` | `git-commit-submit` |
| `C-g` | `keyboard-quit` |
| `C-c C-k` | `git-commit-cancel` |

### Git history defaults and availability

`keys.git_history` owns History input in both profiles.
Prompts still use `keys.minibuffer`. Pointer selection requires the currently
painted observation and viewport. q returns from revision detail to History,
then Status, preserving selection and source ownership.

| Keys | Target |
| --- | --- |
| `C-x g` | `git-status` |
| `q` | `git-quit` |
| `g` | `git-refresh` |
| Tab | `git-show` |
| Return | `git-show-range` |
| `*` | `git-mark` |
| `a`, `U` | `git-sequence-mark`, `git-sequence-unmark` |
| `f` | `git-log-next-page` |
| `A`, `F` | `git-commit-amend`, `git-commit-fixup` |
| `$` | `git-processes` |
| `M-x`, `:` | `execute-extended-command` |
| `C-g` | `keyboard-quit` |
| `C-x C-c` | `save-buffers-kill-terminal` |
| Up, `C-p`, `p` | `previous-line` |
| Down, `C-n`, `n` | `next-line` |
| PageUp, PageDown | `scroll-down-command`, `scroll-up-command` |
| `M-<`, `M->` | `beginning-of-buffer`, `end-of-buffer` |

### Git inspection defaults and availability

`keys.git_inspect` owns revision, reflog, blame and historical-file navigation
in both profiles. Return opens the selected exact parent, revision or historical
blob. v inspects a full revision; it never treats a reflog selector as an argument.
`git-visit-file` visits current source through its canonical native owner.

| Keys | Target |
| --- | --- |
| `C-x g` | `git-status` |
| `q` | `git-quit` |
| `g` | `git-refresh` |
| Tab, Return | `git-toggle-section` |
| `v` | `git-show` |
| `$` | `git-processes` |
| `M-x`, `:` | `execute-extended-command` |
| `C-g` | `keyboard-quit` |
| `C-x C-c` | `save-buffers-kill-terminal` |
| Up, `C-p`, `p` | `previous-line` |
| Down, `C-n`, `n` | `next-line` |
| PageUp, PageDown | `scroll-down-command`, `scroll-up-command` |
| `M-<`, `M->` | `beginning-of-buffer`, `end-of-buffer` |

### Git defaults and availability

`keys.git` applies to native Git navigation in both startup profiles. It keeps
source text read-only and retains the original windows, tree and terminal
owners. User mappings replace individual chords; `C-g` remains reserved for
native cancellation. Prompts use `keys.minibuffer`.

| Keys | Target |
| --- | --- |
| `C-x g` | `git-status` |
| `C-x C-c` | `save-buffers-kill-terminal` |
| `M-x`, `:` | `execute-extended-command` |
| `C-g` | `keyboard-quit` |
| `C-Enter`, `C-c C-c` during exact commit review | `git-commit-submit` |
| `c` | `git-commit` |
| `b`, `y` | `git-refs` |
| `h`, `l l` | `git-log` |
| `l r` | `git-reflog` |
| `B b`, `B B` | `git-blame` |
| `q` | `git-quit` |
| `g` | `git-refresh` |
| `k`, `x d` | `git-discard` |
| `*`, `x m` | `git-mark` |
| `-`, `x u` | `git-unmark` |
| `0`, `x c` | `git-unmark-all` |
| `s` | `git-stage` |
| `u` | `git-unstage` |
| Tab | `git-toggle-section` |
| `?` | `git-dispatch` |
| `$` | `git-processes` |
| Return | `git-visit-file` |
| Up, `C-p`, `p` | `previous-line` |
| Down, `C-n`, `n` | `next-line` |

k or x d restores one tracked unstaged whole path from its captured index entry
after y/n confirmation; C-g cancels and Return does not imply yes. Modified
visiting buffers, including canonical aliases, reject the request. Changing the
file or source context invalidates an open confirmation. Hunk and marked-group
discard, untracked deletion and gitlink replacement are unsupported. Completed
file changes use the existing automatic/ask reload policy; the command never
saves or directly replaces a source buffer.

Marks retain complete comparison identities independently of navigation. Use *
or x m to mark, - or x u to remove a selected mark, and 0 or x c to clear all
marks. At most 256 targets are retained within shared Git memory admission.
When marks exist, s/u checks the entire group: whole paths must support one
direction; text hunks must be nonoverlapping units from one exact path/layer
comparison. Mixed, duplicate, stale or unsupported groups fail before changing
the index. Refresh does not retarget old marks; clear or unmark them explicitly.

Only nonempty groups appear, in Unmerged, Untracked, Unstaged and Staged order.
Selection follows an exact path and group across refresh. Mouse selection
requires the current painted generation, comparison, expansion and dimensions.
Tab toggles a group, path comparison or hunk body; Return visits the current
source, or inspects the selected operation in Processes. Named `git-diff`
opens read-only patch detail. q returns one presentation level;
`q`, `C-g` or Escape dismisses its action menu. Returning from Status keeps
started Git work owned by the editor.

### Org agenda defaults and availability

See [Org agenda defaults and availability](org/bindings.md#org-agenda-defaults-and-availability).

### Org calendar defaults and availability

See [Org calendar defaults and availability](org/bindings.md#org-calendar-defaults-and-availability).

### Native mode selection

See [Org native mode selection](org/bindings.md#native-mode-selection).

### Org date entry and calendar

See [Org date entry and calendar](org/bindings.md#org-date-entry-and-calendar).

### Org timestamp context actions

See [Org timestamp context actions](org/bindings.md#org-timestamp-context-actions).

### Native task clocks

See [Org native task clocks](org/bindings.md#native-task-clocks).

### Clock totals and reports

See [Org clock totals and reports](org/bindings.md#clock-totals-and-reports).

### Session timers

See [Org session timers](org/bindings.md#session-timers).

### Org agenda source discovery

See [Org agenda source discovery](org/bindings.md#org-agenda-source-discovery).

### Native day and week calendars

See [Org native day and week calendars](org/bindings.md#native-day-and-week-calendars).

The read-only bookmark workspace uses `keys.bookmark_list`: arrows, C-n/C-p and
n/p move rows; Return visits; d/u mark or unmark and advance; x deletes marked
records; q/C-g returns and discards unexecuted marks. M-x opens the normal command
prompt and C-x o changes panes. These local keys do not edit source text.

Amend and fixup use the native command catalog. Their first explicit composer
submission opens exact review; a second C-Enter or C-c C-c accepts. q or C-g
in review retains the draft.

### Git refs defaults and availability

`keys.git_refs` owns read-only refs and reviewed local branch/tag actions in both
profiles. Prompts retain `keys.minibuffer`. n creates a branch; p is reserved
for push. Up/Down and C-p/C-n navigate. Annotated tag text uses the retained
native message owner and `keys.git_commit`; its review uses the explicit submit
aliases below.

| Keys | Target |
| --- | --- |
| `TAB` | `git-toggle-section` |
| `t` | `git-tag-create` |
| `T` | `git-tag-delete` |
| `C-Enter` | `git-commit-submit` |
| `C-c C-c` | `git-commit-submit` |
| `C-x g` | `git-status` |
| `q` | `git-quit` |
| `g` | `git-refresh` |
| `n` | `git-branch-create` |
| `S` | `git-branch-switch` |
| `R` | `git-branch-rename` |
| `D` | `git-branch-delete` |
| `?` | `git-dispatch` |
| `$` | `git-processes` |
| `M-x` | `execute-extended-command` |
| `:` | `execute-extended-command` |
| `C-g` | `keyboard-quit` |
| `C-x C-c` | `save-buffers-kill-terminal` |
| `Up` | `previous-line` |
| `Down` | `next-line` |
| `C-p` | `previous-line` |
| `C-n` | `next-line` |
| `PageUp` | `scroll-down-command` |
| `PageDown` | `scroll-up-command` |
| `M-<` | `beginning-of-buffer` |
| `M->` | `end-of-buffer` |
