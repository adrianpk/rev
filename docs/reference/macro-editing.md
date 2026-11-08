# Macro editing and ring menu

All edits replace a checked session snapshot. Named definitions, anonymous key
bindings and ring entries remain independent. Editing never writes the
[saved library](macro-library.md); use explicit named save afterward.

## Definition drafts

C-x C-k Enter opens the current ring head as one schema-1 TOML record in an
editable native buffer. C-x C-k C-e enables ring repeat keys after application.
C-x C-k e selects a named command or an existing session key binding. The
record identity, revision, name and binding pin that target; changing those fields
does not retarget the draft. A named description may be edited.

C-c C-c validates the complete document and replaces only the original target.
Invalid syntax, changed target revisions, removed targets and exceeded bounds
leave the draft editable and the previous definition intact. Normal kill-buffer
closes a draft without applying it and restores its source view and source mode.
Draft text and undo belong to the draft buffer; application adds no source undo.

C-c C-q captures the next native key sequence as inert step tables. Uppercase
characters are literal keys here, without the binding prompt's reserved slot
expansion. C-c C-r accepts a region of complete step-table lines and projects
only those steps into the draft body. It preserves pinned metadata and still
requires C-c C-c. The same 4,096-step and 256 KiB encoded-record limits apply.

C-x C-k l builds a new draft from recent recordable logical input, capped at
300 inputs and 256 KiB. Accepted M-x/colon commands appear once with their native
prompt values. Protected component input, selector navigation, terminal input
and expanded playback are excluded. Opening or editing the draft has no source
effect; applying adds a new session ring entry.

## Step editing

C-x C-k Space starts the selected ring macro through the ordinary checked runner
and asks before the next logical step. The prompt shows that step and ? help.
Waiting for an answer has no automatic macro wake.

| Answer | Effect |
| --- | --- |
| Space or y | Execute one step and ask again. |
| n, d or Backspace | Remove the step from the pending definition. |
| f | Skip this step for the current execution only. |
| Tab | Execute adjacent ordinary text input as a group. |
| c | Continue the remainder without questions. |
| C-k | Remove the remaining steps. |
| i or r | Record native input before, or instead of, the current step; finish with C-j at a complete command. |
| I or R | Capture and execute one native key sequence before, or instead of, the current step. |
| a | Execute the current step, then record input after it. |
| A | Execute the remainder, then record appended input. |
| q or C-g | Cancel definition changes while keeping real source edits. |

Lowercase input uses ordinary command, prompt, completion and source owners.
Its accepted effects already occurred and are not executed twice. Successful
completion replaces the pinned ring definition once. Invalid input, stale targets,
source loss and cancellation preserve the previous published body. Real partial
edits remain in the normal native undo group. Automatic execution shares the
usual time, input, iteration, text and nested-call limits; human waits pause the
active clock, and C-g remains available.

## Read-only ring menu

Run list-keyboard-macros through M-x or modal colon. Rows show position, optional
name, counter, format, rendered counter and a clipped body summary. Row identity
survives reorder; source text and its mode remain owned by the original view.
Tab/BackTab or Left/Right choose a column. Up/Down choose a row. C-SPC starts a
menu row region, which takes precedence over marks, then the current row.

| Key | Action |
| --- | --- |
| # | Move the selected row to a checked zero-based position. |
| C-x C-t | Transpose the selected row upward. |
| c, f or e | Edit counter, format or definition. |
| Enter | Edit the current property column; the name column uses native naming. |
| d, x | Flag rows, then apply checked flagged deletion. |
| m, u, Backspace, U | Mark, clear selected, clear previous or clear all marks and flags. |
| C | Copy selected rows; multiple targets require confirmation. |
| D | Confirm immediate removal of selected ring rows. |
| g, q, ? | Refresh, close to the original source, or show local help. |

Property prompts pin row identity and revision. Stale rows refresh before another
mutation. A canceled confirmation preserves all selected rows. Ring removal never
deletes named snapshots or saved records. Definition editing returns to the menu;
refresh reflects the new revision. Menu input cannot edit source text. Every local
action also has a native [command name and description](commands.md#keyboard-recording-and-replay)
and a configurable [binding](bindings.md#keyboard-recording-and-replay).
