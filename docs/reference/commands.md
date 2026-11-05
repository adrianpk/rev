# Commands

`M-x` opens the command prompt. In Vim Normal or Visual, `:` opens that same
vertical selector with the full shared catalog, matching, history and argument
workflow. Type canonical names; colon adds no Ex aliases, ranges or argument parser. Reported Alt/Meta and compatible Escape input
follow the [terminal input policy](terminal-input.md). Command entry uses the
same admitted command catalog and handlers as keyboard bindings.

Cancelling a selector entered from Visual preserves the native selection only
while its view, point, mark, source revision and shape remain valid. A changed
source or selected view retires it; cancellation never copies old source offsets
back into the editor.

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
Rendering reads only the admitted command catalog and cannot execute a command or
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
| Optional server transport | `lsp-mode`, `lsp-status`, `lsp-connect`, `lsp-disconnect`, `lsp-restart`, `lsp-setup`, `lsp-reload-settings`, `lsp-servers`, `lsp-select-server`, `lsp-install-server`, `lsp-update-server`, `lsp-remove-server`, `lsp-cancel-installation` |
| Native completion | `completion-at-point`, `completion-next`, `completion-previous`, `completion-accept`, `completion-cancel`, `completion-mode`, `completion-auto-mode` |
| Command and session | `execute-extended-command`, `keyboard-quit`, `save-buffers-kill-terminal` |
| Files | `find-file`, `save-buffer`, `write-file`, `revert-buffer` |
| Terminal | `terminal-open`, `terminal-toggle`, `terminal-new`, `terminal-new-buffer`, `terminal-move-to-buffer`, `terminal-move-to-panel`, `terminal-open-here`, `terminal-open-other-pane`, `terminal-list`, `terminal-rename`, `terminal-send-next-key`, `terminal-copy-mode`, `terminal-copy-mode-done`, `terminal-yank`, `terminal-yank-pop`, `terminal-clear`, `terminal-clear-scrollback`, `terminal-beginning-of-line`, `terminal-end-of-line`, `terminal-reset-cursor-point`, `terminal-undo`, `terminal-send-return`, `terminal-send-tab`, `terminal-send-escape`, `terminal-send-backspace`, `terminal-send-delete`, `terminal-send-up`, `terminal-send-down`, `terminal-send-left`, `terminal-send-right`, `terminal-send-prior`, `terminal-send-next`, `terminal-send-space`, `terminal-next-prompt`, `terminal-previous-prompt`, `terminal-next-error`, `terminal-open-link` |
| Shell | `shell-filter-region`, `shell-command-insert`, `shell-command-on-region`, `shell-command`, `shell-command-cancel`, `shell-command-output` |
| AI | `ai-explain`, `ai-prompt`, `ai-implement`, `ai-apply-result`, `ai-discard-result`, `ai-set-directory`, `ai-select-model`, `ai-reset-session`, `ai-disconnect`, `ai-cancel`, `ai-save-command` |
| Buffers | `switch-to-buffer`, `list-buffers`, `kill-buffer` |
| Windows | `split-window-below`, `split-window-right`, `other-window`, `delete-window`, `delete-other-windows`, `windmove-left`, `windmove-down`, `windmove-up`, `windmove-right`, `balance-windows` |
| Editing | `newline`, `insert-tab`, `backward-delete-char`, `delete-char`, `tabify`, `untabify` |
| Motion | `forward-char`, `backward-char`, `forward-word`, `backward-word`, `move-beginning-of-line`, `move-end-of-line`, `beginning-of-buffer`, `end-of-buffer`, `goto-line` |
| Scrolling | `next-line`, `previous-line`, `scroll-up-command`, `scroll-down-command`, `recenter-top-bottom` |
| History | `undo`, `undo-redo` |
| Search | `isearch-forward`, `isearch-backward`, `query-replace` |
| Regions | `set-mark-command`, `exchange-point-and-mark` |
| Kills and retained text | `kill-line`, `kill-word`, `backward-kill-word`, `kill-region`, `kill-ring-save`, `yank`, `yank-pop` |
| Tree | `toggle-file-tree`, `focus-file-tree`, `tree-previous-line`, `tree-next-line`, `tree-expand`, `tree-collapse`, `tree-toggle-directory`, `tree-refresh`, `tree-open`, `tree-create-file`, `tree-create-directory`, `tree-rename`, `tree-delete` |
| Mouse | `mouse-mode` |
| Modal state entry | `modal-enter-normal`, `modal-enter-insert`, `modal-enter-replace`, `modal-enter-insert-after`, `modal-enter-insert-line-first`, `modal-enter-insert-line-end`, `modal-open-line-below`, `modal-open-line-above`, `modal-enter-visual-character`, `modal-enter-visual-line`, `modal-enter-visual-block`, `modal-one-normal-command` |
| Modal source navigation | `modal-forward-char`, `modal-backward-char`, `modal-next-line`, `modal-previous-line`, `modal-line-start`, `modal-line-first-text`, `modal-line-end`, `modal-buffer-end`, `modal-buffer-start`, `modal-repeat-find`, `modal-reverse-find`, `modal-next-paragraph`, `modal-previous-paragraph`, `modal-matching-delimiter`, `modal-page-forward` |
| Modal local targets | `modal-page-backward`, `modal-half-page-forward`, `modal-half-page-backward`, `modal-screen-top`, `modal-screen-middle`, `modal-screen-bottom`, `modal-align-top`, `modal-align-middle`, `modal-align-bottom`, `modal-next-word-start`, `modal-previous-word-start` |
| Modal view and search | `modal-next-word-end`, `modal-next-big-word-start`, `modal-previous-big-word-start`, `modal-next-big-word-end`, `modal-previous-word-end`, `modal-previous-big-word-end`, `modal-find-forward`, `modal-find-backward`, `modal-till-forward`, `modal-till-backward`, `modal-search-forward`, `modal-search-backward`, `modal-search-repeat`, `modal-search-reverse`, `modal-search-word-forward`, `modal-search-word-backward` |
| Modal operators and edits | `modal-delete`, `modal-change`, `modal-copy`, `modal-indent`, `modal-outdent`, `modal-delete-line-end`, `modal-change-line-end`, `modal-copy-line-end`, `modal-delete-forward`, `modal-delete-backward`, `modal-replace-character`, `modal-paste-after`, `modal-paste-before`, `modal-join-lines`, `modal-repeat-last-change` |
| Visual text objects | `modal-select-inner-object`, `modal-select-around-object` |
| Jump traversal | `modal-jump-backward`, `modal-jump-forward` |
| Source wrapping | `visual-line-mode` |
| Gutter | `gutter-toggle-line-numbers`, `gutter-cycle-line-number-mode`, `gutter-set-line-numbers-absolute`, `gutter-set-line-numbers-relative`, `gutter-cycle-current-line`, `gutter-toggle-line-number-side`, `gutter-inspect-numbering`, `gutter-reset-numbering` |
| Bookmarks | `bookmark-set`, `bookmark-set-no-overwrite`, `bookmark-jump`, `bookmark-delete`, `bookmark-save`, `bookmark-reload`, `list-bookmarks`, `bookmark-next`, `bookmark-previous`, `bookmark-markers-mode`, `bookmark-bmenu-delete`, `bookmark-bmenu-unmark`, `bookmark-bmenu-execute-deletions`, `bookmark-bmenu-this-window` |
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

For a task sequence, see [Save and revisit named locations](../how-to/use-bookmarks.md).

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

### Checked source locations

The user-wide collection loads lazily on first bookmark use, enabled marker
demand for a file-backed view, or canonical rename/save-as demand, so closed
records can follow that path change. Live positions follow canonical
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

### Storage and recovery

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

For a whole short file containing `Notes` and a final newline, a stored BOF
record has this form:

