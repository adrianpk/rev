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

## One-shot shell commands

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
