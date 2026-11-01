# Slice 2: Logical navigation, counts and search

Status: reviewing
Delivery set: rev-modal-editing
Plan: `ops/default/plan/rev-modal-editing.md`
Tracker: `ops/default/tracker/rev-modal-editing.md`
Branch: `feat/rev-modal-editing-02-navigation`
PR: `#213`

## Purpose

Resolve ordinary Normal navigation and bounded counts through the existing
coordinate, view, deferred-input and literal-search owners.

## Delivered Behavior

- Grapheme h/l and arrows, logical j/k with desired display column and native
  folds, 0/^/$, word/WORD starts and ends, ge/gE, gg/G and counted line targets.
- Logical-line Unicode f/F/t/T, counted atomic find, same/opposite repetition,
  paragraph boundaries and nested matching parentheses/brackets/braces.
- Full/half-page controls and PageUp/PageDown, H/M/L and zt/zz/zb using native
  wrap/fold/gutter geometry. Large counts remain one semantic command.
- Shared literal smart-case, nonwrapping / and ?, n/N and whole-word */#.
  Confirmed native searches supply continuation; C-g restores the origin view.
- Checked u32 decimal counts, 16 strokes and one scalar character argument
  (at most four UTF-8 bytes within the approved 96-byte bound). Pending syntax
  observes native view, buffer revision and point. Protected recovery, local
  component precedence and explicitly configured leading digits remain usable.

## Implementation Notes

T2.1-T2.3 share one typed request and exhaustively registered handler, committed
together with their target/workflow regressions. T2.4 separately adds boundary,
full-sample, ownership and substantial-rope checks. The command and binding
inventories contain 380 canonical commands; the full sample spells out 104
Normal entries. Emacs factories and Insert bindings retain their native policy.

The shared Odra subset was inspected in the approved reference's navigation,
display and native command implementation/tests; no Odra test was executed.
w starts a new word/punctuation class run, b reaches the native Unicode word
start and e reaches its exclusive end boundary. h/l preserve whole graphemes,
including LF/CRLF; gg/G preserve BOF/EOF. WORD/ge/count forms remain explicit
rev requirements. The binding reference pins every delivered endpoint.

Native b/e and the existing Emacs word commands now share the counted rope
scanner. Find and word traversal consume finite source observations, never a
count-sized command queue. Search extends the canonical KMP matcher with nth
occurrence/whole-word policy and retains confirmed state in the search owner.
Forward search lands after the match; backward search lands at its start.

The maximum-page regression initially failed to settle: repeating preflight
restarted the source traversal, and painting its origin discarded offscreen
work. Native view layout now retains one qualified continuation over two
geometry-only fragments and publishes only its complete destination. Frame
cleanup preserves work owned by the deferred request. Existing 4,096-glyph
steps, 8,192-fragment/1-MiB cache caps and the 256-event FIFO are unchanged.
The regression retains 4,000 wrapped lines, a 12-by-10 pane and u32-max count;
point remains at zero before completion, a queued k runs afterward to logical
row 3,999, and cancellation permits the next l without late page publication.

## Contracts Added or Changed

[MOD04/MOD05/MOD08/MOD09](../../../spec/editor-modal-editing.md) and the paired
[model](../../../spec/editor-modal-editing-model.md) now pin source endpoints,
count/focus lifetime and shared search continuation. Local Org prefix/subtree
ownership shadows an inherited Normal leaf (including C-u), with conflicts
inside each local map still rejecting configuration. The supported sample,
binding/command references, defaults and Unreleased describe delivered behavior.

## Files of Interest

- `src/command/navigation.rs`, `src/coordinates/navigation/`: reusable target
  descriptors and source-bounded coordinate resolution.
- `src/input/grammar.rs`, `src/input/keymap/`: bounded composition and native
  profile/local map resolution.
- `src/session/navigation/`, `src/session/search/`, `src/search/occurrence.rs`:
  checked dispatch, find observation and canonical search continuation.
- `src/view/navigation.rs`, `src/view/visual/scroll.rs`, `src/view/wrap/cache.rs`:
  projected viewport targets and bounded disposable layout progress.
- `tests/modal/`: endpoint, count, search, fold/wrap, ownership and cancellation
  coverage using the existing owned configuration fixture.

## Validation

Exploratory focused checks passed during implementation:

- `cargo check --locked --quiet`.
- `cargo test --locked --test modal --test coordinates --test search --quiet`:
  75 tests passed before the final boundary additions.
- `cargo test --locked --test modal viewport:: -- --nocapture`: four viewport
  regressions passed in 2.82 seconds after the deferred-layout correction.
- `cargo test --locked --test modal boundaries:: -- --nocapture`: five boundary
  checks passed in 1.09 seconds, including u32-max traversal over 4,000 lines.

Final exact-HEAD evidence is recorded by Looper and Acta after the PR-reference
commit. The focused batch protects changed native owners and catalogs before
the canonical ordered batch. It does not execute the delivery-set full gate.

```sh
cargo clippy --locked --lib -- -D warnings
cargo test --locked --lib view::wrap
cargo test --locked --test configuration --test commands --test display --test windows
cargo run --locked --quiet --example source-check -- --source src/command.rs --source src/command/navigation.rs --source src/coordinates.rs --source src/coordinates/motion.rs --source src/coordinates/navigation.rs --source src/coordinates/navigation/find.rs --source src/coordinates/navigation/runs.rs --source src/coordinates/navigation/structure.rs --source src/input.rs --source src/input/dispatch.rs --source src/input/grammar.rs --source src/input/keymap.rs --source src/input/keymap/map.rs --source src/input/keymap/vim.rs --source src/input/keymap/vim_navigation.rs --source src/search.rs --source src/search/occurrence.rs --source src/session.rs --source src/session/layout.rs --source src/session/mode.rs --source src/session/navigation.rs --source src/session/navigation/find.rs --source src/session/navigation/viewport.rs --source src/session/search.rs --source src/session/search/continuation.rs --source src/view.rs --source src/view/navigation.rs --source src/view/visual.rs --source src/view/visual/scroll.rs --source src/view/wrap/cache.rs --source tests/modal.rs --source tests/modal/boundaries.rs --source tests/modal/local_targets.rs --source tests/modal/navigation.rs --source tests/modal/search.rs --source tests/modal/viewport.rs --root tests/modal.rs
cargo run --locked --quiet --example documentation-check
git diff --check
```

Canonical ordered batch:

```sh
cargo fmt --all -- --check
cargo test --locked --test modal --test coordinates --test search --test wrapping --test gutter
git diff --check
```

## Risks and Follow-ups

The frozen delivery-set gate and real-terminal acceptance remain owned by the
controller and later approved slices. Existing unrelated repository-wide source
check failures recorded in Slice 1 remain outside this slice; all changed Rust
files receive the focused source-quality check above. No owned service or
external fixture remains running after these in-process checks.
