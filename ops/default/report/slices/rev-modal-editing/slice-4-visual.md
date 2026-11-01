# Slice 4: Visual selections and block geometry

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-04-visual`
PR: `#215`

## Purpose

Select and edit Character, Line and Block ranges through the native mark, point,
rectangle geometry, prepared edits and ordinary history.

## Delivered Behavior

- v/V/C-v enter Visual Character, Line and Block. Character includes the complete
  destination grapheme; Line includes complete logical lines. Shape conversion
  retains the shared anchor, repeated entry cancels, and o exchanges active ends.
- The Normal navigation/count/find/search grammar extends Visual endpoints.
  Escape/C-[ and C-g recover to Normal. Native pointer drag replaces the shared
  anchor in the active shape; wrap, folds and resize use existing projection.
- d/c/y applies immediately to all three shapes in one transaction. Copy/delete
  returns Normal; successful change enters Insert. Empty valid changes create
  no fabricated edit, and read-only failure preserves every row and endpoint.
- Linear Visual indentation and inner/around word/paragraph objects reuse native
  owners. Block objects/indentation remain unavailable, matching the finite
  approved reference subset.
- Block edges count whole source graphemes, including tabs and joined/wide
  clusters. Both endpoint columns are included; short rows retain only actual
  text and separators remain intact. Emacs rectangles retain display-cell edges.
- Shared block transfer retains native rows and coordinate intent. Modal p/P
  repeats rows horizontally within the expanded-text limit and pastes through
  one native batch; native yank-rectangle can use the same retained block.

## Implementation Notes

T4.1-T4.3 share the native region and rectangle contracts, committed together as
`1938f6d2697d02b5349b34d8a277905fc334cd08`. T4.4 adds focused cross-owner regressions and
this report. The approved reference interaction profile was inspected read-only;
no reference repository tests or runtime changes were performed.

Visual shape belongs to the buffer region mode. The session retains only a
qualifying source stamp, never a second authoritative range. Another view of the
same unchanged buffer can retain the shared shape. A different buffer, published
revision or deactivated native region retires Visual authority before further
input. Native edit publication reconciles all participating views and restores
their individual points through undo/redo.

The rectangle resolver distinguishes display-cell and logical-grapheme policies.
It captures whole logical glyphs, projects their actual display spans for drawing
and carries that coordinate policy into insertion. One native rectangle allocation
is shared with a bounded kill-ring entry; its linear text view joins rows with LF.
A later linear transfer selects linear paste again. Native yank/yank-pop keep their
linear span policy, while native rectangle retention stays independently usable.
Count expansion keeps the existing 64-MiB bound, checked arithmetic and one native
publication. Existing input batching, layout deadlines and idle work are unchanged.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md) and
[tracker](../../../tracker/rev-modal-editing.md) retain the approved MOD06 and
MOD09-MOD12 scope. The [paired model](../../../spec/editor-modal-editing-model.md)
now explains delivered Visual ownership and coordinate policies. Command/binding
inventories contain 399 canonical commands. The exhaustive sample and defaults
spell out 123 Normal bindings, 109 entries in each linear Visual map and 105 in
Block; Replace remains reserved. Rectangle references and Unreleased describe
these usable selection and transfer behaviors.

## Files of Interest

- `src/buffer/region.rs`, `src/input/modal.rs` and `src/session/visual.rs`: native
  shapes, state transitions and source lifetime.
- `src/coordinates/grapheme_columns.rs`, `src/rectangle.rs` and
  `src/rectangle/`: logical versus display columns, drawing and atomic row edits.
- `src/session/visual/`, `src/session/editing.rs` and `src/kill.rs`: shared
  operators, transfer entries and ordinary history.
- `src/session/pointer/selection.rs`: native pointer anchoring in a live shape.
- `tests/modal/visual.rs` and `visual_safety.rs`: transitions, endpoints, transfer,
  permissions, shared views, source retirement, pointer and projection coverage.

## Validation

Exploratory checks passed during implementation:

- `cargo check --locked --quiet` before the final safety additions.
- `cargo test --locked --test modal visual -- --nocapture`: 15 tests passed.
- `cargo test --locked --test rectangles --quiet`: all 42 existing display-cell
  rectangle regressions passed.
- The existing source-check executable passed for all 31 changed Rust files and
  the modal test root, including explanatory comments and complexity limits.

The pointer fixture initially used a UTF-8 byte offset as a terminal column; it
now measures the actual rendered prefix width. The fold fixture now waits for the
native Overview observation and asserts the body is hidden before extending the
selection. Original inclusive selection and complete hidden-source deletion
assertions remain intact.

Final exact-HEAD evidence is recorded by Looper and Acta after the PR-reference
commit. The focused batch protects native owners and inventories before the
canonical ordered batch; neither executes the integrated delivery-set full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --test configuration --test commands --test editing --test core --test display --test coordinates
cargo run --locked --quiet --example source-check -- --source src/buffer.rs --source src/buffer/region.rs --source src/command.rs --source src/command/editing.rs --source src/command/modal.rs --source src/coordinates.rs --source src/coordinates/grapheme_columns.rs --source src/input.rs --source src/input/grammar.rs --source src/input/grammar/operator.rs --source src/input/keymap/defaults.rs --source src/input/keymap/vim.rs --source src/input/modal.rs --source src/kill.rs --source src/rectangle.rs --source src/rectangle/columns.rs --source src/rectangle/kill.rs --source src/rectangle/yank.rs --source src/session.rs --source src/session/editing.rs --source src/session/editing/transfer.rs --source src/session/modal.rs --source src/session/mode.rs --source src/session/pointer/selection.rs --source src/session/rectangle.rs --source src/session/visual.rs --source src/session/visual/editing.rs --source src/session/visual/objects.rs --source tests/modal.rs --source tests/modal/visual.rs --source tests/modal/visual_safety.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

Canonical ordered batch:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test rectangles --test pointer --test windows --test wrapping --test org_editing
git diff --check
```

## Risks and Follow-ups

The controller retains integrated full-gate ownership. Remaining modal states and
terminal/resource acceptance follow the approved later slices. Existing unrelated
repository-wide source-check failures recorded in Slice 1 remain outside this
slice; all changed Rust files receive the focused source-quality check above.
The in-process checks create no persistent external fixture or service.
