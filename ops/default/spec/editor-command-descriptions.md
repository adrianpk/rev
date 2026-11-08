# Command Descriptions

Status: Completed
Completed: 2026-11-07
Validated candidate: `978deacc49f6f5f948cea2070919ce91031f67a2`
Gate log: `A6S4H3BKQFKIG4WEYTWGATYYE4`
Date: 2026-11-07
Approved: product owner; complete all built-in descriptions in one implementation slice
Format: OpenSpec capability addition
Inspection baseline: `c8ccf85472618bf61d8860d998d71cd39ebb1bf2`
Catalog snapshot: 618 canonical built-in commands
Ticket: [TKT-20261106202027](../ticket/solved/20261106202027-complete-command-catalog-descriptions.md)
Plan: [Command Descriptions Delivery](../plan/rev-command-descriptions.md)
Tracker: [Command Descriptions Tracker](../tracker/rev-command-descriptions.md)
Baseline: [Command Keybinding Hints](editor-command-keybinding-hints.md), KBH06

## Purpose

Give every built-in command a useful description beside its name and effective
keybinding in M-x and the shared modal colon selector. This specification defines
the complete description text before implementation; it is not a list of
examples from which the remaining copy must be guessed.

Here, a function means a named editor command admitted by `Command::ALL`, not
an internal Rust helper. Bindings remain optional. The catalog below includes
all 618 built-ins at the inspection baseline, including context-local and
unbound commands. Descriptions explain existing actions; they do not introduce
new commands or change what existing handlers do.

## Source and ownership

The inventory was checked against the compiled `rev::command::Command::ALL`
from the inspection baseline. Behavioral definitions were reconciled with the
[editor bindings](../../../docs/reference/bindings.md),
[Org bindings](../../../docs/reference/org/bindings.md), and their linked native
references. Grouped reference entries have separate descriptions here; for
example, table import and export do not share an ambiguous combined summary.

The existing `Command::description()` accessor returns optional static text,
and the selector already paints a third description column. Implementation
fills that metadata through the existing command/family owners. This change
adds no configuration, runtime state, persistence model or registry of actions.

## ADDED Requirements

### Requirement: CDS01 Define every built-in command

Every canonical command in the live built-in catalog SHALL have exactly one
nonempty, single-line English description. The inventory below SHALL define
the exact selector text for this baseline. A command added during delivery SHALL
receive an explicit definition and matching metadata in the same change.

Aliases SHALL reuse their canonical action's description. Parameterized internal
operations do not become additional named commands merely to supply metadata.

#### Scenario: Complete shipped catalog

- **GIVEN** the current built-in registry
- **WHEN** its description coverage is checked
- **THEN** all 618 baseline names have descriptions matching this inventory
- **AND** missing, duplicate, unknown or empty definitions fail the check.

### Requirement: CDS02 Describe the action accurately

Descriptions SHALL state the user-visible action in plain technical English.
Use short, concrete sentences. Distinguish opposite actions and context-local
operations. Do not infer a description by replacing hyphens in a command name,
copying user arguments or reading source-buffer content.

Short summaries do not replace the existing command contracts. Availability,
confirmation, source protection, cancellation and configured behavior remain
owned by the existing handlers and references. A description SHALL NOT imply
that invoking an action bypasses them.

#### Scenario: Opposite table actions

- **GIVEN** `org-table-import` and `org-table-export`
- **WHEN** their candidates are displayed
- **THEN** each summary identifies its own direction of data transfer.

### Requirement: CDS03 Keep metadata with command identity

The implementation SHALL populate the existing read-only accessor through
canonical command/family metadata. It SHALL NOT add a second dispatch registry,
lookup by rendered row index or filesystem access during drawing.

User-defined AI commands retain the existing optional-metadata contract. Their
instructions, provider details and user data SHALL NOT become descriptions.
Missing optional descriptions remain blank, without dashes or placeholders.

#### Scenario: User command without a summary

- **GIVEN** a registered user-defined command without description metadata
- **WHEN** the selector lists it beside described built-ins
- **THEN** its description field stays empty and its invocation remains usable.

### Requirement: CDS04 Preserve selector behavior

M-x and modal colon SHALL use the same descriptions and existing three-column
presentation: name, adjacent effective shortcut, description. Effective hints
continue to come from the actual originating keymaps and configured overrides;
they SHALL NOT be hardcoded into this inventory.

Preserve name-only matching, recency/frequency ranking, Tab text, Return
invocation, selection and usage identity. Clip descriptions at terminal-cell
boundaries and omit them where space is unavailable. Preserve existing colors,
selected-row contrast and NO_COLOR behavior; do not redesign the selector.

#### Scenario: Learn a bound command

- **GIVEN** a sufficiently wide selector with `save-buffer` bound to C-x C-s
- **WHEN** its row is displayed
- **THEN** the row shows its canonical name, effective shortcut and description
- **AND** Tab completes only `save-buffer`.

#### Scenario: Narrow or monochrome selector

- **GIVEN** a narrow terminal or NO_COLOR presentation
- **WHEN** descriptions do not fit beside the higher-priority fields
- **THEN** ordinary clipping and omission preserve names and complete hints
- **AND** selection and command invocation retain their existing meaning.

### Requirement: CDS05 Keep documentation and future additions aligned

A native catalog guard SHALL compare the live built-ins and their nonempty
single-line descriptions with this inventory. Extend existing catalog checks;
do not create one test or terminal journey per command. Review representative
bound/unbound and context-local rows in both editing profiles, including optional
user-command metadata. Retain the existing Unicode and narrow-layout coverage.

Update the command reference, one relevant User Guide example and Unreleased
notes for the completed behavior. Markdown remains the source of the Spore guide;
do not maintain a separate description list in publication templates.

#### Scenario: A new command omits its description

- **GIVEN** a future canonical command is added without a summary
- **WHEN** the native catalog guard runs
- **THEN** it fails and identifies the missing command.

Current catalog: 624 built-in names: 623 canonical commands and the `quit` alias.
The alias reuses the exit action's description and effective shortcut. Four Org
visibility commands were added by the Org Initial Visibility delivery.
`tree-up-directory` is added by File Tree Parent Navigation. The 618-command snapshot
and completed delivery evidence below retain their original baseline meaning.

## Approved description inventory

Each row gives a canonical name and its exact plain-text selector description.
Sections group entries for review; section names are not runtime categories.
Full behavior and prerequisites remain in the linked command references.

### Editing and session

