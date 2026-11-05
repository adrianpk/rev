# Better Gutter

Status: Approved
Date: 2026-11-06
Format: OpenSpec change specification
Source: owner discussion and supplied gutter screenshot; one specification for the gutter design
Approved: product owner, 2026-11-06; specification, fixed-symbol Nerd Fonts flag, preparation commits, slicing, push and one autonomous Looper delivery through dev authorized
Visual decisions: owner selected a flat Nerd Fonts bookmark, a solid square fallback for ordinary fonts and the three breakpoint states below
Baselines: [Editor Gutter](editor-gutter.md), especially GUT01-GUT05; [Native Bookmarks](editor-bookmarks.md), BMK03
Existing models: [Editor Gutter Model](editor-gutter-model.md), [Native Bookmarks Model](editor-bookmarks-model.md)
Configuration model: [Better Gutter Model](editor-better-gutter-model.md)
Related behavior: [Soft Wrapping](editor-soft-wrap.md), [Pointer Interaction](editor-pointer-interaction.md), [Org Mode](editor-org-mode.md)

## Purpose

Give bookmarks a recognizable symbol and a dedicated place outside document
text. Define the same gutter's future breakpoint and folding positions without
adding an empty folding column to today's editor.

This is one specification. The current change relocates and restyles bookmark
markers. The breakpoint and folding clauses define how those capabilities will
fit into this gutter when implemented; they do not introduce a debugger or a
new structural-folding engine.

## Current implementation

The current gutter allocates the number field first and then a one-cell
bookmark field from the remaining text-side area. Its default glyph is `●`
and its default palette role is `#e0af68`. The marker is presentation rather
than source text, but its position beside the first text cell makes it look
like a yellow dot inside the code.

The relevant owners are the existing gutter geometry, bookmark projection and
configuration. Bookmark commands, names, anchors, persistence and relocation
remain under the completed bookmark specification. Bookmark storage is
unchanged; the companion model defines the new boolean configuration contract.

## Agreed layout

For the ordinary left gutter, the order from the panel edge toward the text is:

```text
Current delivery: [markers][line numbers][separator][document text]
Future, enabled:  [markers][line numbers][folding][separator][document text]
Future, disabled: [markers][line numbers][separator][document text]
```

The brackets identify areas, not literal characters or extra padding. Existing
number padding remains part of the number field. The folding area has no width
or additional padding when absent. Existing right-side gutter placement mirrors
the arrangement, keeping markers on the outside and folding toward the text.

The marker area carries the bookmark symbol now. Future breakpoints use the
circular state symbols defined below in that same area, rather than inside
the document or in another permanent empty column. Folding indicators have
their own area on the other side of the numbers. How coincident bookmarks and
breakpoints share a row belongs to the future debugger contract; this change
does not invent a collision policy.

## Configuration and compatibility

- Retain `bookmarks.markers`, `palette.gutter.bookmark`, `gutter.side` and the
  existing number preferences. Add one startup boolean, `gutter.nerd_fonts`,
  defaulting to `false`.
- Retain `bookmark-markers-mode` as the active-view visibility command. The
  same native command remains available through M-x, modal colon and configured
  keybindings. No new factory shortcut is required.
- Replace the compiled yellow-dot default with the solid square `■` (U+25A0)
  for ordinary fonts. For a compatible Nerd Fonts terminal, use the flat
  bookmark `` (U+F02E, `nf-fa-bookmark`). Both use the bookmark palette role
  in blue, with `#7aa2f7` as the compiled bookmark palette default.
- Compile both symbols into rev. The user selects only whether to use the
  Nerd Fonts symbol, not an arbitrary character, code point or icon name.
  Both selected symbols occupy one cell under the shared display policy.
- Retire `bookmarks.marker` as a public customization option. For compatibility
  with existing generated configurations, continue accepting its legacy string
  input but ignore it when selecting the glyph. Its old value cannot override
  the boolean or reject otherwise valid old configuration. Do not retain an
  arbitrary-glyph renderer path. Remove it from the current sample and value
  catalog; document the compatibility behavior in migration guidance.
- Preserve explicit palette overrides without rewriting user configuration.
  A configuration containing the old bookmark color continues to override the
  new compiled blue until that palette entry is changed or removed.
- Do not expose a nonfunctional folding setting now. The future TOML setting
  that enables or disables its gutter indicators must also add or remove their
  complete column. Its exact key and other folding behavior are future work.

The selected alternatives are:

| Startup setting | Marker | Meaning |
| --- | --- | --- |
| `gutter.nerd_fonts = false` (default) | `■` (U+25A0) | Solid square for ordinary terminal fonts, colored blue by rev. |
| `gutter.nerd_fonts = true` | `` (U+F02E) | Flat bookmark for Nerd Fonts or a compatible font, colored blue by rev. |

The owner rejected the emoji-style `🔖` and selected the square rather than a
thin rectangle or full-cell block. The marker must remain distinct from a
breakpoint circle by shape even without color. Terminal fonts own the glyph
drawing; rev owns its foreground color. The factory path requires no additional
font installation or embedded font. Existing color overrides and `NO_COLOR`
behavior remain applicable.

