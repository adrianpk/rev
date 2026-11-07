# File Tree Parent Navigation

Status: Completed
Date: 2026-11-07
Completed: 2026-11-08
Inspection baseline: `6fde63810e95b8e3a0e61b72714696354993d9c0`
Format: OpenSpec capability addition
Approved: product owner requested specification, slicing, plan and one Looper delivery
Baseline: [File Navigation](editor-file-navigation.md), [Tree Selection](editor-tree-selection.md), [Contexts](editor-projects-workspaces.md)
Plan: [Parent Navigation Delivery](../plan/rev-tree-parent-navigation.md)
Tracker: [Parent Navigation Tracker](../tracker/rev-tree-parent-navigation.md)

## Purpose

Let users move to the parent of an automatic file-tree root while retaining
the directory they came from as the selected, visible entry. An explicitly
opened directory remains the session's fixed reference.

## Existing behavior and ownership

Opening a standalone file gives its containing directory to the automatic
tree-root policy. Later file visits may follow their containing directories.
Opening a directory explicitly anchors its root. Named Project and Workspace
roots are also fixed; configured `tree.root = "cwd"` retains its stable root.

This addition preserves these distinctions. Going up changes the displayed
automatic tree root, not the process working directory or editing context.
It does not turn automatic browsing into an explicit directory anchor.
Existing Left/C-b collapses or selects an already displayed parent; Enter
expands a directory or opens a file. Those operations retain their behavior.

Reuse current native Tree, Session, path admission, listing and selection
owners. There is no new configuration or persisted data model. Future directory
sandboxing remains a separate specification; a navigation anchor alone is not
a filesystem security boundary.

## Command and interaction

| Command | Description | Default key |
| --- | --- | --- |
| `tree-up-directory` | Show the parent directory of an unanchored file tree. | `^` in the tree |

The command belongs to the shared compiled command catalog, descriptions,
effective binding annotations and ordinary keymap remapping. Both editing
profiles use the tree binding. Modal source editing retains its own caret
motion. Named invocation uses the existing command interface and originating
tree focus. A source-focused invocation follows existing tree-command focus
admission without changing source or moving the tree unexpectedly.

## ADDED Requirements

### Requirement: TUP01 Preserve fixed roots and automatic file browsing

The editor SHALL permit upward root navigation only for an automatic,
unanchored tree. Explicit directory anchors, Project roots, Workspace member
roots and the configured cwd policy SHALL remain fixed. A blocked request SHALL
leave state intact and SHALL NOT enumerate outside the fixed root.

#### Scenario: Standalone file

- **GIVEN** a standalone file whose tree root is its containing directory
- **WHEN** the user invokes tree-up-directory in the tree
- **THEN** the tree displays the containing directory's parent
- **AND** later file visits retain the existing automatic root policy.

#### Scenario: Explicit directory, Project, Workspace or cwd policy

- **GIVEN** any of those fixed-root contexts
- **WHEN** the user invokes tree-up-directory
- **THEN** the editor reports the fixed-root boundary and preserves the root,
  root membership, listing, selection and editing context.

#### Scenario: Filesystem root

- **GIVEN** an eligible automatic tree rooted at the filesystem root
- **WHEN** the user invokes tree-up-directory
- **THEN** the editor reports that no parent is available without changing state.

### Requirement: TUP02 Publish a checked parent listing and return selection

The editor SHALL observe only the parent directory's immediate children using
existing filters, ordering, symlink rules and bounded listing limits. It SHALL
publish the new root, entries and selection together only after admitting that
observation. The directory just left SHALL be selected and scrolled into view,
with focus retained in the tree.

#### Scenario: Return to the previous directory

- **GIVEN** an eligible tree rooted at src
- **WHEN** the user moves up
- **THEN** its parent is displayed with src selected and visible
- **AND** existing directory expansion permits returning without searching.

#### Scenario: Consecutive upward navigation

- **GIVEN** several unanchored ancestor directories
- **WHEN** the user moves up twice
- **THEN** each transition selects the immediate directory just left.

#### Scenario: Unreadable, missing or unrepresentable parent result

- **GIVEN** a failed parent observation or a result that cannot represent the
  previous directory because of filtering, disappearance or existing limits
- **WHEN** the user requests upward navigation
- **THEN** the previous root, listing, selection, scroll and focus remain intact
- **AND** the editor reports the reason using its ordinary diagnostics.

A partial bounded listing that contains the previous directory may be admitted
with the existing incomplete-listing diagnostic. Do not increase traversal
limits or substitute an unrelated selected entry.

### Requirement: TUP03 Share the named action and tree binding

The editor SHALL expose tree-up-directory with the exact description above,
through ordinary named invocation and the default tree-local caret binding in
both profiles. Configured remapping and effective keybinding hints SHALL use
the same canonical action. Preserve source-editing bindings.

#### Scenario: Keyboard, named command and remapping

- **GIVEN** tree focus in either editing profile
- **WHEN** the action runs through the default key, named command or an admitted
  configured replacement binding
- **THEN** the same root policy, selected return entry and diagnostics apply.

#### Scenario: Source-focused invocation

- **GIVEN** source focus or an active source selection
- **WHEN** an unrelated source caret motion or a refused tree command executes
- **THEN** ordinary editing behavior and the source selection remain intact.

### Requirement: TUP04 Preserve editing state and bounded presentation

Upward navigation SHALL NOT replace the file/directory/Project/Workspace
context, rebind or close buffers, reload source, change points, marks, dirty
state or undo history, or create project/state files. Keep independent editing
views, existing listing bounds and all current tree presentations.

#### Scenario: Dirty file and independent views

- **GIVEN** an unsaved file with split views and an eligible automatic tree
- **WHEN** upward navigation succeeds or fails
- **THEN** buffer identity, source text, dirty state, undo history, points and
  marks are unchanged in both views.

#### Scenario: Presentation variants

- **GIVEN** minimal or decorated presentation with either Nerd Fonts setting
- **WHEN** the tree moves up
- **THEN** the same root and selected return entry are presented through
  existing layout and selection behavior.

Decorated presentation SHALL retain its existing Nerd Fonts icons, semantic
colors, connectors, spacing and selected-entry treatment after each transition.
Reuse the normal root and directory rows rather than introducing a plain-text
parent row, duplicate navigation entries or a separate visual style. Respect
existing display-cell widths and clipping so narrow trees stay aligned.

## Completed delivery evidence

TUP01-TUP04 are delivered through canonical
[PR #312](https://forge.adrianpk.com/adrianpk/rev/pulls/312), integrated into
`dev` at `a662621b69d4bec9205296040d912772887a32e3`. The controller passed the
unchanged selected gate on `d75db7c33994b51eeb6a3a01c0b05647105ab7c8`, log
`FU4RKA7ANFGNAMEYBVMUXMDHTI`. The
[Slice 1 closure](../report/slices/rev-tree-parent-navigation/slice-1-parent.md#delivery-set-closure)
records the exact command, 107 selected tests, catalog checks and local
Markdown/Spore results. Plan and tracker are Completed; the report is delivered.
Closure changes no requirement, scenario, command definition or tested input.