| Command | Description |
| --- | --- |
| `backward-char` | Move back one grapheme. |
| `backward-delete-char` | Delete the preceding grapheme. |
| `backward-kill-word` | Kill toward the preceding word start. |
| `backward-word` | Move to the preceding word start. |
| `balance-windows` | Equalize existing pane divisions, preserving focus and startup split policy. |
| `beginning-of-buffer` | Move to buffer start. |
| `context-close` | Resolve current source work, close its context and return to the startup screen. |
| `copy-rectangle-as-kill` | Retain selected rectangle rows. |
| `dashboard-open` | Retain current work and show the native startup actions. |
| `delete-char` | Delete the following grapheme. |
| `delete-other-windows` | Keep only the selected view. |
| `delete-window` | Close the selected view, retaining its buffer. |
| `end-of-buffer` | Move to buffer end. |
| `exchange-point-and-mark` | Exchange point and mark. |
| `execute-extended-command` | Find and invoke a command. |
| `find-file` | Open a file or directory. |
| `forward-char` | Move over one grapheme. |
| `forward-word` | Move to the next word end. |
| `generate-new-buffer` | Add an unnamed buffer in the active context. |
| `goto-line` | Go to a one-based logical line number. |
| `insert-tab` | Insert indentation using the active tabs and spaces settings. |
| `isearch-backward` | Incremental literal backward search. |
| `isearch-forward` | Incremental literal forward search. |
| `keyboard-quit` | Cancel the current interaction or deactivate the region. |
| `kill-buffer` | Close a buffer with modified-text protection. |
| `kill-line` | Kill to logical line end, then its separator. |
| `kill-rectangle` | Remove and retain the rectangle as one edit. |
| `kill-region` | Remove the active linear region into the ring. |
| `kill-ring-save` | Copy the active linear region. |
| `kill-word` | Kill toward the next word end. |
| `list-buffers` | Select from the live buffer inventory. |
| `mouse-mode` | Toggle native pointer support for this session without writing settings. |
| `move-beginning-of-line` | Move to visual line start when wrapped, otherwise logical line start. |
| `move-end-of-line` | Move to visual line end when wrapped, otherwise logical line end. |
| `newline` | Insert the buffer's preferred line separator. |
| `next-line` | Move down one visual row when wrapped, otherwise one logical row; retain the intended display column. |
| `open-directory` | Choose an existing directory and replace the context after source decisions and owned cleanup. |
| `other-window` | Cycle editing views. |
| `previous-line` | Move up one visual row when wrapped, otherwise one logical row; retain the intended display column. |
| `query-replace` | Confirm each literal replacement. |
| `quit` | Exit rev after resolving unsaved work and active processes. |
| `recenter-top-bottom` | Cycle point's line through center/top/bottom. |
| `rectangle-mark-mode` | Toggle display-column selection. |
| `revert-buffer` | Reload a checked disk image; confirm before discarding unsaved text. |
| `save-buffer` | Save to the current destination. |
| `save-buffers-kill-terminal` | Exit rev after resolving unsaved work and active processes. |
| `scroll-down-command` | Move back one page. |
| `scroll-up-command` | Advance one page. |
| `set-mark-command` | Activate a linear region at point. |
| `split-window-below` | Split into upper/lower views. |
| `split-window-right` | Split into left/right views. |
| `switch-to-buffer` | Select a live buffer by name. |
| `tabify` | Compress the marked whitespace at effective tab stops when shorter. |
| `text-mode` | Select ordinary text editing. |
| `undo` | Undo a buffer-local group. |
| `undo-redo` | Reapply the next redo group. |
| `untabify` | Expand marked literal tabs while preserving text columns. |
| `visual-line-mode` | Toggle word-aware wrapping for this buffer and all its views without changing source or settings. |
| `windmove-down` | Focus the nearest overlapping pane below. |
| `windmove-left` | Focus the nearest overlapping pane to the left. |
| `windmove-right` | Focus the nearest overlapping pane to the right. |
| `windmove-up` | Focus the nearest overlapping pane above. |
| `write-file` | Save the current buffer to another destination. |
| `yank` | Insert the newest linear ring entry. |
| `yank-pop` | Rotate the current yank's exact insertion range. |
| `yank-rectangle` | Insert retained rows with required padding. |

### Projects and Workspaces

| Command | Description |
| --- | --- |
| `project-create` | Choose a directory, name it and open the registered Project. |
| `project-duplicate-as-workspace` | Resolve source decisions, retain the Project and open an independent Workspace copy of its file session. |
| `project-forget` | Remove Project metadata; active live work becomes a temporary directory. |
| `project-list` | Choose a registered Project and restore its file session. |
| `project-promote` | Register the temporary directory while retaining live work. |
| `project-promote-to-workspace` | Convert a Project atomically to a fresh Workspace; active dirty text stays live. |
| `project-rename` | Select a Project and change only its display name. |
| `project-set-root` | Select a Project and an existing replacement directory without moving files. |
| `project-switch-project` | Restore a Project selected by name/path. |
| `workspace-add-directory` | Append an existing directory to the active Workspace. |
| `workspace-add-project` | Append a Project directory without importing its saved files. |
| `workspace-create` | Name and open an empty Workspace. |
| `workspace-forget` | Remove Workspace metadata; active work resolves source decisions and closes the context. |
| `workspace-list` | Choose a Workspace and restore its file session. |
| `workspace-open` | Restore a Workspace selected by name/path. |
| `workspace-remove-root` | Resolve only files losing coverage and remove their root membership. |
| `workspace-rename` | Select a Workspace and change only its display name. |
| `workspace-set-root-path` | Select a Workspace/root identity and an existing replacement directory. |

### File tree

| Command | Description |
| --- | --- |
| `focus-file-tree` | Focus the visible tree. |
| `toggle-file-tree` | Show or hide the left tree. |
| `tree-collapse` | Collapse or select the parent. |
| `tree-create-directory` | Create a directory at the selected directory. |
| `tree-create-file` | Create an empty file at the selected directory. |
| `tree-delete` | Confirm removal, including separate recursive consent. |
| `tree-expand` | Lazily expand a directory. |
| `tree-next-line` | Select the next visible entry. |
| `tree-open` | Open a file or toggle a directory. |
| `tree-previous-line` | Select the preceding visible entry. |
| `tree-refresh` | Reload already opened tree branches. |
| `tree-up-directory` | Show the parent directory of an unanchored file tree. |
| `tree-rename` | Rename one entry and reconcile visited paths. |
| `tree-toggle-directory` | Toggle a directory; leaves report an error. |

### Completion

| Command | Description |
| --- | --- |
| `completion-accept` | Apply the selected completion as one undoable edit. |
| `completion-at-point` | Show completion suggestions at point. |
| `completion-auto-mode` | Toggle automatic requests while retaining explicit invocation. |
| `completion-cancel` | Dismiss the popup without editing source. |
| `completion-mode` | Toggle completion for this session. |
| `completion-next` | Select the next completion candidate. |
| `completion-previous` | Select the previous completion candidate. |

### Language servers

| Command | Description |
| --- | --- |
| `lsp-cancel-installation` | Request cancellation of the active server management operation. |
| `lsp-connect` | Restore this buffer eligibility without enabling globally disabled LSP. |
| `lsp-disconnect` | Exclude this canonical buffer across all its views until restored or closed. |
| `lsp-install-server` | Install or select a supported managed language server. |
| `lsp-mode` | Toggle optional language-server support for this session. |
| `lsp-reload-settings` | Reload LSP settings and associations while preserving runtime choices. |
| `lsp-remove-server` | Remove a confirmed installer-owned payload only when unreferenced and stopped. |
| `lsp-restart` | Restart the eligible current root; preserve global mode and buffer exclusions. |
| `lsp-select-server` | Persist a compatible configured source, ready managed installation or native-only choice. |
| `lsp-servers` | List configured language servers, availability and installation prerequisites. |
| `lsp-setup` | Set up an installed or managed language server. |
| `lsp-status` | Show language-server status for the current file. |
| `lsp-update-server` | Verify a replacement before retargeting selected managed profiles; retain the old payload. |

