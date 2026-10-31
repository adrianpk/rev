# File and Directory Navigation

Status: accepted
Date: 2026-10-31
Approved: product owner, 2026-10-31; selected five-topic delivery, Emacs interaction criterion and explicit Looper start request
Plan: [Wrap and Navigation](../plan/rev-wrap-navigation.md)
Tracker: [Wrap and Navigation Tracker](../tracker/rev-wrap-navigation.md)
Format: OpenSpec capability proposal
Source: product-owner directory, initial tree, path completion and pointer requests
Model: [File and Directory Navigation Model](editor-file-navigation-model.md)
Baselines: [rev Editor](rev-editor.md), [Editor Interaction](editor-interaction.md), [Editor Pointer](editor-pointer-interaction.md)
Controls: [Editor Configuration](editor-configuration.md)
Readiness: accepted opening, root, completion and click policies; implementation is owned by rev-wrap-navigation

## Purpose

Make opening a file or directory equally ordinary. A directory is a complete
browsing target; a file is an editing target. CLI opening, find-file and the
tree share resource identity and preserve existing buffers and unsaved work.
The supporting tickets divide this interaction into bounded changes.

## Confirmed decisions

- Accept files and directories from startup and C-x C-f, including relative
  and absolute paths. `rev .` opens the current directory for browsing.
- An explicit directory opens its visible tree with immediate entries shown.
  A file opens its document without forcing the tree to appear.
- An explicitly chosen directory takes precedence over automatic file-parent
  rooting. Without an explicit choice, associate the tree with the file's
  parent, reconciling existing configurable root behavior before delivery.
- First tree display expands the root once; descendants remain lazy.
- Tab completes the highlighted path, initially the first suggestion.
- Clicking an entry selects it and, by default, activates it through the
  existing native tree action. mouse.tree_activate_on_click = false retains
  selection-only behavior. This adapts Dired activation to the existing tree.

## ADDED Requirements

### Requirement: NAV01 Distinguish opening targets

Startup and find-file SHALL admit both file and directory targets. Directory
acceptance SHALL finish a directory-open operation without requiring an inner
file. File acceptance SHALL retain ordinary file/new-file behavior. Completion
and final acceptance SHALL remain distinct operations.

#### Scenario: Start in a directory

- **GIVEN** an accessible current directory
- **WHEN** the user runs `rev .`
- **THEN** its tree is visible with immediate entries exposed
- **AND** the directory is not loaded as editable file text.

#### Scenario: Accept a directory while editing

- **GIVEN** an unsaved document and an accessible directory in find-file
- **WHEN** the user explicitly accepts that directory as the opening target
- **THEN** rev presents its browsing context
- **AND** the document, unsaved changes and history remain available.

### Requirement: NAV02 Preserve explicit root intent

An accepted explicit directory SHALL own the session's chosen tree root until
another explicit directory choice replaces it. Visiting a file beneath that
root SHALL NOT narrow it to the file's parent. Without an explicit root choice,
file-parent rooting SHALL apply according to the reconciled configuration
precedence. Directory identity SHALL reuse existing checked path/alias rules.
This context SHALL NOT grant AI transmission or filesystem authority.

#### Scenario: Visit a nested file

- **GIVEN** a directory explicitly selected as the tree root
- **WHEN** the user opens a file inside its nested source directory
- **THEN** the chosen tree root remains the explicit directory
- **AND** normal document identity and buffer reuse remain authoritative.

### Requirement: NAV03 Initialize first-level expansion once

The first display of a newly initialized root SHALL reveal its immediate
entries. Child directories SHALL stay collapsed until explicitly expanded.
Hide/show SHALL preserve deliberate later selection and expansion state.
Directory observation SHALL remain bounded and outside rendering.

#### Scenario: Restore a deliberately collapsed root

- **GIVEN** an initialized tree whose root the user has collapsed
- **WHEN** the tree is hidden and shown again
- **THEN** its deliberate collapsed state is retained
- **AND** initial expansion is not reapplied on every toggle.

### Requirement: NAV04 Complete the selected path without accepting it

A nonempty candidate list SHALL visibly select its first entry initially.
Tab SHALL fill the selected entry, including after list navigation and when
other candidates share a prefix. Completing a directory SHALL retain the
prompt and offer its immediate children; completing a file SHALL fill the
draft without opening it. Disappeared selections SHALL NOT resolve through
an unrelated replacement at the same list index.

#### Scenario: Continue through a selected directory

- **GIVEN** alpha/ and alpine/ matching a find-file draft
- **WHEN** the user highlights alpine/ and presses Tab
- **THEN** the draft names alpine/ and its children become candidates
- **AND** the prompt remains active without opening a file or accepting a root.

### Requirement: NAV05 Select the clicked tree entry