```toml
version = 1

[[bookmarks]]
name = "notes-start"
path = "/work/notes.txt"
position = 0
before = ""
after = "Notes\n"
at_start = true
at_end = true
```

The example path is illustrative. Positions count source Unicode scalars,
not bytes, screen columns or wrapped rows. Non-UTF-8 native paths use an encoded
`path_bytes` array instead of `path`; exactly one is present. An empty store has
only `version = 1`. View policy, transient IDs and lookup indexes are not stored.

### Native list and ordered visits

`C-x r l` (`list-bookmarks`) opens a native read-only `*Bookmarks*` view in the
selected pane. It lists names lexically with file paths and observed resolved,
unresolved, stale or unavailable status; opening and painting do not read target
sources. Up/Down, C-p/C-n or p/n moves between rows. Return visits the selected
stable record through the checked jump owner. A failed or canceled visit retains
the list and originating source.

| Observed status | Meaning |
| --- | --- |
| resolved | A valid anchor in the current live source revision. |
| unresolved | Stored or closed-file context has not been checked against a live source. |
| stale | An edit removed the original point; retained source history can restore it. |
| unavailable | A checked target path could not be opened. |

Status is an observation, not a background filesystem poll. A failed context
check can retain an unresolved record with its diagnostic until another visit.

In the list, `bookmark-bmenu-delete` (`d`) marks a record and advances;
`bookmark-bmenu-unmark` (`u`) clears its mark and advances.
`bookmark-bmenu-execute-deletions` (`x`) atomically removes marked records;
`bookmark-bmenu-this-window` (Return) visits the selected record.
`q` or C-g returns and discards unexecuted marks. Quit also discards those marks.
The local keys are configurable in `keys.bookmark_list`. M-x retains ordinary
prompt ownership; C-x o changes pane focus without editing the list's source.

Collection changes invalidate menu observations and clear staged marks before
an action can affect a replacement row. The list refreshes and asks for a retry.
Returning rechecks the original source revision; a changed or closed origin
retains the current valid source view with a diagnostic. Source text, dirty state
and undo history are independent of row navigation and bookmark deletion.

`M-x bookmark-next` and `M-x bookmark-previous` visit revision-current resolved
bookmarks within the current canonical file, in scalar position order with
lexical name ties, wrapping at each end. There are no factory shortcuts.
The first command includes targets at point; repetition advances by stable
record identity, so several names at one point are visited individually.
Changing direction steps back through that same order. Point movement, source
edits, pane changes and collection mutations reset the repetition observation.
Unresolved, stale and unavailable records are retained but skipped; an empty
eligible set reports a clear result and suggests a checked jump by name.

### Bookmark markers

`M-x bookmark-markers-mode` toggles markers for the current editing view. Splits
inherit its effective visibility; other views retain their policy. Enabled
markers reserve one cell on `gutter.side` independently of number labels, after
protecting available number geometry and at least three text columns. Only the
first visual fragment of a resolved logical line receives the configured glyph.
Multiple names on a line share that glyph and remain independently navigable.
Hiding markers never deletes records. Storage admission occurs on first native
marker demand for a file-backed view; painting never loads, relocates or saves.

## Reload a visiting file

Run `M-x revert-buffer` or `:revert-buffer` to read a fresh checked disk image.
A modified buffer requires confirmation to discard its unsaved contents; `n` or
`C-g` retains the current text. Edits, file replacement or association changes
invalidate pending work and consent. Missing, nonregular, invalid UTF-8, NUL or
oversized sources leave the buffer intact. The configured reload byte limit
applies, and the source file is never written by this command.

Reload is one undoable native change across shared panes and retains prior
history. Undo restores the previous text as modified relative to disk; redo
returns to the clean reload image. Read-only visiting buffers can refresh while
remaining read-only. Same-byte observations update checked file evidence without
an edit or cursor movement. Startup external-change policies and bounds are
listed in [defaults](defaults.md).

## External file changes

Visiting files are observed through their parent directories, including hidden
buffers. Atomic rename, direct writes, deletion and recreation remain observable.
Shared panes use one subscription and one native reload. Idle observation waits
for kernel events without scanning the project or polling file contents.

The default `files.external_changes = "reload"` refreshes clean visiting buffers.
With `"ask"`, a passive notice and `disk: revert-buffer` status retain the current
text until explicit reload. Dirty buffers retain unsaved text under either
policy, with a conflict notice and the existing save overwrite confirmation.
Notices name the buffer and reload action without taking input focus; another
external version can notify again after dismissing an earlier echo.

Missing, invalid or oversized images retain text and history. Recreation at the
same admitted filename triggers fresh validation. Registration limits or failures report unavailable coverage while
`revert-buffer` remains usable. Queue overflow schedules finite recheck batches. Successful save/reload observations coalesce, and undo of an automatic
reload stays dirty instead of being immediately overwritten.

## Selection explanations through Pi

Select a nonempty linear region and run `M-x ai-explain` or `:ai-explain`.
This optional command starts a checked Pi 1.3.0 child only when invoked. Set
`ai.pi.command` to the installed executable name/path and optionally set
`ai.pi.model`; authenticate through Pi before retrying an authentication failure.
The executable is trusted and receives literal adapter arguments without a shell.

The command sends the captured selection and its native source metadata. Its
**directory context** starts at the explicit AI directory override, otherwise the
opened tree root, otherwise the
visiting file's parent, otherwise the editor's launch directory. It never climbs
to an inferred repository root. Pi chooses which sources to inspect through the
internal read, list and literal-search tools; shell, edit/write tools, automatic
extensions, MCP, skills, templates and discovered instruction files are disabled.
Directory operations reuse only compatible private history and an owned idle
child, according to the settings below. Input-only operations always start with
`--no-session` and close their child when finished.
Ordinary editing starts no Pi process or AI state files.

A prompt acceptance notice is pending work. Completion waits for `agent_settled`
and a valid final explanation. A native read-only `*AI Explanation*` buffer shows
a source-named heading followed by the explanation in Org markup. The trailing
`REQUEST` drawer holds editor-supplied backend, model, input/context policy,
history, captured source revision/range and directory root when applicable.
It starts folded; TAB on its opener toggles it. Native Org visibility remains
independent per pane, including manual show-all. Setting `org.enabled = false`
leaves the result readable as plain text, without Org coloring or folding.
Disabling syntax coloring alone still permits the drawer to start folded.
No Org file is created on disk.

Default explanation guidance uses a software engineering perspective for code,
configuration, logs, diffs and technical documents. Clearly different material
receives a relevant perspective without a persona announcement. The instruction
asks for an overview for whole files and focused detail for short selections.
Org format guidance is separate from the literal instruction; saved prompts
and their requested perspective or length are preserved. Model compliance and
wording remain probabilistic. rev supplies the surrounding title and request
facts; they are not fields invented by the model. The source is never saved
or changed by explanation. Editing may continue during generation; if source or
focus changes, use `switch-to-buffer` to inspect the result. C-g or modal Escape
owned by editing cancels and fences late results; a prompt's cancellation remains
owned by that prompt. A stopping child keeps the sole operation slot until cleanup
finishes. Errors retain the source and require an explicit retry.

The adapter admits LF records up to 2 MiB, protocol queues up to 4 MiB/256 records,
a 64 KiB stderr tail and at most eight model turns. The configured input/result
limits also apply. Cancellation sends clear_queue then abort, allows at most five
seconds for acknowledgements/settlement, then closes stdin with a two-second stop
grace and a two-second forced-reap deadline. No native API fallback is selected.

## Guarded Pi directory reads

The packaged internal resource is extracted to a checked private cache only for
an invoked directory operation. It requires Linux descriptor paths and Pi 1.3.0;
no separate JavaScript installation or user plugin setup is needed. Admission
checks the effective three-tool allowlist before starting model work.