### AI commands

| Command | Description |
| --- | --- |
| `ai-apply-result` | Apply the selected proposal once to its original checked target. |
| `ai-cancel` | Cancel the active invocation or AI-owned input/result. |
| `ai-discard-result` | Discard the pending result without editing source. |
| `ai-disconnect` | Stop the Pi connection while keeping its history and results. |
| `ai-explain` | Explain the selected text using the chosen AI context. |
| `ai-implement` | Generate an implementation at point from an instruction. |
| `ai-prompt` | Request an AI transformation of the selection or insertion at point. |
| `ai-reset-session` | Start fresh owned history on the next explicit invocation. |
| `ai-save-command` | Save the successful literal instruction as a reusable command. |
| `ai-select-model` | Choose a Pi-reported provider/model for this editor session. |
| `ai-set-directory` | Choose a session-local AI context root. |

### Bookmarks

| Command | Description |
| --- | --- |
| `bookmark-bmenu-delete` | Mark the selected record for deletion and advance. |
| `bookmark-bmenu-execute-deletions` | Execute checked marked deletions. |
| `bookmark-bmenu-this-window` | Visit the selected stable record in this pane. |
| `bookmark-bmenu-unmark` | Unmark the selected record and advance. |
| `bookmark-delete` | Complete a name and remove that bookmark. |
| `bookmark-jump` | Complete a name and visit its checked location. |
| `bookmark-markers-mode` | Toggle marker visibility for the selected editing view. |
| `bookmark-next` | Visit the next resolved current-file record, wrapping and stepping same-position names. |
| `bookmark-previous` | Visit the previous resolved current-file record in source/name order. |
| `bookmark-reload` | Validate disk and confirm replacing unsaved bookmarks. |
| `bookmark-save` | Checkpoint the user collection without saving sources. |
| `bookmark-set` | Name or replace the current file position. |
| `bookmark-set-no-overwrite` | Name the current position without replacement. |
| `list-bookmarks` | Open the read-only native bookmark list. |

### Gutter

| Command | Description |
| --- | --- |
| `gutter-cycle-current-line` | Cycle absolute, zero and hidden current labels. |
| `gutter-cycle-line-number-mode` | Alternate absolute and relative, retaining visibility. |
| `gutter-inspect-numbering` | Report effective policy and startup/view override sources. |
| `gutter-reset-numbering` | Restore this view's immutable startup policy. |
| `gutter-set-line-numbers-absolute` | Show absolute logical line numbers. |
| `gutter-set-line-numbers-relative` | Show distances from this view's logical point. |
| `gutter-toggle-line-number-side` | Move this view's number field left or right. |
| `gutter-toggle-line-numbers` | Hide or restore numbers, retaining mode. |

### Modal commands

| Command | Description |
| --- | --- |
| `modal-align-bottom` | Align point at the viewport bottom without moving it. |
| `modal-align-middle` | Align point at the viewport middle without moving it. |
| `modal-align-top` | Align point at the viewport top without moving it. |
| `modal-backward-char` | Previous whole grapheme, including a whole LF/CRLF boundary. |
| `modal-buffer-end` | End of buffer; with a count, start of that one-based logical line. |
| `modal-buffer-start` | Start of buffer; with a count, start of that one-based logical line. |
| `modal-change` | Retain/delete the target, then enter Insert after successful publication. |
| `modal-change-line-end` | Change through the logical content-end boundary, then enter Insert. |
| `modal-copy` | Copy the target to the shared kill ring without editing or moving point. |
| `modal-copy-line-end` | Copy through the logical content-end boundary, retaining linear shape. |
| `modal-delete` | Delete one checked target into the shared kill ring. |
| `modal-delete-backward` | Delete counted preceding whole graphemes within this logical line, preserving retained text. |
| `modal-delete-forward` | Delete counted whole graphemes within this logical line, preserving retained text. |
| `modal-delete-line-end` | Delete through the logical content-end boundary; a count selects following lines. |
| `modal-enter-insert` | Enter Insert at point. |
| `modal-enter-insert-after` | Enter Insert after the current grapheme, stopping at logical EOL. |
| `modal-enter-insert-line-end` | Enter Insert at logical EOL. |
| `modal-enter-insert-line-first` | Enter Insert at the first non-whitespace boundary. |
| `modal-enter-normal` | Return to Normal, preserving the native point. |
| `modal-enter-replace` | Enter persistent Replace with an underline cursor. |
| `modal-enter-visual-block` | Select inclusive logical grapheme columns, convert shape or leave a repeated Block state. |
| `modal-enter-visual-character` | Select the current grapheme, convert shape or leave a repeated Character state. |
| `modal-enter-visual-line` | Select complete logical lines, convert shape or leave a repeated Line state. |
| `modal-find-backward` | Counted previous occurrence of one Unicode scalar within the logical line; land on its grapheme start. |
| `modal-find-forward` | Counted next occurrence of one Unicode scalar within the logical line; land on its grapheme start. |
| `modal-forward-char` | Next whole grapheme, including a whole LF/CRLF boundary. |
| `modal-half-page-backward` | Move backward by half a page. |
| `modal-half-page-forward` | Half-page forward. |
| `modal-indent` | Indent each intersecting logical line by one active indentation unit. |
| `modal-join-lines` | Join logical lines, adjusting the whitespace between them. |
| `modal-jump-backward` | Revisit an older checked location, with an optional count. |
| `modal-jump-forward` | Revisit a newer checked location, with an optional count. |
| `modal-line-end` | Logical content-end boundary before LF/CRLF. A count selects the corresponding following logical line. |
| `modal-line-first-text` | First non-whitespace grapheme of the logical line. |
| `modal-line-start` | Logical line start. |
| `modal-matching-delimiter` | Move to the matching parenthesis, bracket or brace on the current logical line. |
| `modal-next-big-word-end` | Exclusive end of the next whitespace-separated WORD. |
| `modal-next-big-word-start` | Next start of a whitespace-separated WORD. |
| `modal-next-line` | Move to the next logical line, retaining the desired display column. |
| `modal-next-paragraph` | Next empty/whitespace-only logical-line boundary, clamped to EOF. |
| `modal-next-word-end` | Next Unicode word's exclusive end boundary, skipping punctuation. |
| `modal-next-word-start` | Start of the next word or punctuation run, skipping whitespace. |
| `modal-one-normal-command` | Admit one complete Normal command, then resume compatible insertion. |
| `modal-open-line-above` | Open one logical line above and enter Insert. |
| `modal-open-line-below` | Open one logical line below and enter Insert. |
| `modal-outdent` | Outdent each intersecting logical line by one active indentation unit. |
| `modal-page-backward` | Full-page backward through native geometry and page overlap. |
| `modal-page-forward` | Full-page forward through native geometry and page overlap. |
| `modal-paste-after` | Paste retained text after the current grapheme, or after this logical line for linewise text. |
| `modal-paste-before` | Paste retained text at point, or before this logical line for linewise text. |
| `modal-previous-big-word-end` | Exclusive end of the preceding WORD, excluding the run containing the point's predecessor. |
| `modal-previous-big-word-start` | Previous start of a whitespace-separated WORD. |
| `modal-previous-line` | Move to the previous logical line, retaining the desired display column. |
| `modal-previous-paragraph` | Previous empty/whitespace-only logical-line boundary, clamped to BOF. |
| `modal-previous-word-end` | Exclusive end of the preceding Unicode word, excluding the run containing the point's predecessor. |
| `modal-previous-word-start` | Previous Unicode word start, skipping punctuation. |
| `modal-repeat-find` | Repeat the last successful find in its original direction; till skips its prior match. |
| `modal-repeat-last-change` | Resolve and repeat the latest admitted semantic change; a count overrides its count. |
| `modal-replace-character` | Replace counted complete graphemes within this logical line with one printable Unicode scalar. |
| `modal-reverse-find` | Repeat the last successful find in the opposite direction; retain the original repeat direction. |
| `modal-screen-bottom` | Bottom displayed text row; count inward from the bottom. |
| `modal-screen-middle` | Middle displayed text row, including wrapped fragments and folds. |
| `modal-screen-top` | Top displayed text row; count inward from the top. |
| `modal-search-backward` | Open the shared backward literal search prompt. |
| `modal-search-forward` | Open the shared forward literal search prompt. |
| `modal-search-repeat` | Continue the confirmed search in its original direction. |
| `modal-search-reverse` | Continue the confirmed search in the opposite direction. |
| `modal-search-word-backward` | Search backward for the Unicode word at point with complete word boundaries. |
| `modal-search-word-forward` | Search forward for the Unicode word at point with complete word boundaries. |
| `modal-select-around-object` | Select a counted word or paragraph with its surrounding whitespace. |
| `modal-select-inner-object` | Select a counted inner word or paragraph through native endpoints. |
| `modal-till-backward` | Grapheme immediately after the counted previous occurrence. |
| `modal-till-forward` | Grapheme immediately before the counted next occurrence. |

