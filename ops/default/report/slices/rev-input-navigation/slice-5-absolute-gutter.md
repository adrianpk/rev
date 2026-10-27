# Slice 5: Render absolute logical line numbers

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-05-absolute-gutter`
PR: [#85](https://forge.adrianpk.com/adrianpk/rev/pulls/85)
Integrated: `d4707f0381b0a20b4c0558724c321710b3ef8f20`

## Purpose

Render absolute logical line numbers inside editing panels while preserving
canonical text, terminal-cell geometry and the ordinary editing surfaces.

## Delivered Behavior

- Factory views show right-aligned one-based labels in a four-digit minimum
  field with zero left padding, one right padding cell and a one-cell separator.
- Empty buffers and final editable empty lines are numbered. Viewport padding
  beyond EOF has no label; tabs, Unicode and CRLF follow canonical logical rows.
- Ordinary labels, current-line labels, background and separator use semantic
  roles in the existing frame palette. Current labels also use bold emphasis,
  reset before document/status cells.
- Document clipping, syntax, linear/rectangular selection, search and cursor
  placement share the final text area. Horizontal reveal uses its reduced width
  while tab stops remain document-relative, on either side of the number field.
- The frame output boundary admits Unicode `│` or ASCII `|` separators at the
  same reserved cell. Literal document separators retain their source rendering.

## Implementation Notes

NumberRow stores checked integer metadata only for real visible source rows.
A label is at most 20 digits; each rendered field is bounded by admitted geometry.
Hidden or geometrically suppressed fields return before row materialization.
The renderer paints number fields and document text as separate cell-bounded
areas, then retains existing status, border, prompt and completion owners.
`View::gutter_geometry` is the shared partition used by rendering, horizontal
reveal and search clipping. Existing row projection translates all document
style/selection spans and the cursor through that final text panel exactly once.

The four compiled gutter roles extend the existing immutable frame palette.
Their TOML leaves and operational gutter command identities remain assigned to
Slice 7. Relative calculation and current-line alternatives remain Slice 6.

## Contracts Added or Changed

GUT01 keeps labels outside canonical text/history. GUT03 uses real logical rows;
GUT04 uses checked stable widths; GUT05 assigns final text bounds; GUT10 uses
safe fixed separators, semantic colors and non-color current-line emphasis.

## Files of Interest

- `src/gutter/row.rs`: checked absolute source-row metadata and separator choice.
- `src/display/gutter.rs`: bounded field materialization inside panel content.
- `src/display.rs`, `compose.rs`, `style.rs`, `selection.rs`: separate field/text
  painting and semantic terminal output.
- `src/config/palette.rs`: four native semantic gutter colors.
- `tests/gutter/rendering.rs`, `tests/gutter/alignment.rs`, `tests/support/surface.rs`:
  actual row/SGR cells, decimal-width and final-origin assertions.
- Existing display, panel, search, rectangle, syntax and window regressions:
  retained source contracts with final text-cell expectations.
- `scripts/configuration_ui/gutter.py`: two owned controlling PTY journeys.
- `docs/reference/gutter.md`: delivered absolute numbering behavior.

## Validation

- PASS: `cargo test --test gutter --test display --test panels` (61 native tests,
  including 17 gutter groups, zero skipped).
- Absolute frames cover empty/final-empty source rows, unnumbered EOF padding,
  CRLF, Unicode/tabs and checked integer labels through `usize::MAX`. Actual SGR
  cells verify all four palette roles, non-color emphasis, style reset and the
  typed ASCII separator output path without changing literal document separators.
- PASS: `cargo test --test gutter alignment::` reproduces and corrects reveal
  using full panel width instead of final text width. Six left/right and narrow/
  wide cases retain exact tab/Unicode clipping and hidden-state cursor placement.
- PASS: `cargo test --test panels projected_overlays` retains nested-panel
  syntax/selection/search/cursor, peer-point, border/status and source assertions
  at the final text origin. Native rendered edit/undo/redo cases cross 9999/10000
  coherently using ordinary viewport mapping and canonical logical points.
- Initial historical render assertions addressed the former full-width document
  origin. Expected ranges/colors now address final text cells; raw source,
  copied rectangle contents, logical endpoints, history and dirty-state checks
  remain. Tab goldens retain their exact original text extent. The seam fixture
  removes six prefix cells so its wide glyph still crosses the same protected
  edge after reserving the six-cell field.
- The syntax-color oracle previously replaced every equal RGB escape globally.
  Native gutter colors share two syntax defaults, so it now recolors only the
  baseline's captured source cells and compares the complete decoded cell map.
  Registry/standalone raw output equality and all source/cursor/history assertions
  remain; an explicit capture assertion prevents a gutter color proving syntax.
- PASS: `cargo test --test search selected_surface` verifies region precedence,
  search underline and reset at actual document cells alongside shared views.
- PASS: `python3 scripts/check-pty.py --suite absolute-gutter` (two owned sessions,
  zero skipped). Unicode/BOM/mixed separators and an empty monochrome file verify
  real/padding labels, current emphasis, physical cursor, edit/save/undo, exact
  source bytes and terminal/descriptor restoration. Existing eight-second and
  one-MiB PTY limits remain unchanged.
- PASS: `python3 scripts/check-config-reference.py` retains the currently admitted
  59 catalog entries, 25 configurable RGB roles and 111 bindings. The four native
  gutter roles do not advertise unsupported startup keys.
- Focused source analysis passes unchanged safe-Rust, parameter, nesting and
  complexity limits. The SGR fixture decoder separates RGB and other attributes
  after bold support exceeded cyclomatic ten, preserving every prior code and
  unexpected-code rejection.

PASS at the integrated candidate: controller evidence `IEKFELNPRVBKZWGWRGSE5HA5SC` records 85 native tests and this ordered canonical list:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test coordinates --test panels`
- `python3 scripts/check-source.py`
- `git diff --check`

PASS on the same candidate: focused evidence `6NOINAR4Q7ZBHZM5UCKOBJGA7J` records 288 native tests, two owned PTYs and this ordered list:

- `cargo test --test commands --test windows --test search --test rectangles --test syntax`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-pty.py --suite absolute-gutter`
- `git diff --check`

## Risks and Follow-ups

None recorded.
