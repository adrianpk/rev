# Org Heading Presentation

Status: draft
Date: 2026-10-28
Reviewed: 2026-10-31
Version: 0.1
Format: OpenSpec capability proposal
Source: product-owner heading clarification, reference screenshot and verified Org Superstar glyph sequence
Model: [Org Heading Presentation Model](editor-org-heading-presentation-model.md)
Org owner: [Org Mode](editor-org-mode.md)
Configuration owner: [Editor Configuration](editor-configuration.md)
Delivery boundary: future follow-up to delivered Org; no implementation or slicing authorized
Readiness: heading scope and six reference glyphs confirmed; listed presentation details remain open

## Purpose

Present Org heading stars as configurable level-specific bullets while keeping
the document's actual star prefix intact. The request concerns headings whose
levels are written with one, two, three or more leading stars, not list-item
indentation. Match the supplied six-level reference and allow user overrides.

The terminal owns font selection, size, glyph coverage and ligature shaping.
This feature selects displayed characters; it does not install fonts, choose
a font family per heading or add a font-rendering engine to rev. The selected
reference characters are Unicode symbols rather than Nerd Fonts private-use
code points. Their appearance still depends on the terminal's chosen font.

## Confirmed reference

Heading levels are one-based. The screenshot's top heading is level 1.
The characters below were checked against the owner's existing glyph list,
not inferred solely from pixels.

| Level | Source prefix | Display marker | Code point |
| --- | --- | --- | --- |
| 1 | `*` | ◉ | U+25C9 |
| 2 | `**` | ○ | U+25CB |
| 3 | `***` | ✸ | U+2738 |
| 4 | `****` | ✿ | U+273F |
| 5 | `*****` | ◆ | U+25C6 |
| 6 | `******` | ▶ | U+25B6 |

Repeated leading stars are visually concealed with level indentation retained.
The reference establishes these six levels; behavior beyond the configured
sequence is still an explicit design choice. It does not establish new colors
or require importing another editor's theme or font configuration.

## ADDED Requirements

### Requirement: OGP01 Level-specific heading markers

Recognized Org headings SHALL select a display marker using their actual
structural level and the admitted configured sequence. The reference sequence
SHALL be the proposed factory sequence. A heading-level edit SHALL recompute
the marker from current source. Glyphs SHALL not be inserted into document
text to represent a heading level.

#### Scenario: Present the reference levels

- **GIVEN** recognized headings at levels 1 through 6
- **WHEN** the reference presentation is enabled
- **THEN** their markers are respectively ◉, ○, ✸, ✿, ◆ and ▶
- **AND** the heading hierarchy is still determined by source stars.

### Requirement: OGP02 Presentation preserves source and history

Pretty heading presentation SHALL leave source bytes, buffer modification
state, history and save behavior unchanged. Text extraction/copy SHALL use the
admitted canonical source rather than substitute display markers. Toggling
presentation SHALL not create a document edit or undo group.

#### Scenario: Save a decorated heading

- **GIVEN** a level-3 heading displayed with ✸
- **WHEN** the document is saved
- **THEN** its source still contains the original three-star prefix
- **AND** enabling the presentation did not itself mark the document modified.

#### Scenario: Copy canonical heading text

- **GIVEN** an admitted selection includes a decorated heading prefix
- **WHEN** ordinary text copy reads that selection
- **THEN** it obtains the canonical selected stars and text
- **AND** the display glyph does not become document content or a history entry.

### Requirement: OGP03 Structural recognition owns decoration

Presentation SHALL use the native Org owner to identify a heading prefix.
Literal blocks, examples, ordinary multiplication/text and list-item markers
SHALL not be decorated merely because they contain stars. Folding, semantic
coloring and structural editing SHALL retain the same source ownership.

#### Scenario: Keep a literal example literal

- **GIVEN** a source/example block contains a line beginning with stars
- **WHEN** heading presentation is enabled
- **THEN** the line retains its literal-block interpretation and text
- **AND** no outer heading or pretty marker is invented for it.

### Requirement: OGP04 Correct source and display geometry

The projection SHALL retain explicit source/display mappings for concealed
stars, the visible marker and heading text. Cursor movement, hit testing,
selection, clipping, horizontal scrolling and gutter numbering SHALL keep
their canonical meanings. Initial glyph admission SHALL require one visible
grapheme occupying one cell under the supported width policy; control sequences,
blank/zero-width entries and unsupported widths SHALL be rejected coherently.
Preserving the source prefix's cell width is the proposed initial geometry.

