# File and Directory Navigation

Status: draft
Date: 2026-10-31
Format: OpenSpec capability proposal
Source: product-owner directory, initial tree, path completion and pointer requests
Model: [File and Directory Navigation Model](editor-file-navigation-model.md)
Baselines: [rev Editor](rev-editor.md), [Editor Interaction](editor-interaction.md), [Editor Pointer](editor-pointer-interaction.md)
Controls: [Editor Configuration](editor-configuration.md)
Readiness: requested outcomes captured; acceptance gestures, root-policy precedence and pointer activation remain open

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
- Clicking an entry should select the row clicked. Whether the same click
  also opens it, and whether a preference is useful, remain undecided.

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
SHALL NOT select or open an unrelated entry. File opening or directory
expansion on that same click requires the separately admitted activation policy.

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
| File/directory target and root semantics | [Directory opening](../ticket/open/20261028094803-open-directory-targets-from-cli-and-find-file.md) |
| First-level initialization | [Initial tree expansion](../ticket/open/20261028153707-expand-file-tree-root-on-first-display.md) |
| Selected Tab completion | [Highlighted path completion](../ticket/open/20261028170748-complete-highlighted-find-file-path-with-tab.md) |
| Pointer row selection | [Clicked tree entry](../ticket/open/20261030231016-select-the-clicked-file-tree-entry.md) |

## Decisions before planning

- The exact gesture for accepting a directory versus descending into it;
  Tab remains completion and must not silently become final acceptance.
- Full precedence with existing configured tree-root policy, launch context
  and file visits outside an explicitly selected root.
- Whether one click merely selects or also activates an entry; directory
  activation, repeated clicks, blank/root rows and any configurable policy.
- Listing limits, candidate refresh identity, symlink/unavailable-path cases
  and geometry-aware first display in a narrow terminal.

## Baseline reconciliation and validation

Prepare a complete amendment to Editor Interaction UI03's shared-prefix Tab
policy and the delivered tree initialization/pointer focus contracts before
implementation. Historical completed specifications and solved tickets remain
evidence of their original delivery, not proof of these requested changes.

Validate CLI and actual find-file input routes, empty and unreadable roots,
Unicode/spaces, explicit and automatic roots, Tab after list selection,
hide/show, pointer hits after scroll/resize, cancellation and dirty buffers.
AI may depend on directory ownership; navigation does not depend on AI.
No implementation, slicing or execution tracker is created by this draft.
