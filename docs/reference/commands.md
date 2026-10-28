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
| Native mode | `org-mode`, `text-mode` |
| Org visibility | `org-cycle`, `org-shifttab`, `org-overview`, `org-content`, `org-show-all`, `org-hide-subtree`, `org-show-children`, `org-show-subtree`, `org-hide-drawer`, `org-show-drawer`, `org-toggle-drawer` |
| Org tasks | `org-todo`, `org-todo-next`, `org-todo-previous`, `org-todo-set`, `org-inspect-todo` |
| Org tags | `org-set-tags-command`, `org-remove-tag`, `org-inspect-tags`, `org-match-tags` |
| Org properties | `org-set-property`, `org-delete-property`, `org-inspect-property`, `org-edit-property-owner` |
| Org headings | `org-insert-heading`, `org-meta-return`, `org-next-visible-heading`, `org-previous-visible-heading`, `org-forward-heading-same-level`, `org-backward-heading-same-level`, `org-up-heading-safe` |
| Org list context | `org-metaleft`, `org-metaright`, `org-metaup`, `org-metadown` |
| Org checkboxes | `org-ctrl-c-ctrl-c`, `org-toggle-checkbox`, `org-toggle-radio-button`, `org-toggle-checkbox-presence`, `org-update-statistics-cookies`, `org-insert-item-checkbox` |
| Org lists | `org-insert-item`, `org-indent-item`, `org-outdent-item`, `org-indent-item-tree`, `org-outdent-item-tree`, `org-move-item-up`, `org-move-item-down`, `org-renumber-list` |
| Org subtrees | `org-promote-subtree`, `org-demote-subtree`, `org-move-subtree-up`, `org-move-subtree-down` |
| Files | `find-file`, `save-buffer`, `write-file` |
| Buffers | `switch-to-buffer`, `list-buffers`, `kill-buffer` |
| Windows | `split-window-below`, `split-window-right`, `other-window`, `delete-window`, `delete-other-windows` |
| Editing | `newline`, `insert-tab`, `backward-delete-char`, `delete-char`, `tabify`, `untabify` |
| Motion | `forward-char`, `backward-char`, `forward-word`, `backward-word`, `move-beginning-of-line`, `move-end-of-line`, `beginning-of-buffer`, `end-of-buffer`, `goto-line` |
| Scrolling | `next-line`, `previous-line`, `scroll-up-command`, `scroll-down-command`, `recenter-top-bottom` |
| History | `undo`, `undo-redo` |
| Search | `isearch-forward`, `isearch-backward`, `query-replace` |
| Regions | `set-mark-command`, `exchange-point-and-mark` |
| Kills and retained text | `kill-line`, `kill-word`, `backward-kill-word`, `kill-region`, `kill-ring-save`, `yank`, `yank-pop` |
| Tree | `toggle-file-tree`, `focus-file-tree`, `tree-previous-line`, `tree-next-line`, `tree-expand`, `tree-collapse`, `tree-toggle-directory`, `tree-refresh`, `tree-open`, `tree-create-file`, `tree-create-directory`, `tree-rename`, `tree-delete` |
| Mouse | `mouse-mode` |
| Gutter | `gutter-toggle-line-numbers`, `gutter-cycle-line-number-mode`, `gutter-set-line-numbers-absolute`, `gutter-set-line-numbers-relative`, `gutter-cycle-current-line`, `gutter-toggle-line-number-side`, `gutter-inspect-numbering`, `gutter-reset-numbering` |
| Rectangles | `rectangle-mark-mode`, `copy-rectangle-as-kill`, `kill-rectangle`, `yank-rectangle` |

See [Rectangles](rectangles.md) for display-column selection and copy boundaries.

See [File tree](file-tree.md) for root, layout and keyboard focus.

## Line navigation

