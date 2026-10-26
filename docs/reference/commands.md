# Commands

`M-x` opens the command prompt. Reported Alt/Meta and compatible Escape input
follow the [terminal input policy](terminal-input.md). Command entry uses the
same fixed command registry and handlers as keyboard bindings.

## Matching and invocation

- Type a lowercase hyphenated name or part of it. Matching is case-sensitive:
  prefixes come first, followed by other substrings; each group is alphabetical.
  Up to three candidates appear beside the command text within available width.
- Tab completes the first candidate. Further Tabs cycle that original matching
  set and wrap. Editing the query starts a new matching set.
- Return invokes the exact name when present, otherwise the first matching
  candidate. An empty query invokes nothing. Unknown names report `Unknown command`
  and return to editing without changing document text. Tab with no matches keeps
  command focus and reports `No matching command`.
- Up or C-p recalls earlier commands; Down or C-n moves toward newer commands
  and then restores the draft query. Navigation stops at the oldest entry. The
  session retains up to 64 distinct accepted command names, most recent first.
  Accepting a name records the invocation even if its handler later refuses it.
  Canceled or unknown input does not enter history; history is not persisted.
- C-g cancels command entry. Text, cursor edits and single-line paste remain in
  the prompt. The existing 4096-character prompt bound and control rejection apply.
  Releases and repeated Return do not invoke a command twice.

Argument-taking commands continue with their existing path, buffer or decision
prompts. Named edits use the same read-only admission and history as bindings.
Opening and editing M-x is transient input, so a named `yank-pop` can still use
the current yank range; cancellation or an unrelated invoked command ends it.
Names are literal registry entries, with no expression or language interpretation.

See [Focused prompts](prompts.md) for argument stages, in-prompt editing and
save/close cancellation.

`isearch-forward` and `isearch-backward` open [incremental search](search.md)
with the same handlers as C-s and C-r.
`query-replace` opens the same literal replacement prompts as M-% and requires
an explicit decision for each occurrence.

## Implemented catalog

| Group | Names |
| --- | --- |
| Command and session | `execute-extended-command`, `keyboard-quit`, `kill-terminal` |
| Files | `find-file`, `save-buffer`, `write-file` |
| Buffers | `switch-to-buffer`, `list-buffers`, `kill-buffer` |
| Windows | `split-window-below`, `split-window-right`, `other-window`, `delete-window`, `delete-other-windows` |
| Editing | `newline`, `insert-tab`, `backward-delete-char`, `delete-char` |
| Motion | `forward-char`, `backward-char`, `forward-word`, `backward-word`, `move-beginning-of-line`, `move-end-of-line`, `beginning-of-buffer`, `end-of-buffer` |
| Scrolling | `next-line`, `previous-line`, `scroll-up-command`, `scroll-down-command`, `recenter-top-bottom` |
| History | `undo`, `undo-redo` |
| Search | `isearch-forward`, `isearch-backward`, `query-replace` |
| Regions | `set-mark-command`, `exchange-point-and-mark` |
| Kills and retained text | `kill-line`, `kill-word`, `backward-kill-word`, `kill-region`, `kill-ring-save`, `yank`, `yank-pop` |
| Tree | `toggle-file-tree`, `focus-file-tree`, `tree-previous-line`, `tree-next-line`, `tree-expand`, `tree-collapse`, `tree-toggle-directory`, `tree-refresh`, `tree-open`, `tree-create-file`, `tree-create-directory`, `tree-rename`, `tree-delete` |
| Rectangles | `rectangle-mark-mode`, `copy-rectangle-as-kill`, `kill-rectangle`, `yank-rectangle` |

See [Rectangles](rectangles.md) for display-column selection and copy boundaries.

See [File tree](file-tree.md) for root, layout and keyboard focus.
