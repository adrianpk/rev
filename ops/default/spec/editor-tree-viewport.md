# File Tree Width and Horizontal Navigation

Status: Completed
Date: 2026-11-12
Format: OpenSpec; workspace specification contract
Inspection baseline: `6746978285aa7cd5739b704cc0ab214e47e9050c`
Source: [TKT-20261108175339](../ticket/solved/20261108175339-improve-file-tree-panel-width-and-horizontal-navigation.md)
Model: [Tree Viewport Policy](editor-tree-viewport-model.md)
Plan: [Tree Viewport Delivery](../plan/rev-tree-viewport.md)
Tracker: [Tree Viewport Tracker](../tracker/rev-tree-viewport.md)
Baselines: [Better File Tree](editor-better-file-tree.md), [Tree Selection](editor-tree-selection.md), [File Navigation](editor-file-navigation.md)

## Purpose

Let users identify long and deeply nested tree entries through bounded stable
automatic fitting, explicit width controls and a shared horizontal viewport.
The source ticket contains the approved product decisions. This specification
adds their verifiable contract without reopening that alignment.

## Requirements

### Requirement: TVP01 Configured width policy

The editor SHALL admit `tree.auto_width` as a boolean, default `true`, through
the existing startup configuration, merge and provenance pipeline. In automatic
mode, `tree.width` SHALL be the preferred minimum, with its existing default
24 and validation. In fixed mode it SHALL be the preferred width. Both SHALL
respect `tree.max_fraction`, its existing one-third default and native small
frame geometry. Runtime changes SHALL remain session-only.

#### Scenario: Default, explicit and rejected configuration

- **GIVEN** omitted, explicit true, explicit false or an invalid auto-width value
- **WHEN** startup admits the configuration
- **THEN** omission and true select automatic sizing, false selects fixed sizing,
  and invalid values reject the candidate through the existing diagnostic path
- **AND** included/root overrides retain existing precedence and provenance.

#### Scenario: Fixed policy restoration

- **GIVEN** `tree.auto_width = false` and a configured preferred width
- **WHEN** the user changes width or mode and later resets width
- **THEN** the configured fixed width policy resumes
- **AND** the configuration file bytes remain unchanged throughout.

### Requirement: TVP02 Stable bounded automatic fitting

Automatic desired width SHALL use the furthest rendered right edge of every
row in the admitted expanded model, including rows outside the vertical
viewport. It SHALL account for indentation, disclosure markers, icons, spaces
and sanitized labels in terminal display columns. Folded descendants SHALL NOT
contribute until expansion admits them. Width observation SHALL perform no
filesystem discovery during drawing, selection movement or terminal resizing.

Within the same tree context, automatic sizing SHALL retain a desired-width
high-water mark from the preferred minimum. Expansion and explicit refresh can
increase it. Folding, selection, vertical scrolling, focus changes and
hiding/showing SHALL NOT reduce it. Actual geometry SHALL be capped by the
existing fraction and small-frame constraints without losing desired width.

#### Scenario: Expanded offscreen and folded rows

- **GIVEN** short visible rows, an expanded long offscreen row and a still folded
  directory containing a longer name
- **WHEN** the tree computes width
- **THEN** the expanded offscreen row contributes and the hidden descendant does not
- **AND** selection and vertical scrolling leave width unchanged
- **WHEN** that directory expands and later folds
- **THEN** width can grow to its cap and remains stable after folding.

#### Scenario: Small frame and restored room

- **GIVEN** an automatic desired width established in a larger terminal
- **WHEN** the terminal becomes narrow and later returns to its original size
- **THEN** native geometry safely clamps or hides the tree and restores the
  retained desired width when room returns
- **AND** the source retains its native usable editing area.

### Requirement: TVP03 Explicit session width ownership

`tree-widen` and `tree-narrow` SHALL change desired width by four terminal
display columns, within admitted native bounds. A manual action SHALL own width
until a later explicit width or mode action. Automatic fitting SHALL NOT undo
a manual width, including a width below the configured automatic minimum.
Navigation, expansion, refresh and root/context changes SHALL retain it.

`tree-reset-width` SHALL clear session width/mode overrides, the previous
automatic high-water mark and horizontal offset, then apply configured policy
to the currently expanded model. `tree-auto-width-mode` SHALL switch runtime
automatic ownership; enabling it fits the current model from the preferred
minimum, and disabling it retains the current desired width as a session fixed
choice. Neither action SHALL rewrite TOML or change selection/root identity.

#### Scenario: Manual width survives tree transitions

- **GIVEN** automatic sizing and a tree with room for a four-column step
- **WHEN** the user narrows, moves, expands, refreshes, changes focus and changes roots
- **THEN** the chosen desired width remains authoritative
- **AND** a temporary frame constraint preserves it for later restoration.

#### Scenario: Reset and automatic mode

- **GIVEN** a manual width and a retained automatic high-water mark
- **WHEN** the user resets
- **THEN** the configured mode resumes with fresh current-model measurement
- **AND** automatic reset can exceed 24 when labels or configuration require it
- **WHEN** the user explicitly enables or disables automatic sizing
- **THEN** the mode transition takes ownership according to this requirement.