### Shell commands

| Command | Description |
| --- | --- |
| `shell-command` | Run a shell command and retain its output. |
| `shell-command-cancel` | Cancel the owned invocation; originating editing C-g also cancels after inner input owners. |
| `shell-command-insert` | Insert a successful shell command's output at point. |
| `shell-command-on-region` | Run a shell command with the selected text as input. |
| `shell-command-output` | Visit the last retained read-only plain-text result or diagnostics. |
| `shell-filter-region` | Replace the selection with a successful shell command's output. |

### Integrated terminals

| Command | Description |
| --- | --- |
| `terminal-beginning-of-line` | Move to the frozen copy line start. |
| `terminal-clear` | Send C-l through ordinary child input. |
| `terminal-clear-scrollback` | Clear retained history and copy projections, preserving live screen/input. |
| `terminal-copy-mode` | Freeze retained output for native navigation, selection and search. |
| `terminal-copy-mode-done` | Resume live input without sending Return to the child. |
| `terminal-end-of-line` | Move to the frozen copy line end. |
| `terminal-list` | Select or inspect retained terminal sessions. |
| `terminal-move-to-buffer` | Move the current terminal into the selected editing pane. |
| `terminal-move-to-panel` | Dock the same session in the lower panel. |
| `terminal-new` | Create an independent lower-panel session. |
| `terminal-new-buffer` | Create an independent terminal directly in the selected editing pane. |
| `terminal-next-error` | Visit an actually recognized retained error location. |
| `terminal-next-prompt` | Navigate the next captured OSC 133 prompt. |
| `terminal-open` | Open or select the instance default interactive terminal in the lower panel. |
| `terminal-open-here` | Create a session from the current resource's directory context. |
| `terminal-open-link` | Explicitly activate the validated link at live or frozen point. |
| `terminal-open-other-pane` | Select a terminal in the next pane, creating a right split when needed. |
| `terminal-previous-prompt` | Navigate the previous captured OSC 133 prompt. |
| `terminal-rename` | Give the selected terminal buffer a unique name without changing its identity. |
| `terminal-reset-cursor-point` | Align copy point with a still-represented live cursor. |
| `terminal-send-backspace` | Send Backspace with the live mode-aware VT encoder. |
| `terminal-send-delete` | Send Delete with the live mode-aware VT encoder. |
| `terminal-send-down` | Send Down with the live mode-aware VT encoder. |
| `terminal-send-escape` | Send Escape with the live mode-aware VT encoder. |
| `terminal-send-left` | Send Left with the live mode-aware VT encoder. |
| `terminal-send-next` | Send PageDown with the live mode-aware VT encoder. |
| `terminal-send-next-key` | Send the next key directly to the terminal process. |
| `terminal-send-prior` | Send PageUp with the live mode-aware VT encoder. |
| `terminal-send-return` | Send Return with the live mode-aware VT encoder. |
| `terminal-send-right` | Send Right with the live mode-aware VT encoder. |
| `terminal-send-space` | Send Space with the live mode-aware VT encoder. |
| `terminal-send-tab` | Send Tab with the live mode-aware VT encoder. |
| `terminal-send-up` | Send Up with the live mode-aware VT encoder. |
| `terminal-toggle` | Hide or restore the lower terminal panel without restarting the child. |
| `terminal-undo` | Send C-_ to the child without source undo. |
| `terminal-yank` | Paste the newest retained text using active bracket framing. |
| `terminal-yank-pop` | Rotate only while the preceding paste remains wholly queued. |

### Git commands

