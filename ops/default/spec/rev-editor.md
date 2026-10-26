# rev Editor

Status: accepted
Approved: product owner, 2026-10-26
Date: 2026-10-26
Format: OpenSpec baseline requirements and scenarios
Model: [rev Editor Model](rev-editor-model.md)
Reference: `rev-technical-specification.md`, dated 2026-10-26
Reference SHA-256: `7dd838f03fa9c9fdd8dbf952913c8590fbd0a750e936422b6bd169867f27b5cd`

## Purpose

Define the complete reference implementation of `rev`: a small, non-modal,
terminal text editor written in Rust, with Emacs-style editing, rope-backed
buffers, split windows, a side file tree, and Tree-sitter highlighting.

This document adapts the supplied technical specification to the OpenSpec
document format defined by the shared workspace specification contract. It
retains the product requirements and acceptance identities A01 through A20.
The requested deliverable is a working editor. Teaching methods, exercises,
lesson pacing, and work deliberately left for a learner are outside this
implementation contract.

The source document remains the provenance reference. This baseline and its
model are accepted by the product owner. The approved delivery plan and tracker
own execution authority and state. Markdown scenarios state
acceptance obligations; they are not evidence that tests have run.

## Scope and priority

Priorities, in order:

1. Correct editing and preservation of user data.
2. Predictable Emacs-style keyboard interaction.
3. Small, direct, readable Rust code.
4. Responsive terminal operation with rope-based text storage.
5. Syntax highlighting and a simple file tree.

Compatibility concerns the specified commands, keys, cursor behavior, and
screen organization. It does not import the complete GNU Emacs feature set.
The baseline excludes a GUI, Lisp or another scripting runtime, plugins or
package installation, LSP or language-completion engines, Git UI, Org, embedded
shells, a Dired-style editable directory listing, and built-in compilers,
formatters, test runners, or other IDE subsystems. Tree-sitter is for highlighting.

MUST and SHALL define acceptance obligations. SHOULD is a strong expectation;
deviations require a recorded reason. MAY permits bounded optional behavior
after the required baseline is satisfied. A **Design default** retains a
choice labeled as such in the reference, rather than claiming a separate
product-owner decision. Accepted adaptation clarifications are identified below.

## Vocabulary and invariants

| Term | Meaning |
| --- | --- |
| Buffer | One editable rope, file association, mark, and per-buffer history. |
| File | A path and its disk content, distinct from its visiting buffer. |
| Window or view | A terminal viewport referencing a buffer, with independent point and scroll. |
| Frame | The single terminal screen; no additional OS window. |
| Point | Insertion position belonging to the displayed window. |
| Mark | A position associated with the buffer, used with point to form a region. |
| Region | The ordered half-open text range between point and mark. |
| Kill ring | Bounded session history of copied or killed linear text. |
| Rectangle | Text selected by a line range and display columns; stored separately from the kill ring. |
| Minibuffer | Bottom prompt and echo area for commands, arguments, and messages. |
| Command | A built-in Rust operation with a lowercase hyphenated public name. |
| Tree pane | The optional left-side file navigator. |

`C-` means Ctrl, `M-` means Meta/Alt or a supported Escape prefix, `RET` means
Enter, `DEL` means Backspace, and `SPC` means Space. `C-x 2` is a sequence of
chords, not one simultaneous keypress.

The following invariants apply across requirements:

- All editable text has one canonical rope owner per buffer.
- Windows share text without sharing point or viewport state.
- Rope character positions, UTF-8 byte offsets, grapheme boundaries, and
  terminal cells are distinct units with explicit conversions.
- Editing, history, position updates, and syntax invalidation describe the
  same accepted change. An ordinary rejected edit leaves confirmed text intact.
- Saving never substitutes a success message for a failed write, and a later
  failure cannot conceal a file replacement that already occurred.
- Terminal presentation and syntax failures do not destroy canonical text.
- Keyboard bindings and `M-x` enter the same command implementation.

## Requirements

### Requirement: R01 Product identity and runtime boundary

Source: sections 1-3, 16, 23-24. Acceptance: A02, A17.

The executable MUST be named `rev`. It MUST run in a terminal, accept ordinary
text without an insert-mode transition, and implement commands directly in
Rust. Normal execution MUST work without a network connection. Temporary
prompt, prefix, search, and tree focus states are part of non-modal editing.
The baseline MUST respect the exclusions under Scope and priority.

**Design default:** Linux terminals are the primary target. Other platforms
MAY work through the selected crates; broad platform certification is not
required. The interface uses built-in defaults and one fixed palette.

#### Scenario: Ordinary offline editing

- **GIVEN** a supported terminal with no network connection
- **WHEN** the user launches `rev` and types ordinary text
- **THEN** an editable terminal buffer receives the text directly
- **AND** no interpreter, language server, or external editor service is required.

### Requirement: R02 Launch and initial file

Source: section 4.1. Acceptance: A01, A04.

Launching `rev` without arguments MUST create an initially empty editable
buffer and one editing window. **Design default:** one positional path opens
that file, or a new unsaved visiting buffer if the path does not exist.
Multiple positional paths MAY be added after the baseline is complete.
Initialization failures MUST be legible in the original terminal and produce
a nonzero exit status when startup cannot complete.

#### Scenario: Launch a nonexistent path

- **GIVEN** a path whose parent is accessible and whose file does not exist
- **WHEN** the user launches `rev` with that path
- **THEN** the editor shows an empty unsaved buffer associated with the intended path
- **AND** it does not create or overwrite a file merely by opening it.

### Requirement: R03 Terminal lifetime and cleanup

Source: sections 4.2, 17. Acceptance: A01, A20.

A small RAII session guard MUST own raw mode, alternate-screen entry, and
cursor visibility obligations. The editor MUST enable raw input, enter the
alternate screen, and hide the terminal cursor while rendering its own cursor.
Output MUST be buffered and flushed at deliberate render boundaries.

Partial initialization, ordinary exit, ordinary I/O failure, and recoverable
unwinding MUST restore acquired terminal state where possible: disable raw
mode, restore the original screen, and show the cursor. Recovery after
`SIGKILL`, termination without unwinding, or terminal-emulator failure is not
a guarantee.

#### Scenario: Failure after raw-mode entry

- **GIVEN** raw mode has been enabled
- **WHEN** later initialization or ordinary terminal I/O fails
- **THEN** guard cleanup attempts each restoration obligation already acquired
- **AND** the failure is reported without claiming restoration steps that failed.

### Requirement: R04 Safe quit and buffer close

Source: sections 4.3, 7.1, 17. Acceptance: A05, A19, A20.

`C-x C-c` MUST invoke `save-buffers-kill-terminal`. Modified buffers MUST be
guarded by an explicit save, discard, or cancel decision before closing them
or leaving the session. Cancel MUST preserve the session and unsaved text.
An unsuccessful required save MUST prevent an automatic close or exit.
`C-g` MUST cancel a pending interaction without quitting.

