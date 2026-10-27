# Commands

`M-x` opens the command prompt. Reported Alt/Meta and compatible Escape input
follow the [terminal input policy](terminal-input.md). Command entry uses the
same fixed command registry and handlers as keyboard bindings.

## Matching and invocation

- Type a lowercase hyphenated name or part of it. Matching is case-sensitive.
  `minibuffer.match` selects `prefix-then-substring` (the default), `prefix` or
  `substring`. The default ranks prefixes before other substrings; each rank
  retains the registry's alphabetical order. Substring mode has one alphabetical
  rank; prefix mode excludes other substrings.
- The query appears above a vertical list with one command per row. Up to
  `minibuffer.candidate_limit` rows are shown (default 8, admitted range 1..=32),
  clipped to geometry after complete editing minima. The selected command uses
  the selection palette roles, with reverse video under `NO_COLOR`. No matches
  display `No matching commands` in an unselected row.
- Up/Down or C-p/C-n selects the previous/next candidate, stopping at either
  end without changing the query. The selected row stays visible as the list
  scrolls. Tab fills the selected command name without executing or cycling it;
  the original matching set remains available for further navigation.
- Return invokes an explicitly selected candidate, otherwise an exact typed name
  or the first match for a nonempty query. Opening M-x and pressing Return with
  an empty query reports `No command entered`; explicit navigation or completion
  is required to choose from the unfiltered list. Unknown names report
  `Unknown command` and restore the workspace without changing document text.
  Tab or navigation with no matches retains command focus and reports
  `No matching command`.
- Editing the query resets selection and scroll origin. Cursor movement and
  refused edits preserve them. M-p recalls earlier commands; M-n moves toward
  newer commands and then restores the draft query. History recall starts a new
  matching set; navigation stops at the oldest entry. The session retains up to
  128 distinct accepted command names, most recent first. Accepting a name records
  the invocation even if its handler later refuses it. Canceled or unknown input
  does not enter history; history is not persisted.
- C-g cancels command entry. Text, cursor edits and single-line paste remain in
  the prompt. The existing 4096-character prompt bound and control rejection apply.
  Releases and repeated Return do not invoke a command twice.

Candidate space borrows presentation rows without scrolling the retained editing
workspace. When no candidate rows fit, the query and cancellation remain usable.
Resize preserves the draft and selection; growing restores a coherent list.
Rendering reads only the fixed command registry and cannot execute a command or
observe the filesystem.

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

See [Bindings](bindings.md) for the complete key map and [Scope and defaults](defaults.md)
for file, history and retained-text policies.

## Implemented catalog

| Group | Names |
| --- | --- |
| Command and session | `execute-extended-command`, `keyboard-quit`, `save-buffers-kill-terminal` |
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

## Supplied retention preferences

Library sessions supplied with an admitted snapshot use `history.group_limit`
(1..=65536 complete groups, default 512) and `history.payload_target_bytes`
(1..=1073741824 logical fragment bytes, default 67108864). At each completed edit,
oldest whole groups evict until both targets hold, always retaining the newest
coherent group even when it alone exceeds the payload target. This soft target
is not a process-memory maximum. Applying a different buffer policy leaves
published undo/redo records intact until the next edit; branching then discards
redo and applies retention through the canonical history owner. Pruning never
marks text saved or changes the logical saved-state identity.

`kill_ring.capacity` admits 1..=4096 linear entries (default 32). A new entry
replaces the oldest when full; yank-pop wraps over the retained entries, including
at capacity one. With `kill_ring.coalesce = true` (the default), consecutive
compatible kills append forward or prepend backward. Setting it false retains
separate entries. Copy and intervening commands still end continuity. Rectangular
text retains its separate owner. These preferences retain only session memory.
