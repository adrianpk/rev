# Slice 8: Preserve rectangle tab semantics

Status: delivered
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `fix/rev-config-ui-08-rectangle-capture`
PR: [#57](https://forge.adrianpk.com/adrianpk/rev/pulls/57)
Integrated: `7672b1eed5709c05d0221f705603ba12628c203f`

## Purpose

Separate rectangle copy and kill capture semantics for tabs while preserving
source-policy geometry, whole graphemes, short rows and atomic history.

## Delivered Behavior

- Copy expands all selected tab cells into retained spaces, including complete
  tabs, without changing source text, cursor, mark, saved state or history.
- Kill retains complete selected tabs as literal U+0009. Selected partial tab
  cells become retained spaces; deletion materializes only the unselected edge
  cells and leaves untouched tabs literal.
- Derives capture spans from the source buffer's effective tab width at every
  row. Wide, combining and ZWJ clusters remain complete even at touched edges.
- Retains only existing selected text on short/empty rows, with no right padding.
  LF/CRLF separators remain outside retained row fragments and deletion spans.
- Keeps rectangular text separate from the linear kill ring. Linear copy/yank
  retains literal tabs while rectangle copy holds expanded spaces.
- Preserves one checked publication and undo/redo operation for a multi-row kill,
  including point/mark restoration and truthful clean/modified state.

## Implementation Notes

- T8.1: `8271ccd8fa68ee05af02a412d3832088b5beedfe` adds explicit capture
  intent, retains the partial-tab deletion representation and updates the
  contradictory whole-tab copy/insertion expectations and user reference.
- T8.2: `d858fcd8bbf2e6eff2d6d1bbff02eeda4958aa8a` verifies configured
  source widths, Unicode, CRLF, short/read-only rows and independent stores.
- T8.3: `a74eff35b6f54c2b87066d2c15a0c96bc221b787` ports the accepted
  width-four row examples into original golden fixtures,
  strengthens the partial-edge matrix and checks configured-width kill replay.
  All three new capture tests and the affected existing focused cases pass.
- Each part records whether its source is a literal tab. Copy expands its actual
  selected cell span; kill uses literal source chunks for complete tabs. Partial
  tabs already retain unselected padding and prefix information for deletion.
- Source columns still come from the common coordinate owner. Capture and
  deletion derive parts synchronously from the borrowed buffer; no terminal
  surface, filesystem discovery or second display-width calculation is involved.
- The accepted `a\tb` / `c\td` example at width 4 retains `a   b` / `c   d`
  for copy and `a\tb` / `c\td` for kill. LF and CRLF fixture variants preserve
  the original source on copy and restore it exactly through kill undo.
- Copy-time whole-tab and copied-shape expectations now assert spaces. Existing
  kill, partial-edge, wide-cluster and failed-admission assertions remain active.
  The rectangle reference and Unreleased notes describe the visible correction.

## Contracts Added or Changed

- `Rectangle::capture` receives an explicit private `Capture::Copy` or
  `Capture::Kill` intent. Both use the same source cell/grapheme boundaries.
- Copy's retained rows contain spaces for selected tabs; kill's retained rows
  may contain complete literal tabs. Requested width metadata and actual
  per-row fragments remain distinct when Unicode edges expand a selection.
- Session retention publishes the new rectangle only after any required kill
  succeeds. Existing refusal, independent-ring and history ownership remain.

## Files of Interest

- `src/rectangle.rs`: explicit capture intent and source-part serialization.
- `src/session/rectangle.rs`: copy/kill capture selection and retained ownership.
- `tests/rectangles/capture.rs`: source-policy, accepted row examples and kill
  replay across representative widths.
- `tests/rectangles.rs`, `rectangles/kill.rs`, `rectangles/yank.rs`: source-tab
  copy expectations, independent stores, partial edges and copied-row insertion.
- `docs/reference/rectangles.md`, `CHANGELOG.md`: observable copy/kill behavior.

## Validation

Development checks:

- `cargo test --test rectangles tab_`: passed; three existing cases.
- `cargo test --test rectangles copied_shapes`: passed; adapted insertion case.
- `cargo test --test rectangles source_policy`: passed; five admitted widths.
- `cargo test --test rectangles separate_store`: passed; literal linear tab.
- `cargo test --test rectangles capture::`: passed; three capture cases.
- `cargo test --test rectangles kill::tab_fragments`: passed; strengthened matrix.
- `cargo clippy --lib --test rectangles -- -D warnings`: passed.

All six ordered checks passed on the clean final PR head
`7672b1eed5709c05d0221f705603ba12628c203f`, controller validation
`RSTOMMXIYRVKDJNLZI37TEXLBN`: 38 rectangle and 24 coordinate tests.
PR #57 integrated that exact commit into dev by Rebase + Fast-forward.
Commands, in canonical order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test rectangles`
- `cargo test --test coordinates`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Cross-width rectangle insertion and automatic path-change policy transitions
remain assigned to Slice 9. Startup configuration activation remains assigned
to Slice 20; configured-width capture is tested through supplied snapshots.