`C-q` MUST NOT remain a quit binding. Quoted insertion is not added to the
baseline by removing that prototype shortcut. Terminal interception of a
required chord MUST be documented rather than silently changing its meaning.

#### Scenario: Cancel quit with unsaved text

- **GIVEN** one or more buffers contain unsaved edits
- **WHEN** the user invokes `C-x C-c` and cancels the decision
- **THEN** the buffers, windows, and unsaved text remain available for editing.

#### Scenario: Save fails during quit

- **GIVEN** the user chooses to save a modified buffer while quitting
- **WHEN** the required save fails
- **THEN** the editor reports the failure and retains the buffer
- **AND** it does not complete the quit as if saving succeeded.

### Requirement: R05 Screen layout and status

Source: sections 5.1-5.2. Acceptance: A01, A05, A20.

The frame MUST contain editing windows, a per-window mode/status indication
or a clearly equivalent indication, a bottom minibuffer/echo area, and an
optional left file tree. Initially the tree is hidden and no prompt is active.
The selected editing window MUST be visually identifiable.

Status MUST show buffer/file name, dirty state, current line and column,
syntax mode or `Text`, and read-only state when applicable. Errors MUST appear
without erasing document content. The column convention MUST be documented
and remain consistent with the explicit coordinate domains.

#### Scenario: Save error remains visible with document intact

- **GIVEN** an active buffer and its status indication
- **WHEN** an ordinary save error is reported
- **THEN** the echo area identifies the failure
- **AND** the document and its dirty indication remain visible.

### Requirement: R06 Resize and limited geometry

Source: section 5.3. Acceptance: A06, A20.

A resize MUST recompute layout before the next draw and preserve valid editor
positions. Tiny or zero dimensions MUST NOT cause arithmetic errors, panics,
or out-of-bounds drawing. A split that cannot fit MUST preserve the existing
layout and show a message. An optional pane MAY collapse or temporarily hide
when space is insufficient, retaining editing state.

#### Scenario: Resize nested windows to zero and back

- **GIVEN** nested editing windows and an open tree
- **WHEN** the terminal reports zero width or height and later usable dimensions
- **THEN** presentation safely suspends or adapts and later resumes
- **AND** the buffers and valid point positions survive.

### Requirement: R07 Viewport and composed cell rendering

Source: sections 5.4, 16. Acceptance: A03, A08, A15, A20.

Motion MUST reveal point in the active viewport. Each window MUST retain its
own point and scroll. Rendering MUST use terminal-cell widths for wide text,
combining sequences, tabs, and displayed control characters. It MUST NOT
assume that a Rust `char` or UTF-8 byte occupies one cell.

The focused region MUST be visibly highlighted across rows and scrolling.
Syntax, selection, search indication, and cursor styles MUST compose in a
stable order with readable text and status. **Design defaults:** no soft wrap,
a horizontal viewport for long lines, and four-cell tab stops. Actual tabs
remain valid buffer content.

#### Scenario: Reveal mixed-width text

- **GIVEN** a long line containing a tab, CJK text, and a combining sequence
- **WHEN** point moves beyond the visible horizontal range
- **THEN** scrolling reveals point at a valid grapheme boundary
- **AND** drawing stays within its assigned cell rectangle.

### Requirement: R08 Canonical rope and shared mutations

Source: sections 6.1, 6.4. Acceptance: A06, A16.

Every editable buffer MUST store canonical text in a rope, including newly
created, loaded, pasted, undone, and redone text. A mutable `String` or
`Vec<String>` MUST NOT replace that store. Temporary strings for I/O, visible
fragments, queries, status, and names are permitted.

An edit MUST update the rope and its undo history together. All views of that
buffer MUST observe the resulting text. File I/O and per-key editing MUST NOT
rebuild the whole document. Storage MUST NOT require one allocation per
character or one canonical string per logical line.

`ropey` is the reference's starting candidate. An equivalent rope MAY be used
with a recorded justification. This specification adds no crate dependency.

#### Scenario: Edit through either shared view

- **GIVEN** two windows displaying one rope-backed buffer
- **WHEN** text is inserted through either window
- **THEN** both show the same updated document
- **AND** the operation is recorded once in that buffer's history.

### Requirement: R09 Qualified positions and explicit conversion

Source: section 6.2. Acceptance: A03, A10, A15, A16.

Rope character positions, Tree-sitter UTF-8 byte offsets and row/byte columns,
and terminal rows/display columns MUST be distinct. Conversions MUST be
explicit and tested. Editing and cursor motion MUST NOT bisect an encoded
Unicode scalar. **Design default:** motion uses user-perceived grapheme
clusters where possible, while rope character positions remain a valid
internal representation.

The model qualifies prepared ranges with their buffer and source text state.
A stale or foreign range MUST be rejected or deliberately recomputed against
the current text before mutation; raw offsets cannot silently change units.
This clarification carries a core invariant observed in `ruru` into `rev`.

#### Scenario: Convert a multibyte range for highlighting

- **GIVEN** a line with ASCII, an emoji, and a combining sequence
- **WHEN** an edit range is passed from the rope to Tree-sitter and drawing
- **THEN** the byte range and display cells describe the intended text
- **AND** neither consumer treats a character index as a byte or cell offset.

### Requirement: R10 File text fidelity

Source: sections 6.3, 17. Acceptance: A04, A16, A19.

The baseline MUST support UTF-8. Invalid UTF-8 and likely binary input MUST
produce a clear error instead of lossy decoding and later corruption. Existing
LF and CRLF line endings MUST be preserved. New line endings SHOULD follow
the prevailing style. An unrelated edit MUST NOT change final-newline presence.
A recognized UTF-8 BOM SHOULD be preserved.

Loading, editing, saving, and reopening MUST preserve unaffected text. Rope
indices, line counts, and byte offsets MUST remain consistent across edits,
paste, undo, and redo. Encoding policy MUST NOT normalize unrelated content.

#### Scenario: Preserve CRLF and a missing final newline

- **GIVEN** valid UTF-8 with CRLF lines and no final newline
- **WHEN** the user edits one word, saves, and reopens the file
- **THEN** unaffected bytes, line separators, and final-newline absence match.

#### Scenario: Reject invalid UTF-8

- **GIVEN** an existing active buffer and an invalid UTF-8 file
- **WHEN** the user attempts to open that file
- **THEN** a readable error is shown
- **AND** the active document is not replaced with a lossy decoded copy.

### Requirement: R11 Buffer identity and lifecycle

Source: sections 7.1, 17-18. Acceptance: A05, A06, A19.

A buffer MAY be unnamed, associated with a new unsaved path, or visiting an
existing file. Live names MUST be unique; equal basenames MUST be distinguishable
without losing path identity. Opening an already visited canonical file SHOULD
activate its existing buffer. The same open machinery MUST serve command and
tree access.

Buffers MUST survive window deletion. Killing a buffer shown in several
windows MUST safely replace each reference with an existing buffer or a new
empty buffer. Dirty state MUST describe edits relative to the last successful
save; only successful saving can establish a new saved state. Read-only
buffers MUST reject text mutation through every command entry point.

