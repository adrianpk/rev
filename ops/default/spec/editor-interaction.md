# Editor Interaction

Status: Completed
Completed: 2026-10-27
Validated candidate: `95713efb7f79fbc72417615a921841f8d49730ed`
Gate validation: `FWDUCVRJCYBKFXEWEFLKPKA35E`
Date: 2026-10-27
Approved: product owner; panels, vertical M-x and find-file completion join the configuration delivery.
Format: OpenSpec capability additions
Baseline: [rev Editor](rev-editor.md), R05-R07, R12, R14-R15, R23, R26-R27
Plan: [Configuration and Interaction Delivery](../plan/rev-configuration-ui.md)

## Purpose

Give rev's existing panes a small structural presentation owner and make its
command and file prompts discoverable. Buffer, view, layout, command and file
owners retain their existing authority.

UI03 now incorporates the complete approved [navigation amendment](editor-file-navigation.md)
through Slice 6 of [Wrap and Navigation](../plan/rev-wrap-navigation.md). The
completed candidate and gate above identify the original interaction delivery.

## Native Org projection compatibility

[Native Org](editor-org-mode.md) retains these panel, focus, clipping and prompt
contracts. Agenda/habit/column projections and source-edit/capture drafts have
explicit retained source/view ownership. Moving between them does not make a
projection's display row canonical text. Pending source jobs and active countdowns
share the native event owner; quiescent Org states add no polling or repaint loop.
NO_COLOR retains readable source, markers and diagnostics. This additive contract
does not relabel the historical candidate or completed interaction gate above.

## ADDED Requirements

### Requirement: UI01 Common panel composition

Editing panes, file tree and completion area SHALL use a common presentation
contract for outer/content bounds, reserved separators, clipping, status and
active state. The binary split tree SHALL remain the semantic layout owner.
Display-coordinate translation SHALL be derived once from these bounds for
cursor, syntax, regions and search; string prefixes SHALL NOT be an independent
source of geometry.

Default presentation SHALL use continuous Unicode shared separators and clean
junctions, without an enclosing outer frame around every pane. A narrow ASCII
fallback MAY be used for terminals lacking the required glyphs; geometry is
identical. Horizontal and vertical boundaries SHALL reserve actual cells and
respect editing/status/prompt minima. Style layers SHALL consume palette roles.

#### Scenario: Nested panels and tree survive resize

- **GIVEN** a visible tree and nested splits sharing a tabbed Unicode buffer
- **WHEN** focus changes and the terminal shrinks to tiny dimensions and grows
- **THEN** every rendered cell and cursor remains within its owning bounds
- **AND** buffer text, independent points, pending input and valid focus survive.

### Requirement: UI02 Vertical command completion

M-x SHALL display its query followed by one candidate per row, using up to
`minibuffer.candidate_limit` rows as geometry permits. It SHALL retain stable
registry order within the configured match ranks. The selected candidate SHALL
remain visible while navigating; an empty match has a clear empty state.

Up/Down and C-p/C-n SHALL select candidates; M-p/M-n SHALL navigate session
command history. Tab SHALL complete the selected command name without executing
it. Return SHALL dispatch the selected or exact typed command once. The existing
empty-query/no-match error behavior SHALL remain defined; opening M-x SHALL NOT
execute the first command by accident. C-g SHALL cancel, discard the draft and
restore prior editing focus. Rendering SHALL NOT perform command execution.

#### Scenario: Filter and choose a command

- **GIVEN** multiple commands match a typed prefix
- **WHEN** the user moves to a later row and presses Return
- **THEN** the visible selected command invokes its existing Rust handler once
- **AND** command arguments continue through the existing prompt flow.

#### Scenario: Tiny prompt retains the draft

- **GIVEN** a long Unicode query and selected candidate
- **WHEN** the completion panel cannot fit candidate rows after a resize
- **THEN** the query and cancellation remain usable within available geometry
- **AND** growing the terminal restores a coherent candidate list and selection.

### Requirement: UI03 Filesystem path completion

C-x C-f and M-x find-file SHALL share one native file/directory prompt. Its
initial directory SHALL be the current visiting file's parent, otherwise the
selected directory context or launch cwd. Relative/absolute paths, spaces,
Unicode, ./, ../ and leading ~/ SHALL work without shell evaluation.

A nonempty list SHALL visibly select its first result. Tab SHALL complete that
selected result, including after explicit Up/Down or C-p/C-n navigation, rather
than stopping at a shared prefix. Completing a directory SHALL retain its
separator and keep the prompt active for child completion. Query changes and
refreshes SHALL preserve checked candidate identity or report its disappearance.

Return after explicit navigation SHALL accept that candidate. Otherwise it
SHALL accept the typed draft, including a genuinely new file, without forcing
the first suggestion. An accepted directory SHALL close the prompt and open
its browsing context; an empty draft SHALL accept the base directory. New files
remain in-memory until save. Existing file identity/buffer reuse and unsaved
text SHALL be preserved. C-g and failed operations SHALL preserve prior usable
editor state. Completion and selection SHALL NOT create filesystem content.

#### Scenario: Complete first and choose whether to open

- **GIVEN** alpha/ and alpine/ match a typed path component
- **WHEN** Tab completes the selected result and the user browses its children
- **THEN** find-file stays active without opening a file or selecting a root
- **AND** Return can later accept a chosen file or the directory itself.

#### Scenario: Keep a genuinely new filename

- **GIVEN** suggestions alongside a typed filename that does not exist
- **WHEN** Return accepts that draft without explicit candidate navigation
- **THEN** rev opens the ordinary unsaved new-file buffer
- **AND** neither completion nor acceptance creates a disk file.

### Requirement: UI04 Bounded and safe candidate observation

The provider SHALL observe only the current directory component, never recurse.
It SHALL cache the observation while filtering that component; drawing SHALL
perform no filesystem reads. Each requested observation SHALL be bounded by
4096 entries and 512 KiB of candidate-name bytes, stop at its limit, and show an
explicit incomplete-list diagnostic. Manual path entry remains available.
An explicit completion or directory transition MAY refresh stale observations.
No background scanner or watcher is required.

Names SHALL be displayed as safe text without emitting controls. UTF-8 names
SHALL retain their exact opening identity independently of clipped labels;
non-UTF-8 names SHALL produce an explicit unsupported-name indication and SHALL
NOT be opened under a lossy replacement path. Listings SHALL sort directories
before files, then by case-sensitive filename bytes. Exact path errors SHALL
not replace the current buffer or discard the draft. Symlink destinations may
be followed only by an explicit accepted path, without recursive traversal.

#### Scenario: Directory becomes unreadable

- **GIVEN** a completion draft whose directory becomes unreadable or disappears
- **WHEN** completion refreshes its observation
- **THEN** the prompt displays the filesystem error and retains the editable draft
- **AND** C-g returns to the unchanged editing buffer.

## Acceptance and references

Native geometry/provider/state-machine checks and real PTY journeys SHALL cover
all four requirements, in combination with configuration, Unicode tabs, shared
views and resize. Existing file-publication and terminal-cleanup guarantees
remain authoritative. Ticket paths and slice assignments are in the master plan.

The controller-owned full gate passed UI01-UI04 on the validated candidate
above. The final Slice 30 report records native geometry/state checks, terminal
journeys, file-publication and cleanup evidence alongside configuration checks.
