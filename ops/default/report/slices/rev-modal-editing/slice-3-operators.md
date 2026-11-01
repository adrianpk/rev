# Slice 3: Operators and shared text transfer

Status: delivered
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-03-operators`
PR: `#214`
Merged commit: `e7fdb956486a1a347ed9b66f6f604471b9b3f746`
Canonical validation: `CZHCWJ4BP6JJ76FP4Q6PZEJXMG`, three configured checks and 210 tests passed
Focused validation: `4QS6SUSRZSF5KQNCBZQOMKW2EJ`, Clippy, 416 tests, 31 changed Rust files and documentation passed

## Purpose

Compose Normal editing operators with checked native targets and preserve one
atomic edit, shared text transfer and ordinary multi-view undo.

## Delivered Behavior

- d/c/y and </> compose with motions, multiplied checked counts, doubled logical
  lines and finite inner/around word, WORD, paragraph, pair and quote objects.
  Closing pair spellings are aliases; nesting and escaped delimiters are handled.
- dd/cc/yy, D/C/Y, grapheme x/X and r<char>, p/P, J and indentation publish through
  native prepared edits. Change enters Insert only after successful admission;
  an empty valid change enters Insert without fabricating history.
- Pending operators leave point/text untouched, including / and ? previews.
  Missing, stale, read-only and overflow targets preserve text, history and prior
  transfer. One 2d3w removes six words and one undo restores participating views.
- Linear/linewise intent belongs to the existing bounded kill ring. Native
  kill/yank/yank-pop and modal commands share entries and compatible coalescence.
  Linewise paste respects LF/CRLF, counts and unterminated EOF.

## Implementation Notes

T3.1-T3.3 share the typed editing request, target resolvers and native edit/transfer
publication, committed together as `ae9c695b33d1c24fd5626eb400b7f6ba3ef734f7`.
T3.4 adds cross-owner safety regressions and corrects prefix feedback so it does
not end a compatible kill chain before the semantic command completes. A failed
operator still ends that chain. The pure search-target extraction also preserves
native confirmed word queries after an initial unsuccessful search.

The approved reference implementation and tests were inspected read-only.
Operators preserve its half-open motion endpoints: de retains the separator;
dfX leaves X at the destination; dtX leaves the preceding grapheme too. Native
e already resolves an exclusive word end. Y copies through logical line-end
with linear intent; yy copies complete logical lines. Small x/X preserve the
previous transfer. The binding reference pins these endpoint choices explicitly.

Objects traverse rope-backed source observations rather than a parser. Pair
counts consume actual enclosing pairs, and quote counts remain line-local.
Checked multiplication and the existing 16-stroke/u32 grammar reject overflow.
Count-expanded paste/replacement text has a documented 64-MiB allocation bound;
a single ordinary transfer retains native admission. No count-sized command
queue, modal register store, worker or idle poll is introduced.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md),
[tracker](../../../tracker/rev-modal-editing.md) and
[paired model](../../../spec/editor-modal-editing-model.md) retain native edit,
history, source and input ownership. This slice implements the operator/transfer
portion of MOD05/MOD06/MOD12. The canonical catalog now has 394 commands, and the
complete supported sample spells out 120 Normal bindings. Binding/command/default
references and Unreleased describe the delivered commands and their limits.

## Files of Interest

- `src/command/editing.rs`, `src/input/grammar/operator.rs` and
  `src/input/keymap/vim_editing.rs`: typed intent and bounded composition.
- `src/coordinates/navigation/objects/`: source-backed finite text objects.
- `src/session/editing/`, `src/session/edit.rs`, `src/kill.rs` and
  `src/session/yank.rs`: shared prepared edits, transfer shapes and history.
- `src/session/search.rs` and `src/session/search/continuation.rs`: checked
  operator search without preview point movement.
- `tests/modal/operators.rs`, `objects.rs`, `transfer.rs` and
  `operator_safety.rs`: endpoints, bounds, Unicode, ownership and restoration.

## Validation

Exploratory checks passed during implementation:

- `cargo check --locked --quiet`.
- `cargo test --locked --test modal --test editing --quiet`: 38 modal and 96
  editing tests passed before the final safety additions.
- `cargo test --locked --test modal operator_safety:: -- --nocapture`: nine
  safety tests passed, including 2,000 nested pairs and rejected u32-max paste.
- `cargo test --locked --test modal transfer::direct_edits -- --exact`: passed,
  including a counted join across a blank line.
- The existing source-check executable passed for all 31 changed Rust files and
  the modal test root, including explanatory test comments and complexity bounds.

Final exact-HEAD evidence is recorded by Looper and Acta after the PR-reference
commit. The focused batch protects changed native owners and catalogs before
the canonical ordered batch; it does not execute the delivery-set full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --test configuration --test commands --test search --test core --test windows --test display
cargo run --locked --quiet --example source-check -- --source src/command.rs --source src/command/editing.rs --source src/coordinates/navigation.rs --source src/coordinates/navigation/objects.rs --source src/coordinates/navigation/objects/pairs.rs --source src/coordinates/navigation/objects/paragraphs.rs --source src/coordinates/navigation/objects/words.rs --source src/input/dispatch.rs --source src/input/grammar.rs --source src/input/grammar/operator.rs --source src/input/keymap.rs --source src/input/keymap/vim.rs --source src/input/keymap/vim_editing.rs --source src/kill.rs --source src/session.rs --source src/session/edit.rs --source src/session/editing.rs --source src/session/editing/direct.rs --source src/session/editing/lines.rs --source src/session/editing/range.rs --source src/session/editing/transfer.rs --source src/session/layout.rs --source src/session/navigation/find.rs --source src/session/search.rs --source src/session/search/continuation.rs --source src/session/yank.rs --source tests/modal.rs --source tests/modal/objects.rs --source tests/modal/operator_safety.rs --source tests/modal/operators.rs --source tests/modal/transfer.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

Canonical ordered batch:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test editing --test rectangles --test recovery
git diff --check
```

## Risks and Follow-ups

The controller retains integrated full-gate ownership; terminal acceptance and
remaining modal states follow the approved later slices. Existing unrelated
repository-wide source-check failures recorded in Slice 1 remain outside this
slice; all changed Rust files receive the focused source-quality check above.
The in-process checks create no persistent external fixture or service.