#### Scenario: Reopen one visited file

- **GIVEN** a file already visited by a live buffer
- **WHEN** that canonical file is opened again through a command or the tree
- **THEN** its existing buffer is selected under the retained reuse default
- **AND** no independent divergent editable copy is introduced.

#### Scenario: Kill a shared modified buffer

- **GIVEN** two windows display the same modified buffer
- **WHEN** the user confirms discarding and killing that buffer
- **THEN** both windows receive valid replacement buffers
- **AND** deleting the buffer does not leave stale window references.

### Requirement: R12 File and buffer command interface

Source: section 7.2. Acceptance: A04, A05, A11.

The commands below MUST be implemented and available through the common
command handler. The buffer list MAY be a transient selectable overlay or an
internal read-only view; it MUST NOT require an extension system.

| Keys | Command | Behavior |
| --- | --- | --- |
| `C-x C-f` | `find-file` | Prompt for a path; open or activate its buffer. |
| `C-x C-s` | `save-buffer` | Save to the visited path; prompt if unnamed. |
| `C-x C-w` | `write-file` | Prompt for a destination and save the current text there. |
| `C-x b` | `switch-to-buffer` | Prompt with completion across live buffer names. |
| `C-x k` | `kill-buffer` | Choose and close a buffer with unsaved-change protection. |
| `C-x C-b` | `list-buffers` | Present a simple selectable live-buffer list. |

#### Scenario: Select duplicate basenames

- **GIVEN** two live buffers visiting different files with the same basename
- **WHEN** the user opens the buffer list or buffer-switch prompt
- **THEN** their labels distinguish both paths
- **AND** selecting either label activates the correct buffer.

### Requirement: R13 Safe file publication

Source: sections 7.3, 17. Acceptance: A04, A19.

Saving MUST report path, permission, disk-full, read-only, and other I/O errors
without discarding the rope. Failed saves MUST preserve dirty state and MUST
NOT report success. Save-as to an occupied path MUST require confirmation.
A detected external modification since open or last save MUST trigger a warning
before overwrite; conservative metadata checks are sufficient, and eliminating
all races with external writers is not promised.

**Design default:** write a temporary file in the destination directory and
replace the destination only after the new content is written successfully,
where supported. Appropriate permissions SHOULD be preserved where practical.
Unavailable guarantees MUST be disclosed. A later failure MUST distinguish
an unpublished attempt from a replacement already published; it cannot claim
that the original destination was preserved if replacement already happened.
The model defines the accepted outcome and saved-state rules.

#### Scenario: Write fails before replacement

- **GIVEN** a modified buffer and an existing destination
- **WHEN** writing the temporary output fails before replacement
- **THEN** the destination's existing content is preserved under the atomic-save default
- **AND** the buffer remains dirty and the failure is reported.

#### Scenario: External modification warning is canceled

- **GIVEN** the destination changed externally after the editor last observed it
- **WHEN** the user cancels the overwrite warning
- **THEN** no replacement is attempted
- **AND** the edited rope remains available and dirty.

### Requirement: R14 Nested splits and independent views

Source: section 8. Acceptance: A06, A07.

`C-x 2` / `split-window-below` MUST create upper/lower windows.
`C-x 3` / `split-window-right` MUST create left/right windows. Both operations
MUST be nestable in either order. A new split initially shows the same buffer
and approximately the same viewport as the original; original focus remains.
The views then maintain independent point and scroll while sharing the rope.
Repeated split, close, and resize operations MUST preserve a valid layout.

A small recursive split tree is sufficient. Numeric prefixes, explicit split
sizes, balancing, and a generalized docking engine are outside the baseline.

#### Scenario: Mix split directions

- **GIVEN** one editing window
- **WHEN** the user splits below and then splits one resulting window right
- **THEN** three usable views form the requested nested layout
- **AND** moving point in one does not move point in the others.

### Requirement: R15 Window deletion and focus cycling

Source: section 8. Acceptance: A07.

`C-x 0` / `delete-window` MUST remove the selected view while retaining its
buffer. Deleting the sole editing window MUST be refused or be a no-op with
a message. `C-x 1` / `delete-other-windows` MUST retain the selected editing
window. `C-x o` / `other-window` MUST cycle through editing windows in a
deterministic order and wrap.

**Design default:** the tree does not participate in that cycle; its own
command transfers focus. Window deletion MUST NOT discard unsaved text.

#### Scenario: Close a view of unsaved text

- **GIVEN** a modified buffer displayed in two windows
- **WHEN** the user deletes one window
- **THEN** the buffer and its unsaved text survive in the session
- **AND** focus selects a valid remaining editing window.

### Requirement: R16 Emacs-style motion

Source: section 9.1. Acceptance: A03.

The following keys and command names MUST retain the listed intent. Motion at
document boundaries MUST clamp or be a safe no-op. Word motion MUST be
consistent for Unicode text and punctuation without requiring language-aware
structural navigation. Arrow, Home, End, and Page keys MAY mirror these commands.

| Keys | Command | Behavior |
| --- | --- | --- |
| `C-f` | `forward-char` | Next character/grapheme. |
| `C-b` | `backward-char` | Previous character/grapheme. |
| `C-n` | `next-line` | Next logical line, retaining intended column when possible. |
| `C-p` | `previous-line` | Previous logical line with consistent intended-column behavior. |
| `C-a` | `move-beginning-of-line` | Logical line start. |
| `C-e` | `move-end-of-line` | Logical line end. |
| `M-f` | `forward-word` | Next word boundary. |
| `M-b` | `backward-word` | Previous word boundary. |
| `M-<` | `beginning-of-buffer` | Buffer start. |
| `M->` | `end-of-buffer` | Buffer end. |
| `C-v` | `scroll-up-command` | Page forward. |
| `M-v` | `scroll-down-command` | Page backward. |
| `C-l` | `recenter-top-bottom` | Recenter the selected view around point. |

#### Scenario: Retain intended column across a short line

- **GIVEN** point at a display column beyond the end of the next logical line
- **WHEN** `C-n` moves onto that short line and then onto a longer line
- **THEN** the short line uses a valid boundary
- **AND** the later move recovers the intended column where possible.

### Requirement: R17 Direct insertion and deletion

Source: sections 9.2-9.3. Acceptance: A02, A03, A09.

Printable input MUST insert at point. Enter MUST insert a line break, and Tab
MUST insert a tab unless a different behavior is explicitly documented.
Backspace MUST delete the preceding character/grapheme without crossing buffer
start. `C-d` / `delete-char` MUST delete the following character/grapheme without
crossing buffer end.

`C-k` / `kill-line` MUST kill to line end, or the line separator when already
at line end. `M-d` / `kill-word` and `M-DEL` / `backward-kill-word` MUST kill in
their respective directions. Region deletion MUST update point and mark
consistently. Every affected view MUST retain valid positions after an edit.
Control/Meta combinations MUST enter the keymap rather than insert their
printable representation or unknown control bytes.

#### Scenario: Kill a CRLF line separator

