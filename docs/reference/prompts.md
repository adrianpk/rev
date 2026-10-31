# Focused prompts

The active prompt owns keyboard input until its current stage accepts, reports
an error or is canceled. Find-file retains focus on ingress errors so its path
can be corrected. Command names, paths, buffer names and decisions use
separate focused contexts. Their text does not enter the document underneath.
M-x and find-file queries appear above their vertical candidates when surplus
rows fit; other argument and decision prompts retain the bottom prompt row.

## Stages

| Operation | Sequence |
| --- | --- |
| M-x | Command query, then the selected command's handler and any arguments. |
| Goto line | Signed decimal line number; invalid or overflowing input retains the draft for correction. C-g preserves point and viewport. |
| Query replace | Literal query, literal replacement, then a fresh y/n/q decision for each occurrence. C-g stops at any stage; accepted edits remain independently undoable. |
| Find file | Selected or typed path; directories open their tree, files open/reuse, and ingress errors retain the draft. |
| Write file | Destination path, then an explicit decision if the destination is occupied or a different referent requires consent, then save. |
| Save buffer | Save the visited path, or request a destination if unnamed; external changes require a separate explicit decision. |
| Switch buffer | Live buffer name; an empty argument selects the current buffer. |
| Kill buffer | Live buffer name; clean text closes immediately. Modified text asks whether to save, discard or cancel. |
| Session exit | Each modified buffer in stable identity order, then save/discard/cancel; an unnamed save requests a path and save conflicts require separate consent. Exit follows only all decisions and completed required saves. |
| Save before closing | Named text follows its save checks; unnamed text requests a destination. A destination conflict requires its own decision. Close occurs only after a completed save of the captured target. |

Buffer-name Tab completion uses live names and their common prefix; Return
selects an exact name. Find-file starts in the visiting file's parent directory,
otherwise the chosen directory context or launch cwd. Its blank draft filters that base directory; relative paths use
that base, while absolute paths and leading `~/` choose explicit directories.
The query stays above one candidate per row, bounded by
`minibuffer.candidate_limit` and available geometry. Directory labels end in `/`.

In find-file, Up/Down or C-p/C-n moves the selection without changing the draft.
Tab fills an explicitly selected or unique name, or extends the shared prefix at
whole-grapheme boundaries. Completing a directory keeps its separator and lists
that component; deleting or editing the path back to a parent refreshes it.
Typing within the same component reuses the cached observation. Explicit Tab can
refresh disk changes; drawing and resize do not read disk. Incomplete observations
show a warning and require an explicit selection for completion rather than
assuming uniqueness from a partial list. No match or refused completion retains
the draft and focus. C-g cancels without visiting a candidate or editing text.

Return after explicit candidate navigation accepts that exact observed path,
independently of its safe display label. With a freely typed draft, Return accepts
the draft even when suggestions exist. The initial highlight alone does not
choose a candidate. An accepted directory closes the prompt and opens its
visible, first-level-expanded tree without replacing document text. An empty
accepted draft opens the base directory. Tab completion stays inside the prompt.
Directory symlinks remain list leaves until explicitly accepted. A removed
selected path reports an error instead of becoming a new file implicitly.

File acceptance uses canonical visit ownership: aliases reuse existing dirty
buffers and their history without rereading or saving. A new filename creates
only an in-memory buffer with the startup language and new-file format defaults;
it creates neither files nor missing parents. Existing files retain their bytes,
separators, BOM and final-newline state. Failed ingress retains the path, focus
and original buffer; edit and retry, or cancel with C-g. Unsupported byte names
remain visible but cannot open under a replacement spelling.

Write/save path arguments keep their draft on Tab and display
`Enter a path; C-g cancels`. [M-x completion and history](commands.md) operate
only in command focus. See the [filesystem candidate contract](path-completion.md)
for observation bounds and exact path/label identities.

Query replace has separate literal argument and occurrence-decision contexts;
its multiline paste and cancellation rules are in [Search and replacement](search.md).