Tool paths are relative to a canonical root. Every component is opened without
following symbolic links, and root identity plus opened file/directory handles
are checked around IO. Hard links, special files, invalid UTF-8, NUL-containing
files and regular files over 1 MiB are rejected. Root replacement retires the
grant. Missing sources report unavailability; an admitted native snapshot can
still represent an unsaved file whose disk entry disappeared.

Discovery excludes `.git`, `.pi`, common credential paths such as `.env`, SSH
keys and credential/key files, and resolved editor/Pi private directories. These
path exclusions do not classify arbitrary source contents as safe to transmit.
Select a root whose ordinary source files may be sent to the chosen Pi model.
The configured executable and packaged adapter are trusted process code; cwd is
not an operating-system sandbox.

Read ranges use Unicode scalar offsets, at most 32768 scalars per call. Listing
supports an optional literal path substring and bounded recursion. Search is
case-sensitive literal text, includes native snapshots and labels disk matches
as disk observations. Results report completeness, skipped entries and source
provenance. Each traversal visits at most 4096 entries and 16 nested directory
levels. Search inspects at most 128 files/4 MiB and returns bounded excerpts.

One invocation admits at most 32 tool calls, eight model turns and 4 MiB of
returned discovery text. Before source IO, a tool reserves 256 KiB of remaining
return capacity; an insufficient remainder ends discovery conservatively.
Compaction never resets these counters or the native request deadline. Discovery
returns data only and cannot expand the original native editing target.

## Reusable AI commands

After a successful operation, run `ai-save-command` and enter a new name without
its `ai-` prefix. This saves the successful literal instruction and its input,
context, output and optional model policy, never the generated text or a captured
source/result. Saving requires a valid current configuration graph; a changed
root/include or competing publisher rejects the transaction. Includes, comments,
unrelated values and other definitions retain their existing representation.
Existing names are refused; choose another name or edit the definition explicitly
in configuration.

The chosen saved command becomes callable immediately through M-x or modal
colon. Only that definition activates: other disk changes and newly persisted
keybindings follow ordinary restart behavior. C-g cancels the name prompt without
writing. No process starts merely because a definition is configured or saved.

Definitions use `ai.commands.<name>` in the existing user TOML graph. At most 64
names are admitted. Names contain 1..64 lowercase ASCII letters/digits/hyphens,
start with a letter, and cannot collide with compiled command/action names.
The callable identity is `ai-<name>`. Each complete record requires a nonempty
literal `prompt` of at most 16384 UTF-8 bytes, `input`, `context` and `output`.
An optional `model` selects Pi's provider/model without changing credentials.
Shell substitutions, template interpolation and executable handlers do not run.

Input `selection` requires a nonempty linear region; `point` captures an insertion
anchor; `buffer` explicitly captures the complete native buffer. Output
`explanation` is read-only. Output `insert` requires point input; `replace`
requires selection or buffer input. Every invocation captures fresh source and
target identities. Application retains the same explicit acceptance and one-step
undo as builtin proposals, including stale-target rejection.

Context `input` transmits only the selected input envelope, with no directory
tools, peer snapshot map or retained history. It always starts a fresh ephemeral
child, including after broader directory work. Context `directory` permits the
existing guarded discovery/snapshot policy and compatible private history.
Changing that boundary first closes an idle child; retry after owned cleanup.
A definition's model overrides the session-local choice, then the configured
Pi model; an absent model uses those existing choices.

```toml
[ai.commands.table-total]
prompt = "Add a totals row and preserve the table layout."
input = "selection"
context = "input"
output = "replace"

[keys.editor]
"M-j" = "ai-table-total"

[keys.vim_normal]
"g t" = "ai-table-total"
```

Complete records merge by name through normal include/root precedence; duplicate
keys within one TOML source reject. Definition admission precedes final keymap
reference validation, so included maps can refer to a root definition. Existing
context, prefix and reserved cancellation/recovery rules still apply. The full
compiled catalog remains available alongside admitted saved names.

## Private Pi sessions and controls

`ai-set-directory` opens a native path prompt for a session-local AI root. Relative
paths resolve against the current AI root. The chosen canonical directory does
not change the file tree, visiting file or editing target. `ai-select-model`
queries Pi and offers only its reported provider/model identifiers in the native
name selector. The choice applies to this editor session; it does not write
configuration or authentication data. Both commands are available through M-x
and modal colon without a factory key prefix.

With `ai.pi.persist_sessions = true`, directory history lives under the XDG state
root: `rev/ai/pi/sessions.toml` binds a canonical root, exact configuration/grant
observations and resolved provider/model to a relative file under
`rev/ai/pi/sessions/`. Pi owns that JSONL history and reconstruction. The native
adapter stores bounded binding metadata, not a second transcript. New invocations
capture fresh source revisions and grant counters; earlier source/tool messages
remain historical and cannot replace the current native editing target.

The version-1 index admits at most 16 bindings and 64 KiB. Each history is limited
to 32 MiB, including an OS file-size limit on the owned persistent Pi child.
Private directories/files require modes 0700/0600 and current-user ownership.
No-follow checks, identity observations and cooperating index/history locks
reject unsafe paths, conflicting writes and simultaneous use of one history.
Unknown schemas, corrupt/oversized records and unsafe histories are preserved
and fail explicitly. Correct the owned store or set persistence false to use
in-memory directory context; no automatic eviction or unknown-history overwrite
occurs.

`ai-reset-session` stops the current child and makes the next explicit invocation
under that directory/model start fresh owned history. Only its verified previous
owned history can be removed after the replacement binding is published.
`ai-disconnect` closes the owned child while preserving history and any completed
result. `ai-cancel` cancels the active invocation or AI-owned prompt/result through
its native owner. None of these commands changes source text or Pi credentials.

`ai.pi.idle_timeout_ms` keeps a compatible directory child available after a
settled result (default 900000 ms); zero closes it immediately. The idle supervisor
blocks on notification until a new explicit request, cancellation or its deadline;
ordinary editor input has no AI idle polling timer. Idle expiry and editor exit
close and reap the owned child. With persistence enabled, later explicit work
restores compatible history in a new child. With persistence disabled, only that
live directory child retains memory; input-only work remains fresh and ephemeral.

Changing root, model or admitted policy crosses an isolation boundary. A child
that is active or stopping retains the single operation slot until cleanup
finishes. Retry explicitly after the stopping notice; no hidden request queue
starts another invocation. A new boundary never adopts the last arbitrary Pi
session or widens an earlier input-only grant.

## Generated text through Pi

Run `M-x ai-prompt` or `:ai-prompt` for a literal instruction. A nonempty linear
selection chooses replacement; otherwise the command captures a point for
insertion. `ai-implement` always captures point insertion, including when a
selection is active. The instruction prompt names the source and chosen scalar
range before execution. Missing required explain selections and rectangular
replacement inputs fail without broadening the target.

Replacement includes its captured selection; point insertion supplies the
instruction and native point metadata. Directory reads use complete snapshots
of the current visiting source and up to 16 admitted current/dirty visiting
buffers within that root, captured when the instruction is submitted and counted against
`max_input_bytes`. These snapshots override disk for their paths. Another
invocation refreshes them; later source edits still invalidate result acceptance.
No whole-directory dump is sent. The instruction accepts at most 16384 UTF-8
bytes, also subject to native prompt and total AI input limits. No reusable
command is saved by executing an instruction. Read-only editing targets reject
before launch; a reload or association change during instruction input requires
a new invocation.