- **GIVEN** point at the end of a line followed by CRLF
- **WHEN** `C-k` kills its line separator
- **THEN** the logical separator is removed coherently and retained for yanking
- **AND** all views remain at valid positions.

### Requirement: R18 Mark and linear region

Source: section 10.1. Acceptance: A08.

`C-SPC` / `set-mark-command` MUST set mark at point and activate the region.
Subsequent movement MUST extend or shrink its visible range in either
direction. `C-x C-x` / `exchange-point-and-mark` MUST exchange endpoint positions
and activate the region as appropriate. `C-g` / `keyboard-quit` MUST cancel or
deactivate the active selection or pending interaction.

Mark belongs to the buffer; point and viewport belong to the view. They MUST
NOT collapse into one global cursor. Ordinary edits SHOULD deactivate the
transient selection consistently with Emacs-style behavior.

#### Scenario: Extend a backward region

- **GIVEN** mark set at point in the selected view
- **WHEN** the user moves backward and exchanges point and mark
- **THEN** the ordered range denotes the same selected text
- **AND** its visual highlighting follows the current endpoints.

### Requirement: R19 Linear kill ring and yank replacement

Source: sections 9.2, 10.1. Acceptance: A09.

`C-w` / `kill-region` MUST remove the active region into the kill ring.
`M-w` / `kill-ring-save` MUST copy it without editing. `C-y` / `yank` MUST insert
the newest ring entry. `M-y` / `yank-pop` MUST replace precisely the text from
the immediately preceding yank/yank-pop sequence with an older entry.
Unrelated commands MUST invalidate that special replacement state.

The ring MUST hold multiple entries. **Design default:** retain 32 linear
entries in session memory. Consecutive directional kills SHOULD append or
prepend appropriately; an intervening non-kill command breaks coalescing.
Empty-ring and inapplicable yank-pop actions MUST preserve text and show a
short message. OS clipboard integration and cross-session ring persistence
are not required.

#### Scenario: Rotate and then invalidate a yank

- **GIVEN** at least two kill-ring entries
- **WHEN** the user invokes `C-y`, then `M-y`, then an unrelated command
- **THEN** the first `M-y` replaces exactly the inserted range with the older entry
- **AND** a later `M-y` cannot replace text using the invalidated range.

### Requirement: R20 Rectangle selection and block editing

Source: section 10.2. Acceptance: A10.

The following rectangle commands MUST be implemented. Rectangle columns MUST
be display columns, not byte offsets. Operations MUST NOT divide a grapheme,
combining sequence, or double-width character into invalid text. Copying a
short source line MUST NOT invent trailing text. Insertion MAY add spaces only
where needed to reach the target column. The rectangle store MUST be separate
from the linear kill ring. Each rectangle mutation MUST undo as one coherent
operation over all affected lines.

| Keys | Command | Behavior |
| --- | --- | --- |
| `C-x SPC` | `rectangle-mark-mode` | Toggle rectangle selection for the active interaction. |
| `C-x r k` | `kill-rectangle` | Remove and retain the selected rectangle. |
| `C-x r M-w` | `copy-rectangle-as-kill` | Retain the rectangle without editing. |
| `C-x r y` | `yank-rectangle` | Insert the retained rows at point across successive lines. |

The clarification table defines boundary behavior for tabs and wide
clusters. Additional clear/open/number/register operations are outside scope.

#### Scenario: Rectangle spans short and mixed-width lines

- **GIVEN** selected display columns across a tab, a wide grapheme, and a short line
- **WHEN** the user copies, kills, and yanks the rectangle
- **THEN** whole Unicode text and adjacent unselected content remain valid
- **AND** copy adds no absent trailing content and insertion pads only as needed.

#### Scenario: Undo a multi-line rectangle

- **GIVEN** a rectangle command changed several lines
- **WHEN** the user invokes undo once
- **THEN** every affected line is restored as one transaction.

### Requirement: R21 Per-buffer undo and redo

Source: section 11. Acceptance: A12, A19.

Every text mutation MUST be undoable while its history remains available.
History MUST belong to its buffer. `C-/`, `C-_`, and `C-x u` MUST invoke `undo`
when their sequences can be reported by the terminal. `M-x undo-redo` MUST
reapply an undone edit. Unavailable navigation MUST preserve text and show a
short message.

**Design default:** conventional undo/redo, with redo discarded after a new
edit. This documented choice does not promise Emacs's full undo-of-undo model.
Consecutive ordinary insertions SHOULD form sensible groups. Large paste,
region kill, and rectangle changes SHOULD each form one transaction.
Undo/redo MUST restore rope content, valid cursor positions, dirty state, and
syntax invalidation. Saving MUST NOT discard history. Dirty tracking MUST
support returning to a retained saved state rather than relying only on an
ever-increasing revision counter.

#### Scenario: Undo back to the saved state

- **GIVEN** a successfully saved buffer followed by one text transaction
- **WHEN** the user undoes that transaction
- **THEN** the saved text and clean indication are restored
- **AND** redo can restore the edited text and dirty indication.

#### Scenario: Branch after undo under the conventional default

- **GIVEN** an edit has been undone
- **WHEN** the user makes a different text edit
- **THEN** the former redo future becomes unavailable
- **AND** undo still reverses the new edit coherently.

### Requirement: R22 One static command registry

Source: sections 12.1-12.2, 18.3. Acceptance: A11, A17.

Bindings and `M-x` MUST invoke the same Rust command handler. The registry
SHOULD be a direct static mapping to functions or a small enum. Every listed
command MUST work; a functional-looking stub is not an implemented command.
Public names MUST be lowercase and hyphenated, preferring the corresponding
Emacs name. Internal identifiers use idiomatic Rust naming under R36.

The registry MUST NOT introduce a command interpreter or public plugin
protocol. Names MUST NOT imply absent Emacs subsystems.

#### Scenario: Equivalent named and bound command

- **GIVEN** equivalent sessions with the same active buffer and layout
- **WHEN** one invokes `C-x 3` and the other invokes `M-x split-window-right`
- **THEN** both execute the same operation and produce equivalent split behavior.

### Requirement: R23 Minibuffer command entry and arguments

Source: section 12.3. Acceptance: A04, A05, A11.

`M-x` MUST activate a labeled command prompt. Typing MUST filter known commands;
prefix or substring matching SHOULD be supported with deterministic ordering.
A small candidate list MAY
appear without needlessly obscuring editing. `TAB` MUST select or complete a
candidate; `RET` MUST invoke it. Unknown names MUST report a concise error,
return control to editing, and leave document text unchanged.

Argument-taking commands MUST prompt in a predictable sequence. Focused
minibuffer editing MUST NOT edit the document underneath. `C-g` MUST cancel
without invoking the pending command. **Design default:** session-local
command history with previous/next navigation.

#### Scenario: Cancel a partially entered file command

- **GIVEN** a file command is waiting for a path in the minibuffer
- **WHEN** the user edits the prompt and presses `C-g`
- **THEN** the prompt closes without opening or writing a file
- **AND** its typed argument has not entered the document.