The user enables the alternative with an ordinary configuration value:

```toml
[gutter]
nerd_fonts = true
```

The flag expresses terminal font support; it does not install or select a font.
Rev performs no automatic font detection. A terminal missing the opted-in glyph
may display its missing-character symbol. The square is the safe factory
choice, not an automatic substitution performed by the terminal. These rules
describe ordinary rev usage independently of any development environment.

## MODIFIED Requirements

### Requirement: BMK03 Derive current gutter markers

The gutter SHALL reserve a dedicated marker area on the outside of its number
field while the per-view marker policy is on, whether or not numbers are
visible. On the left, markers SHALL precede numbers; right-side placement SHALL
mirror this order. The existing number format and separator SHALL remain
supported. The marker area SHALL occupy one terminal cell. Its width SHALL
remain stable between bookmarked and unmarked rows while the same policy is
effective.

A resolved logical source line SHALL display one bookmark symbol. Multiple
bookmark names on that line SHALL share the symbol and remain individually
navigable. Only its first visual fragment SHALL receive a marker; continuation
rows, filler, hidden folded bodies and surrogate headings SHALL NOT acquire
one. A target inside a fold appears after ordinary bookmark navigation reveals
it through the existing owner.

Lookup SHALL use the current buffer/revision index rather than scan all stored
records for every row. Paint SHALL perform no filesystem IO, relocation or
persistence. Visibility, numbering, scrolling, focus and resize SHALL NOT change
bookmark identity. Narrow panes SHALL preserve the existing three-column text
minimum and omit the marker area before sacrificing available complete number
geometry. Requested presentation SHALL return when space permits it.

#### Scenario: Bookmark before the number field

- **GIVEN** a resolved bookmark on line 17 with left-side numbers enabled
- **WHEN** the view is rendered
- **THEN** its bookmark symbol appears to the left of the number field
- **AND** document text begins after the gutter's complete reserved area.

#### Scenario: Independent marker visibility and numbering

- **GIVEN** a resolved bookmark and enabled markers
- **WHEN** numbers are hidden or switched between absolute and relative mode
- **THEN** the marker retains its logical source-line identity outside the text
- **AND** hiding markers later does not delete or alter the bookmark.

#### Scenario: Wrapped, folded and narrow views

- **GIVEN** a bookmarked source line with wrapped continuations or folded content
- **WHEN** the view is scrolled or resized
- **THEN** only its visible first source fragment may carry the marker
- **AND** insufficient width hides the complete marker area without clipping a
  symbol, shrinking protected text space or marking an unrelated row.

## ADDED Requirements

### Requirement: BTG01 Recognizable bookmark presentation

The factory bookmark marker SHALL be the solid square `■` with
`gutter.nerd_fonts = false`. With that boolean set to `true`, the marker SHALL
be the flat bookmark ``. Both symbols SHALL be compiled into rev and SHALL use the
same independently configurable blue bookmark role rather than a yellow dot
or a skeuomorphic emoji. Their shapes SHALL distinguish them from the circular
symbols reserved for future breakpoints. The user SHALL NOT need to find,
copy or enter a glyph or code point. Custom bookmark characters SHALL NOT be
part of the supported configuration surface.

The boolean SHALL use the existing composed TOML loader, source provenance,
candidate validation and immutable startup snapshot. Invalid types SHALL reject
the candidate through ordinary configuration failure behavior. Changing the
file affects the next startup; no new runtime font command is introduced.

#### Scenario: Factory bookmark appearance

- **GIVEN** factory marker settings and a resolved bookmark
- **WHEN** the line is displayed
- **THEN** its marker is a solid blue square in the marker area
- **AND** ordinary text rendering does not include a floating yellow dot.

#### Scenario: Nerd Fonts bookmark appearance

- **GIVEN** a terminal using a font containing `nf-fa-bookmark`
- **AND** startup configuration sets `gutter.nerd_fonts = true`
- **WHEN** a bookmarked line is displayed
- **THEN** the marker is the flat bookmark silhouette, colored blue by rev
- **AND** no emoji-style bookmark is substituted.

#### Scenario: Built-in symbols preserve alignment

- **GIVEN** either boolean choice and its corresponding built-in symbol
- **WHEN** marked and unmarked lines are drawn, or the bookmark is removed
- **THEN** the whole glyph stays inside its reserved area
- **AND** number alignment and document origin do not move between those rows.

#### Scenario: Existing configuration has a legacy custom marker

- **GIVEN** otherwise valid existing TOML containing a `bookmarks.marker` string
- **WHEN** the settings candidate is prepared
- **THEN** the legacy string neither rejects the candidate nor controls the glyph
- **AND** `gutter.nerd_fonts` selects the square or the flat bookmark normally.

### Requirement: BTG02 Folding column exists only when enabled

The current delivery SHALL allocate zero cells for the future folding column,
including zero folding-specific padding. Existing number padding and the
separator are separate concerns and SHALL retain their configured behavior.