A completed `*AI Result*` is a proposal in a native read-only Org buffer. A
source-named heading precedes a source block for the captured language, or an
example block for plain text. The same initially folded `REQUEST` drawer follows
the proposal. Native Org block coloring applies; this does not add embedded
language token highlighting. Source warnings and apply/discard actions remain
outside the folded metadata. Run
`ai-apply-result` while that result is selected to accept it, or
`ai-discard-result` to close it and return to source. Explanations have no apply
action. One pending result must be applied or discarded before a new invocation.
C-g and editing-owned modal Escape also discard a selected result; canceling a
separate command selector still cancels that selector alone.

Application checks the original buffer, revision, visiting association and live
read-only state again. Focus changes cannot choose another insertion target.
Local edits, undo back to identical text, external reloads and closed sources
reject stale publication while keeping the proposal inspectable. An altered
result cannot reuse its old payload revision. Explicit acceptance consumes its
authority once and makes one ordinary native undo step, without saving the file.
Shared views, source return positions and undo/redo use the native transaction.
Generated bytes are preserved, including whitespace, line endings and fences;
no model-supplied path, command or extra result field is accepted.
The original generated payload is retained separately from the display. Org
comma escaping, display line endings, titles, blocks and request metadata are
never recovered by stripping markup and never enter accepted source text.

## Interactive terminals

Open a shell with `M-x terminal-open` or modal `:terminal-open`. It starts in
the lower panel; `terminal-new-buffer` opens directly in the main area.
`terminal-move-to-buffer` and `terminal-move-to-panel` move the same live session.
Use C-g to return to the editor and M-x for host commands while a terminal owns
focus. Ordinary terminal bytes belong to its child in either startup profile.

Follow [Use an Integrated Terminal](../tutorials/user-guide/16-integrated-terminal.md)
for a shell/lazygit exercise, the [task guide](../how-to/use-integrated-terminal.md)
for common operations, or the [terminal reference](integrated-terminal.md) for
the complete command, input, copy/paste, configuration and lifecycle contracts.

## One-shot shell commands

Follow [Run Shell Commands](../tutorials/user-guide/15-shell-commands.md) for a
sort-and-undo exercise, or see the [shell reference](shell-commands.md) for exact
commands, bounds and lifecycle.

Run `M-x shell-command` or `:shell-command`, then accept a literal command such
as `printf 'hello\n'`. The selected shell receives the accepted command as one
argument after `shell.args`, with stdin immediately closed and no source text.

The shell prompt retains the last 32 admitted command strings for this session;
M-p/M-n recall them and restore the current draft. Cancelled or rejected input
leaves history unchanged. In non-modal editing, M-! runs a command, M-\| sends
the region, C-u M-! inserts output and C-u M-\| filters the region. These are
exact sequences; modal Normal/Visual retain C-u motion. Linear Visual selections
survive colon entry; blocks are rejected. Configured bindings use the same handlers.

Its directory is captured before the prompt: the visiting file's parent, the
explicitly opened directory for unnamed buffers, or the editor launch directory.
The shell inherits the editor environment. No source file is saved automatically.

One invocation runs at a time, within `shell.timeout_seconds` and the aggregate
`shell.max_output_bytes` limit. Editing remains available; active-pane status
shows the shell running or stopping.
Completion is observed by native input polling or frame production, and a finished
job leaves no shell worker or input deadline. Use
`shell-command-cancel` or C-g in its originating editing context to cancel it;
a prompt consumes its own cancellation first. Shutdown cleans up the owned
process group after normal quit decisions succeed.

Completion retains exact valid UTF-8 stdout in a read-only plain-text buffer.
`shell-command-output` visits the last retained result without automatic focus
changes. Nonzero exit also retains stderr/status in `*Shell Diagnostics*`.
Timeout, cancellation, invalid UTF-8, NUL and output overflow report diagnostics
without exposing partial output as a successful result. Closed or changed output
identities are unavailable; they are never recreated or overwritten by name.

Use `shell-command-on-region` to send the current nonempty linear selection as
literal stdin data and show stdout. `shell-filter-region` sends the same bounded
snapshot and replaces that region with complete successful stdout.
`shell-command-insert` sends no source input and inserts at the point captured
before its prompt. Select the complete document with the native mark and buffer
motions for whole-buffer input. Empty, absent and rectangular regions are rejected.

Filters and insertion require the original buffer revision, file association and
writable state before launch and publication. Pending file observation, reload,
external disk changes and closed or edited sources reject automatic application
while retaining valid stdout. Switching panes never redirects the target or takes
focus. Successful replacement/insertion uses one ordinary undo step; empty
replacement deletes the region, while empty insertion creates no history entry.
No whitespace or newline normalization changes the stdout applied to source.

## Native Git Status

C-x g opens native Status in both editing profiles, from the visiting file or
selected directory and launch-directory fallback. The full canvas belongs to
Git navigation while source buffers, layout and terminals remain retained.
Status reads disk and Git state; it does not save an unsaved source buffer.