#### Scenario: Edit a heading level

- **GIVEN** a decorated level-2 heading and a source-valid point in its prefix
- **WHEN** the user adds a third star through an ordinary edit
- **THEN** current recognition presents the level-3 marker
- **AND** cursor/selection state still identifies the actual edited source.

#### Scenario: Click or select across a decorated prefix

- **GIVEN** a current projection containing concealed stars and a marker
- **WHEN** a pointer or selection action targets that area
- **THEN** it resolves through checked current source mappings
- **AND** a rendered cell or stale row cannot authorize an unrelated source edit.

### Requirement: OGP05 Declarative startup configuration

Enablement, the marker sequence and supported leading-star presentation SHALL
be declarative TOML preferences using the existing startup validation and
override rules. The exhaustive sample SHALL describe them only when delivered.
Unspecified values SHALL use complete built-ins. Glyph list/byte limits and
depth-overflow behavior SHALL be fixed before implementation. An invalid
candidate SHALL follow existing coherent configuration failure handling.

#### Scenario: Override the marker sequence

- **GIVEN** a valid custom marker sequence in startup TOML
- **WHEN** a new session admits that configuration
- **THEN** recognized heading levels use the selected sequence
- **AND** changing the preference does not rewrite the Org document.

### Requirement: OGP06 Native source and pretty-view controls

A compiled native command SHALL let the user switch heading presentation
between decorated and readable source in the active Org view. Startup defaults
and session-local view overrides SHALL remain distinct. Existing command entry
and, when delivered, Vim colon entry SHALL reach the same handler. Controls
SHALL not change the editor interaction profile or persist an override implicitly.

#### Scenario: Return to visible source

- **GIVEN** a decorated Org view with unsaved edits
- **WHEN** the user selects ordinary source presentation
- **THEN** its heading stars are readable without decoration
- **AND** the same buffer, edits, history and source point remain available.

### Requirement: OGP07 Terminal font ownership and honest fallback

rev SHALL document that font selection and programming ligatures belong to
the hosting terminal. It SHALL not claim reliable detection of installed
glyph coverage or identical font rasterization. User-selected Nerd Fonts glyphs
may be admitted under the same width rules, but their font availability is an
external requirement. Readable source presentation SHALL remain available.
Programming operators SHALL not be rewritten into Unicode symbols to imitate
font ligatures. Compatibility checks SHALL preserve original text and positions.

#### Scenario: Use a terminal without the selected glyph coverage

- **GIVEN** the terminal renders a configured symbol poorly or as a missing glyph
- **WHEN** the user switches to source presentation or an admitted simpler sequence
- **THEN** heading text remains editable and structurally correct
- **AND** rev does not pretend it installed or automatically identified a suitable font.

### Requirement: OGP08 Bounded projection and delivery isolation

Decoration SHALL reuse native Org recognition and visible projection, with
revision/settings invalidation and bounded retained state. It SHALL not add
an idle scan, whole-project redraw loop or another Org parser. Acceptance SHALL
cover the reference markers, configuration, literal contexts, source fidelity,
Unicode/cell geometry, source/pretty switching and real terminal interaction.
Completed Org delivery records SHALL remain historical evidence; this draft
SHALL NOT be presented as delivered heading decoration.

#### Scenario: Refresh after a local heading change

- **GIVEN** a large Org document with an existing recognized outline
- **WHEN** one heading changes level
- **THEN** current decoration updates through the admitted recognition/projection owners
- **AND** source is not reparsed or scanned as an unrelated project-wide operation.

## Open design details

- Cycling versus retaining the final marker after the configured sequence ends.
- Final TOML/command names, default enablement, leading-star indentation and
  per-view override/reset/split inheritance.
- Reveal-on-edit behavior inside the prefix and precise hidden-cell hit mapping.
- Finite list/payload limits and real-terminal width/ligature fixtures.

## Delivery relationship

This is a presentation follow-up to ORG16/ORG17, not a claim that heading decoration is
already implemented. The model records proposed preference spelling and source
mappings without adding them to the shipped schema. Versioning this draft does
not add list-item prettification, install fonts or start implementation/slicing.
