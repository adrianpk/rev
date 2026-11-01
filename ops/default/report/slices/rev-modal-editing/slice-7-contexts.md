# Slice 7: Colon commands and completion ownership

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-07-contexts`
PR: `#218`

## Purpose

Share native command entry, pane operations and completion with modal editing,
while preserving component input, native selections and atomic source ownership.

## Delivered Behavior

- Colon in Normal and every Visual shape opens the same vertical selector as
  M-x: full canonical names, query matching, candidates, completion, history,
  availability, argument prompts and handlers. No Ex parser or alias catalog is
  introduced. Insert, Replace and prompt punctuation remains literal input.
- Cancelling a compatible Visual selector retains its native shape and anchor.
  Changed view, stamped point, source revision, mark or shape retires that
  selection without restoring old offsets. Selector cancellation leaves an
  outer Org source draft active; native save/abort still owns that draft.
- C-w h/j/k/l focuses the nearest overlapping native pane, s/v splits below/right,
  c closes this pane, o retains it and = balances existing divisions. Missing
  neighbors are no-ops. Balancing preserves topology, focus, source, history and
  the startup split ratio for later splits.
- Automatic native or optional LSP suggestions require Insert/Replace. Manual
  completion uses the shared canonical command. Popup keys, colored kinds,
  syntax colors, selected resolution/imports and one-step undo remain native.
  State changes retire queued/list/resolve work; late replies cannot reopen or
  publish it. Insert C-o includes asynchronous completion in its command lifetime.
- Tree, search, prompts, inventories, bookmarks and local Org contexts keep input
  priority. Popup Escape/C-g cancels locally before an editing Escape returns to
  Normal. Tab/C-i, held Return, pointer focus and bracketed paste keep their
  native ownership and do not become modal jumps or automatic suggestions.

## Implementation Notes

T7.1-T7.3 share dispatch and lifetime boundaries and are committed together as
`f5912b8048c057810d58be8b149152e036f4e6f0`. T7.4 supplies representative input/transport regressions,
reference reconciliation and this report.

The command origin is observation evidence, not a second selection. Native mark,
Visual geometry and source remain authoritative while the selector owns input.
Its cancellation is dispatched before outer draft/capture cancellation. Retained
tree, bookmark and agenda prompt handlers precede ordinary editor cancellation,
preserving their focused owner; native draft/provider work remains intact.

Five neutral native window commands add directional focus and balance. Direction
selection requires orthogonal half-open overlap, then ranks directional gap and
center distance with native screen order for ties. Balance keeps overrides only
for live layout forks. It reserves internal separators before sharing pane cells;
recursive native minima and suspended-layout behavior remain authoritative.

Completion captures editing state beside its existing generation. State, native
view/point/revision, syntax/source observations, provider identity and read-only
checks qualify presentation and publication. Native atomic completion updates
input context and retires unsupported repeat ownership. No worker, queue, event
batch or deadline limit changes; modal state introduces no idle poll.

## Contracts Added or Changed

The [plan](../../../plan/rev-modal-editing.md) and
[tracker](../../../tracker/rev-modal-editing.md) retain MOD07-MOD12. The
[paired model](../../../spec/editor-modal-editing-model.md) and targeted R22/R23/R33
[baseline](../../../spec/rev-editor.md) wording record shared colon dispatch,
selection cancellation and completion ownership without rewriting prior evidence.

Both command inventories now contain 409 commands. The complete supported sample
and defaults declare 137 Normal bindings, 77 Insert, 76 Replace, 119 Visual
Character, 119 Visual Line and 115 Visual Block bindings. References and
Unreleased describe the delivered command, pane and completion behavior.

## Files of Interest

- `src/input/keymap/vim.rs`, `src/command/window.rs` and `src/command.rs`:
  colon/pane factories and shared canonical identities.
- `src/window/navigation.rs`, `division.rs`, `geometry.rs` and `layout.rs`:
  native directional targets and separator-aware balanced geometry.
- `src/session/extended.rs`, `prompt.rs` and `windows.rs`: checked Visual command
  origin, selector-local cancellation and ordinary native window handlers.
- `src/session/completion.rs` and `jumps/once.rs`: modal request qualification,
  native publication and complete one-Normal-command continuation.
- `tests/modal/contexts.rs`, `completion.rs`, `context_lsp.rs` and
  `context_safety.rs`: command, pane, draft, popup, transport and input boundaries.
- `src/session/completion/tests.rs`: a state-only stale result at the same native
  view, point, revision and generation must fail admission.

## Validation

Exploratory `cargo check --locked --quiet` passed during implementation. All 15
new context tests passed through
`cargo test --locked --test modal contexts -- --nocapture`.
The existing source-check executable passed all 21 changed Rust files and the
modal root before the final selector fixture and reporting changes.

The three-pane balance assertion exposed missing internal-separator accounting;
the allocation was corrected while retaining equal-size, focus and topology
assertions. The custom Normal editing fixture now uses an unbound q rather than
conflicting with the native z prefix. The selected-import fixture uses opaque
`index` data, matching the existing shell peer's request-ID parser; its import,
state-preservation and atomic-undo assertions remain unchanged. The first
controller-focused batch exposed a retained-tree focus regression in the existing
`workspace::tree_focus` test. Selector cancellation now delegates retained
component prompts to their native handlers before the editor fallback. The
original assertion passed with
`cargo test --locked --test pointer workspace::tree_focus -- --nocapture`; the
modal selector regression retains the same focus assertion.

The canonical command batch then exposed stale yank-pop continuity after
selector C-g in the existing `palette::named_yank` case. Cancellation now breaks
the native kill chain and clears the captured yank before the editor fallback,
while retained component handlers keep their existing behavior. The unchanged
assertions passed with
`cargo test --locked --test commands palette::named_yank -- --nocapture`.

Final exact-HEAD outcomes and receipt identities are recorded by the controller
and Acta after the PR-reference commit. The focused batch covers changed native
owners, configuration, pointer/display/recovery, bookmark/agenda dependencies, source quality and
documentation. It precedes the canonical ordered batch below; neither is the
integrated delivery-set full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --lib
cargo test --locked --test configuration --test windows --test pointer --test display --test recovery --test bookmarks --test org_agenda
cargo run --locked --quiet --example source-check -- --source src/command.rs --source src/command/window.rs --source src/input/keymap/vim.rs --source src/session.rs --source src/session/completion.rs --source src/session/completion/tests.rs --source src/session/extended.rs --source src/session/jumps/once.rs --source src/session/prompt.rs --source src/session/windows.rs --source src/window.rs --source src/window/division.rs --source src/window/geometry.rs --source src/window/layout.rs --source src/window/lifecycle.rs --source src/window/navigation.rs --source tests/modal.rs --source tests/modal/completion.rs --source tests/modal/context_lsp.rs --source tests/modal/context_safety.rs --source tests/modal/contexts.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test completion --test lsp --test core --test commands --test panels --test path_completion --test org_source
git diff --check
```

## Risks and Follow-ups

The owned LSP peer exercises actual framed transport, selected imports,
cancellation and late list/resolve responses; it does not establish installed
rust-analyzer or real-terminal acceptance. Those paths, resource measurements,
User Guide progression and the unchanged integrated full gate remain in Slice 8.