| Command | Behavior |
| --- | --- |
| `git-tag-create` | Collect a local tag name, lightweight/annotated kind (lightweight by default) and complete target ID; check Git signing policy and explicitly review before creation. Annotated tags add native message composition and retained failure drafts. |
| `git-tag-delete` | Review the selected tag's exact full name, object and peeled target; delete only that unchanged local ref. |
| `git-merge` | Review the exact selected ref/commit and merge into the checked current branch with fast-forward allowed. |
| `git-merge-fast-forward` | Review an exact integration target and require Git's fast-forward-only policy. |
| `git-merge-no-fast-forward` | Review an exact integration target and require a merge commit when integration has work. |
| `git-rebase` | Review ordinary rebase of the checked current branch onto the exact selected ref's commit; require clean disk/index and unmodified visiting sources. |
| `git-merge-continue` | Refresh and confirm continuation of a compatible merge with saved staged resolutions; verify the exact resulting parents and index tree. |
| `git-merge-abort` | Refresh and confirm abort of a compatible merge; verify restoration of its exact original branch and object. |
| `git-rebase-autosquash` | Prepare and explicitly review at most 256 ordered pick/fixup/squash actions and full source IDs before rebasing onto the exact selected target; no interactive external editor or executable todo commands. |
| `git-rebase-continue` | Refresh and confirm continuation of a compatible Git-owned rebase with saved staged resolutions and exact lineage. |
| `git-rebase-skip` | Refresh and explicitly confirm skipping the current compatible rebase step; expose the actual sequence or completed skipped outcome. |
| `git-rebase-abort` | Refresh and confirm abort of a compatible rebase; verify its original branch and object. |
| `git-bisect-start` | Collect explicit bad/good boundaries, resolve full commit IDs and review the original branch/object before starting manual bisect. |
| `git-bisect-good` | Review the exact current candidate and record the user-supplied good verdict. |
| `git-bisect-bad` | Review the exact current candidate and record the user-supplied bad verdict. |
| `git-bisect-skip` | Review and skip the current candidate; retain ambiguity when skipped commits prevent a unique first bad. |
| `git-bisect-reset` | Review and restore the checked original branch/object; reject missing or changed original lineage. |
| `git-cherry-pick-continue` | Refresh actual Git state and confirm saved staged resolutions, then apply the remaining reviewed targets in order; require unchanged retained cherry-pick lineage. |
| `git-cherry-pick-skip` | Refresh actual Git state and confirm skipping only the current target, then retain the reviewed remaining order; require unchanged retained cherry-pick lineage. |
| `git-cherry-pick-abort` | Refresh actual Git state and confirm restoration of the exact reviewed original branch and HEAD, including a verified completed prefix; require unchanged retained cherry-pick lineage. |
| `git-revert-continue` | Refresh actual Git state and confirm saved staged resolutions, then apply the remaining reviewed targets in order; require unchanged retained revert lineage. |
| `git-revert-skip` | Refresh actual Git state and confirm skipping only the current target, then retain the reviewed remaining order; require unchanged retained revert lineage. |
| `git-revert-abort` | Refresh actual Git state and confirm restoration of the exact reviewed original branch and HEAD, including a verified completed prefix; require unchanged retained revert lineage. |
| `git-operation-continue` | Refresh actual Git state and review one compatible family-specific continuation bound to that exact sequence. |
| `git-operation-skip` | Refresh actual Git state and review a compatible skip; reject a missing or wrong-family sequence. |
| `git-operation-abort` | Refresh actual Git state and review one compatible abort with exact original lineage; process cancellation is a separate action. |
| `git-clone` | Review literal clone arguments and an explicit absent destination, then verify the created regular repository. |
| `git-clone-shallow` | Prepare a depth-one clone at an explicit destination and verify its shallow state and full HEAD. |
| `git-clone-bare` | Review a bare clone destination and verify its separate Git-only repository identity. |
| `git-clone-mirror` | Review a mirror clone destination and verify bare and mirror configuration without a later push. |
| `git-init` | Prepare initialization at an explicit absent destination and verify its unborn repository identity. |
| `git-ignore` | Visit the ordinary worktree .gitignore file without inserting a pattern or saving text. |
| `git-ignore-exclude` | Visit the ordinary common Git info/exclude file through its native visiting owner. |
| `git-submodule-add` | Review a submodule metadata name, source URL and literal path, then verify its gitlink and checkout. |
| `git-submodule-list` | Review recursive submodule status and inspect the actual bounded listing without repository effects. |
| `git-submodule-update` | Review one exact gitlink path and initialize or check out its indexed commit without forcing dirty files. |
| `git-submodule-sync` | Review one declared submodule path and verify its registered URL against .gitmodules. |
| `git-submodule-deinit` | Review non-forced deinitialization of one checked clean submodule, preserving its parent gitlink. |
| `git-subtree-add` | Review an explicit prefix, source repository and pinned full branch object before importing a subtree. |
| `git-subtree-merge` | Review a full local commit and subtree prefix, retaining actual merge/conflict state. |
| `git-subtree-pull` | Pin one explicit source branch before fetching and merging into a reviewed subtree prefix. |
| `git-subtree-push` | Review one explicit endpoint and full destination branch, then verify its published subtree object and tree. |
| `git-subtree-split` | Review an exact commit and prefix, then inspect the full split object while preserving local HEAD and saved files. |
| `git-notes-add` | Review a full notes ref, exact target object and literal message; verify the stored note bytes. |
| `git-notes-remove` | Review one existing note on a full object ID and remove only that note. |
| `git-notes-prune` | Review exact missing-object note targets and prune only those candidates. |
| `git-sparse-checkout-enable` | Review enabling cone mode and verify actual tracked-file materialization. |
| `git-sparse-checkout-disable` | Review disabling sparse checkout and verify that tracked files return. |
| `git-sparse-checkout-reapply` | Review reapplying the captured cone selection to a checked clean working tree. |
| `git-sparse-checkout-set` | Review literal relative directories and replace the cone selection without forcing saved changes. |
| `git-sparse-checkout-add` | Review literal relative directories and add them to the captured cone selection. |
| `git-format-patch` | Review an explicit local commit range and complete mailbox bytes, then save to one absent output file. |
| `git-save-diff` | Review two local commits and a complete binary diff, then save to one absent output file. |
| `git-request-pull` | Review local commit anchors and one explicit published remote branch, then save request text without sending mail. |
| `git-worktree-list` | Open a bounded NUL-record worktree list with raw paths, full HEADs, distinct Git/common directories, lock reasons and prune state. |
| `git-worktree-create` | Review a new linked worktree and new local branch at the exact observed HEAD, using a checked absent destination. |
| `git-worktree-visit` | Open checked Git Status for the selected worktree identity while retaining native source buffers and windows. |
| `git-worktree-lock` | Review an exact linked-worktree path and explicit one-line lock reason. |
| `git-worktree-unlock` | Review unlocking one exact linked-worktree record without changing its saved files. |
| `git-worktree-prune` | Review all captured missing unlocked worktree records and prune only while the complete registry remains unchanged. |
| `git-worktree-remove` | Review non-forced removal of one clean linked worktree; protect current/main roots, lock state and retained native resources. |
| `git-stash-list` | Open a bounded exact stash list; subjects and ordinal labels never supply mutation authority. |
| `git-stash-create` | Review and stash saved tracked and untracked changes, excluding ignored files; protect affected unsaved buffers. |
| `git-stash-show` | Inspect the selected full stash object as bounded read-only patch detail with explicit partial status. |
| `git-stash-apply` | Review and apply one full stash object to checked clean saved state while keeping its stash entry. |
| `git-stash-pop` | Apply one full stash object and drop only the revalidated captured entry after success; retain the stash on conflict. |
| `git-stash-branch` | Review a new branch at the selected stash base, restore its full object and remove its checked entry only on success. |
| `git-stash-drop` | Review removal of one exact stash entry without changing the saved working files or index. |
| `git-conflict-explain` | Capture exact roles, objects, saved disk text and visiting unsaved result text for the existing optional input-only AI explanation owner; retain a read-only result with no edit/stage/continue authority and mark changed sources as historical. |
| `git-conflict-use-base` | Review a complete compatible Base side and replace only the exact working file, preserving unmerged index stages. |
| `git-conflict-use-current` | Review the operation-relative Current side and replace only the checked working file without staging. |
| `git-conflict-use-applied` | Review the operation-relative Applied side and replace only the checked working file without staging. |
| `git-conflict-delete` | Review removal of one exact supported result from disk and index, staging its deletion while leaving continuation separate. |
| `git-conflict-stage` | Review and stage one exact saved text result or saved absence; recheck complete stages, file identity, bytes, Git conversion and visiting-buffer stamps. |
| `git-conflicts` | Refresh actual Git-owned sequence state and exact selectable unmerged paths; Return inspects the selected conflict. |
| `git-conflict-visit` | From Operation, inspect the selected exact unmerged path and its available Base/Current/Applied index stages; from Conflict, visit the canonical editable result without staging or continuing. |
| `git-operation-status` | Refresh compatible Status/Refs and actual Git-owned sequence family, full current/original/onto identities, step and exact conflict paths, including external operations. |
| `git-push` | Prepare and explicitly confirm one selected or marked exact local branch, its configured upstream remote and full destination; push one object without force or expanded publication and verify the actual push endpoint. |
| `git-fetch` | Fetch the exact selected configured remote in Refs with its observed transport/refspec policy; refresh tracking facts and Status without merge or rebase. |
| `git-refs` | Observe local/remote-tracking branches, both tag kinds, configured remote names, HEAD and available upstream facts with exact names and object IDs. |
| `git-branch-create` | Collect a new local name and confirm the selected ref's commit or checked HEAD; create without implicit tracking. |
| `git-branch-switch` | Confirm the selected local branch; require complete clean disk/index and unchanged unmodified repository-backed buffers. |
| `git-branch-rename` | Collect a new name and confirm the exact selected local branch; preserve its object and reject replacement of an existing name. |
| `git-branch-delete` | Confirm exact local deletion; require a noncurrent branch merged into checked HEAD and retain Git's non-forced protection. |
| `git-show-range` | Compare the marked full base and selected full endpoint in that order; keep the exact read-only range through refresh. |
| `git-cherry-pick` | Prepare the selected History revision or independent ordered marks; choose a mainline number for each merge, review full targets/parents and confirm that exact order. |
| `git-revert` | Prepare and confirm the selected or independently marked ordered revisions for revert, including each merge mainline; preserve the requested order. |
| `git-sequence-mark` | Append the selected History revision once to a bounded ordered set, retaining the independent range base. |
| `git-sequence-unmark` | Remove the selected revision from the ordered set, retaining remaining order and the range base. |
| `git-reflog` | Inspect bounded HEAD reflog entries; volatile selectors stay display metadata, while revision visits use full objects. |
| `git-blame` | Attribute the selected current source at exact HEAD, retaining original full objects, paths and line locations. |
| `git-log` | Open bounded ancestry from current branch HEAD; detached or bare repositories collect an explicit root, resolved once to a full commit. |
| `git-show` | Inspect the selected History/reflog/blame revision with full metadata and first-parent diff; in revision detail, compare a selected parent or collect another complete parent ID. |
| `git-log-next-page` | Continue the pinned root's exact topological ancestry; expose completion and the configured retained-page limit. |
| `git-commit-amend` | Prepare the exact observed HEAD and existing message; explicitly review before committing the checked index as its replacement. |
| `git-commit-fixup` | Resolve one entered revision to a full immutable commit, compose its standard fixup subject/body and explicitly review the target; keep current history in place. |
| `git-commit` | Open a native subject/body composer from current complete Status; retain cancelled or failed text for this session. |
| `git-commit-submit` | Explicitly commit the checked index with the accepted message; verify actual HEAD, parent, message, tree and unchanged index. In tag message composition, open or accept the annotated tag review. In prepared argument composition, check input and open exact review, then execute only the accepted plan. |
| `git-commit-cancel` | Return before process start; retain commit/tag annotation drafts or cancel prepared arguments and restore their source owner. |
| `git-status` | Check installed Git and the actual repository, then open structured Status. |
| `git-quit` | Return from the innermost Git menu or restore the retained editing context; started work continues. |
| `git-refresh` | Replace Status only after a checked bounded observation succeeds; retain confirmed facts with a stale indication on failure. |
| `git-toggle-section` | Toggle a Status group, path comparison or hunk; in inspection, open the selected exact parent, revision or read-only historical file. |
| `git-discard` | Confirm restoration of one tracked unstaged whole path from its index; reject unsaved canonical sources and verify unchanged index entries and actual checkout bytes/mode. |
| `git-mark` | In Refs, retain one exact local branch push source independently of selection. In Status/Diff, mark the exact current path or supported hunk; retain its complete comparison independently of the cursor. |
| `git-unmark` | In Refs, clear the independent push source. Otherwise, Remove the exact selected path or hunk mark. |
| `git-unmark-all` | Clear all path and hunk marks. |
| `git-stage` | Stage the selected exact disk path, supported unstaged text hunk or complete compatible marked group after complete preflight; preserve source buffers and verify the index. |
| `git-unstage` | Unstage the selected staged path, supported text hunk or complete compatible marked group against its captured HEAD baseline, including unborn HEAD. |
| `git-diff` | Inspect the selected exact path/layer or hunk in read-only detail; q returns to Status. |
| `git-visit-file` | Visit the selected canonical current source without replacing unsaved text, including from blame or historical detail; in Processes, inspect the selected operation. |
| `git-dispatch` | Show the currently available native Git actions. |
| `git-processes` | Inspect active operations followed by bounded recent records, including after leaving Git. |
| `git-process-show` | Open the selected operation's phase, elapsed time, exit evidence and safely displayed retained diagnostics. |
| `git-process-cancel` | Request bounded cancellation of the explicitly selected live operation; an expired identity cannot target a replacement. |

