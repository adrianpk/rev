# Slice 32: Select and copy display-column rectangles

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-32-rectangle-copy`
PR: `#32`
Integrated: Rebase + Fast-forward into dev at `42b879a78263a3cb5dd6dcc24cd920a9173a14f9`.

## Purpose

Select and copy rectangles in display columns using the same Unicode/tab policy
as drawing, retaining source row fragments independently of the linear kill ring.

## Delivered Behavior

- C-x SPC and M-x rectangle-mark-mode toggle rectangle selection. Activation
  reuses an active mark or starts at point; rows include both endpoints and
  columns are ordered independently. Normal motion grows or reverses selection.
- The selected pane highlights each row's actual selected cells. Touched wide
  graphemes, combining clusters and joined emoji remain whole. Short and empty
  rows acquire no synthetic selection beyond their existing source text.
- C-x r M-w and M-x copy-rectangle-as-kill share the real copy handler. Reported
  Alt-w and compatible Escape-w work inside the nested rectangle prefix.
  Copying requires active mode and a nonzero display-column span.
- Partially selected tabs contribute their selected existing cells as spaces;
  complete selected tabs retain their source bytes. Row fragments exclude LF/CRLF
  separators and preserve original Unicode and control text. Copy adds no absent
  trailing source cells and never substitutes drawn control notation for text.
- One session-local rectangle stores ordered rows and the requested display width.
  It is separate from the linear ring, whose copy/yank behavior stays independent.
  A successful copy replaces the block; empty/inactive or focused-prompt refusals
  preserve the retained block.
- Copy is allowed on read-only text and changes no rope, revision, cursor, mark,
  modification state or undo history. C-g, toggling off, normal mark entry and
  successful text publication end rectangle selection; refused edits preserve it.
  Shared views retain a common buffer mark/mode and independent selected points.

## Implementation Notes

- T32.1: `c68bd85ba5702d8c6a7f46d2be23557b1c010655` adds mark mode,
  qualified geometry, row extraction/storage, bound/named routing, projection,
  three native tests and product reference. Task completion at
  `d7b17247f5265a2678196aeed944e88a49e435c3` includes catalog and existing
  nested-chord fixture expectations; all seven ordered checks passed there.
- T32.2: `3b0d40c28bfd16ed6bf468024067410cfa08fe29` expands the native
  suite to twelve cases and adds two real PTY journeys. All seven ordered checks
  passed, with 333 native tests and 66 terminal cases.
- Buffer selection mode distinguishes inactive, linear and rectangle states.
  Prepared text publication captures that mode, so a mode change invalidates an
  earlier candidate just like mark movement. Stored mark affinity and canonical
  edit/history ownership remain in the buffer.
- Geometry retains a source Stamp and borrows Glyphs from canonical rope lines.
  Drawing and extraction use the same selected parts: complete touched clusters
  and precise tab-cell fragments. Explicit copy owns only the retained row text;
  redraw does not flatten or copy the selected document into a second text store.
- No new dependency or arbitrary source-length cap is introduced. The store retains
  one block, proportional to the explicitly copied existing source fragments.
  The compiled coordinate policy defines its four-cell tabs and Unicode width;
  requested width remains distinct from row byte/scalar counts and wide expansion.

## Contracts Added or Changed

- The implemented catalog exposes rectangle-mark-mode and copy-rectangle-as-kill.
  Nested Meta suffixes retain their keymap and cannot invoke linear M-w.
- Rectangle activation is buffer-owned with the mark; selected-pane projection
  derives its other endpoint from that view's qualified point. Clipping and
  scrolling change presentation without changing copied source geometry.
- Rectangle copy publishes its completed owned block only after extraction
  succeeds. Empty, inactive and focused-input refusals retain the previous block.
- Source-qualified mode admission rejects foreign/stale endpoints and interior
  grapheme positions without altering selection or history.
- docs/reference/rectangles.md documents bindings, cell boundaries, store ownership
  and selection lifetime; the command catalog and reference index link it.
  CHANGELOG.md records the delivered selection and copy workflow.

## Files of Interest

- `src/rectangle.rs`: qualified geometry, selected parts and retained rows/width.
- `src/buffer/region.rs`, `src/buffer/edit.rs`: mode lifetime and checked admission.
- `src/session/rectangle.rs`: shared command handler and separate session store.
- `src/command/rectangle.rs`, `src/input/rectangle.rs`, `src/input/bindings.rs`:
  discoverable commands and nested terminal bindings.
- `src/display/selection.rs`: existing-cell projection and complete cluster edges.
- `tests/rectangles.rs`: twelve native selection/copy and lifetime cases.
- `scripts/check-pty.py`: real bound/named rectangle journeys and nested Meta reports.
- `docs/reference/rectangles.md`, `docs/reference/commands.md`,
  `docs/reference/index.md`, `CHANGELOG.md`: reader-facing behavior and discovery.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 333 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test rectangles`: passed; twelve tests, zero skips. Cases cover
  reversed endpoints, partial/full tabs, CJK/emoji/combining/control text, CRLF,
  short/empty rows, clipped/scrolled chunk-spanning source, mode admission,
  independent stores, read-only/history behavior, shared views and prompt focus.
- `python3 scripts/check-pty.py --suite terminal`: passed; sixty-six cases,
  zero skips. New journeys verify actual C-x SPC/C-x r M-w and named commands,
  wide/tab highlighting, reversed endpoints, independent linear yank, scroll,
  zero/tiny resize, cancellation, empty history and exact unchanged BOM/CRLF bytes.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