Pointer input on a rendered tree entry SHALL select that entry while focusing
the tree. Row identity SHALL come from the current rendered viewport and
listing observation, including scrolling and clipping. Invalid/stale hits
SHALL NOT select or open an unrelated entry. With mouse.tree_activate_on_click = true (the factory default), that same
click SHALL activate the selected entry through the native tree owner. Files
open in the remembered editing pane; directories expand/collapse without
changing the chosen root. False SHALL select/focus only. Repeated clicks are
individual actions; no double-click detector or timing threshold is needed.

#### Scenario: Click a different visible entry

- **GIVEN** a tree with one selected row and another visible entry
- **WHEN** the user clicks the other entry
- **THEN** focus and tree selection identify the clicked entry
- **AND** any subsequent action addresses that identity rather than the old row.

### Requirement: NAV06 Preserve state on unsuccessful navigation

Cancellation, inaccessible directories, changed paths and failed target
admission SHALL preserve the prior usable root, buffers, dirty text and
history. Feedback SHALL distinguish failed observation from a verified empty
directory. Path handling SHALL preserve spaces, Unicode and admitted literal
spelling without recursive discovery or incidental filesystem writes.

#### Scenario: Fail to open an unreadable directory

- **GIVEN** a usable browsing context and an unsaved document
- **WHEN** a requested directory cannot be observed
- **THEN** rev reports that failure and preserves the prior context
- **AND** the failed listing is not displayed as a successfully empty root.

## Ticket ownership

| Change | Ticket |
| --- | --- |
| File/directory target and root semantics | [Directory opening](../ticket/solved/20261028094803-open-directory-targets-from-cli-and-find-file.md) |
| First-level initialization | [Initial tree expansion](../ticket/solved/20261028153707-expand-file-tree-root-on-first-display.md) |
| Selected Tab completion | [Highlighted path completion](../ticket/in_progress/20261028170748-complete-highlighted-find-file-path-with-tab.md) |
| Pointer row selection | [Clicked tree entry](../ticket/ready/20261030231016-select-the-clicked-file-tree-entry.md) |

## Accepted interaction profile

- Return accepts the explicitly navigated candidate, or the typed draft when
  selection has not been changed explicitly. A directory is a complete opening
  target; acceptance closes the prompt and shows that root, rather than merely
  descending inside find-file. An empty accepted draft denotes the prompt's
  base directory. A genuinely new file draft retains unsaved-file behavior.
- Tab fills the highlighted/default-first candidate; a completed directory
  keeps its separator and exposes its children while the prompt remains open.
  Tab never commits a root or opens a file. Retain the selected-path identity
  check and fail without substituting an unrelated refreshed row.
- Root precedence: an explicit directory target wins. Without one, an explicit
  configured tree.root = "cwd" is honored; "auto" and "file-parent" use the
  visiting file's parent when present and launch cwd otherwise. Visiting files
  inside or outside a chosen explicit root preserves it; a subsequent explicit
  directory target replaces it. Path spelling alone does not establish intent.
- Explicit directory opening requests a visible first-level-expanded tree even
  when startup tree.visible is false. File opening does not force visibility
  either way. A narrow terminal may temporarily hide the tree without losing
  the requested root, initialization or focus intent.
- First presentation observes immediate entries once, reusing bounded native
  listing. Subsequent hide/show respects deliberate collapse and selection.
  Continue existing hidden-file, ordering, symlink and refresh rules. Keep the
  existing finite listing bounds and incomplete-list diagnostics; no recursive
  discovery is added.
- Primary click selects the checked rendered entry, then activates it by
  default under NAV05. Dired is the activation reference; rev retains native
  expand/collapse for directories rather than introducing a directory buffer.
  A click on blank tree space focuses without an entry action. Root-row clicks
  use existing root activation. Failed actions retain the valid tree selection
  and dirty buffers. The setting is startup-only under the normal TOML loader;
  global mouse disablement continues to suppress all mouse interaction.

## Baseline reconciliation and validation

Prepare a complete amendment to Editor Interaction UI03's shared-prefix Tab
policy and the delivered tree initialization/pointer focus contracts before
implementation. Historical completed specifications and solved tickets remain
evidence of their original delivery, not proof of these requested changes.

Validate CLI and actual find-file input routes, empty and unreadable roots,
Unicode/spaces, explicit and automatic roots, Tab after list selection,
hide/show, pointer hits after scroll/resize, cancellation and dirty buffers.
AI may depend on directory ownership; navigation does not depend on AI.
Implementation, slicing and closure belong to the single rev-wrap-navigation
delivery. This accepted proposal is not evidence that its behavior has shipped.

## MODIFIED Requirements

Target: [Editor Interaction UI03](editor-interaction.md). Apply this complete
replacement during delivery, including the corresponding consolidated baseline
clause. Historical completed gate evidence must remain identifiable.

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

## Behavioral references

- [Emacs visiting files and directories](https://www.gnu.org/software/emacs/manual/html_node/emacs/Visiting.html).
- [Emacs Dired activation](https://www.gnu.org/software/emacs/manual/html_node/emacs/Dired-Visiting.html).
