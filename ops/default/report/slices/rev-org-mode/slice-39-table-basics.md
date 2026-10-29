# Slice 39: navigate and align native Org tables

Status: reviewing
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
T39.3 verifies canvas columns, source format, shared positions/marks, read-only
publication, stale prompts, cancellation, limits and ordinary undo/redo. Two owned
terminal journeys exercise default keys and M-x through real editing and saves.

## Implementation Notes

The existing structural scanner identifies real table/row/cell ranges from the
canonical rope snapshot. Layout measures actual glyphs with the buffer's tab
policy and terminal column. Original nonempty cell spans remain retained runs;
only gaps become replacements. One canonical buffer transaction maps shared
positions and provides ordinary undo. No buffer identity is invented for a
rendered fragment. Leading combining marks are measured together with the real
preceding padding space. Field destinations use complete rendered grapheme
boundaries. All analysis runs on the existing single cancellable worker.

## Contracts Added or Changed

- ORG12: original cell source, table motion/alignment and checked row insertion.
- ORG17: working table commands, contextual bindings and typed bounded input.

## Files of Interest

- `src/org/table/`: source observation, layout, cell retention and operation plans.
- `src/session/tables.rs`: source-qualified prompts, context dispatch and publication.
- `src/coordinates/display.rs`: shared native glyph/tab measurement for text fragments.
- `tests/org_tables.rs` and `tests/org_tables/`: 17 real buffer/history workflows.
- `scripts/configuration_ui/org_tables.py`: two owned RGB/NO_COLOR journeys.
- `docs/reference/org-tables.md`: command behavior and native profile.

## Validation

- `cargo test --test org_tables -- --nocapture`: four workflows pass for source alignment/idempotence, field/row motion, row/separator insertion, creation/cancellation/invalid dimensions and one-operation undo/redo.
- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed.
- `cargo test --test commands command_sequences -- --nocapture`: passed.
- `cargo test --test commands catalog_reference -- --nocapture`: passed.
- `python3 scripts/check-config-reference.py`: 155 schema/catalog leaves, 54 RGB roles and 246 bindings agree.
- Changed-source quality passes at unchanged parameter, nesting and cyclomatic limits.

- `cargo test --test org_tables metadata -- --nocapture`: five workflows pass for affiliated metadata/formulas, ragged/empty cells, alignment cookies, creation boundaries and literal block rejection. Two initial failures exposed width-cookie classification and adjacent table joining; both are corrected and the unchanged assertions pass.

- `cargo test --test org_tables -- --nocapture`: 17 workflows passed, zero failed/ignored/skipped. Tabs at widths 4/8, wide/combined/ZWJ/control glyphs, empty/ragged cells, real displayed columns, BOM/mixed CRLF/EOF save bytes, shared point/mark mapping, stale folds, read-only behavior, cancelled/stale requests, separator/column/width/dimension bounds and single-operation history are covered.
- `cargo test --test org_tables --lib org::table -- --nocapture`: one internal canonical-buffer cancellation/deadline/source-byte-limit workflow passed; integration tests were filtered by this internal test selector.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tables`: two owned RGB/NO_COLOR sessions passed, zero skipped. They assert exact canvases, metadata/formula retention, real command bindings, creation, field navigation, row insertion, undo/redo, prompt cancellation, literal rejection, unchanged pre-save bytes, exact BOM/CRLF/EOF save bytes, complete fixture inventory, idle silence and terminal cleanup. Expected cost: two short sessions plus the existing development build.
- Initial coordinate assertions exposed incorrect isolated-combining-mark width and an invalid field destination inside its rendered grapheme. Both production defects are corrected; the original column and boundary assertions pass. One RET fixture expectation used the wrong scalar split location and was corrected. The terminal driver initially expected ordinary typing to replace empty padding; its exact source expectations now model insertion before the following alignment command. Resource limits, fixture inventory, persisted-byte assertions and terminal deadlines remain unchanged.

## Risks and Follow-ups

All three tracker tasks are complete. Final clean-HEAD controller receipts and
canonical PR references remain pending. Formula-changing row insertion rejects
atomically; checked reference rewrites belong to the recorded Slice 40.
