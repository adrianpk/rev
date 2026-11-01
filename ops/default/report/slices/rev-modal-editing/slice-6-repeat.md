# Slice 6: Replace state and semantic repeat

Status: delivered
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-06-repeat`
PR: `#217`
Merged commit: `c8508ea30b60318764a316f633a78ebe569329c2`
Canonical validation: `FYXNQG57C3DL52F2GJUEF3B55I`, three configured checks and 242 tests passed
Focused validation: `MFKEIWGHQT45BVQXOKIJZRI62P`, Clippy, 548 tests, 32 changed Rust files and documentation passed

## Purpose

Overwrite whole graphemes in Replace and repeat a bounded semantic change at a
fresh native editing target, preserving ordinary undo and input ownership.

## Delivered Behavior

- R enters persistent Replace with an underline cursor and compatible native
  modified controls. Text replaces complete graphemes on the current logical
  line, inserts at EOL and preserves LF/CRLF. Joining input extends the preceding
  cluster. Escape/C-[ returns to Normal at the checked native point.
- Connected Insert/Replace input, a preceding change or a logical line opening
  forms one native undo/redo group. Native source, view, anchor and permission
  checks still govern each publication. Empty insertion entry preserves the
  previous completed repeat record.
- Dot resolves an entry, operator/object, direct edit, indentation or retained
  paste against current source. An explicit count replaces the recorded count.
  It preserves linear, linewise and logical-block intent, stays Normal and
  publishes one native history transaction. Failure preserves the record.
- A completed record can be used in another compatible pane or buffer. A paste
  retains its admitted row/text payload even after another shared copy. Visual
  repetition retains grapheme/line counts or block height and width; a Visual
  count override selects the linear/line count or block width.
- Retained text is limited to 1,024 bytes. Oversized or unsupported changes keep
  their complete source and ordinary undo while becoming unavailable to dot.
  Unfinished entries cannot inherit another view or external source revision.
  Prompt text, consent and provider/asynchronous workflows are never replayed.

## Implementation Notes

T6.1-T6.3 share source publication, history and repeat admission, committed together
as `0a8c9fec6f277371afcce1b584eb773fa4a7ee03`. T6.4 adds component, transfer and deferred-input safety regressions
and this report. The approved reference's replacement/repeat implementation and
reference text were inspected read-only; no reference tests or changes ran.

The input owner adds a typed Replace state and an independent factory map. Shared
text admission counts complete incoming graphemes and accounts for input joining
the preceding cluster. EOL clamps source removal before the existing separator.
Normal and Visual cursor presentation and native Emacs entry remain unchanged.

Native history continuation composes reversible fragments rather than storing a
second document or raw event log. It checks the first transaction stamp and exact
before/after point, mark and anchor observations. A Visual block's empty top row
can insert before its first removed row within the same owned history group.
Ordinary payload accounting, whole-group pruning and canonical initiating-point
restoration remain authoritative, including multi-pane undo/redo.

The repeat owner keeps one completed semantic record and an observed entry
candidate. First admitted input retires the previous completed record. The entry
candidate reads its bounded payload from native grouped history, separating a
native line-opening/change separator from inserted user text. Overflow retires
its whole candidate; it never retains or publishes a partial prefix. Completed
records contain operation descriptors and bounded text, with no authoritative
old source offset. Undo/redo preserves the completed semantic record.

A fresh repeat checks focus, point, read-only state and current target before
native publication. Its preflight uses the existing deferred-layout owner for
that retained intent's new target. Count expansion retains the existing 64-MiB
materialization bound and uses doubling rather than a count-sized command loop.
No input batch, queue, deadline or worker limit changed, and no idle poll was added.

Destructive entry controls outside the representable insertion, source-bound
accepted-search ranges and unrelated native editing workflows retire repetition.
Native controls and unsupported workflows still execute through their ordinary
owners; a later dot cannot replay their prompts or a stale earlier edit.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md) and
[tracker](../../../tracker/rev-modal-editing.md) retain MOD03, MOD05, MOD12 and MOD14.
The [paired model](../../../spec/editor-modal-editing-model.md) records delivered
Replace/repeat and native grouping ownership. The canonical command inventories
contain 404 commands. The exhaustive sample/defaults declare 127 Normal bindings,
77 Insert bindings and 76 Replace bindings, with existing Visual maps preserved.
References and Unreleased describe replacement, bounded repeat and ordinary undo.