### Requirement: R24 Prefix handling and cancellation

Source: sections 4.3, 12.4. Acceptance: A08, A10, A11, A20.

The input interpreter MUST recognize partial and nested sequences, including
`C-x` and `C-x r`. A prefix MUST wait for its next chord without inserting
prefix text. Invalid sequences MUST preserve document content, clear the
prefix, and show a short message. `C-g` MUST cancel prefixes, prompts, search,
and transient interaction according to their active context. A complete
sequence SHOULD dispatch exactly one logical command.

Pending input MUST NOT prevent event processing needed for terminal cleanup.
One explicit focused input context MUST determine whether the event belongs
to a prompt, tree, search, or editing command.

#### Scenario: Cancel a nested rectangle prefix

- **GIVEN** the interpreter has received `C-x r`
- **WHEN** the user presses `C-g` and then types ordinary text
- **THEN** the prefix is canceled
- **AND** the ordinary text is handled by the restored editing context once.

### Requirement: R25 Real terminal input and paste

Source: sections 12.5, 16. Acceptance: A02, A10, A12, A20.

Input MUST use a terminal-aware event library. Key presses MUST NOT be applied
again as releases. Repeats MAY repeat input only where appropriate. Resize
events MUST NOT become editing commands. Unknown escape sequences and terminal
control bytes MUST NOT become accidental saved text. No mouse is required.

Meta handling MUST support combinations the selected library can distinguish
and SHOULD support compatible Escape-prefixed chords. Standalone Escape and
a Meta prefix MUST use a small, consistent disambiguation policy. Required
target-terminal checks include `C-SPC`, `C-/`, `M-<`, `M->`, and rectangle keys;
the alternate undo bindings MUST work where reportable. Unavoidable terminal
limitations MUST be documented.

Bracketed paste SHOULD insert literal pasted text as one undoable operation.
Text that resembles key sequences MUST NOT execute commands during paste.
Unicode MUST survive insertion, motion, selection, save, and reopening.

#### Scenario: Paste text resembling editor commands

- **GIVEN** bracketed paste is supported
- **WHEN** a multi-kilobyte paste contains newlines and text resembling `C-x C-c`
- **THEN** that text is inserted literally in one undoable operation
- **AND** no quit or prefix command is invoked by the pasted content.

### Requirement: R26 Incremental literal search

Source: section 13. Acceptance: A13.

`C-s` / `isearch-forward` and `C-r` / `isearch-backward` MUST search the active
buffer incrementally. The query MUST appear in the minibuffer; typing MUST
update the match without editing the rope. Repeating the search chord MUST
select the next/previous match. `RET` MUST accept the match; `C-g` MUST cancel
and restore the starting point. Failed search MUST preserve a valid position.

Search MUST cross rope chunk and UTF-8 boundaries. Match highlighting SHOULD
make the result visible. **Design default:** literal smart-case search,
case-insensitive without uppercase query characters and case-sensitive
otherwise. Regex and project-wide search are outside this baseline.

#### Scenario: Search across a rope boundary and cancel

- **GIVEN** a matching Unicode string spans internal rope chunks
- **WHEN** the user searches for it and then cancels with `C-g`
- **THEN** the complete string can be found without changing content
- **AND** cancellation restores the original point.

### Requirement: R27 Interactive query replace

Source: section 13. Acceptance: A13.

`M-%` / `query-replace` MUST prompt for search and replacement text and offer
yes, no, and quit decisions for occurrences. Only accepted occurrences MUST
change. History MUST remain meaningful, using the same checked editing path
as other commands. Replacement is literal and confined to the active buffer;
regex replacement and multi-file replacement are outside scope.

#### Scenario: Replace, skip, then stop

- **GIVEN** three occurrences in the active buffer
- **WHEN** the user accepts the first, skips the second, and quits at the third
- **THEN** only the first occurrence changes
- **AND** undo reverses the accepted replacement without changing skipped text.

### Requirement: R28 Left tree, root, and focus

Source: sections 14.1-14.2. Acceptance: A14.

The file explorer MUST be a narrow left-side tree with hierarchy and a clear
selected entry. It MUST show/hide without changing open buffers. All actions
MUST be keyboard-accessible, including predictable transfer between tree and
editor focus. Opening a file MUST use the common `find-file` machinery and
display it in the selected editing window.

**Design defaults:** `M-x toggle-file-tree` toggles the tree, with optional
`C-x t t`; the root is the current directory or the initial file's parent
where appropriate. Tree-specific public names MAY use `tree-open`, `tree-close`,
`tree-refresh`, `tree-create`, `tree-rename`, and `tree-delete`. Exact tree keys
are implementation defaults. Git metadata MUST NOT be required or scanned.

#### Scenario: Open a visited file from the tree

- **GIVEN** the left tree is focused and a selected file already has a buffer
- **WHEN** the user presses Enter on that file
- **THEN** the selected editor window displays its existing buffer
- **AND** showing or hiding the tree does not close that buffer.

### Requirement: R29 Tree traversal and refresh

Source: sections 14.2-14.3, 17. Acceptance: A14.

Up/Down MUST move among visible entries. Right/Left or Enter on a directory
MUST provide expansion and collapse. Refresh MUST reload entries without
restarting. Children SHOULD load lazily; traversal MUST avoid symlink cycles.
**Design default:** symlinked directories remain leaves unless support is
deliberately extended and validated.

Read and traversal failures MUST report the relevant path/operation and leave
the tree usable. Filesystem watching is not required. Icons, Git decorations,
project tabs, bookmarks, and tree plugins are outside the requested tree.

#### Scenario: Expand an inaccessible or cyclic entry

- **GIVEN** visible entries include an inaccessible directory and a symlink cycle
- **WHEN** the user navigates and attempts expansion
- **THEN** inaccessible access reports an error without breaking navigation
- **AND** the cycle cannot cause unbounded recursive traversal.

### Requirement: R30 Safe tree filesystem operations

Source: sections 7.3, 14.2-14.3, 17. Acceptance: A14, A19.

The tree MUST support creating a file or directory at the selected directory,
renaming with conflict checks, deleting with confirmation, and refresh.
Creation and rename MUST prompt for names at the selected location and check
conflicts before modifying disk. A
nonempty directory MUST NOT be recursively deleted without a separate explicit
confirmation identifying that it contains children. Rename/delete MUST require
clear confirmation where data loss is possible.

Failed operations MUST preserve a usable tree and show the path/error.
Successful rename of a visited file MUST update the buffer association.
Tree and buffer presentations MUST reflect opens, saves, renames, and removals
without restart. Open buffers MUST retain their rope content even when a disk
path is removed. The model records association reconciliation.

#### Scenario: Rename a visited file

- **GIVEN** a tree entry is visited by an open buffer
- **WHEN** rename succeeds after required conflict handling
- **THEN** the buffer follows the new visited path
- **AND** the tree and buffer label reflect the successful rename.

#### Scenario: Cancel nonempty-directory deletion