Tab on a path expands its exact comparison; Tab on a hunk expands its body.
One path and one hunk body expand at a time. `git-diff` opens all retained patch
lines in read-only detail, including explicit binary, rename, submodule,
malformed and truncated results. g refreshes Status and the retained comparison;
old detail is labeled stale until replacement succeeds. Return visits the real
current file, preserving an existing modified source buffer. A deleted path or
directory has no current source file to visit. q returns one presentation level.

From Refs, the native `git-log` command opens the exact selected ref's commit
as the History root while retaining the checked current working branch. History
P and V prepare cherry-pick and revert. With no ordered marks, the selected
revision is the one target. a/U append/remove independent ordered marks; * keeps
its separate range-base role. Each merge requires a parent number and final
review shows that full parent object. y, C-Enter or C-c C-c confirms; Return does
not submit. A stopped ordered request exposes the actual Git family and current
revision with its retained original context and verified completed prefix.

b or y in Status opens Refs. Its independent `keys.git_refs` map keeps n for
branch creation, S for switch, R for rename and D for non-forced deletion.
Choose a ref or configured remote with Up/Down or C-p/C-n. Tab opens
read-only full-name/object/type/upstream detail; Up/Down scroll and Tab or q
returns to the list. Remote names remain reachable beyond the ref rows. Prompts collect new names; the explicit
review shows the exact accepted name and full target. Only y accepts; Return
is not acceptance, n or C-g cancels. Existing names, moved refs/HEAD, modified
source buffers and dirty switch state reject. g refreshes compatible Status
and Refs; failed observations retain confirmed facts. Tags show their actual
object and peeled commit separately. Configured remote names are shown without
exposing URL values. q returns to Status; started work retains its ownership.
Changed disk files enter the existing automatic-or-ask reload policy. Retained
history refreshes through the owned read worker: unchanged HEAD retains its
pinned pages, while a changed HEAD starts current ancestry. Refresh failure
keeps the preceding history stale and does not replay the ref mutation.

t in Refs creates a local tag. Enter a name, accept the default lightweight kind
or explicitly select annotated, then accept or enter a full target object ID.
Git checks the name, target and current signing policy before review. Lightweight
review shows the name, kind and full target; y accepts, n or C-g cancels. A policy
requiring signing rejects the lightweight choice without changing its kind or
disabling signing. Select annotated explicitly to compose an annotation in the
native message editor. Return inserts a newline; C-Enter or C-c C-c opens the
scrollable exact review, and the same aliases explicitly accept it. q or C-g
from review retains the draft. The existing `keys.git_commit` aliases own native
message editing for both commits and annotated tags. Source publication and
saving remain guarded while this transient message owner is active.

Git supplies tagger identity/date and performs configured signing. Signed
annotation review includes its required final newline within the 65536-byte
message limit, so the signature remains recognizable to Git. A failed tag
creation preserves annotation text and actual process diagnostics. After g,
start t with the retained name, explicitly choose annotated and accept the same
target to recover its draft. T reviews the selected local tag's full ref object
and peeled target before deletion. Creation and deletion do not publish tags;
HEAD and index remain unchanged. An annotated tag retains its distinct tag
object and target instead of treating both as a commit ID.

h or l l in Status opens History. `keys.git_history` keeps navigation separate
from Status in both profiles. Tab inspects the exact selected revision; f admits
one more finite ancestry page. g verifies current HEAD and preserves the pinned
root, admitted pages and compatible selection; a changed HEAD requires Status
refresh and explicit History reentry. Missing objects and page limits are explicit.
Revision detail includes full parent IDs, raw commit metadata, exact old/new
changed paths and a bounded read-only patch. Invoke `git-show` in detail to
compare another listed full parent ID. Return on a parent compares it; Return
on a changed path opens its immutable historical bytes. Deleted paths use
the comparison's old object. q returns to History, then Status. A/F in
History prepares amend of checked HEAD or fixup of the selected full revision;
existing explicit commit review remains required. Inspection never checks out
historical files or replaces current source text.

In History, * records one range base and Return compares that full base with
the selected endpoint in their explicit order. a appends an exact revision and
U removes it from the independent ordered set. Base changes retain revision
order; sequence changes retain the base. Duplicate and excess marks reject;
at most 256 ordered revisions remain retained. `git-unmark` removes the selected
base and `git-unmark-all` clears both History roles. Incompatible repository/root
marks remain explicit until cleared or their pages are admitted again.

