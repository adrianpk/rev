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

The editor and [Org command catalog](org/commands.md) together list every
compiled M-x name once.

| Group | Names |
| --- | --- |
| Command and session | `execute-extended-command`, `keyboard-quit`, `save-buffers-kill-terminal` |
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
| Source wrapping | `visual-line-mode` |
| Gutter | `gutter-toggle-line-numbers`, `gutter-cycle-line-number-mode`, `gutter-set-line-numbers-absolute`, `gutter-set-line-numbers-relative`, `gutter-cycle-current-line`, `gutter-toggle-line-number-side`, `gutter-inspect-numbering`, `gutter-reset-numbering` |
| Bookmarks | `bookmark-set`, `bookmark-set-no-overwrite`, `bookmark-jump`, `bookmark-delete`, `bookmark-save`, `bookmark-reload` |
| Rectangles | `rectangle-mark-mode`, `copy-rectangle-as-kill`, `kill-rectangle`, `yank-rectangle` |

See [Rectangles](rectangles.md) for display-column selection and copy boundaries.
See [File tree](file-tree.md) for root, layout and keyboard focus.

## Line navigation

With wrapping enabled, Up/Down and `C-n`/`C-p` move through visual rows.
`C-a`/`C-e` address the current visual row. Following Emacs, `C-e` stops on
the character where a continued row wraps; at a source EOL it reaches the
insertion boundary. `C-k` kills to that visual end, including wrap whitespace.
At the visual end, the next kill consumes the wrap character or real source
newline. Kills retain the ordinary kill ring, undo and read-only admission.
Disabling wrapping restores logical-line motion and kills.

Paging and recentering use visual rows while wrapped. Goal columns follow
terminal cells and are recalculated when pane width changes. Each split keeps
its own point and viewport over the same source text.

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

See [Org native task workflows](org/commands.md#native-task-workflows).

## Named bookmarks

`C-x r m` (`bookmark-set`) records the current file position under a name.
Return with an empty draft uses the visited basename shown as the default;
type a name to distinguish positions in the same file. Names are case-sensitive,
retain spaces literally, and contain 1..256 UTF-8 bytes without control characters.
Setting an existing name replaces its location. `C-x r M`
(`bookmark-set-no-overwrite`) refuses an occupied name. `C-x r b`
(`bookmark-jump`) visits a name; `M-x bookmark-delete` removes it.

These commands use a vertical name list with the configured minibuffer matching
policy and row limit. Up/Down or C-p/C-n selects a candidate; Tab fills the
highlighted name without accepting it. Return accepts an explicitly selected
candidate or the exact typed name. The initial highlight alone does not select
a name. C-g cancels; errors retain the draft for correction. Names are shared
across files, with a configurable cap of 1024 records by default and 4096
encoded path bytes per record. Reaching the limit never evicts another name.

Modified, read-only and intended new file buffers are eligible. Unnamed and
synthetic buffers cannot supply a bookmark. Commands do not save source text,
change its modified flag, or add source undo steps. Aliases reuse the canonical
live file buffer, including unsaved text, and visiting a bookmark preserves an
explicit file-tree root. Rectangle bindings under C-x r remain available.

The user-wide collection loads lazily on first bookmark use or a canonical
rename/save-as demand, so closed records can follow that path change. Live positions follow canonical
edits, structural moves and source undo/redo. Insertions at the point use left
affinity. Removing the original position makes it stale; source undo can restore
it, or an explicit set can replace it. Undo does not recreate deleted names or
replace a bookmark set after that source transaction.

Closing a buffer retains bounded context, and jumping rechecks that evidence
against the reopened file. A changed position can relocate through one unique
context match; ambiguous, missing or unavailable targets retain the original
view and record. The factory limits capture 64 scalars on each side and search
at most 262144 source scalars, in passes of at most 16384. C-g or new input
cancels pending resolution; resize and key releases retain it. Checked context
is location evidence, not proof of historical identity after duplicated edits.

An already open canonical buffer takes precedence, including its unsaved text.
Discarded text is never recreated for a bookmark. Successful file or directory
rename updates associated records, including closed targets; save-as follows
the canonical buffer destination. Jumps reveal folded ancestors and wrapped
fragments in the selected view without changing other splits' fold state.

`M-x bookmark-save` stores the collection without saving any source document.
By default, quitting saves bookmarks after all source save/discard decisions.
Set `bookmarks.save = "on-change"` to checkpoint explicit set/delete operations,
rename and source-save boundaries; text edits only dirty evidence for the next
checkpoint. `"manual"` saves only on explicit request and leaves unsaved bookmark
changes unsaved on ordinary quit. A failed automatic quit checkpoint offers
`retry`, `discard` or `cancel` in a name prompt; C-g also cancels. Discard permits
that exit without changing source text or pretending the collection was stored.

`M-x bookmark-reload` validates disk before offering to discard dirty in-memory
bookmarks. No or C-g retains both states; yes rechecks disk before replacement.
The default destination is the existing XDG user data root's `bookmarks.toml`;
`bookmarks.file` can select an absolute destination. Storage is independent of
launch directory and tree root. Embedded library sessions retain host ownership
and use persistence only with an explicit configured destination.

The strict version-1 TOML stores names, lossless native paths, scalar positions
and bounded before/after context in lexical name order. Unknown fields/versions,
duplicate names, malformed evidence, unreadable files, more than the configured
entry cap or more than 16 MiB reject the complete candidate. Invalid stores and
unsaved memory are retained; native in-memory operations remain available.
After correcting storage, explicitly reload before replacing an unknown store.

A nonblocking sibling file lock serializes cooperating writers. Saves compare
exact observed identity and bytes, stage and sync in the same directory, then
atomically replace and verify publication. A conflicting or busy store requires
retry or explicit reload; explicit save alone never authorizes an overwrite.
A failure after publication remains unresolved until actual candidate bytes
and sync completion reconcile. Newly owned data directories/files are private
to the user. The stable sibling lock file may remain after the OS lock releases.