### Requirement: TVP04 Shared horizontal viewport

`tree-scroll-left` and `tree-scroll-right` SHALL move one shared horizontal
offset by one terminal display column per invocation, bounded to current
expanded content and actual panel width. Every row's indentation, markers,
icons and label SHALL use the same offset. Panning SHALL preserve panel width,
selected entry, expansion, root, buffers and source history.

The offset SHALL survive row changes, vertical navigation, focus changes and
hiding/showing. Root/context changes and width reset SHALL return it to zero.
Shorter content or wider geometry SHALL clamp only excess offset. Clipping
SHALL preserve whole displayed glyphs, including wide and combining sequences,
while the logical pan step remains one display column.

#### Scenario: Reveal an ending at the width cap

- **GIVEN** several aligned rows and a selected nested name beyond the width cap
- **WHEN** the user pans right repeatedly and then left
- **THEN** the name's ending and beginning are accessible within finite bounds
- **AND** all rows share the same displacement with unchanged identity and width.

#### Scenario: Retention and clamping

- **GIVEN** a nonzero horizontal offset
- **WHEN** selection, vertical viewport, focus or tree visibility changes
- **THEN** the offset remains unchanged unless current content/geometry requires clamping
- **WHEN** the root/context changes or width resets
- **THEN** the offset returns to zero.

### Requirement: TVP05 Native command and key routes

All six commands SHALL have descriptions and effective keybinding hints in M-x
and the modal command selector. Both interaction profiles SHALL use these
defaults, through existing configurable keymaps and input ownership:

| Binding | Context | Action |
| --- | --- | --- |
| `>` / `<` | Focused tree | Widen / narrow by four columns |
| `C-x t >` / `C-x t <` | Editor and tree | The same widen / narrow actions |
| `C-x t =` | Editor and tree | Reset width policy |
| Left / Right | Focused tree | Pan left / right by one column |
| Enter | Focused tree | Open file or toggle directory |
| `C-f` / `C-b` | Focused tree | Expand / collapse or select parent |

Unmodified angle brackets in an editing buffer SHALL keep normal input behavior.
Arrow panning SHALL NOT expand, collapse, open an entry or change roots.
Existing command-prefix cancellation and configured overrides SHALL retain
their ownership and effective-hint behavior.

#### Scenario: Routes and source input

- **GIVEN** either native interaction profile and tree or source focus
- **WHEN** local, prefixed or named width actions are used in their admitted context
- **THEN** they execute the same corresponding command
- **AND** ordinary source angle-bracket input, pending-prefix cancellation and
  effective override hints remain correct.

#### Scenario: Directory actions remain available

- **GIVEN** a selected directory or file and focused tree
- **WHEN** Left/Right, Enter or control navigation is used
- **THEN** arrows pan only, Enter opens/toggles and `C-f`/`C-b` keep directory operations.

### Requirement: TVP06 Presentation and interaction consistency

Minimal/decorated, ordinary Unicode/Nerd Font and monochrome presentations SHALL
share measurement, clipping and selection geometry. Pointer targeting SHALL
continue to resolve admitted row/path identities after resizing and panning.
Active-buffer following, expansion state, root membership and pending pointer
identity checks SHALL retain their existing contracts.

#### Scenario: Unicode, pointer and source preservation

- **GIVEN** shallow long, deeply nested and nested long names containing spaces,
  combining sequences or wide characters, across the existing presentation variants
- **WHEN** the user fits, resizes, pans, follows an active buffer and clicks a row
- **THEN** drawn cells, highlight and pointer identity agree without splitting glyphs
- **AND** width/pan actions alone leave source bytes, point, mark, dirty state,
  history and open-buffer/root identities unchanged.

## Acceptance and boundaries

Extend existing native tree, input, configuration, display, pointer and context
fixtures. Extend the existing Better File Tree native/owned-PTY journey rather
than creating a parallel terminal framework. Verify actual arrow and prefixed
routes in both profiles, including source insertion and terminal restoration.
Native cells establish the Unicode/presentation matrix; PTY evidence establishes
the real executable/input journey, with the existing physical-font limitation.

This change owns the tree viewport. It does not introduce generic split resizing,
filesystem discovery from rendering or a persisted UI-session format. No product
decisions remain open. Delivery status requires fresh controller-owned acceptance
on the exact integrated candidate and reconciled source-ticket closure.

## Completion evidence

TVP01-TVP06 are delivered through merged canonical PRs #30 and #31. The
controller passed the unchanged integrated gate at `1c98ee49bc6af413b99148f270d1395dc80f9512`,
receipt `DYYC4ATKJZTUCR4UA2GE5UDS43`. The
[delivery-set closure](../report/slices/rev-tree-viewport/slice-2-horizontal.md#delivery-set-closure)
records the exact command, 639 passing native tests, executable/documentation
acceptance and evidence limits. The source ticket is solved with resolution fixed.