l r opens Reflog; B b or B B in Status blames the selected exact path, with the
visiting repository-backed source as the fallback. Blame uses checked HEAD,
so unstaged or unsaved source text is not attributed as committed content.
`keys.git_inspect` owns revision, reflog, blame and historical-file navigation:
Return opens retained detail, v inspects the exact selected revision, and q
returns to its originating list. Blame retains each original object, filename
and line through renames; historical source visits read the exact blob without
checkout or source publication. Historical bytes remain read-only and visibly
bounded, including binary or truncated content. `git-visit-file` explicitly
visits the current canonical source; existing modified buffers remain owned by
the editor. A historical path with no current file reports that fact.

The `keys.git` map owns q, g, s/u, Tab, Return, ? and list navigation in both profiles.
M-x and colon use the shared command catalog. Unmerged, Untracked, Unstaged
and Staged sections appear in that order when nonempty. Clean, bare, unborn,
detached, unavailable and truncated states remain explicit. Pointer selection
uses a current frame and semantic path identity; display escaping never
reconstructs a filesystem argument.

`$` opens Processes and Return inspects the selected operation. Contextual C-g
in Processes or process detail explicitly cancels that selected live operation;
inside a menu or prompt it dismisses only that input owner. Escape dismisses
input without cancelling a child. Each q returns one level and preserves work.
Application exit identifies active Git work and requires a fresh decision
before termination, alongside unsaved-source and terminal decisions. Accepted
cleanup retains ownership until reaping; failure keeps the editor open. Stopping
a process does not abort a Git sequence or claim repository rollback.

In Status, c opens normal commit composition directly. A following c is message
text; plain Return inserts a newline. `keys.git_commit` inherits ordinary native
editor keys in both profiles: C-Enter or C-c C-c submits, C-g or C-c C-k cancels
and retains the draft. Git keeps identity, hooks and signing policy; rev never
saves or stages source text implicitly. Failed or uncertain effects retain the
message and actual child diagnostics in Processes. Drafts are session-only,
limited to 65536 bytes per context, with bounded native undo history.

`git-commit-amend` prepares the observed HEAD and existing message.
`git-commit-fixup` prompts for one revision, resolves a full immutable target and
prepares its `fixup!` subject; retain that exact subject while editing the body.
For these modes, the first C-Enter or C-c C-c opens a read-only exact review.
The review shows the complete target, checked HEAD, message byte count and
final-newline state; Up/Down scroll the wrapped content. Plain Return does not
accept. C-Enter or C-c C-c explicitly accepts; q or C-g cancels and retains
the draft. A moved HEAD or changed index rejects before the mutator. Amend
preserves Git author/parent semantics; fixup creates one ordinary child commit
with the standard message and does not run autosquash.

### Native Git fetch

Select a configured remote row in Refs, then use & or M f, or invoke
`git-fetch` through M-x or modal colon. M b reopens Refs. Missing selection
reports its requirement instead of choosing an upstream or path. Fetch honors
the selected remote's configured transport and refspecs, including the installed
Git credential/helper policy. Configured pull policy does not trigger integration.
The native request checks complete current refs, HEAD and its private effective
configuration before launch. Changed configuration requires a new checked request.

Fetch uses `git.transport_timeout_seconds` and the existing aggregate output,
metadata, record and session limits. Leaving Git retains owned work; cancellation
and deadlines retain actual possible effects. Processes keeps safe argv, actual
exit, outcome and checked local facts. Raw transport/helper and configuration
output is withheld because it can contain credentials. A failed or unverifiable
fetch is not automatically replayed. Refreshed tracking facts are local
observations; they do not promise exclusive access against other Git clients.

### Native Git push

In Refs select one local branch and invoke p / `git-push`, or mark it with *
and navigate to a configured remote before M p. The remote selection constrains
the configured upstream; it cannot replace it. The one push-source mark stays
independent of index/path/hunk marks and History range/sequence marks. - clears
it. A changed marked object requires clearing or marking its current object again.

Preparation requires exactly one `branch.<name>.remote` and full branch
`branch.<name>.merge`, consistent with Git's observed upstream relation. Missing
or contradictory tracking rejects. The native scrollable review shows the full
source ref, full source object, configured remote name and full destination ref.
Only y accepts; Return does not submit, and n or C-g cancels. Configuration and
source facts are checked again before launch; an accepted review is used once.

Mirror push, automatic tag following, recursive submodule publication, multiple
push URLs, conflicting push remotes or configured refspecs reject with a visible
reason. A compatible explicit source/destination refspec is allowed. Git's
`push.default` cannot broaden the explicit full-object refspec. The installed
backend receives explicit no-force/no-expansion flags after compatible policy
admission. Hooks, credentials, push signing and receiver protections remain Git-owned.

The result refreshes local facts and queries the privately captured effective
push endpoint for the exact full destination. A successful child alone does not
prove publication. Failed children, unavailable verification, differing remote
objects and cancellation/timeout after possible effects remain distinct in
Processes. Failure is never automatically retried. Recognized noninteractive credential/authentication failures get a fixed safe
diagnosis. Transport/helper output and credential-bearing endpoints remain private; records retain safe argv, actual
exit and checked outcomes under the existing finite transport/output limits.

### Native Git integration

In Refs select one exact commit-bearing ref. = or m f chooses fast-forward-only
merge; i or m m permits fast-forward or merge; I or m n requires a merge commit
when Git has integration work. e or r r chooses ordinary rebase. Native review
shows the selected full ref/commit, current branch/HEAD and explicit policy.
y, C-Enter or C-c C-c accepts; Return does not accept. n or C-g cancels.

Integration requires complete current refs, a checked existing current branch,
clean index/worktree and unchanged unmodified repository-backed source buffers.
Stale HEAD/ref facts reject. Configured autostash, rebase updateRefs or branch
mergeOptions which would alter the reviewed merge request require an explicit
configuration revision. Requests do not save source or introduce autostash,
interactive editors or executable todo actions. Git retains hooks/signing and
its receiver-independent repository protections.

`git-operation-status` opens the read-only Git Operation view. A conflict stop
shows its actual family, current/original/onto objects and available step,
plus exact conflict paths. Processes reports InProgress separately from verified
integration and retains the actual failed child exit. q returns to Status
without aborting Git state; process cancellation also does not abort it.
Changed source reconciles through the existing reload/ask policy.

Sequence metadata admits 64 ordinary files and 256 merge targets, within the
existing detail, 1 MiB worker scratch and 32 MiB shared session limits. Symlinked
or oversized state cannot authorize work. Snapshot publication checks sequence
bytes and HEAD again. Integration uses `git.operation_timeout_seconds`;
observation, output, record and cleanup bounds retain their existing meanings.

Conflict inspection retains full index object IDs, modes and bounded side bytes.
For rebase, Current is the destination/onto side and Applied is the original
commit being replayed. For revert, Base is the reverted commit and Applied is
its selected parent. Missing, binary, non-text, structural and partial sides
remain visible. The read-only view never replaces unsaved source text.

In Conflict, 1/2/3 prepares a complete compatible side replacement, K reviews
exact deletion from disk and index, and x prepares staging of saved bytes or
absence. y, C-Enter or C-c C-c accepts the exact review; n or C-g cancels it.
An unsaved visiting result blocks these actions. Binary, non-text, type-change,
submodule, incompatible-mode and partial observations remain inspection only.
Changing the saved file, index stages, HEAD or sequence invalidates confirmation.
Removing markers does not stage or continue; continue is always separate.

Conflict H uses the configured optional Pi owner with copied input only and no
directory discovery or tools. Saved disk text and any visiting unsaved result
remain distinct snapshots. Exact stage IDs, modes, roles and unavailable content
are explicit; binary or partial content is not presented as complete text.
Context exceeding the existing Git detail or AI input cap rejects as a whole.
An unavailable provider leaves the manual conflict view usable. The normal
read-only AI result and discard workflow preserve native source ownership.
Index/file metadata and native revisions conservatively invalidate source
currency; an explanation never replaces exact Git checks for changing actions.

