# Slice 10: Define panel geometry contracts

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `refactor/rev-config-ui-10-panel-contracts`
PR: [#59](https://forge.adrianpk.com/adrianpk/rev/pulls/59)

## Purpose

Prepare one small panel presentation contract for editing panes, tree and
completion consumers while retaining the existing semantic split tree.

## Delivered Behavior

- Panel records expose outer/content bounds, an optional final status row and
  owner-supplied active state. Content-local cursor cells and spans translate
  through checked bounds; status and separator cells stay outside content.
- Layout derives shared horizontal and vertical separator spans, including
  nested T/cross junctions, without an enclosing frame or a terminal-sized raster.
- Composition reserves one prompt row and clamps optional completion content
  plus its separator to surplus rows after the full recursive editing minimum.
  Every leaf retains three columns, a content row and a status row.
- Tiny or zero dimensions suspend the whole editing projection. Reprojection
  retains live ViewIds, layout order, independent points and buffer/history
  ownership. Activity follows the existing selected view.

## Implementation Notes

- T10.1: `964812e837b27a4f406d2ae75730bd478b1d48ea` adds `Panel` and
  checked construction of the existing shared `Area` rectangle type. No second
  rectangle convention or buffer/view registry is introduced.
- T10.2: `407b116d69ce421cd51fa862b9e57f5fcd02b471` extends the existing
  layout traversal and recursive allocation with explicit separator spacing.
  Current runtime geometry keeps its existing spacing; the new panel projection
  reserves actual shared cells on both axes.
- T10.3: `d73cc5f1dfe467877e40e04c67ed7d8eaafbe5e9` adds the projection
  grid, full native extent arithmetic and live activity checks. The reversed-range negative fixture uses an explicit range record
  while preserving its rejection assertion under Clippy.
- Separator glyphs derive from neighboring reserved cells over compact spans.
  Panel content, status, separators, auxiliary content and prompt partition the
  admitted surface. No rendering or input consumer is migrated in this layer.

## Contracts Added or Changed

- `Area::new` validates half-open terminal-cell endpoints, including valid empty
  bounds and native maxima. `Panel` derives status/content and clips translated
  cells without deciding Unicode grapheme boundaries or semantic input focus.
- `Composition::new` projects the authoritative `Windows`/`Layout` into read-only
  panel records, separator spans, optional auxiliary content and a prompt row.
  Insufficient recursive space returns no partial editing composition.
- Shared panel minima include a horizontal separator row between stacked
  subtrees. Existing runtime split admission, geometry and rendering remain
  unchanged until consumer migration.

## Files of Interest

- `src/panel.rs`: presentation records and content clipping/translation.
- `src/panel/composition.rs`, `border.rs`: reservations and separator topology.
- `src/window/geometry.rs`, `layout.rs`: shared rectangle/allocation owner.
- `tests/panels.rs`, `tests/panels/geometry.rs`: native contract tests.

## Validation

Development checks:

- `cargo test --test panels cell_clipping`: passed.
- `cargo test --test panels panel_bounds`: passed.
- `cargo test --test panels native_bounds`: passed.
- `cargo check --lib`: passed.
- `cargo test --test panels junction`: passed; two junction cases.
- `cargo test --test panels auxiliary_rows`: passed.
- `cargo test --test panels geometry::`: passed; three geometry cases, including
  4608 small projections and native dimension extremes.
- `cargo clippy --lib --test panels --test windows -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/panel.rs --source src/panel/composition.rs --source src/panel/border.rs --source src/window/layout.rs --source src/window/geometry.rs --root tests/panels.rs --source tests/panels/geometry.rs`: passed.

All six ordered checks passed on the clean final PR head
`e3763fd30b8ab0aeb433d25c44e4f43d7bf1eb13`, controller validation
`HHR6ATRPUOOJEL222DBMUJLWMN`: 9 panel and 38 window tests.
PR #59 integrated that exact commit into dev by Rebase + Fast-forward.
Commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test panels`
- `cargo test --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Editing renderer migration remains assigned to Slice 11; tree and auxiliary
composition consumers remain assigned to Slice 12. This preparatory contract
does not change visible separators or input semantics.
