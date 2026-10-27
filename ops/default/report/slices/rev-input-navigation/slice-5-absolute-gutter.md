# Slice 5: Render absolute logical line numbers

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-05-absolute-gutter`
PR: pending

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
- `tests/gutter/rendering.rs`, `tests/support/surface.rs`: actual row and SGR cells.

## Validation

- PASS: `cargo test --test gutter` (15 native groups, zero skipped).
  New groups cover actual absolute frames, real/padding-row boundaries including
  `usize::MAX`, semantic colors/current emphasis and the real ASCII output path.
- Focused source analysis passes unchanged safe-Rust, parameter, nesting and
  complexity limits. The SGR fixture decoder delegates non-RGB attributes to a
  small owner after adding bold support exceeded the cyclomatic-ten limit;
  every prior supported code and unexpected-code rejection remains asserted.
- PASS: `cargo test --test gutter alignment::` reproduces and corrects reveal
  using full panel width instead of final text width. Six left/right and narrow/
  wide cases assert exact tab/Unicode clipping, cursor placement and hidden-state
  geometry with unchanged canonical text/history.
- PASS: `cargo test --test panels projected_overlays` retains the existing real
  nested-panel syntax/selection/search/cursor assertions at the final text origin;
  peer points, border/status cells and source bytes remain protected.
- Broader display regressions and exact clean-HEAD evidence are in progress.

## Risks and Follow-ups

None recorded.