- **GIVEN** a directory contains files
- **WHEN** the user cancels the explicit nonempty-directory confirmation
- **THEN** no recursive deletion occurs
- **AND** navigation and open buffer contents remain usable.

### Requirement: R31 Bundled Tree-sitter highlighting

Source: sections 15.1-15.2, 15.4. Acceptance: A15, A17.

Supported languages MUST use bundled or compiled Tree-sitter grammars and
highlight queries. Supported captures MUST distinguish common keywords,
strings, comments, types, functions, numbers, and punctuation/operators as
provided by the grammar. Highlighting MUST be visual only and preserve
document content. Unsupported files MUST remain fully editable as plain text.

**Design default:** Rust first, selected by extension, with a safe plain-text
fallback and one small readable palette. Extra fixed grammars MAY be added,
but no language list beyond Rust is approved by this baseline. Runtime grammar
installation, semantic completion, keyword popup completion, and a substitute
language-server framework MUST NOT be introduced.

#### Scenario: Highlight Rust and retain plain-text fallback

- **GIVEN** a Rust file and a file with an unsupported extension
- **WHEN** the user opens and edits both
- **THEN** Rust keywords, strings, and comments receive suitable styles
- **AND** the unsupported file remains editable with `Text` status.

### Requirement: R32 Incremental syntax state and failure isolation

Source: sections 6.4, 15.1, 15.3. Acceptance: A15, A16, A20.

Parsed buffers SHOULD retain parser and tree state. Each edit MUST translate
to the correct Tree-sitter byte range and row/byte-column edit. Parsing SHOULD
update incrementally rather than unconditionally restarting on every key.
Coloring SHOULD be limited to visible text where practical. The renderer MUST
NOT treat syntax byte columns as display columns.

Incomplete syntax, parser failure, or query failure MUST NOT block editing.
The editor MUST fall back to plain text or still-valid styling. Stale syntax
coordinates MUST NOT be applied to a different text state as if current.
Region, cursor, tree, prompt, and status readability MUST survive highlighting.

#### Scenario: Parser failure during a shared-buffer edit

- **GIVEN** two views display a parsed buffer
- **WHEN** the user edits multibyte text and syntax processing fails
- **THEN** the shared rope and history retain the accepted edit
- **AND** both views remain usable without invalid stale highlight ranges.

### Requirement: R33 Small ownership structure and event flow

Source: sections 18.1-18.3. Acceptance: A11, A16, A18.

One executable and a small collection of ordinary Rust modules SHOULD own the
application. `App` SHOULD own buffers, the layout, ring, input state, tree, and
application messages. The terminal guard owns restoration. Buffers own text,
file identity, mark, history, and content-specific parsing. Windows reference
stable buffer identities and own their point and viewport.

Drawing SHOULD borrow application state immutably and output mutably only
while drawing. Event handlers SHOULD borrow the smallest needed mutable state.
Avoid pervasive `Rc<RefCell<_>>`, trivial wrapper modules, one-use abstraction
layers, and traits without a concrete substitution need. The conceptual owners
in the model are not a mandate for a multi-crate workspace or a module per row.

The event flow MUST normalize one event, route it to the focused context,
resolve a complete command where needed, perform its state transition, update
dependent positions/history/syntax, and redraw/flush as appropriate.

#### Scenario: A command has one mutation owner

- **GIVEN** a text command is reachable by binding and name
- **WHEN** either entry point executes it
- **THEN** the same buffer-owned transition updates text and dependent state
- **AND** drawing reads that state without maintaining another editable copy.

### Requirement: R34 Minimal dependencies

Source: sections 1, 18.4. Acceptance: A17, A18.

Dependencies MUST be few, explicit, and justified. Suitable starting candidates
are `crossterm`, `ropey`, `tree-sitter` with a Rust grammar and highlight query,
optionally `tree-sitter-highlight`, and small Unicode segmentation/width crates.
Versions SHOULD be selected, fixed, and tested when introduced. The prototype's
`crossterm` 0.29 reference is not a promise that it is the correct final version.

A generic TUI framework, async runtime, database, large logging framework,
plugin manager, or scripting engine MUST NOT be added without demonstrated
need and scope authority. Reviewing another editor MUST NOT silently turn it
into a runtime, build, or path dependency.

#### Scenario: Standalone build and runtime

- **GIVEN** the approved source and its declared build dependencies
- **WHEN** the editor is built and later run offline
- **THEN** it does not need an adjacent editor checkout, Ruby runtime, or runtime grammar installer.

### Requirement: R35 First-party safe Rust

Source: section 18.5. Acceptance: A18.

Every first-party crate root MUST enforce `#![forbid(unsafe_code)]`.
First-party source MUST NOT contain unsafe blocks, raw-pointer manipulation,
custom FFI, or unsafe trait implementations. A source check or lint MUST
confirm that boundary. Mature dependencies MAY internally use unsafe code;
the first-party policy does not claim to constrain their implementations.

#### Scenario: Unsafe implementation is rejected

- **GIVEN** the first-party safety policy is enabled
- **WHEN** an implementation introduces an unsafe block or unsafe implementation
- **THEN** the mandatory safety checks reject that candidate.

### Requirement: R36 Complexity and naming limits

Source: sections 19.1-19.2. Acceptance: A18.

Production functions MUST have cyclomatic complexity at most 10, targeting 5
or fewer where feasible. Every function/method MUST have at most five parameters,
including a receiver. Nesting SHOULD remain at most two logical levels; three
is the exceptional ceiling. Decompose cohesive responsibilities rather than
hide unrelated inputs in a parameter object or add score-reducing indirection.
Complexity MUST be measured or reviewed by disciplined branch counting;
formatting and ordinary Clippy alone do not prove the cap.

Identifiers MUST be concise and precise: one meaningful word where possible,
at most two when needed, with idiomatic Rust casing. Use editor vocabulary such as
`buffer`, `point`, `mark`, `view`, `ring`, and `cell`. Avoid opaque abbreviations,
letter soup, sprawling names, and comments that compensate for poor naming.
Canonical public Emacs command strings are exempt from the internal word limit.

#### Scenario: Review a completed increment

- **GIVEN** a candidate completes editor behavior
- **WHEN** its code is checked for acceptance
- **THEN** complexity, parameter counts including receivers, nesting, and naming are examined explicitly
- **AND** an over-cap production function is decomposed before acceptance.

### Requirement: R37 Restrained comments and expected errors

Source: sections 19.3-19.4. Acceptance: A18, A19, A20.

Implementation comments MUST be exceptional and explain non-obvious reasons
or contracts. Rustdoc is appropriate for public contracts not evident from
signatures. Narration of obvious private functions, fields, variables, and
branches, dead commented-out code, decorative blocks, and boilerplate headers
MUST NOT be added. Detailed rationale belongs in specifications or design notes.

Fallible operations MUST return typed results with useful operation/path
context. Recoverable production conditions MUST NOT use `unwrap`, `expect`,
or panic. Tests MAY use them to abort a failing test. Missing files, malformed
input, small terminals, parser errors, and unavailable history are ordinary
outcomes. Cleanup SHOULD use RAII and normal unwinding.

