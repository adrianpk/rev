# Org Heading Presentation

Status: Completed
Completed: 2026-11-03
Date: 2026-10-28
Reviewed: 2026-11-03
Version: 0.2
Approved: product owner, 2026-11-03; explicit continuation to finalize and approve the heading-presentation specification
Format: OpenSpec capability specification
Source: product-owner heading clarification, reference screenshot and verified Org Superstar glyph sequence
Model: [Org Heading Presentation Model](editor-org-heading-presentation-model.md)
Org owner: [Org Mode](editor-org-mode.md)
Configuration owner: [Editor Configuration](editor-configuration.md)
Delivery boundary: approved three-slice autonomous delivery through dev; explicit Looper startup requested 2026-11-03
Readiness: delivered through three canonical slices; controller full gate passed at the candidate recorded in the linked tracker
Plan: [Heading Presentation Delivery](../plan/rev-org-headings.md)
Tracker: [Heading Presentation Tracker](../tracker/rev-org-headings.md)

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
After level 6, cycle through the sequence: level 7 uses ◉ and level 8 uses ○.
A startup preference can instead retain the final configured marker for deeper
levels. This does not establish new colors or require importing another
editor's theme or font configuration.

## Reference comparison and settled profile

The [Org manual](https://orgmode.org/manual/Hard-indentation.html) distinguishes
hiding leading stars from removing source indentation. Its
[Org Indent mode](https://orgmode.org/manual/Org-Indent-Mode.html) also introduces
virtual indentation of body text. The
[org-superstar reference](https://github.com/integral-dw/org-superstar-mode#custom-utf8-bullets-for-heading-lines)
cycles heading markers by default and can repeat the last entry instead.
These are behavioral references; rev does not load either Emacs facility.

The heading-only baseline decorates the original source prefix. The composed
profile adds the independent buffer-local [Org Visual Indentation](editor-org-indentation.md)
policy, default true; its outer margins align heading titles and bodies at 2N
cells where width permits. Disabling that policy uses a flat outline: hidden
leading stars occupy no screen cells, so each pretty marker starts at column
zero. Source/visible-star presentation keeps its real prefix width.

Within an enabled indentation margin, preserve the original prefix width: a level-N heading uses
N-1 blank cells followed by its one-cell marker, then the original separator
and title. With leading-star hiding disabled, the first N-1 asterisks stay
visible and only the final star is replaced. With pretty presentation disabled,
every source star stays visible. Reuse the existing Org heading palette roles.

Defaults are `pretty = true`, the six reference markers,
`hide_leading_stars = true` and `cycle_bullets = true`, under `[org.headings]`.
The model defines their exact types, admission bounds and precedence. These
settings are delivered in the native binary, complete startup catalog and
packaged sample.

`org-pretty-headings-mode` toggles pretty presentation in the active Org view.
`org-reset-heading-presentation` clears its override and restores the immutable
startup choice. Both are compiled commands available through M-x and modal
colon, without a new default keybinding. Runtime changes do not write TOML.
The view retains its override across buffer switches, ignores it in non-Org
buffers, and resumes it when Org is active again. A split copies that view's
choice; later changes in either pane remain independent. A source edit or
reload invalidates derived mappings, not the view's explicit choice.

The prefix does not automatically reveal on cursor entry or editing. Original
stars retain canonical positions. In indented view their separate cells remain
addressable; in flat pretty view hidden stars share the marker's cursor cell.
Clicking the marker targets the last star. Character motions and edits still
operate on source stars. Source/pretty switching may reflow a flat heading.
Copy returns the original selected text.

## ADDED Requirements

### Requirement: OGP01 Level-specific heading markers

Recognized Org headings SHALL select a display marker using their actual
structural level and the admitted configured sequence. The reference sequence
SHALL be the factory sequence. A heading-level edit SHALL recompute
the marker from current source. Glyphs SHALL not be inserted into document
text to represent a heading level.

#### Scenario: Present the reference levels

- **GIVEN** recognized headings at levels 1 through 6
- **WHEN** the reference presentation is enabled
- **THEN** their markers are respectively ◉, ○, ✸, ✿, ◆ and ▶
- **AND** the heading hierarchy is still determined by source stars.

#### Scenario: Resolve levels beyond the sequence

- **GIVEN** the six factory markers and a level-7 heading
- **WHEN** `cycle_bullets` is true
- **THEN** its marker is ◉
- **AND** its structural level remains 7.

#### Scenario: Retain the last marker instead

- **GIVEN** the six factory markers and `cycle_bullets` false
- **WHEN** headings at levels 7 and 8 are displayed
- **THEN** both use ▶
- **AND** their different structural levels and source prefixes remain intact.

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
The complete source prefix's cell width SHALL remain unchanged within enabled
Org indentation margins, including while point is inside it. When indentation
is disabled and leading stars are hidden, those cells SHALL collapse and the
marker SHALL begin at the left margin. Shared word-wrap, source-to-cell and
selection owners SHALL account for that collapse without changing source data.
Enabled [Org indentation](editor-org-indentation.md) changes the available source
width and continuation margin through that same checked projection; its virtual
cells SHALL never become canonical source positions.

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
Unspecified values SHALL use complete built-ins. A marker list SHALL contain
1..=32 entries, each at most 32 UTF-8 bytes and one printable non-whitespace
grapheme of terminal width one under the existing native width policy. Control
characters, empty entries and zero/wide-cell markers SHALL be rejected. The
total admitted marker payload is at most 1024 bytes. An invalid
candidate SHALL follow existing coherent configuration failure handling.

#### Scenario: Override the marker sequence

- **GIVEN** a valid custom marker sequence in startup TOML
- **WHEN** a new session admits that configuration
- **THEN** recognized heading levels use the selected sequence
- **AND** changing the preference does not rewrite the Org document.

#### Scenario: Reject a marker that changes cell geometry

- **GIVEN** an empty sequence, a control character or a two-cell marker
- **WHEN** startup configuration is validated
- **THEN** the whole candidate follows the existing configuration failure policy
- **AND** no partial heading preference or edited document is published.

### Requirement: OGP06 Native source and pretty-view controls

A compiled native command SHALL let the user switch heading presentation
between decorated and readable source in the active Org view. Startup defaults
and session-local view overrides SHALL remain distinct. M-x and the delivered
modal colon selector SHALL reach the same handlers named above. Controls SHALL
not change the editor interaction profile or persist an override implicitly.

#### Scenario: Return to visible source

- **GIVEN** a decorated Org view with unsaved edits
- **WHEN** the user selects ordinary source presentation
- **THEN** its heading stars are readable without decoration
- **AND** the same buffer, edits, history and source point remain available.

#### Scenario: Keep independent pane choices

- **GIVEN** two panes displaying the same Org buffer
- **WHEN** pretty presentation is toggled in one pane
- **THEN** only that pane's presentation changes
- **AND** splitting it inherits its current choice while reset restores only
  the selected view's configured default.

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
If native structural observation is incomplete, stale or unavailable, undecided
prefixes SHALL remain readable source until current evidence is available.
Turning syntax coloring off SHALL not disable heading recognition or make
unverified star-like text eligible for decoration. Respect `org.enabled` and
the existing Org observation, cancellation and work limits. Retain one bounded
startup marker list and viewport-derived projection rather than another
per-line or whole-document decoration store. Acceptance SHALL include both
editing profiles, splits, reload, soft wrap, disabled coloring, literal contexts
and read-only Org result buffers.
Completed Org delivery records SHALL remain historical evidence; this contract
SHALL NOT be presented as delivered heading decoration.

#### Scenario: Refresh after a local heading change

- **GIVEN** a large Org document with an existing recognized outline
- **WHEN** one heading changes level
- **THEN** current decoration updates through the admitted recognition/projection owners
- **AND** source is not reparsed or scanned as an unrelated project-wide operation.

## Acceptance fixtures and planning boundary

Use headings at levels 1 through 8, a custom one-marker sequence, visible and
hidden leading stars, and plain source view. Include source/example blocks with
star-like lines, a malformed/incomplete observation, Unicode titles and long
wrapped headings. Test insertion/deletion of stars, copy, save, undo, pointer
selection, split panes and external reload without changing source ownership.

Exercise the native view at 40x12 and 80x24 and through resize. A real Linux PTY
journey SHALL verify emitted reference glyphs, source/pretty switching,
cancellation, cursor position and terminal restoration under both profiles.
Rendering protocol evidence does not prove font coverage or ligature shaping;
visual review uses the terminal's existing configured font, without installing
fonts or making a new font a build/test prerequisite.

No product decisions remain open for this capability. The approved delivery plan
identifies the observation/projection owners, proportional native/PTY checks and
projection-cost evidence. The owner's separate startup request authorizes its
one retained controller; the completed earlier Org gate remains historical.

## Delivery relationship

This presentation follow-up to ORG16/ORG17 is delivered through PRs #238-#240.
The model's four preferences and source/view mappings are implemented and the
paired plan/tracker are Completed. The unchanged controller full gate passed
at `18f625997de4c0ac2e1ae50d6fea667fe14f2c2d`, receipt
`T2QVMEBIU5HJTNBIOEXKDCUFSU`, closing OGP01-OGP08. The
[Slice 3 closure](../report/slices/rev-org-headings/slice-3-acceptance.md#delivery-set-closure)
records exact-candidate tests, bounded terminal/resource observations and local
guide validation. Earlier Org delivery gates retain their historical identity.
