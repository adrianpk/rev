# Better File Tree

Status: Approved
Date: 2026-11-06
Approved: product owner, 2026-11-06; specification/model, preparation commits, plan, slicing, push and one autonomous Looper delivery through dev
Format: OpenSpec change specification
Source: owner request and supplied file-tree screenshot
Baselines: [File and Directory Navigation](editor-file-navigation.md), [Configuration](editor-configuration.md), [Better Gutter](editor-better-gutter.md)
Configuration model: [Better File Tree Model](editor-better-file-tree-model.md)
Plan: [Better File Tree Delivery](../plan/rev-better-file-tree.md)
Tracker: [Better File Tree Tracker](../tracker/rev-better-file-tree.md)
Current interface: [File tree reference](../../../docs/reference/file-tree.md)
Related follow-up: [Tree selection and active-buffer synchronization](../ticket/open/20261031093408-highlight-tree-selection-and-sync-active-buffer.md)

## Purpose

Make the file tree easier to scan with a distinct root, folder states, file-kind
symbols and semantic colors. Use this decorated presentation by default while
retaining the current minimal presentation through one TOML preference.

Both presentations use the same tree, entry identities and interactions. The
supplied screenshot informs presentation; its multiple roots do not introduce
a workspace or multi-root model into rev.

## Current implementation

The tree already has lazy directory observations, one-time initial expansion,
keyboard navigation, pointer activation, root policies and explicit refresh.
`src/tree/entry.rs` builds plain labels with `>` or `v` for directories, `@`
for symlinks, two spaces per depth, a directory slash and an incomplete-list
suffix. `src/display/tree.rs` paints a focused selected row with the existing
tree-selection palette. Other entries have no file-kind decoration.

This change adds a presentation choice over those observations. Persistent
selection feedback and following the active editing buffer remain in the linked
ticket; this specification does not decide their multi-pane behavior.

## Configuration

Add `tree.decorated`, a startup boolean with compiled default `true`. Keep
`tree.visible` and all other tree defaults unchanged: the richer presentation
does not itself open the tree. A missing new leaf selects the decorated mode,
including in an existing configuration. Explicit `false` selects the minimal
presentation.

Reuse `gutter.nerd_fonts` as the existing declaration that the terminal font
supports Nerd Fonts. Do not add a second font preference or rename this key.
Its compiled default remains `false`; the decorated factory presentation must
therefore be useful with ordinary terminal fonts.

| `tree.decorated` | `gutter.nerd_fonts` | Presentation |
| --- | --- | --- |
| `false` | Either value | Current minimal labels and selection styling. |
| `true` | `false` | Decorated labels with ordinary symbols and semantic colors. |
| `true` | `true` | The same decorated layout with compiled Nerd Fonts icons. |

For example, a user who prefers the existing presentation sets:

```toml
[tree]
decorated = false
```

The flag describes configured support, not detected font availability. rev does
not inspect installed fonts, install fonts or infer support from the connection
used to run the editor. An incorrectly enabled Nerd Fonts flag can produce
missing-glyph boxes; setting it to `false` selects the ordinary symbols.

## Visual vocabulary

These fixed symbols and colors are approved design choices. They are compiled
defaults, not a user-maintained icon map. Use flat symbols rather than
emoji. The color applies to the kind icon and entry name; selection overrides
it when needed for contrast.

| Entry kind | Ordinary symbol | Nerd Fonts icon | Palette role | Default |
| --- | --- | --- | --- | --- |
| Root | `◆` | `` U+F1B2 | `palette.tree_entries.root` | `#7aa2f7` |
| Directory | `/` | `` U+F07B closed; `` U+F07C open | `palette.tree_entries.directory` | `#e0af68` |
| Source file | `λ` | `` U+F121 | `palette.tree_entries.source` | `#7aa2f7` |
| Document | `≡` | `󰈙` U+F0219 | `palette.tree_entries.document` | `#e0af68` |
| Configuration | `=` | `` U+E615 | `palette.tree_entries.configuration` | `#bb9af7` |
| Other file | `·` | `` U+F15B | `palette.tree_entries.file` | `#c0caf5` |
| Symlink | `@` | `` U+F481 | `palette.tree_entries.symlink` | `#7dcfff` |

Keep `palette.tree` as the existing background scalar. The new nested
`palette.tree_entries` table contains only entry foreground roles and does not
change the meaning of an existing palette key. Keep `palette.tree_selected`
and `palette.tree_selected_text` for the selected row.

The decorated prefix has a disclosure field and a kind-icon field, separated
from the name. The disclosure field uses `▸` for closed directories and `▾`
for open directories, including the root; it is blank for leaves. It remains
present with either font profile, so an ordinary directory slash is not the
only indication of directory state. Names at the same depth align; a change of
icon must not shift the name. Retain the existing two-cell nesting step.

Keep directory slashes, the visible symlink distinction and incomplete-list
diagnostics. Root emphasis may use bold with its palette role. No connector
lines or additional permanent pane width are required by this design.

## ADDED Requirements

### Requirement: BFT01 Select presentation without changing tree behavior

rev SHALL use decorated presentation by default and SHALL accept
`tree.decorated = false` to retain the current minimal presentation. The
preference SHALL use the existing startup configuration precedence, validation
and fallback rules. It SHALL affect presentation only, without changing
visibility intent, root selection, sorting, hidden entries, expansion, refresh,
entry activation, bindings or modal/non-modal input ownership.

#### Scenario: Existing configuration adopts the new default

- **GIVEN** valid configuration that omits `tree.decorated`
- **WHEN** the user starts rev and shows the file tree
- **THEN** the tree uses decorated presentation
- **AND** the initial visibility and first-level expansion rules are unchanged.

