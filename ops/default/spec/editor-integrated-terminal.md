# Integrated Terminal Sessions

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.2
Format: OpenSpec capability proposal
Source: product-owner terminal presentation request and clarification to inherit Odra behavior with a Rust/Alacritty implementation
Model: [Integrated Terminal Model](editor-integrated-terminal-model.md)
Editor owner: [rev Editor](rev-editor.md)
Controls: [Editor Configuration](editor-configuration.md)
Delivery boundary: future terminal delivery; no implementation or slicing authorized
Readiness: behavioral reference and Rust engine direction established; exact dependency pin, native catalog integration and measured limits remain delivery preparation

## Purpose

Run interactive terminals inside rev. Opening a terminal uses a lower panel by
default; the user can instead open it directly as a terminal-backed buffer in
the main editing area. An existing session can move between these placements
in either direction while preserving its running process and terminal state.

A terminal-backed buffer makes full-area tools such as lazygit usable through
the editor's normal buffer navigation. Supporting that application does not
create a built-in Git UI or a Magit-like subsystem in rev.

## Reference and confirmed direction

Odra's integrated-terminal-sessions specification, reviewed at commit
f701c830b88e2088bb8de93ba9fee2dd0cd111f1, is the behavioral reference for session,
terminal-state, buffer/view, input, copy/paste and lifecycle ownership. rev
preserves those semantics with its requested placement controls and a Rust
terminal engine. Its native commands and TOML replace Odra's programmable
surface; Project/Workspace abstractions are not introduced.

- The default placement is a usable lower terminal panel.
- Opening directly in a normal buffer location is an explicit alternative.
- Multiple terminal sessions are supported as independent running sessions.
- Move the same live session from the lower panel to a buffer and back.
- Preserve the child process, interactive application, working state and
  retained terminal output across those moves.
- Use alacritty_terminal as the selected Rust terminal engine, with no
  libghostty-vt or Zig build/runtime requirement for this feature.
- Inherit Odra's session, close, input, copy/paste and presentation behavior.

## Rust terminal engine