## Files of Interest

- `src/input/modal.rs`, `keymap/vim.rs`, `src/command/modal.rs` and
  `src/command/editing.rs`: Replace, its factories and counted dot requests.
- `src/session/repeat/`: observed entry lifetime, finite payload capture, fresh
  targets, Visual geometry, transfer shape and deferred preflight.
- `src/session/edit.rs`, `paste.rs` and `editing/`: native text admission,
  ordinary edit requests and one-transaction semantic replay.
- `src/buffer/edit/continuation.rs`, `grouping.rs` and `history.rs`: checked
  reversible-fragment composition, source anchors and canonical history.
- `src/rectangle.rs`: native logical-block geometry for fresh repeat targets.
- `tests/modal/repeat.rs` and `repeat_safety.rs`: Unicode/EOL, counts, target
  resolution, limits, permissions, prompts, shared views and deferred FIFO input.

## Validation

Exploratory checks passed during implementation:

- `cargo check --locked --quiet` before the final safety and preflight additions.
- `cargo test --locked --test modal repeat -- --nocapture`: all 18 replacement
  and repetition tests passed after those additions.
- The existing source-check executable passed all changed Rust files and the
  modal test root before the final safety fixture additions; final checks below
  cover the complete changed set of 32 Rust files.

The short block fixture now expects both inclusive columns to be removed. The
read-only fixture uses an explicitly read-only canonical buffer, since disk mode
bits alone do not establish the editor's native read-only state. The deferred
fixture uses a counted viewport target on the existing 4,000-line source, 12x10
view and unchanged five-second settlement deadline. A logical-line target
completed synchronously and could not exercise the original no-early-edit/FIFO
assertions; those assertions remain intact against the deferred viewport target.

Final exact-HEAD evidence is recorded by Looper and Acta after the PR-reference
commit. The focused batch checks native owners and inventories before the
canonical ordered batch. Neither runs the integrated delivery-set full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --lib
cargo test --locked --test configuration --test commands --test core --test display --test rectangles --test windows
cargo run --locked --quiet --example source-check -- --source src/buffer/edit.rs --source src/buffer/edit/continuation.rs --source src/buffer/edit/grouping.rs --source src/buffer/edit/history.rs --source src/command.rs --source src/command/editing.rs --source src/command/modal.rs --source src/input.rs --source src/input/grammar.rs --source src/input/keymap/defaults.rs --source src/input/keymap/vim.rs --source src/input/modal.rs --source src/input/org.rs --source src/rectangle.rs --source src/session.rs --source src/session/edit.rs --source src/session/editing.rs --source src/session/editing/direct.rs --source src/session/editing/range.rs --source src/session/editing/transfer.rs --source src/session/modal.rs --source src/session/mode.rs --source src/session/paste.rs --source src/session/repeat.rs --source src/session/repeat/capture.rs --source src/session/repeat/preflight.rs --source src/session/repeat/replay.rs --source src/session/repeat/selection.rs --source src/session/repeat/text.rs --source tests/modal.rs --source tests/modal/repeat.rs --source tests/modal/repeat_safety.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

Canonical ordered batch:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test editing --test recovery --test coordinates
git diff --check
```

## Risks and Follow-ups

The controller owns the integrated full gate. Completion/colon ownership and
terminal/resource acceptance remain with their recorded slices. Existing
unrelated repository-wide source-check failures remain outside this slice;
every changed Rust file receives focused source-quality validation. These
in-process checks create no persistent external fixture or service.