#### Scenario: Report an ordinary inaccessible file

- **GIVEN** a path cannot be read
- **WHEN** the user invokes `find-file`
- **THEN** an operation/path error is shown without a normal-use backtrace or panic
- **AND** the current document and terminal session remain usable.

### Requirement: R38 Mandatory increment checks

Source: section 19.5. Acceptance: A18.

Every completed implementation increment MUST pass these commands:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

It MUST also pass the first-party unsafe check and explicit review of R36-R37.
Repository tooling for those checks SHALL be defined during approved planning
and implementation; this document does not claim that tooling already exists.
Tests MUST target observable behavior rather than elaborate private mocks.

#### Scenario: Tests pass but a required quality check fails

- **GIVEN** a candidate passes its behavioral tests
- **WHEN** a mandatory lint, safety, complexity, or parameter check fails
- **THEN** the increment remains incomplete until the required check passes.

### Requirement: R39 Behavior, integration, and adversarial coverage

Source: sections 17, 20.1-20.3. Acceptance: A01-A20.

Terminal-independent behavior MUST be testable without starting a terminal.
Coverage MUST include rope boundary edits; empty and multiline text; UTF-8 and
grapheme/cell conversion; CRLF and final-newline fidelity; regions and mark;
kill rotation/coalescing/yank validity; rectangles and atomic undo; buffer
identity; nested windows and focus; command/prefix cancellation; per-buffer
history and dirty state; tree/path safety; and syntax coordinate conversion.

Integration coverage MUST exercise the actual delivered entry points for:
new text and save/reopen; multiple buffers; nested shared-buffer splits;
linear and rectangular copy/kill/yank; yank-pop; undo/redo; name/binding parity;
tree open/rename and path reconciliation; Rust highlighting and plain-text
fallback; resize with tree and splits; failed saves; canceled dirty quit; and
terminal restoration. PTY-driven interactions SHOULD be used where practical.
Pure state tests MUST NOT rely on terminal timing or exact color sequences.

Adversarial cases MUST include emoji, combining marks, tabs, CJK, empty and
one-line files, long lines, CRLF, tiny geometry, repeated split/close, large
paste containing key-like text, failed search, canceled conflicting rename,
canceled nonempty delete, invalid yank-pop, post-raw-mode I/O failure,
symlink cycles, and inaccessible directories. Failures MUST preserve the
relevant text, history, layout, and terminal invariants.

#### Scenario: Acceptance evidence is incomplete

- **GIVEN** the editor can draw highlighted text and pass isolated logic tests
- **WHEN** data-safety and terminal-lifetime scenarios have not been exercised
- **THEN** that evidence does not establish complete baseline acceptance.

### Requirement: R40 Representative performance validation

Source: sections 6.4, 20.4. Acceptance: A16, A18, A20.

Rendering SHOULD process visible text where practical. Search and edits MUST
avoid whole-file copies per key. Syntax SHOULD update incrementally. A
documented representative UTF-8 fixture with tens of thousands of mixed-length
lines SHALL exercise startup, insert/delete, scrolling, split redraw, and
highlighting. Record the fixture, hardware context, and measurements.

Acceptance requires no visible pathological behavior such as unbounded rebuild
per key, repeated full-document copying, or sustained idle repaint loops.
Numeric latency or maximum-file-size guarantees MUST NOT be claimed without
measurements on identified hardware. Broad performance infrastructure is not
required to validate the representative workload.

#### Scenario: Measure a large mixed-width source fixture

- **GIVEN** a documented large UTF-8 fixture and recorded hardware context
- **WHEN** startup, editing, scrolling, shared splits, and highlighting are exercised
- **THEN** the recorded results identify any pathological work
- **AND** claims are limited to the workloads and measurements actually observed.

## Retained design defaults

| Area | Retained baseline default |
| --- | --- |
| Invocation | Zero arguments or one path; absent file opens an unsaved visiting buffer. |
| Tree | Initially hidden, left side, current directory or initial file parent as root. |
| Tree toggle | `toggle-file-tree`; optional `C-x t t`. |
| Tree focus | Separate from `other-window` cycling. |
| Text display | No soft wrap; horizontal scrolling; visual tab stops every four cells. |
| Character motion | User-perceived grapheme clusters where possible. |
| Line endings | Preserve existing LF/CRLF; inserted newlines follow prevailing style. |
| Kill ring | 32 in-memory linear entries; separate rectangle storage. |
| History | Conventional per-buffer undo/redo; new edit invalidates redo. |
| Search | Literal smart-case; interactive in-buffer replacement. |
| Syntax | Bundled Rust grammar and query; plain-text fallback; one fixed palette. |
| Save | Same-directory temporary output and replacement after successful write. |
| Commands | Static Rust registry; session-local command-name history. |
| Platform | Linux terminal first, without broad platform certification. |

## Accepted adaptation clarifications

These accepted choices make cross-component behavior reviewable. They do not
claim that the source already specified every detail.

| ID | Accepted clarification | Reason and boundary |
| --- | --- | --- |
| D01 | Protect modified unnamed buffers on quit as well as file-backed buffers. | Preserve the source's general no-silent-loss acceptance rule. |
| D02 | Qualify prepared ranges with buffer identity and text revision; map every live view after accepted edits. | Prevent stale ranges and cross-view position corruption without requiring a public anchor framework. |
| D03 | Track a retained saved text state independently of monotonic publication revisions. | Undo to a saved state can become clean; failed saving cannot move the save point. |
| D04 | Rectangle edges touching a wide grapheme include it whole. A partially selected tab is represented by the selected existing cells as spaces; untouched tabs remain unchanged. Copy never pads beyond end of line. | Define deterministic cell-based operations without invalid Unicode or invented trailing source text. |
| D05 | Save outcomes distinguish not published, published with confirmed completion, and published with a later unresolved failure. Only a fully successful save clears dirty state or satisfies a required save-before-exit. | Preserve truthful publication and conservative data-safety behavior after replacement. |
| D06 | A new file association becomes effective only after publication; a published but uncertain attempt retains its target and warning. | Keep failed save-as attempts from falsely relocating a buffer while retaining actual publication facts. |
| D07 | Tree deletion of a visited path retains its buffer and marks the disk association missing; recreating it requires an explicit save. | Filesystem deletion cannot erase unsaved in-memory text or silently recreate a path. |

The model describes these proposals without prescribing a custom rope,
branching history, background workers, or a reusable extension framework.
No additional language grammar or user configuration system is selected.

## Related Rust design evidence

The product owner requested a bounded review of `ruru` to avoid unnecessary
semantic divergence. Static source review used revision
`bb1b96924bad0475a25d33a446a452b577df044c`. No `ruru` tests or benchmarks were
run for this adaptation. These are reviewed design references, not fresh
runtime-validation claims and not a dependency declaration.