Use the [alacritty_terminal library](https://docs.rs/alacritty_terminal/0.26.0/alacritty_terminal/)
behind a rev-owned adapter. The inspected reference release is 0.26.0. Its
Term/grid API supplies terminal state and renderable content; its tty and
event_loop modules also provide PTY-related components. The exact production
pin and feature set are established with the implementation candidate.

Keep process, PTY, emulator, buffer, view and render ownership explicit even
when the same crate supplies several mechanisms. Review its existing PTY/IO
facilities before introducing another PTY dependency. No separate PTY crate is
selected by this specification.

The [crate manifest](https://docs.rs/crate/alacritty_terminal/0.26.0/source/Cargo.toml.orig)
is the dependency reference. Integrate the library into rev's native render
and event ownership; do not launch the Alacritty desktop application or adopt
its window, font or application-configuration layer. The hosting terminal
continues to own fonts and ligatures. Dependency installation and Cargo changes
are not performed by this specification update.

## ADDED Requirements

### Requirement: TRM01 Default panel and direct buffer entry

Native commands SHALL support opening a terminal in the lower panel by default
and opening one directly as a terminal-backed buffer in the main editing area.
Buffer placement SHALL participate in ordinary buffer selection and native
view ownership. Panel geometry SHALL provide a usable interactive area rather
than a fixed single output line. Exact geometry and typed preference bounds
are implementation preparation under the native command contract below.

#### Scenario: Open the default terminal

- **GIVEN** an editor with room for an admitted lower panel
- **WHEN** the user invokes ordinary terminal creation
- **THEN** a live terminal is presented below the editing area
- **AND** ordinary source buffers remain available.

#### Scenario: Open a terminal directly as a buffer

- **GIVEN** a user wants the main editing area for an interactive tool
- **WHEN** they invoke direct terminal-buffer creation
- **THEN** a new terminal is selected as a terminal-backed buffer
- **AND** using the lower panel first is not required.

### Requirement: TRM02 Relocate one live session

Moving between the lower panel and a main-area buffer SHALL change presentation
ownership without replacing the terminal buffer identity, process/session or
terminal state. The session SHALL receive the newly admitted geometry through
its PTY owner. A failed move SHALL preserve the prior placement and live state.
Moving SHALL not copy a transcript into a new ordinary text buffer or relaunch
the child application.

#### Scenario: Move lazygit to a buffer and back

- **GIVEN** lazygit is running in a lower-panel terminal
- **WHEN** the user moves that terminal to the main buffer area and later back
- **THEN** the same process and application state remain active
- **AND** the terminal receives the current dimensions without restarting lazygit.

#### Scenario: Reject an unavailable placement

- **GIVEN** a live terminal and a relocation whose target geometry cannot be admitted
- **WHEN** the move is rejected
- **THEN** the original session and valid presentation remain available
- **AND** no orphan process or duplicate terminal is created.

### Requirement: TRM03 Multiple sessions with stable identities

rev SHALL own multiple independent terminal sessions and expose their stable
terminal-buffer identities through native navigation. Selecting or relocating
one SHALL not change another's process, input, output or working state. The
first buffer SHALL be named terminal, then terminal-2, terminal-3 and so on;
explicit rename SHALL preserve identity. One terminal-backed buffer MAY be
selected in multiple ordinary panes as views of the same session. The focused
live view SHALL own PTY resize authority; other views SHALL not create another
PTY or emulator. Each view retains its own viewport/copy navigation.

#### Scenario: Switch independent terminals

- **GIVEN** two live terminals with different applications or working directories
- **WHEN** the user selects and relocates one
- **THEN** only its attachment and geometry change
- **AND** the other session retains its state and receives no redirected input.

#### Scenario: Present one terminal in two panes

- **GIVEN** two panes show the same terminal-backed buffer
- **WHEN** focus and usable pane geometry change
- **THEN** one session/emulator receives geometry from the focused live view
- **AND** no second child is created and the other view retains its viewport.

### Requirement: TRM04 Real interactive terminal behavior

The capability SHALL own a local PTY process and admitted terminal-emulation
state sufficient for the selected interactive-shell/TUI scope, including
lazygit. Cursor movement, alternate-screen updates, colors, input modes and
resize SHALL operate within the terminal view. Child terminal-control bytes
SHALL be interpreted by that owner, not directly forwarded to overwrite the
hosting editor's screen. A process-output log alone SHALL not count as an
interactive terminal.

#### Scenario: Run an interactive full-screen tool

- **GIVEN** an admitted terminal session and lazygit available to its shell
- **WHEN** the user launches and interacts with lazygit
- **THEN** its live screen and keyboard behavior are usable in the terminal view
- **AND** its output does not take over unrelated editor panes.

### Requirement: TRM05 Session lifetime is distinct from visibility

Hiding, moving or detaching a presentation SHALL not terminate the process,
including detaching the final view. Closing a live terminal-backed buffer or
explicitly terminating its session SHALL request concise confirmation. A
declined confirmation SHALL preserve the session and buffer. Confirmed closure
SHALL own process termination, waiting and complete resource cleanup.

Editor exit with live terminals SHALL ask once whether to terminate those
sessions or cancel exit. Processes SHALL not implicitly survive an editor
restart. Normal child exit SHALL retain bounded final output and its actual
exit status in an inspectable buffer until explicit closure. Status text SHALL
not be injected into the child's output. A terminal buffer SHALL not be saved
as an ordinary source file.

#### Scenario: Hide the lower panel

- **GIVEN** a lower-panel terminal has a running process
- **WHEN** the user hides its presentation
- **THEN** the session remains owned and can be shown again
- **AND** hiding is not treated as permission to kill the process.

#### Scenario: Observe a process exit

- **GIVEN** an owned terminal application exits
- **WHEN** rev observes its completion
- **THEN** the session reports the actual outcome under the retention policy
- **AND** it does not remain labeled as a live interactive process.

#### Scenario: Decline terminal termination

- **GIVEN** a terminal-backed buffer owns a running shell or lazygit process
- **WHEN** the user attempts to kill that buffer and declines confirmation
- **THEN** its process, buffer and retained output remain intact
- **AND** the attempt is not treated as a pane-only hide or completed closure.

### Requirement: TRM06 Contextual keyboard and pointer ownership

Focused live terminal input SHALL be routed to the child under an explicit
terminal context. Ordinary editor Emacs/Vim motions SHALL not accidentally
consume interactive application input. Factory C-g/M-x recovery and the C-x
editor prefix SHALL remain reachable. Terminal-local commands resolve before
unclaimed input is encoded for the child. C-q/terminal-send-next-key SHALL send
exactly one following normalized key literally to the child, including a key
otherwise intercepted by the host.

Return, Tab, Backspace, Delete, arrows and paging SHALL use native child-mode
encoding in live mode. C-y/M-y use admitted terminal yank/yank-pop; C-l clears
the visible terminal screen. C-c C-t enters copy mode. Printable colon belongs
to the child while live terminal input owns focus; Vim host command entry is
available after selecting a compatible editor/copy context. Escape belongs to
the child unless a stronger host transient explicitly owns it.

When the child enables mouse reporting, ordinary pointer/wheel input SHALL be
encoded for it. Without that reporting, native terminal selection/scrolling
owns those events. Shift SHALL retain local selection/viewport control;
Control-click SHALL explicitly activate an admitted terminal hyperlink.

#### Scenario: Focus an interactive tool

- **GIVEN** a live terminal owns focus and an ordinary source buffer is also visible
- **WHEN** the user types an admitted child key sequence
- **THEN** it reaches the focused terminal session
- **AND** it neither edits the source buffer nor invokes an unrelated modal motion.

### Requirement: TRM07 Native commands and preferences

Creation, direct-buffer creation, session selection, relocation in both
directions and visibility controls SHALL be concrete native operations in
rev's existing command catalog. M-x and, when delivered, Vim colon entry SHALL
invoke the same operations in compatible host contexts. Appropriate panel,
directory, shell and terminal preferences SHALL use existing TOML/startup and
session-override conventions. Default shell creation SHALL use the user's
configured interactive shell; executable arguments/environment SHALL remain
structured values rather than implicitly evaluated shell strings.

Use rev's selected directory context, with the current file's containing
directory and launch directory as the existing admitted fallbacks. An existing
session SHALL retain its own working state. There SHALL be one instance-global
default terminal identity; no Project/Workspace registry is introduced.
terminal-open selects the live default or creates one; terminal-new always
creates an independent session. If the default exited, opening creates a new
default while retaining the old exited buffer. terminal-toggle remembers the
last selected terminal and lower-panel size, and restores previous focus/layout
when hidden without spawning a process on each toggle.

#### Scenario: Use either command-entry profile

- **GIVEN** an available terminal relocation operation
- **WHEN** it is selected through the active profile's command-entry surface
- **THEN** the same native session/view operation executes
- **AND** no alternate terminal implementation is selected by the keybinding style.

### Requirement: TRM08 Bounded IO and failure containment

Sessions, PTY handles, queues, emulator state, scrollback and render work SHALL
have explicit ownership and finite limits before implementation. IO SHALL not
block ordinary editing or create an idle busy loop. Creation, resize, output,
failure and cleanup SHALL preserve unrelated buffers and sessions. Resource
failure SHALL not leave an unowned process or a half-published attachment.

#### Scenario: Fail without losing editor state

- **GIVEN** ordinary unsaved buffers and an attempted terminal operation
- **WHEN** process creation or session/view admission fails
- **THEN** rev reports the operation's actual failure and retains existing editor state
- **AND** no hidden orphan session or partial source edit is published.

### Requirement: TRM09 External applications retain their own behavior

An application launched explicitly in the terminal SHALL run as a child tool,
not become a built-in rev subsystem. rev SHALL own terminal presentation and
process lifecycle without implementing lazygit's Git operations. This native
terminal capability SHALL not expand the restricted tool grants of AI commands
or silently become an external harness backend.

#### Scenario: Use lazygit without a built-in Git UI

- **GIVEN** lazygit is launched by the user inside a terminal buffer
- **WHEN** the user interacts with that program
- **THEN** lazygit owns its application operations and rev owns its session/view
- **AND** no Magit-like command layer or broader AI authority is implied.

### Requirement: TRM10 Rust engine and editor-owned rendering

Terminal emulation SHALL use alacritty_terminal through a narrow native
adapter. rev SHALL retain process/session, editor-buffer, attachment and output
ownership. The feature SHALL not require Zig, libghostty-vt, an Alacritty
desktop installation or a separate graphical window. The library's screen,
modes and damage observations SHALL be lowered through rev's admitted terminal
renderer. Library selection SHALL not replace the hosting terminal's font
selection or the editor's startup-selected Emacs/Vim interaction profile.

#### Scenario: Run the embedded terminal with the Rust library

- **GIVEN** a built rev candidate using the pinned Rust engine and no Zig toolchain or Alacritty desktop application
- **WHEN** the user opens and interacts with an integrated terminal
- **THEN** the terminal works inside the existing rev screen
- **AND** process and presentation behavior follows the native session contract.

#### Scenario: Keep fonts under the hosting terminal

- **GIVEN** rev runs in a terminal with a selected font and supported ligature behavior
- **WHEN** an alacritty_terminal-backed buffer is displayed
- **THEN** rev supplies terminal cells/styles under the same hosting frontend
- **AND** the embedded engine does not select another font or import desktop Alacritty settings.

### Requirement: TRM11 Retained output copy paste and external effects

Scrollback SHALL remain bounded independently of pane height. Manual retained
output navigation SHALL suspend tail-follow for that view without pausing the
child; returning to the tail resumes following. Copy mode SHALL use a stable
text projection and native selection/search/transfer semantics. Return leaves
copy mode without sending a child Return. C-g first clears an active selection,
then leaves copy mode when no selection remains. Copying SHALL retain the copy
viewport so more fragments can be selected.

User paste SHALL use native retained-text/clipboard authority and the child's
active bracketed-paste mode. Yank-pop SHALL act only while the preceding paste
is actually replaceable under the admitted input policy. Child-requested
clipboard reads/writes SHALL be denied initially, without blocking explicit
user copy/paste. Hyperlinks SHALL require explicit validated activation. Other
unsupported host-effect or graphics requests SHALL remain inert or receive a
compact bounded fallback; raw output SHALL never grant editor-command authority.

#### Scenario: Inspect output while the child continues

- **GIVEN** copy mode is inspecting older output and the child emits more text
- **WHEN** the user selects and copies a fragment
- **THEN** the stable viewport/selection remain usable and output continues under bounds
- **AND** Return leaves copy mode without executing a child command.

#### Scenario: Deny an unsolicited clipboard request

- **GIVEN** child output requests reading or replacing the system clipboard
- **WHEN** the terminal engine reports the effect
- **THEN** the native policy denies it without applying that host effect
- **AND** explicit user copy/paste remains available.

## Native command contract

Reuse Odra's terminal-prefixed native command identities where the operation
exists. Adapt its working-context selection to rev's file/directory model.
All commands below require concrete native behavior, not functional-looking
stubs or a programmable facade.

| Command | Contract |
| --- | --- |
| terminal-open | Select the live default or create it in the lower panel. |
| terminal-new | Create an independent lower-panel session. |
| terminal-new-buffer | Create an independent terminal directly in the selected main-area view. |
| terminal-move-to-buffer | Present the selected live terminal as a main-area buffer without restarting it. |
| terminal-move-to-panel | Dock that same terminal in the lower panel. |
| terminal-open-here | Create a session in the current resource's containing directory. |
| terminal-open-other-pane | Present/select a terminal through ordinary other-pane policy. |
| terminal-toggle | Show/hide the remembered lower presentation, retaining session and size. |
| terminal-list, terminal-rename | Select/inspect retained sessions and rename without changing identity. |
| terminal-copy-mode, terminal-copy-mode-done | Enter retained-text navigation and return to live input. |
| terminal-yank, terminal-yank-pop | Apply the shared retained-text and checked terminal-paste contract. |
| terminal-clear, terminal-clear-scrollback | Explicitly clear the live screen or retained output under their separate policies. |
| terminal-next-prompt, terminal-previous-prompt | Navigate recognized prompt boundaries, using admitted shell reports such as OSC 133 when available. |
| terminal-next-error | Navigate an actually recognized result location; unavailable evidence does not authorize a guessed source jump. |
| terminal-open-link | Explicitly validate and activate the link at the selected terminal location. |
| terminal-beginning-of-line, terminal-end-of-line, terminal-reset-cursor-point | Navigate copied output or align its copy point with the live cursor where representable. |
| terminal-send-next-key, terminal-send-* | Forward the literal-next input or the corresponding native Return/Tab/Escape/deletion/movement/paging/space encoding. |
| terminal-undo | Send the admitted child-side undo input; do not apply source-document undo to live terminal state. |

## Implementation preparation

Behavior follows Odra with the explicit rev placement and Rust-engine choices
above. Additional reference text may refine the draft but is not a prerequisite
for resolving behavior already covered by that reference.

- Pin alacritty_terminal and the admitted feature/transitive set; verify its
  PTY/IO/resize/exit integration and ordinary Rust build without a Zig toolchain.
- Lower the admitted VT styles/modes/damage and host effects into rev's renderer
  and input owners; advertise only actual supported terminal capabilities.
- Define the typed TOML catalog, sensible panel geometry, session/IO/scrollback
  limits and exact Linux integration gates from representative measurements.
- Validate real lazygit/shell operation, relocation/mirrored views, focus and
  literal-next recovery, copy/paste, process closure and idle/resource behavior.

Before planning, reconcile the editor's current embedded-shell exclusion and
buffer/file command admission with a complete accepted terminal capability
delta. Preserve historical completed delivery and gate records. Versioning this
draft does not authorize implementation, slicing, dependency
installation or process launch.