## Editing and decisions

Argument text supports Left/Right or C-b/C-f, Home/End or C-a/C-e, Backspace
or C-h, and Delete or C-d. Movement and deletion respect whole graphemes.
Single-line Unicode paste inserts at the prompt cursor. The existing limit is
4096 Unicode scalars; control-containing and oversized payloads reject wholly.
Successful edits clear outdated input/completion feedback without changing focus.

Save decisions accept only a fresh y/n press. Invalid keys retain the question;
Return has no affirmative default, and paste cannot answer. At the modified-buffer
close question, y means save and then close, n means discard and close, and C-g
cancels. At a save conflict question, n cancels the save and any pending close.
Repeated Return or a held decision suffix cannot accept the next stage.

## Completion, failure and cancellation

Return consumes ordinary arguments once before their handler continues. Goto-line
reopens invalid numeric input with its draft and feedback. Find-file
keeps its draft until target admission succeeds or C-g cancels. Directory
acceptance closes the prompt and presents the chosen tree. An unknown buffer or failed save returns to editing
with its error and retains the source text. A failed required save keeps the buffer open. A later independent
save does not inherit an earlier canceled close request.

C-g drops the current pending interaction, including nested path/decision stages.
It preserves the selected buffer, point and current navigation state; normal C-g
region deactivation still applies. Canceling M-x does not reset the intended
vertical column or recenter cycle. A name that is accepted runs its actual command
with the same navigation behavior as its binding.

Resize remains actionable during every stage, including tiny or zero geometry.
The prompt and pending operation survive; presentation follows the current
terminal size. A restored editing context accepts fresh input once, without
replaying canceled text or carrying save/close authority into another command.

## Session exit

`C-x C-c` and `M-x save-buffers-kill-terminal` use the same coordinator. A clean
session exits immediately. Each modified buffer is visited once, including
hidden buffers, modified unnamed text and removed-path warnings. Several
windows of the same buffer do not repeat its question. The selected editing
view and workspace focus remain in place while the bottom area names the buffer.

At each quit question, y saves, n authorizes discard for this exit only, and
C-g cancels the entire exit. An unnamed save requests its destination; an
occupied target or changed/missing visited file requires separate consent.
Declining that save confirmation stops exit, rather than discarding the buffer.
C-g at any path or decision stage returns to the live session. All buffers,
windows, points, marks and unsaved history remain; completed earlier saves stay
saved. Discard choices do not mark text clean or survive a canceled attempt.

Read-only rejection, missing parents, failed writes and publication with a later
unresolved completion all stop exit and report the actual save result. Only a
complete save of the current buffer state advances to the next buffer. A later
independent save cannot resume a stopped exit. The terminal guard restores raw
mode, alternate screen, bracketed paste and cursor state after an accepted exit.

A save observes its destination before temporary output and rechecks it after
staging, before replacement. Changes detected there stop publication and clean
up the owned temporary file. The external destination stays as observed; retry
saving to obtain a fresh conflict decision. This is a conservative metadata
check, not a guarantee against changes in the final interval before replacement.

## Supplied prompt preferences

Executable startup and library sessions supplied with an admitted snapshot use
`minibuffer.input_limit` (1..=65536 Unicode scalars, default 4096) across command,
argument, incremental-search and query-replace text. The bound covers initial
text, typed keys, whole paste payloads, completion and history recall. An excess
reports `Prompt input limit exceeded` and retains the current draft, cursor and
focus; C-g remains available. Grapheme editing remains independent of this scalar
count. A short command query may invoke a longer fixed catalog name, but that
name cannot be completed or recalled into text past the bound.

`minibuffer.history_limit` admits 1..=4096 distinct accepted command names
(default 128), newest first. Repeating a command promotes it without consuming
another entry; overflow evicts the oldest. Cancellation retains session history
and rejected input adds nothing. History is not persisted. Startup reads these
preferences once; source edits take effect after restarting.
