# Slice 39: navigate and align native Org tables

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-39-table-basics`
PR: pending

## Purpose

Deliver native table basics under the
[approved plan](../../../plan/rev-org-mode.md#slice-39-navigate-and-align-native-org-tables)
and [tracker](../../../tracker/rev-org-mode.md#slice-39-navigate-and-align-native-org-tables).

## Delivered Behavior

T39.1 introduces `org_tables` and working creation, alignment, cell/row motion,
row insertion and separator commands. Table TAB/S-TAB, RET, C-c C-c and M-RET
reach native context actions; non-table RET retains ordinary newline editing.
T39.2 retains names, captions, attributes, formula spelling, source cell markup
and surrounding paragraphs. Ragged rows gain explicit empty fields, while
width cookies remain literal source without affecting numeric alignment.
Creating beside existing tables preserves separate table boundaries.

## Implementation Notes

The existing structural scanner identifies real table/row/cell ranges from the
canonical rope snapshot. Layout measures actual glyphs with the buffer's tab
policy and terminal column. Original nonempty cell spans remain retained runs;
only gaps become replacements. One canonical buffer transaction maps shared
positions and provides ordinary undo. No buffer identity is invented for a
rendered fragment. All analysis runs on the existing single cancellable worker.

## Contracts Added or Changed

- ORG12: original cell source, table motion/alignment and checked row insertion.
- ORG17: working table commands, contextual bindings and typed bounded input.

## Files of Interest

- `src/org/table/`: source observation, layout, cell retention and operation plans.
- `src/session/tables.rs`: source-qualified prompts, context dispatch and publication.
- `src/coordinates/display.rs`: shared native glyph/tab measurement for text fragments.
- `tests/org_tables.rs`: real buffer/history table workflows.
- `docs/reference/org-tables.md`: command behavior and native profile.

## Validation

- `cargo test --test org_tables -- --nocapture`: four workflows pass for source alignment/idempotence, field/row motion, row/separator insertion, creation/cancellation/invalid dimensions and one-operation undo/redo.
- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed.
- `cargo test --test commands command_sequences -- --nocapture`: passed.
- `cargo test --test commands catalog_reference -- --nocapture`: passed.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog leaves, 54 RGB roles and 246 bindings agree.
- Changed-source quality passes at unchanged parameter, nesting and cyclomatic limits.

- `cargo test --test org_tables metadata -- --nocapture`: five workflows pass for affiliated metadata/formulas, ragged/empty cells, alignment cookies, creation boundaries and literal block rejection. Two initial failures exposed width-cookie classification and adjacent table joining; both are corrected and the unchanged assertions pass.

## Risks and Follow-ups

Edge validation remains assigned to T39.3.
Final ordered controller checks remain pending.