### Prepared clone and init arguments

Status C C, C s, C b and C m open regular, shallow, bare and mirror clone
preparation. I I prepares init. Named clone/init commands also work from ordinary
editing or a chosen file-tree directory without an existing Git repository.
The native text draft contains one JSON array of literal Git arguments. Fill its
empty source/destination parameters; keep its family, options and parameter
positions. Shallow preparation uses depth one. Clone and init require an absent
explicit destination with an existing canonical ordinary parent directory.

The shared native composer uses ordinary editor text keys in both profiles.
C-Enter or C-c C-c first checks input, captures saved source state and opens
exact review of the executable, literal argv, directory and destination. A second
C-Enter or C-c C-c executes only that captured plan. Plain Return never accepts
review. q or C-g cancels review; C-g cancels composition. No shell expansion,
implicit source save, operation retry or destination deletion is performed.
Status changes follow the existing observation/reload owners. Git process
inspection retains bounded exit/cleanup facts; transport output is withheld to
protect credentials. Failure after launch does not claim repository rollback.

Status i t and i p visit the actual .gitignore and common Git info/exclude
files. Existing unsaved visiting identities stay owned; no pattern is invented.

### Prepared submodule arguments

Status o a, o l, o u, o s and o d prepare submodule add, recursive status listing,
update, URL sync and non-forced deinit. Fill the add metadata name, source URL
and literal repository-relative path, or the selected path for update/sync/deinit.
Update
uses the indexed gitlink commit and explicit checkout mode. The captured path
must stay inside the working tree through ordinary directory components.

The same two-step native composer opens exact executable/cwd/argv/target review.
Preparation captures the parent sequence, index, saved bytes, configuration and
selected child repository state. Changed evidence refuses execution. Update and
deinit refuse dirty child state; unsaved repository-backed buffers protect all
changing routes. Deinit also protects retained native visiting/directory owners.
No force, hidden stash, save, commit or retry is implied. Listing retains its
actual bounded output under git-processes; other raw configuration/transport
output stays private. Successful changing results verify actual gitlinks,
checkouts, URL registration or deinitialization. Relative sync URLs whose
resolved value cannot be verified remain explicitly unverified after execution.


### Prepared subtree arguments

Status O a/O m/O f/O p/O s prepare subtree add, merge, pull, push and split.
Each route requires a checked working tree, complete refs, clean saved state,
no active Git sequence and an explicit literal repository-relative prefix.
Add requires an absent prefix with an existing ordinary parent. Other routes
require an existing ordinary directory. Missing optional Git subtree support
reports its installed availability before any changing child.

Add and pull name a source repository and one full refs/heads branch; read-only
preparation pins its complete object ID. Merge names an exact local commit.
Split names an explicit commit, such as HEAD, and retains its resulting full
commit object and actual stdout under process inspection. Native composition,
read-only preparation and exact review remain separate from execution.

Push requires an explicit repository URL or absolute path and one full
refs/heads destination. Review shows that full branch and the exact argv operand
used by Git subtree, which adds refs/heads itself. Configured remote names,
active URL rewrites, implicit followTags and recursive publication are refused.
The checked remote branch and captured effective configuration must remain
unchanged through review. No force, inferred upstream or additional publication
is requested.

Postchecks inspect actual branch/ref state, saved paths outside the prefix,
imported and split tree objects, and the remote subtree object after push.
Unsaved native sources remain protected. A real merge conflict retains Git's
actual sequence and reports in-progress state; git-operation-status inspects it
before a separately reviewed continuation or abort. Failures preserve uncertainty
and never retry, clean up or claim rollback automatically.

### Prepared notes arguments

Status T a/T r/T p prepare notes add, remove and prune. Fill one full refs/notes
name and an explicit existing target object for add/remove. Add also requires a
literal nonempty message; it refuses to overwrite an existing note. Preparation
resolves the full target ID and captures the exact target-to-note map and Git
configuration. Prune review lists the full missing-object targets returned by
Git's read-only dry run. A changed ref, note map, prune set, saved source or
configuration invalidates review before execution.

Execution verifies the exact resulting note map. Add checks the actual note
blob against the reviewed message after Git's normal whitespace cleanup; remove
and prune must preserve all other notes. HEAD, unrelated refs and saved index,
working and untracked bytes stay checked. Bare repositories support the same
notes operations through their actual Git context. Successful add reports the
full note blob ID. Process inspection owns actual exit/cleanup facts; failed postchecks remain explicitly unverified.

### Prepared sparse-checkout arguments

Status > e/> d/> r/> s/> a prepare enable, disable, reapply, set and add.
The routes require a checked clean working tree, complete saved/index/ref state,
no active sequence and no unsaved repository-backed buffers. Enable and set
select cone mode without sparse index compression. Set/add accept one or more
literal repository-relative directory arguments after --. Paths must use
ordinary components outside .git. The reviewed --skip-checks option permits
literal pattern metacharacters after native path checks. Git cone patterns cannot
represent a newline directory, so that input refuses before execution.
Raw supported path bytes remain exact at the Git owner;
native JSON text supports ordinary Unicode paths without shell or glob expansion.

Review captures effective configuration, the selected worktree's private sparse
patterns, complete index flags and actual materialization. Existing enabled
non-cone patterns or inconsistent materialized excluded files refuse these
prepared routes. Reapply/add require enabled cone mode. Parent selections absorb
redundant child directories according to Git's cone selection.

After execution, checks inspect the actual cone configuration, selected directory
list, index skip flags and tracked-file presence. HEAD, index objects, saved
changes and unrelated configuration remain checked. Linked worktrees retain
separate pattern and materialization state. No force, implicit save/stash,
automatic cleanup, retry or rollback is performed. Plain Return and cancellation
follow the same native argument composer contract as other prepared families.


### Prepared patch and request output

Status W c/W s/W r prepare format-patch, saved diff and request-pull output.
These drafts contain a JSON object with an arguments array and an explicit
output path, for example {"arguments":["diff","--binary","--full-index",
"--no-ext-diff","--no-textconv","--no-color","HEAD~1","HEAD","--"],
"output":"review.diff"}. Relative output names resolve against the captured
repository root. The destination must be absent, with an ordinary canonical
parent outside Git metadata. Existing entries are never overwritten.

Format-patch requires one local BASE..END commit range; saved diff requires two
local commit expressions. Preparation freezes full IDs and disables replacement
objects. Binary/full-index output permits real Git mailbox/plain-patch consumers.
Only saved commit state is exported; an unsaved native source stays owned and is
not saved implicitly. A bare repository can supply the same checked commit range.

Request-pull requires an explicit repository URL or absolute path, a local start
commit and one full remote refs/heads branch. Its end parameter can also use
LOCAL:REMOTE to select a different local commit expression. The remote branch
must already name the reviewed local end commit; this route does not publish it.
Active endpoint URL rewrites refuse preparation. Request text is saved locally;
no mail is sent.

The shared owner captures complete output during read-only preparation. Review
shows the exact executable/mode/cwd/argv, destination, byte count and Git blob
hash of the captured bytes. This hash does not write an object to the database.
A separate acceptance regenerates the output and requires exact byte equality,
unchanged source/configuration and the same published request-pull anchor.
Output exceeding the existing detail/scratch or aggregate process limits rejects
as a whole. No partial export file is presented as a complete result.

File publication reuses the existing owned staging lifecycle through a captured
directory descriptor and an atomic hard link that cannot replace another entry.
Postchecks read the actual saved bytes. find-file visits the explicit output;
process inspection retains the actual producer's exit/cleanup facts. Changed
review evidence refuses execution. Failed writes preserve uncertainty and never
retry, delete the chosen destination or claim rollback.