| Command | Description |
| --- | --- |
| `git-apply-mailbox` | Review one complete local mailbox, then apply its decoded patches as checked commits. |
| `git-apply-patch` | Review one complete local patch file, then apply its sealed bytes to the checked index and working tree. |
| `git-bisect-bad` | Review the exact current candidate and record the user-supplied bad verdict. |
| `git-bisect-good` | Review the exact current candidate and record the user-supplied good verdict. |
| `git-bisect-reset` | Review and restore the checked original branch/object; reject missing or changed original lineage. |
| `git-bisect-skip` | Review and skip the current candidate; retain ambiguity when skipped commits prevent a unique first bad. |
| `git-bisect-start` | Review good and bad revisions and start a manual bisect. |
| `git-blame` | Attribute the selected or visiting source at checked HEAD. |
| `git-branch-create` | Confirm a new local branch at the selected full commit. |
| `git-branch-delete` | Confirm non-forced local deletion. |
| `git-branch-rename` | Confirm the exact local branch and new name. |
| `git-branch-switch` | Confirm a checked clean local branch switch. |
| `git-cherry-pick` | Review exact ordered revisions and explicit merge mainlines before applying them. |
| `git-cherry-pick-abort` | Refresh and review abort of the exact retained cherry-pick request. |
| `git-cherry-pick-continue` | Refresh and review continue of the exact retained cherry-pick request. |
| `git-cherry-pick-skip` | Refresh and review skip of the exact retained cherry-pick request. |
| `git-clone` | Review literal clone arguments and an explicit absent destination, then verify the created regular repository. |
| `git-clone-bare` | Review a bare clone destination and verify its separate Git-only repository identity. |
| `git-clone-mirror` | Review a mirror clone destination and verify bare and mirror configuration without a later push. |
| `git-clone-shallow` | Prepare a depth-one clone at an explicit destination and verify its shallow state and full HEAD. |
| `git-commit` | Open native commit text directly; a subsequent c is literal message text. |
| `git-commit-amend` | Prepare exact observed HEAD composition, then explicitly review the replacement commit. |
| `git-commit-cancel` | Cancel before process start; retain commit/tag drafts and restore the source owner. |
| `git-commit-fixup` | Resolve one exact target, compose its fixup body and review before submission. |
| `git-commit-submit` | Submit the active native composer; prepared arguments require checked review before execution. |
| `git-conflict-delete` | Review exact deletion from disk and index, leaving continue separate. |
| `git-conflict-explain` | Ask AI to explain the selected conflict without applying a resolution. |
| `git-conflict-stage` | Review staging of captured saved text or absence; protect unsaved results and reject stale evidence. |
| `git-conflict-use-applied` | Review complete operation-relative Applied replacement without staging. |
| `git-conflict-use-base` | Review complete Base replacement of only the checked working file. |
| `git-conflict-use-current` | Review complete operation-relative Current replacement without staging. |
| `git-conflict-visit` | Inspect selected stages, then visit the canonical editable result. |
| `git-conflicts` | Inspect actual Git sequence state and exact selectable conflict paths. |
| `git-diff` | Inspect the exact selected comparison as read-only detail. |
| `git-discard` | Confirm one tracked unstaged whole path from its index; protect dirty visiting buffers. |
| `git-dispatch` | Show the available native Git actions. |
| `git-fetch` | Fetch the selected configured remote without integration. |
| `git-format-patch` | Review an explicit local commit range and complete mailbox bytes, then save to one absent output file. |
| `git-ignore` | Visit the ordinary worktree .gitignore file without inserting a pattern or saving text. |
| `git-ignore-exclude` | Visit the ordinary common Git info/exclude file through its native visiting owner. |
| `git-init` | Prepare initialization at an explicit absent destination and verify its unborn repository identity. |
| `git-log` | Open exact bounded ancestry; detached HEAD collects an explicit root. |
| `git-log-next-page` | Continue exact ancestry from its pinned root. |
| `git-mark` | Mark one exact current path or supported hunk independently of navigation. |
| `git-merge` | Review merge with fast-forward allowed. |
| `git-merge-abort` | Refresh and confirm restoration of the original merge context. |
| `git-merge-continue` | Refresh and confirm continuation of a compatible merge. |
| `git-merge-fast-forward` | Review one exact target and require fast-forward-only integration. |
| `git-merge-no-fast-forward` | Review merge-commit policy. |
| `git-notes-add` | Review a full notes ref, exact target object and literal message; verify the stored note bytes. |
| `git-notes-prune` | Review exact missing-object note targets and prune only those candidates. |
| `git-notes-remove` | Review one existing note on a full object ID and remove only that note. |
| `git-operation-abort` | Review abort against exact original lineage and fresh Git state. |
| `git-operation-continue` | Review continuation against the fresh actual compatible family. |
| `git-operation-skip` | Review skipping against the fresh actual compatible family. |
| `git-operation-status` | Inspect actual Git-owned sequence state and conflicts. |
| `git-process-cancel` | Cancel the selected live operation through its exact identity; menus and prompts keep first refusal. |
| `git-process-show` | Inspect the selected operation; the Return binding dispatches through git-visit-file. |
| `git-processes` | Inspect active and recent owned operations without cancelling them. |
| `git-push` | Review one exact source branch and its configured upstream destination before publication. |
| `git-quit` | Dismiss the innermost menu or return to the retained editing context; owned work continues. |
| `git-rebase` | Review ordinary rebase onto the selected exact commit. |
| `git-rebase-abort` | Refresh and confirm restoration of the original rebase context. |
| `git-rebase-autosquash` | Review the finite ordered source plan before autosquash. |
| `git-rebase-continue` | Refresh and confirm continuation after saved staged resolutions. |
| `git-rebase-skip` | Refresh and confirm skipping the current compatible rebase step. |
| `git-reflog` | Inspect bounded exact HEAD reflog entries. |
| `git-refresh` | Request a replacement observation; retain confirmed Status on failure. |
| `git-refs` | Inspect exact native repository refs. |
| `git-request-pull` | Review local commit anchors and one explicit published remote branch, then save request text without sending mail. |
| `git-revert` | Review exact ordered revisions and explicit merge mainlines before reverting them. |
| `git-revert-abort` | Refresh and review abort of the exact retained revert request. |
| `git-revert-continue` | Refresh and review continue of the exact retained revert request. |
| `git-revert-skip` | Refresh and review skip of the exact retained revert request. |
| `git-run-command` | Compose literal Git argv and review executable, cwd and unknown postconditions before execution. |
| `git-run-shell-command` | Explicitly compose one shell command using the configured shell program and arguments, with exact review. |
| `git-save-diff` | Review two local commits and a complete binary diff, then save to one absent output file. |
| `git-sequence-mark` | Append an exact ordered revision without changing the range base. |
| `git-sequence-unmark` | Remove an ordered revision, retaining remaining order and the base. |
| `git-show` | Inspect full revision metadata, changed paths and exact parent comparison. |
| `git-show-range` | Compare the marked exact base and selected endpoint in their explicit order. |
| `git-sparse-checkout-add` | Review literal relative directories and add them to the captured cone selection. |
| `git-sparse-checkout-disable` | Review disabling sparse checkout and verify that tracked files return. |
| `git-sparse-checkout-enable` | Review enabling cone mode and verify actual tracked-file materialization. |
| `git-sparse-checkout-reapply` | Review reapplying the captured cone selection to a checked clean working tree. |
| `git-sparse-checkout-set` | Review literal relative directories and replace the cone selection without forcing saved changes. |
| `git-stage` | Stage the exact selected path, supported text hunk or complete compatible marked group; unsaved source text is retained. |
| `git-stash-apply` | Review and apply one full stash object to checked clean saved state while keeping its stash entry. |
| `git-stash-branch` | Review a new branch at the selected stash base, restore its full object and remove its checked entry only on success. |
| `git-stash-create` | Review and stash saved tracked and untracked changes, excluding ignored files; protect affected unsaved buffers. |
| `git-stash-drop` | Review removal of one exact stash entry without changing the saved working files or index. |
| `git-stash-list` | Open a bounded exact stash list; subjects and ordinal labels never supply mutation authority. |
| `git-stash-pop` | Apply one full stash object and drop only the revalidated captured entry after success; retain the stash on conflict. |
| `git-stash-show` | Inspect the selected full stash object as bounded read-only patch detail with explicit partial status. |
| `git-status` | Discover the current repository and open full-canvas native Status. |
| `git-submodule-add` | Review a submodule metadata name, source URL and literal path, then verify its gitlink and checkout. |
| `git-submodule-deinit` | Review non-forced deinitialization of one checked clean submodule, preserving its parent gitlink. |
| `git-submodule-list` | Review recursive submodule status and inspect the actual bounded listing without repository effects. |
| `git-submodule-sync` | Review one declared submodule path and verify its registered URL against .gitmodules. |
| `git-submodule-update` | Review one exact gitlink path and initialize or check out its indexed commit without forcing dirty files. |
| `git-subtree-add` | Review an explicit prefix, source repository and pinned full branch object before importing a subtree. |
| `git-subtree-merge` | Review a full local commit and subtree prefix, retaining actual merge/conflict state. |
| `git-subtree-pull` | Pin one explicit source branch before fetching and merging into a reviewed subtree prefix. |
| `git-subtree-push` | Review one explicit endpoint and full destination branch, then verify its published subtree object and tree. |
| `git-subtree-split` | Review an exact commit and prefix, then inspect the full split object while preserving local HEAD and saved files. |
| `git-tag-create` | Confirm a local lightweight tag or explicitly compose an annotated tag. |
| `git-tag-delete` | Confirm deletion of the exact local tag object. |
| `git-toggle-section` | Collapse or expand the selected Status group, path comparison or semantic hunk. |
| `git-unmark` | Remove the exact selected path or hunk mark. |
| `git-unmark-all` | Clear all retained path and hunk marks. |
| `git-unstage` | Unstage the selected path, hunk or marked group without changing working files. |
| `git-visit-file` | Visit the canonical current source, preserving an existing modified buffer. |
| `git-worktree-create` | Review a new linked worktree and new local branch at the exact observed HEAD, using a checked absent destination. |
| `git-worktree-list` | List linked worktrees, their branches, lock reasons and prune state. |
| `git-worktree-lock` | Review an exact linked-worktree path and explicit one-line lock reason. |
| `git-worktree-prune` | Review all captured missing unlocked worktree records and prune only while the complete registry remains unchanged. |
| `git-worktree-remove` | Review removal of a clean, unlocked linked worktree. |
| `git-worktree-unlock` | Review unlocking one exact linked-worktree record without changing its saved files. |
| `git-worktree-visit` | Open checked Git Status for the selected worktree identity while retaining native source buffers and windows. |