| Reviewed source in `ruru` | Useful contract for `rev` | Adaptation boundary |
| --- | --- | --- |
| `src/buffer.rs`, `src/coordinates.rs`, `docs/reference/buffer-core.md` | One text publication owner, qualified positions, UTF-8-safe checked ranges. | `ruru` uses qualified byte positions and a custom persistent rope. `rev` may use Ropey's character coordinates with explicit byte conversion; no storage implementation is copied. |
| `src/transaction.rs`, `src/history.rs`, `src/anchors.rs` | Prepare a coherent edit before publication; keep text, history, and dependent positions consistent. | `rev` needs ordinary command transactions, not nested Ruby capabilities, a public anchor registry, or branching retained history. |
| `crates/ruru-view/src/state.rs` | Distinct view identity, point, top position, desired display column, and geometry. | Keep per-view state; do not inherit the reference's surface-size caps, generation families, or accounting system. |
| `crates/ruru-file/src/publication.rs`, `crates/ruru-session/src/saving.rs` | Publication is irrevocable; later failures cannot erase an actual replacement. | Apply truthful outcomes and saved-state reconciliation with a small synchronous file owner where sufficient. |

Rust owns the underlying contracts in this evidence. Ruby's command/configuration
and hosting roles in `ruru` do not transfer to `rev`; the corresponding fixed
product commands and policy remain direct Rust code here.

## Acceptance contract

The A-identities are preserved from the supplied reference. All are required
for complete delivery through ordinary user operations, without debug commands,
code edits, plugins, or external helper programs during normal editing.

| ID | Scenario and pass condition | Owning requirements |
| --- | --- | --- |
| A01 | Launch into an editable empty TUI; exit restores the terminal. | R01-R05 |
| A02 | Printable text enters at point without changing modes. | R01, R17, R25 |
| A03 | Required Emacs motion works with valid Unicode and display positions. | R07, R09, R16-R17 |
| A04 | Find, save, and write-file work with safe error handling and text fidelity. | R02, R10, R12-R13 |
| A05 | Buffer switching, killing, and listing operate independently of windows. | R04, R11-R12 |
| A06 | Both split directions nest; views share text and retain independent point. | R06, R08, R14 |
| A07 | Delete-window, delete-other-windows, and other-window preserve buffers. | R14-R15 |
| A08 | Mark, motion, exchange, and cancellation manage a visible region. | R07, R18, R24 |
| A09 | Region/word/line kills, copying, yank, and yank-pop obey ring semantics. | R17-R19 |
| A10 | Rectangle selection, copy, kill, yank, and atomic undo preserve adjacent text. | R09, R20, R24-R25 |
| A11 | Hyphenated `M-x` commands and bindings invoke the same working handlers. | R12, R22-R24, R33 |
| A12 | Per-buffer undo/redo recover insertion, deletion, and paste coherently. | R21, R25 |
| A13 | Incremental forward/backward search and query-replace preserve text safety. | R26-R27 |
| A14 | The keyboard tree toggles, navigates, expands, opens, creates, renames, and deletes safely. | R28-R30 |
| A15 | Tree-sitter highlights Rust; unsupported text and parser failures remain editable. | R07, R09, R31-R32 |
| A16 | Every editable buffer remains rope-backed across load, edits, and history. | R08-R10, R32, R40 |
| A17 | The delivered product stays within the explicit runtime and feature scope. | R01, R22, R31, R34 |
| A18 | Safe Rust, five-parameter limit, complexity cap, shallow nesting, concise names, tests, and lints pass. | R33-R40 |
| A19 | Dirty close/quit is guarded; failed saves retain text; external changes warn. | R04, R10-R13, R21, R30, R37 |
| A20 | Resize and ordinary failures preserve editing state and terminal lifetime. | R03-R07, R24-R25, R32, R37, R39-R40 |

## Reference coverage

| Reference sections | Adaptation owner |
| --- | --- |
| 1-3: authority, identity, scope, vocabulary | Purpose, Scope and priority, Vocabulary, R01 |
| 4: launch, terminal lifetime, quit | R02-R04 |
| 5: layout, status, resize, viewport | R05-R07 |
| 6: rope, coordinates, text fidelity, efficiency | R08-R10, R32, R40 |
| 7: buffers, file commands, safe saving | R11-R13 and model file/publication records |
| 8: nested windows and focus | R14-R15 and model layout/view records |
| 9: movement, insertion, deletion, modifiers | R16-R17, R25 |
| 10: linear region, kill ring, rectangles | R18-R20 and model region/ring/rectangle records |
| 11: history | R21 and model history/save-state records |
| 12: commands, prompts, prefixes, terminal keys | R22-R25 |
| 13: search and replacement | R26-R27 |
| 14: simple file tree and filesystem safety | R28-R30 |
| 15: Tree-sitter and completion exclusion | R31-R32 |
| 16-17: terminal events, Unicode, ordinary failures | R03-R04, R06-R07, R09-R13, R17-R21, R24-R32, R37, R39 |
| 18: ownership, event flow, dependencies, safe Rust | R33-R35 and model ownership map |
| 19: code quality and mandatory checks | R36-R38 |
| 20: logic, integration, adversarial and performance validation | R39-R40 |
| 21: observable acceptance | A01-A20, preserved above |
| 22: tutor workflow and suggested stages | Pedagogy excluded by owner clarification; functional outcomes retained across R01-R40. No suggested stage is treated as a fixed slice count. |
| 22.3: prototype continuity | Reuse was conditional on correctness. The incomplete local prototype was discarded under explicit owner authorization. |
| 23-24: design defaults and completion | Retained design defaults, Scope and priority, Acceptance contract |
| 25: technical references | References below; these do not expand the specified subset. |

## Delivery boundary

The requested implementation shall be prepared as one complete, bounded Looper
cycle covering the full baseline. The approved plan must enumerate all
required slices and dependencies before execution, with no arbitrary small
slice quota. Detailed supporting documents may distribute implementation
knowledge while one canonical plan and tracker retain execution authority.

The [delivery plan](../plan/rev-editor.md) and
[tracker](../tracker/rev-editor.md) record the approved slice map, decisions, and
execution state. This specification does not replace their validation evidence.

## References

The format follows the shared workspace specification contract's OpenSpec
baseline syntax, referenced at OpenSpec 1.15.0, source revision
`6d04a1b58dedc95729d34e4f223d9d483aae210a`. No CLI, Node.js dependency, or format
service is required.

- [GNU Emacs windows](https://www.gnu.org/software/emacs/manual/html_node/emacs/Windows.html)
- [GNU Emacs mark and region](https://www.gnu.org/software/emacs/manual/html_node/emacs/Mark.html)
- [GNU Emacs killing and moving text](https://www.gnu.org/software/emacs/manual/html_node/emacs/Killing.html)
- [GNU Emacs rectangles](https://www.gnu.org/software/emacs/manual/html_node/emacs/Rectangles.html)
- [Ropey documentation](https://docs.rs/ropey/)
- [Tree-sitter incremental parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- [Tree-sitter highlighting](https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html)
- [Crossterm documentation](https://docs.rs/crossterm/)