#### Scenario: Minimal presentation is selected

- **GIVEN** `tree.decorated = false` and either Nerd Fonts preference
- **WHEN** the user navigates, expands and activates entries
- **THEN** labels retain the existing minimal appearance
- **AND** the same tree commands and pointer actions remain available.

### Requirement: BFT02 Decorate entries with or without Nerd Fonts

In decorated mode, rev SHALL distinguish the root, directories, source files,
documents, configuration files, other files and symlinks using the fixed visual
vocabulary above. `gutter.nerd_fonts` SHALL select the symbol catalog only;
ordinary symbols SHALL retain kind distinctions, disclosure state and semantic
colors. There SHALL be no font detection or executable icon provider.

#### Scenario: Decorated factory presentation

- **GIVEN** the compiled defaults and an ordinary terminal font
- **WHEN** the tree displays a root, directory, source file and document
- **THEN** their ordinary kind symbols and palette roles distinguish them
- **AND** directories show their current open or closed state.

#### Scenario: Configured Nerd Fonts support

- **GIVEN** `tree.decorated = true` and `gutter.nerd_fonts = true`
- **WHEN** the tree is displayed
- **THEN** it uses the compiled Nerd Fonts kind icons
- **AND** ordering, nesting, disclosure state and entry identity are unchanged.

### Requirement: BFT03 Derive file kinds from already observed names

rev SHALL classify entries from their observed type and basename without opening
file contents. Root, symlink and directory identity SHALL take precedence over
filename classification. A symlink SHALL remain a leaf without following its
target to select an icon.

The initial filename catalog SHALL recognize at least the source suffixes
`.rs`, `.go`, `.c`, `.h`, `.js`, `.jsx`, `.ts`, `.tsx`, `.html`, `.htm` and
`.css`; document suffixes `.md`, `.markdown`, `.org`, `.txt` and `.rst`; and
configuration suffixes `.toml`, `.json`, `.yaml`, `.yml`, `.ini`, `.cfg`,
`.conf` and `.xml`. Configuration basenames SHALL include `.editorconfig`,
`.gitignore`, `.gitattributes`, `Cargo.lock`, `Makefile` and `Dockerfile`.
Classification MAY recognize additional names in the same compiled catalog.
ASCII case-insensitive icon matching SHALL NOT change path identity or sorting.
Unrecognized names SHALL receive the other-file symbol and remain actionable.

This is a presentation catalog. It neither requires nor advertises a parser,
language server or language feature for a file receiving a source icon.

#### Scenario: Names are sufficient

- **GIVEN** cached entries named `main.rs`, `notes.org`, `Cargo.toml` and `data.bin`
- **WHEN** the tree is painted
- **THEN** they receive source, document, configuration and other-file decoration
- **AND** no content read, parser, language server or Git process is started.

#### Scenario: Symlink name resembles a directory or source file

- **GIVEN** an observed symlink named `main.rs`
- **WHEN** its row is painted or navigated
- **THEN** it has symlink presentation and remains a leaf
- **AND** rendering does not inspect or traverse its target.

### Requirement: BFT04 Keep geometry and interaction consistent

Decorated labels SHALL use terminal-cell geometry for icon placement, clipping
and selection. Both icon catalogs SHALL use the same bounded prefix width.
Rendering SHALL sanitize filenames through the existing display path and SHALL
clip safely for narrow panes, wide characters and combining characters.

Pointer hit testing SHALL continue to resolve the cached row's canonical entry,
including presses over disclosure, icon, name and remaining row space. Existing
activate-on-click or selection-only policy SHALL apply to the complete row;
decorative fields SHALL NOT introduce an additional activation gesture.

The selected row SHALL remain readable using the existing selection palette
and focus rules. With `NO_COLOR`, structural symbols, labels and reverse-video
selection SHALL remain useful without semantic color. Rendering and ordinary
movement SHALL retain the existing absence of filesystem reads, Git discovery
and recursive scans, and all directory-observation bounds SHALL remain intact.

#### Scenario: Icon and name identify the same entry

- **GIVEN** a clipped decorated row in a narrow tree pane
- **WHEN** the user clicks its icon or its visible name
- **THEN** both resolve to the same observed entry
- **AND** existing click policy determines selection or activation.

#### Scenario: Decoration remains legible without color

- **GIVEN** decorated mode and `NO_COLOR` enabled
- **WHEN** the user navigates between an open directory and a file
- **THEN** the disclosure and kind symbols remain visible
- **AND** focused selection remains identifiable without colored foregrounds.

## Documentation and acceptance

Implementation shall update the complete sample configuration and defaults
reference for the new boolean, palette roles and the extended use of
`gutter.nerd_fonts`. Existing configurations shall not be overwritten. Update
the file-tree reference and its User Guide chapter with a short comparison of
the two presentations; project the Markdown guidance into Spore without drift.

Acceptance shall cover all four boolean combinations, missing and invalid
configuration leaves, precedence, palette overrides, both input profiles,
color and `NO_COLOR`, directory states, file kinds, symlinks, unknown names,
long/Unicode names, clipping, resize and clicks over decorated fields. Compare
minimal mode with the existing labels and verify that painting and movement
consume cached observations only.

## Approved delivery boundary

The owner approved this specification and its model on 2026-11-06 and requested
Conventional Commits, a plan, slicing and one autonomous Looper delivery through
dev. The approved scope includes both presentations, the fixed icon catalogs,
palette roles, configuration and joint acceptance with Markdown/Spore guidance.
The linked selection-synchronization ticket retains its separate scope.
Main promotion, release and public publication remain separately requested.