### Org commands

| Command | Description |
| --- | --- |
| `org-agenda` | Choose agenda sources and open the agenda. |
| `org-agenda-action-results` | Inspect the results of agenda source actions. |
| `org-agenda-bulk-action` | Choose an action for marked agenda entries. |
| `org-agenda-bulk-mark` | Mark the selected agenda entry. |
| `org-agenda-bulk-mark-all` | Mark all visible entries, up to 256 distinct targets. |
| `org-agenda-bulk-unmark` | Unmark the selected agenda entry. |
| `org-agenda-bulk-unmark-all` | Clear explicit marks. |
| `org-agenda-columns` | Choose which property columns the agenda displays. |
| `org-agenda-custom-view` | Choose a configured agenda view by key. |
| `org-agenda-day-view` | Show a one-day agenda. |
| `org-agenda-earlier` | Move a span backward. |
| `org-agenda-filter` | Apply a native tag/property visibility filter. |
| `org-agenda-filter-remove-all` | Restore configured preset filters. |
| `org-agenda-goto-date` | Choose an explicit date. |
| `org-agenda-goto-today` | Show today's agenda. |
| `org-agenda-later` | Move a span forward. |
| `org-agenda-month-view` | Show an aligned calendar month. |
| `org-agenda-redo` | Refresh current sources and captured time. |
| `org-agenda-sort` | Set ordered native sort keys. |
| `org-agenda-span` | Select a named span or 1–366 days. |
| `org-agenda-switch-to` | Visit the selected agenda entry in its source file. |
| `org-agenda-undo` | Undo the last unchanged source action transaction. |
| `org-agenda-week-view` | Show one configured week. |
| `org-agenda-year-view` | Show an aligned calendar year. |
| `org-archive-results` | Inspect archive participants and independent save outcomes. |
| `org-archive-subtree` | Move the complete subtree to the checked archive location with origin metadata. |
| `org-archive-to-archive-sibling` | Move the entry below its checked Archive sibling, creating it when absent. |
| `org-attach` | Dispatch explicit heading attachment operations. |
| `org-attach-attach` | Copy a selected file into the current heading's attachments. |
| `org-attach-copy` | Copy a selected file into the current heading's attachments. |
| `org-attach-link` | Create a hard link to a selected file in the heading's attachments. |
| `org-attach-move` | Move a selected file into the current heading's attachments. |
| `org-attach-open` | Open a selected attachment using its file or viewer rules. |
| `org-attach-open-in-emacs` | Open a selected attachment as text in rev. |
| `org-attach-results` | Inspect the latest attachment operation's results. |
| `org-attach-reveal-in-emacs` | List the current heading's attachments. |
| `org-attach-set-directory` | Set the heading's attachment directory without creating folders. |
| `org-babel-execute-src-block` | Select a configured runner and explicitly confirm a bounded source job. |
| `org-babel-execution-results` | Inspect retained stdout, stderr and source without inserting results. |
| `org-babel-insert-header-arg` | Insert one checked header argument on the opening line. |
| `org-babel-tangle` | Tangle declared blocks from the current canonical document into checked files. |
| `org-babel-tangle-results` | Inspect the actual outcome of each file from the last tangling operation. |
| `org-babel-tangle-single-block` | Tangle only the block at point, retaining its declared named references. |
| `org-babel-view-src-block-info` | Inspect declared language, name and effective header data. |
| `org-backward-heading-same-level` | Visit the previous sibling without crossing a parent boundary. |
| `org-calendar-backward-day` | Select the previous day. |
| `org-calendar-backward-month` | Select one month earlier, clamping its day. |
| `org-calendar-backward-week` | Select one week earlier. |
| `org-calendar-backward-year` | Select one year earlier, clamping its day. |
| `org-calendar-forward-day` | Select the next day. |
| `org-calendar-forward-month` | Select one month later, clamping its day. |
| `org-calendar-forward-week` | Select one week later. |
| `org-calendar-forward-year` | Select one year later, clamping its day. |
| `org-calendar-goto-today` | Select the operation's captured current date. |
| `org-calendar-scroll-three-months-left` | Advance by three months. |
| `org-calendar-scroll-three-months-right` | Move back by three months. |
| `org-calendar-select` | Accept the interpreted date/time with any required offset choice. |
| `org-capture` | Select a template and prepare an editable transient draft. |
| `org-capture-finalize` | Revalidate, insert and save once; retain the draft on failure. |
| `org-capture-kill` | Discard the draft and return to its mapped origin view. |
| `org-clock-cancel` | Remove the active interval. |
| `org-clock-display` | Compare task/subtree/file clock totals and effort. |
| `org-clock-goto` | Reveal the active task heading. |
| `org-clock-in` | Start a task-owned source interval. |
| `org-clock-in-last` | Re-clock the most recent qualified task. |
| `org-clock-out` | Close the active interval with elapsed time. |
| `org-clock-report` | Insert or explicitly update a clocktable. |
| `org-columns` | Show a read-only property-column view of the subtree or file. |
| `org-columns-insert-dblock` | Insert or update a property-column report block. |
| `org-content` | Show every heading without section bodies. |
| `org-ctrl-c-ctrl-c` | Apply the contextual Org action for the table, timestamp, heading or list at point. |
| `org-cycle` | Cycle heading or drawer visibility in native Org. |
| `org-cycle-set-startup-visibility` | Restore the selected Org view's configured or document startup visibility. |
| `org-date-from-calendar` | Insert the previously selected calendar date. |
| `org-dblock-update` | Refresh a recognized clocktable or columnview body. |
| `org-deadline` | Set the current heading's DEADLINE timestamp. |
| `org-delete-property` | Remove the named local base or additive form. |
| `org-demote-subtree` | Demote a complete heading subtree. |
| `org-edit-affiliated-keyword` | Edit an affiliated keyword attached to the current Org element. |
| `org-edit-property-owner` | Explicitly edit a selected effective source owner. |
| `org-edit-special` | Edit the selected source block in a retained draft. |
| `org-edit-src-abort` | Discard the source-block draft and return to its Org document. |
| `org-edit-src-exit` | Apply the source-block draft and return to its Org document. |
| `org-emphasize` | Insert or change source emphasis. |
| `org-export-to-beamer` | Export the document, subtree or region as Beamer source. |
| `org-export-to-beamer-pdf` | Export the document, subtree or region as a Beamer PDF. |
| `org-export-to-html` | Export the document, subtree or region as HTML. |
| `org-export-to-icalendar` | Export the document, subtree or region as an iCalendar file. |
| `org-export-to-latex` | Export the document, subtree or region as LaTeX source. |
| `org-export-to-latex-pdf` | Export the document, subtree or region as a LaTeX PDF. |
| `org-export-to-man` | Export the document, subtree or region as a man page. |
| `org-export-to-markdown` | Export the document, subtree or region as Markdown. |
| `org-export-to-odt` | Export the document, subtree or region as an OpenDocument text file. |
| `org-export-to-org` | Export the document, subtree or region as Org source. |
| `org-export-to-texinfo` | Export the document, subtree or region as Texinfo source. |
| `org-export-to-text` | Export the document, subtree or region as plain text. |
| `org-export-to-unicode` | Export the document, subtree or region as Unicode-formatted text. |
| `org-feed-goto-inbox` | Open a declared canonical inbox without fetching. |
| `org-feed-results` | Inspect retained per-feed import and publication outcomes. |
| `org-feed-update` | Select one declared feed for an explicit update. |
| `org-feed-update-all` | Explicitly update declared feeds with checked native imports. |
| `org-footnote-action` | Follow or create a checked footnote. |
| `org-footnote-new` | Create a footnote at point. |
| `org-footnote-renumber` | Renumber numeric footnotes. |
| `org-forward-heading-same-level` | Visit the next sibling without crossing a parent boundary. |
| `org-goto-calendar` | Inspect the calendar and retain a selected date. |
| `org-habit-graph-date` | Select a view date without editing source. |
| `org-habit-set-graph-window` | Set graph days and preferred column. |
| `org-habit-show-graph` | Open or refresh the native habit agenda for this buffer. |
| `org-habit-toggle-habits` | Toggle habit graph visibility. |
| `org-habit-toggle-habits-only-for-today` | Toggle graphs only for today. |
| `org-habit-toggle-show-all-today` | Include all habits on today's agenda. |
| `org-hide-drawer` | Hide only the current drawer body. |
| `org-hide-all-drawers` | Hide all valid Org drawer bodies in the selected view. |
| `org-hide-subtree` | Hide the containing heading body and descendants. |
| `org-inc-effort` | Cycle declared EFFORT_ALL choices. |
| `org-indent-disable` | Disable visual heading and body indentation in the current Org buffer. |
| `org-indent-enable` | Enable visual heading and body indentation in the current Org buffer. |
| `org-indent-item` | Indent the item while retaining child indentation. |
| `org-indent-item-tree` | Indent the whole item and its nested source. |
| `org-indent-mode` | Toggle visual heading and body indentation in the current Org buffer. |
| `org-insert-entity` | Insert a named Org entity. |
| `org-insert-heading` | Insert a heading at the current level through one checked edit. |
| `org-insert-item` | Split or continue an admitted list item. |
| `org-insert-item-checkbox` | Insert a new unchecked list item. |
| `org-insert-link` | Insert or edit a literal link and description. |
| `org-inspect-effort` | Show original effort spelling and elapsed minutes. |
| `org-inspect-priority` | Show explicit or implicit default priority. |
| `org-inspect-property` | Inspect effective data and source owners. |
| `org-inspect-tags` | Inspect local, ancestor and file memberships with source lines. |
| `org-inspect-todo` | Inspect task state and choices without editing. |
| `org-link-preview` | Explicitly preview the local image at point, in a region or in the current section. |
| `org-link-preview-clear` | Remove this view's disposable previews. |
| `org-link-preview-refresh` | Refresh bounded local previews in the current view. |
| `org-link-preview-region` | Preview the active region, or the buffer when no region is active. |
| `org-list-to-table` | Convert the current contiguous list into a table. |
| `org-mark-ring-goto` | Return to a previously stored Org source position. |
| `org-mark-ring-push` | Store the current Org source position in the mark ring. |
| `org-match-tags` | Test a bounded tag/property expression against the current heading. |
| `org-meta-return` | Insert a heading or continue the precise current list item. |
| `org-metadown` | Move a complete subtree or list item after its next sibling. |
| `org-metaleft` | Promote a subtree or outdent the precise list item. |
| `org-metaright` | Demote a subtree or indent the precise list item. |
| `org-metaup` | Move a complete subtree or list item before its previous sibling. |
| `org-mode` | Select native Org for the current buffer. |
| `org-move-item-down` | Move complete item source after its next sibling. |
| `org-move-item-up` | Move complete item source before its previous sibling. |
| `org-move-subtree-down` | Move a complete heading subtree after its next sibling. |
| `org-move-subtree-up` | Move a complete heading subtree before its previous sibling. |
| `org-next-link` | Move to the next Org link; repeat at the boundary to wrap. |
| `org-next-visible-heading` | Visit the next visible heading. |
| `org-open-at-point` | Resolve and visit the exact link at point. |
| `org-outdent-item` | Outdent an admitted item without detaching children. |
| `org-outdent-item-tree` | Outdent the whole item and its nested source. |
| `org-overview` | Show top-level headings. |
| `org-pretty-headings-mode` | Toggle decorative heading glyphs in the current Org view. |
| `org-previous-link` | Move to the previous Org link; repeat at the boundary to wrap. |
| `org-previous-visible-heading` | Visit the previous visible heading. |
| `org-priority` | Set a priority character; empty/space removes the cookie. |
| `org-priority-down` | Cycle the current heading priority downward. |
| `org-priority-up` | Cycle the current heading priority upward. |
| `org-promote-subtree` | Promote a complete heading subtree. |
| `org-publish` | Publish a configured Org document project. |
| `org-publish-results` | Inspect the results of the last Org publishing operation. |
| `org-refile` | Select a checked heading and move complete subtrees into its canonical buffer. |
| `org-refile-copy` | Copy complete subtrees without removing the source. |
| `org-refile-results` | Inspect each participant and its ordinary save outcome. |
| `org-remove-tag` | Remove exactly one local, case-sensitive tag. |
| `org-renumber-list` | Renumber ordered siblings with retained explicit resets. |
| `org-reset-heading-presentation` | Clear this view's override and restore its immutable startup pretty flag. No TOML write. |
| `org-resolve-clocks` | Explicitly resolve incomplete intervals in the selected entry. |
| `org-return` | Move to the next table row or insert an ordinary newline in other source contexts. |
| `org-schedule` | Set the current heading's SCHEDULED timestamp. |
| `org-search-view` | Search entry text with phrases or signed terms. |
| `org-set-effort` | Store a validated Org duration in local EFFORT. |
| `org-set-property` | Set a local property through name/value prompts. |
| `org-set-tags-command` | Replace locally stored tags; empty clears them. |
| `org-shiftdown` | Decrease the timestamp field at point, or cycle heading priority. |
| `org-shiftleft` | Move a timestamp back one day, or select the previous heading task state. |
| `org-shiftright` | Move a timestamp forward one day, or select the next heading task state. |
| `org-shifttab` | Cycle overview, contents and show-all in this view. |
| `org-shiftup` | Increase the timestamp field at point, or cycle heading priority. |
| `org-show-all` | Show all source in this view. |
| `org-show-all-drawers` | Show all valid Org drawer bodies in the selected view. |
| `org-show-children` | Show direct child headings. |
| `org-show-drawer` | Show the current drawer body. |
| `org-show-subtree` | Show the entire containing subtree. |
| `org-store-link` | Store the checked current location for later insertion. |
| `org-table-align` | Align the current table's columns. |
| `org-table-beginning-of-field` | Move to the beginning of the current table field. |
| `org-table-create` | Create a bounded native table on a blank line. |
| `org-table-delete-column` | Delete the current table column. |
| `org-table-end-of-field` | Move to the end of the current table field. |
| `org-table-export` | Export the current table as CSV or TSV. |
| `org-table-import` | Import table data from a CSV or TSV file. |
| `org-table-insert-column` | Insert an empty table column before the current one. |
| `org-table-insert-hline` | Insert a table separator below the current row. |
| `org-table-insert-row` | Insert a row above the selected table row. |
| `org-table-kill-row` | Remove the selected table row or separator. |
| `org-table-move-column-left` | Move the current table column left. |
| `org-table-move-column-right` | Move the current table column right. |
| `org-table-move-row-down` | Move the current table row or separator down. |
| `org-table-move-row-up` | Move the current table row or separator up. |
| `org-table-next-field` | Move to the next table field. |
| `org-table-next-row` | Move to the next table row. |
| `org-table-previous-field` | Move to the previous table field. |
| `org-table-recalculate` | Recalculate the selected table explicitly, as one undo unit. |
| `org-table-recalculate-cell` | Recalculate the current table field. |
| `org-table-recalculate-column` | Recalculate the current table column. |
| `org-table-sort-lines` | Sort the selected data rows by a field. |
| `org-table-to-list` | Convert a one-column table into a list. |
| `org-tags-view` | Select Boolean tag/property matches. |
| `org-timer` | Insert the current timer value. |
| `org-timer-item` | Insert or continue a timed description list. |
| `org-timer-pause-or-continue` | Pause or continue the session timer. |
| `org-timer-set-timer` | Enter an explicit countdown. |
| `org-timer-show-remaining-time` | Inspect the session timer without source changes. |
| `org-timer-start` | Start or reset the relative timer. |
| `org-timer-stop` | Stop and discard the session timer. |
| `org-timestamp` | Insert or edit an active timestamp through a native date/calendar prompt. |
| `org-timestamp-down` | Decrease the selected timestamp field. |
| `org-timestamp-down-day` | Move the selected timestamp back one civil day. |
| `org-timestamp-inactive` | Insert or edit an inactive timestamp. |
| `org-timestamp-up` | Increase the selected timestamp field. |
| `org-timestamp-up-day` | Move the selected timestamp forward one civil day. |
| `org-todo` | Cycle task state or select an annotated fast key. |
| `org-todo-list` | Select unfinished tasks or explicit states. |
| `org-todo-next` | Next state in the current sequence, including unmarked. |
| `org-todo-previous` | Previous state in the current sequence, including unmarked. |
| `org-todo-set` | Select an admitted state or clear it. |
| `org-toggle-archive-tag` | Toggle the local ARCHIVE tag without removing source. |
| `org-toggle-checkbox` | Toggle checked source and update progress. |
| `org-toggle-checkbox-presence` | Add or remove checkbox source explicitly. |
| `org-toggle-drawer` | Toggle only the current drawer body. |
| `org-toggle-all-drawers` | Toggle all valid Org drawer bodies in the selected view. |
| `org-toggle-radio-button` | Select a radio checkbox at the current level. |
| `org-up-heading-safe` | Visit the parent heading. |
| `org-update-statistics-cookies` | Update current item/heading statistics. |

