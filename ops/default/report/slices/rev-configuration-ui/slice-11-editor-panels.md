# Slice 11: Compose editing panes through panels

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `refactor/rev-config-ui-11-editor-panels`
PR: [#60](https://forge.adrianpk.com/adrianpk/rev/pulls/60)

## Purpose

Activate common panel composition for editing surfaces, including shared
separators and one source of placement for text, overlays and cursor cells.

## Delivered Behavior

- Editing panes draw continuous Unicode shared separators with connected T and
  cross junctions. Each split reserves one cell on its split axis; stacked
  panes retain separate content and status rows without enclosing pane frames.
- Live split admission, view reveal and rendering use the same recursive minima.
  The default half-split preference, layout order and selected identity remain.
  A below split now needs six terminal rows for two minimal editing panes plus
  their shared separator and the global prompt row.
- Text, syntax, region, search, status and hardware cursor positions derive from
  panel content/status bounds. Syntax covers every visible pane; only the
  selected pane owns region/search highlighting and the document cursor.
- Shared buffers retain independent points and navigation. Wide/combining/ZWJ
  text clips at pane seams as whole glyphs or blank edge cells. Tiny frames
  suspend editing projection while retaining layout, input and document state.

## Implementation Notes

- T11.1: `d1ae49100a76035ddd9ea98a48484f1d4fd86e8c` activates panel
  composition and shared separator topology in the editing renderer, removes
  the previous spacing mode and updates exact geometry/admission fixtures.
- T11.2: `7b773d08b9dacd93e9da589f54220246c6c35c74` renders each panel
  directly into one frame instead of creating and offsetting temporary pane
  frames. `Row::projected` translates clipped metadata through `Panel::clip`;
  cursor placement uses `Panel::position`, status uses its reserved bounds and
  search uses selected-panel content.
- T11.3: `ca7ea55c3534c7a894ad0d4dccfa70fe6c19d1f7` adds rendered
  crossing, focus/resize and lower-right overlay evidence.
  Existing resize/minimum and Unicode seam expectations deliberately change for
  actual separator rows and Unicode cells rather than removing assertions.
- Row publication consumes its text and metadata. A full-width row transfers
  directly; partial rows preserve adjacent pane cells through grapheme clipping.
  The frame retains its existing single echo row and terminal write owner.
- The Unreleased note and defaults reference describe the visible presentation
  and separator minimum change.

## Contracts Added or Changed

- Runtime `Layout` geometry now uses shared separator spacing on both axes,
  matching `Composition`. Admission reserves recursive child minima before
  publishing a split; undersized projections remain all-or-nothing.
- Editing rendering consumes `Panel` content/status/activity rather than
  independent area offsets and ASCII divider painting. Native source positions,
  buffer ownership, undo/saved identity and focus traversal remain authoritative.
- Projected syntax/selection spans are already surface-qualified before row
  publication. Search highlights clip through the same content bounds, keeping
  separator/status/prompt cells outside editing overlays.

## Files of Interest

- `src/display.rs`: panel content/status and cursor projection.
- `src/display/compose.rs`: shared frame, row publication and separator drawing.
- `src/display/search.rs`, `src/session/search.rs`: selected-panel highlighting.
- `src/window/layout.rs`, `geometry.rs`: live shared separator minima.
- `tests/panels/rendering.rs`: observable pane and overlay composition.
- `tests/windows.rs`, `tests/windows/resize.rs`: exact geometry and resize cases.
- `CHANGELOG.md`, `docs/reference/defaults.md`: visible defaults.

## Validation

The first controller validation passed panels/display and stopped at
`cargo test --test windows`: `policy::shared_policy` used byte offsets across
a Unicode separator. Its exact text assertion now indexes the fixture's
one-cell characters; the focused policy regression passes.

Development checks:

- `cargo test --test windows mixed_layout`: passed.
- `cargo test --test windows fit_admission`: passed.
- `cargo test --test windows composed_cells`: passed.
- `cargo test --test panels shared_separators`: passed.
- `cargo test --test windows resize::branch_minima`: passed.
- `cargo test --test search selected_surface`: passed.
- `cargo test --test panels projected_overlays`: passed.
- `cargo test --test syntax shared_surfaces`: passed.
- `cargo test --test display initial_frame`: passed.
- `cargo test --test panels rendering::`: passed; four rendered cases.
- `cargo test --test windows resize::`: passed; six resize cases.
- `cargo test --test windows policy::shared_policy`: passed after correcting
  the rendered fixture's byte-versus-cell indexing.
- `cargo clippy --lib --test panels --test display --test windows -- -D warnings`: passed.
- `cargo clippy --test panels -- -D warnings`: passed after final test additions.
- `cargo run --quiet --example source-check -- --source src/display.rs --source src/display/compose.rs --source src/display/search.rs --source src/session/search.rs --source src/window/layout.rs --source src/window/geometry.rs --root tests/panels.rs --source tests/panels/rendering.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/panels/rendering.rs`: passed after final test additions.

The controller stores final results for the exact clean PR head after its
PR-reference follow-up commit. Commands, in canonical order, with focused
search/syntax/tree consumer regressions after the required prefix:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test panels`
- `cargo test --test display`
- `cargo test --test windows`
- `python3 scripts/check-source.py`
- `cargo test --test search`
- `cargo test --test syntax`
- `cargo test --test tree`
- `git diff --check`

## Risks and Follow-ups

Tree presentation and auxiliary composition remain assigned to Slice 12. Their
existing flows retain the current owners while editing panes use panels.