When gutter folding indicators are implemented, their enabling TOML setting
SHALL control the existence of their complete column. Disabled indicators
SHALL reserve zero cells. Enabled indicators SHALL appear between numbers and
the source-facing separator; lines without a fold indicator may then have an
empty cell inside that explicitly enabled column. Neither future feature
availability alone nor bookmark visibility SHALL enable this column.

#### Scenario: Present editor has no folding column

- **GIVEN** the current editor without structural-folding gutter indicators
- **WHEN** numbers and bookmark markers are displayed
- **THEN** no empty folding column or folding-specific padding is allocated.

#### Scenario: Future folding visibility changes geometry

- **GIVEN** a future implementation with working gutter folding indicators
- **WHEN** its TOML visibility setting is enabled for startup
- **THEN** the folding column is included on the text-facing side of the numbers
- **AND** disabling that setting removes the entire column and its padding.

### Requirement: BTG03 Shared marker area for future breakpoints

Future breakpoint indicators SHALL use these owner-selected Unicode symbols
in the existing marker area outside the number field:

| State | Symbol | Code point | Foreground |
| --- | --- | --- | --- |
| Enabled | `●` | U+25CF | Red |
| Disabled | `○` | U+25CB | Gray |
| Pending verification | `◌` | U+25CC | Amber |

The same standard Unicode symbols SHALL apply with or without Nerd Fonts.
Rev SHALL apply the foreground color; no colored emoji or font-specific
breakpoint icon is required. Shape SHALL distinguish these states when color
is unavailable. Exact palette values remain part of the future debugger's
configuration contract.

Defining this presentation SHALL NOT allocate another permanent area, create
breakpoint records or add debugger actions to the current editor. Bookmark
identity and operations SHALL remain independent of any eventual breakpoint
lifecycle.

#### Scenario: Future breakpoint state changes

- **GIVEN** a future debugger displaying a breakpoint in the shared marker area
- **WHEN** it reports enabled, disabled or pending verification
- **THEN** the gutter shows `●` in red, `○` in gray or `◌` in amber, respectively
- **AND** each state uses one cell and remains distinguishable by shape without
  color or Nerd Fonts.

#### Scenario: Bookmark-only editor

- **GIVEN** this delivery with bookmarks and no debugger
- **WHEN** its gutter is rendered or receives pointer input
- **THEN** no breakpoint-only column, placeholder dot or debugger action exists.

### Requirement: BTG04 One geometry for paint and interaction

Gutter partitioning, wrapping width, horizontal scrolling, cursor placement,
completion anchoring, pointer hit testing and selection SHALL use the same
checked terminal-cell geometry. No marker, number, separator or future folding
cell SHALL become document text or a source insertion position. A marker SHALL
NOT overlap an adjacent number, separator or text cell.

Absolute and relative logical numbering, per-view policy, shared buffers,
source grapheme mapping and existing Org visibility SHALL retain their owning
contracts. Presentation changes SHALL NOT modify source bytes, source revision,
dirty state, undo, copied text, search results or saved files. The current
delivery SHALL NOT turn gutter clicks into new bookmark or folding commands.

#### Scenario: Select and copy beside a bookmark

- **GIVEN** a bookmarked line shown in two editing panes
- **WHEN** the user clicks or selects source text beside the gutter and copies it
- **THEN** input maps to the expected source graphemes in each pane
- **AND** the copied text contains no gutter glyphs or padding.

#### Scenario: Relayout preserves editing behavior

- **GIVEN** wrapped text, relative numbers and an active completion popup
- **WHEN** marker visibility, gutter side or pane width changes
- **THEN** text fragments, cursor, pointer mapping and popup anchor use the
  updated geometry coherently
- **AND** source text, bookmark identity and the other pane's policy are retained.

## Documentation and acceptance

Implementation must update the gutter and bookmark reference pages, the
bookmark how-to, and the relevant User Guide example. Show the marker before
the number field with a short text illustration. Configuration reference and
sample values must agree with the approved glyph, width admission and palette.
The Spore guide must derive from the same Markdown material. Future folding and
debugger behavior must remain identified as future behavior, not advertised as
shipped functionality.

Focused acceptance must cover both interaction profiles, both gutter sides,
numbering hidden and visible, marker visibility, multiple names on one line,
wrapped continuations, existing Org folds, digit-width changes, narrow panes,
the square and Nerd Fonts alternatives, pointer/selection/cursor mapping, source-byte
fidelity and the absence of any reserved folding column. Rendering must retain
the current prepared bookmark index and bounded visible-row work.

## Approved delivery boundary

The layout, selected glyphs, marker meanings, boolean font-support setting and
zero-width disabled folding column form the approved delivery contract.
The chosen blue uses an existing palette value; it does not claim that the
reference screenshot uses that exact RGB value.
The future folding key and debugger collision policy are deliberately not
requirements for delivering the current bookmark presentation.

This change document does not alter the Completed status or historical gate
evidence of its baselines. Their current presentation clauses and existing
model/configuration descriptions must be reconciled when this change is
implemented; bookmark storage remains unchanged.