## Command prefix argument descriptions

These delivered helpers retain their existing compiled descriptions. The alias
uses the same description as its canonical command.

| Command | Description |
| --- | --- |
| `prefix-argument` | Set an argument for the next command. |
| `universal-argument` | Set an argument for the next command. |
| `digit-argument` | Add a digit to the pending command argument. |
| `negative-argument` | Set the sign of the pending command argument. |

## Keyboard macro descriptions

The first recording/replay increment adds the following descriptions from the
approved [Keyboard Macros specification](editor-keyboard-macros.md). Later macro
commands enter this shared live catalog only when their native handlers arrive.

| Command | Description |
| --- | --- |
| `kmacro-start-macro-or-insert-counter` | Start recording, or insert the current macro counter. |
| `kmacro-start-macro` | Start or append to a keyboard macro. |
| `kmacro-end-macro` | Finish recording a keyboard macro. |
| `kmacro-end-or-call-macro` | Finish recording, or run the current keyboard macro. |
| `kmacro-end-and-call-macro` | Finish recording and run the current keyboard macro. |
| `kmacro-end-or-call-macro-repeat` | Run the current macro and enable ring repeat keys. |
| `kmacro-call-ring-2nd-repeat` | Run the second macro and enable ring repeat keys. |
| `kmacro-cycle-ring-next` | Select the next older keyboard macro. |
| `kmacro-cycle-ring-previous` | Select the next newer keyboard macro. |
| `kmacro-delete-ring-head` | Remove the current macro from the session ring. |
| `kmacro-swap-ring` | Exchange the first two macros in the ring. |
| `kmacro-redisplay` | Refresh the display during macro playback. |
| `kmacro-insert-counter` | Insert a macro counter value. |
| `kmacro-set-counter` | Set or reset the current macro counter. |
| `kmacro-add-counter` | Add to or restore the current macro counter. |
| `kmacro-set-format` | Set the integer format for macro counters. |
| `apply-macro-to-region-lines` | Run the current macro on logical lines in the region. |
| `kbd-macro-query` | Add a confirmation or editing pause to a macro. |
| `exit-recursive-edit` | Resume a macro after its manual editing pause. |
| `abort-recursive-edit` | Abort a macro editing pause and its playback. |

## Completed delivery evidence

CDS01-CDS05 are delivered through canonical PR #307. All 618 built-ins match
the exact approved inventory, while user-defined commands retain optional
metadata. The controller passed the unchanged gate at `978deacc49f6f5f948cea2070919ce91031f67a2`,
with log `A6S4H3BKQFKIG4WEYTWGATYYE4`. The
[Slice 1 closure](../report/slices/rev-command-descriptions/slice-1-catalog.md#delivery-set-closure)
records its exact command, six selected tests, full catalog guard, representative
M-x/modal colon presentation checks and Markdown/Spore documentation validation.
The ticket is solved with resolution fixed. This closure changes no requirement,
inventory definition, runtime, test, guide input or historical delivery evidence.