`M-g g`, `M-g M-g` and `M-x goto-line` open `Goto line: `. Enter a signed
64-bit decimal integer and press Return. The configured `minibuffer.input_limit`
bounds the draft (default 4096 Unicode scalars). Surrounding whitespace is
ignored; malformed, fractional, nondecimal or overflowing values retain the
draft with feedback so it can be edited and submitted again. Tab displays input
guidance. C-g cancels and retains the original point, viewport and navigation
state; normal region deactivation still applies.

Numbers refer to one-based logical lines. Existing lines go to their beginning;
zero or negative numbers go to buffer start, and numbers past the final line go
to exact buffer end. A final newline therefore admits a final empty logical
line. Navigation reveals the destination in the selected view, preserves other
views and uses the existing mark/region rules. Read-only text remains navigable;
no edit, undo entry, dirty change or save occurs.

## Whitespace conversion

`M-x tabify` and `M-x untabify` act on the marked region. Set mark with `C-SPC`,
move to the other endpoint, then invoke the desired command. To select the whole
document explicitly, use `M-<`, `C-SPC`, `M->` in that order. Without an active
region the command reports `Region is not active`; it never selects the document
implicitly. An empty or already unchanged region reports `No whitespace changed`
and creates no edit, undo entry or dirty-state change.

Untabify expands selected literal tabs to spaces using the buffer's effective
`editor.tab_width` (1..=32), including language overrides. Tabify replaces eligible
selected runs of spaces/tabs with a shorter representation at those same stops.
A one-cell run cannot shorten. Both commands use actual display columns,
including wide/combining text and any unselected prefix of the current line.
Whitespace inside strings or comments participates when selected.

Only selected whitespace changes. Other text, BOM, LF/CRLF separators and final
newline presence remain intact, and non-whitespace text keeps its display columns.
A space can anchor an otherwise zero-width grapheme following an expanded tab,
preserving its existing visible cell. Settings, the Tab key's insertion policy,
other buffers and the kill ring remain separate from conversion.

Each conversion is one checked edit across all selected lines. One undo restores
the original bytes and redo restores conversion; shared views use ordinary point
and viewport mapping. Read-only buffers reject nonempty conversion. C-g while
entering the M-x command cancels without editing and retains normal region
cancellation. There are no factory shortcuts for these commands; an existing
validated editor keymap may bind either name.

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

## Native task workflows

C-c C-t (`org-todo`) cycles the current heading through its declared sequence,
including the unmarked state. Declarations with fast selection keys instead
open a state prompt. `org-todo-set` always prompts: enter a full keyword, its
selection key, or `-`/empty for unmarked. `org-todo-next` and
`org-todo-previous` cycle without fast selection; `org-inspect-todo` reports
current state and the admitted choices without editing.

TODO, SEQ_TODO and TYP_TODO declarations are data. Each declaration defines a
separate sequence, `|` begins finished states, and without `|` the last state
is finished. Keywords must be unique across the file (at most 64). Annotations
such as `WAIT(w@/!)` select a key, entry note and departure timestamp. Target
entry logging takes precedence over departure logging. Unknown leading words
remain title text. Malformed declarations reject task editing and matching.

A transition preserves priorities, tags, planning, properties and unrelated
bytes. Selected CLOSED timestamps, state logs/one-line notes and the immediate
parent's task progress cookies publish in the same transaction. Empty notes
record time only; C-g cancels the entire transition. Undo restores all its
metadata, including shared-view anchors. Read-only buffers admit inspection
and reject transitions before opening an edit prompt.

LOGGING and LOG_INTO_DRAWER inherit explicitly from source ancestors/file
properties. A nonempty LOGGING value resets defaults and annotations; `nil`,
`logdone`, `lognotedone`, and declared keyword annotations are supported.
LOG_INTO_DRAWER accepts `nil`, `t` (LOGBOOK) or an explicit drawer name.
Task timestamps capture the local operation time once, before a note prompt.
The session reads at most 1 MiB of TZif data; UTC or a TZif name/path in TZ is
supported. Unsupported or uncovered local time rejects before mutation.
