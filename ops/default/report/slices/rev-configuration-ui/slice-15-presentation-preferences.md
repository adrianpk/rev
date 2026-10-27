# Slice 15: Apply layout tree and syntax preferences

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `feat/rev-config-ui-15-presentation-preferences`
PR: pending

## Purpose

Apply admitted scrolling, layout, tree and syntax preferences through their
existing owners, preserving canonical documents and retained workspace state.

## Delivered Behavior

- Page distance uses global overlap and current pane content height. Usable
  panes advance at least one row; suspended page input retains point.
- Window and native panel geometry use one preferred ratio on both split axes,
  flooring the first child's available share and clamping to complete recursive
  minima. Tiny resizes suspend all editing leaves together without losing views.
- Tree initial visibility retains editing focus. Preferred width clips to the
  maximum frame fraction; the existing narrow-frame hiding/focus repair remains.
  Auto/file-parent root policy uses initial file context or cwd fallback; cwd
  policy stays explicit. Later visits retain the selected startup root.
- Explicit tree observations filter dotfiles and order directories first only
  when requested. Filename comparison remains native and deterministic, and
  symlinks remain leaves. Drawing and visible movement read only loaded state.
- Syntax enabled, UTF-8 byte cutoff and per-language rust/plain grammar control
  parser admission. Equality is admitted; zero means no configured cutoff.
  Crossing the cutoff drops derived styling, and shrinking restarts from current
  text without changing edit admission, marks, revisions or history.
- Visits, renames and published write-file destinations resolve grammar from
  the registry snapshot. Canceled parser revisions retain their retry bound.

## Implementation Notes

- T15.1: `44543334610ee49689442241acf984cfe071eadc` wires overlap, shared
  ratio geometry and tree preferences, with focused layout/listing cases.
- T15.2: `1c8824ab576504d25c8be3f8bd7698406cb9bb93` moves grammar selection
  into snapshot policy application, adds parser cutoff transitions and corrects
  the root conditional flagged by the initial focused Clippy check.
- T15.3 covers root contexts, tiny recursive geometry, pending tree focus after
  observed paths disappear, parser admission with a real progress callback and
  configured write-file grammar changes followed by undo/redo.
- Buffers retain admitted derived parser policy beside their existing editor
  policy. Windows owns its immutable ratio; native composition delegates to the
  same geometry owner. Tree observes children only on explicit commands.
- Startup root resolution can report a working-directory error through the
  existing view/session construction boundary. Root selection itself reads no
  directory entries. No runtime configuration discovery or reload is added.

## Contracts Added or Changed

- `Windows::with_settings` and `Tree::with_settings` accept an admitted snapshot;
  their existing constructors retain compiled defaults.
- Session construction initializes layout/tree preferences from its registry
  and initial view context. Root, focus and pending input outlive redraw/resize.
- `Buffer::apply_settings` also resolves grammar and syntax admission. Syntax
  policy changes affect derived presentation, not canonical state or history.
- Parser/query creation occurs on transitions into admitted Rust presentation;
  disabled/plain/oversized buffers skip parser work, including progress-aware
  refresh requests. A canceled parse is not restarted by ordinary redraw.

## Files of Interest

- `src/window/geometry.rs`, `layout.rs`, `src/window.rs`: shared ratio projection.
- `src/session/motion.rs`, `src/view/scroll.rs`: bounded page distance/recenter.
- `src/tree.rs`, `tree/entry.rs`, `navigation.rs`, `refresh.rs`: tree policy.
- `src/buffer/policy.rs`, `syntax.rs`, `save.rs`, `visits.rs`: grammar lifecycle.
- `tests/windows/preferences.rs`, `tests/tree/preferences.rs`, `tests/syntax/preferences.rs`: focused preference boundaries.
- `docs/reference/bindings.md`, `file-tree.md`, `syntax.md`: supplied preferences.

## Validation

Development checks:

- `cargo test --test windows preferences::`: passed; ratio, overlap and recursive minima.
- `cargo test --test tree preferences::`: passed; listing, visibility, roots and memory-only draw.
- `cargo test --test syntax current_revision`: passed.
- `cargo test --test syntax preferences::`: passed; grammar, cutoff, policy transitions and parser admission.
- `cargo test --test syntax saved_grammar`: passed; published destinations and undo/saved-state truth.
- `cargo test --test tree listing_policy`: passed after adding refresh coverage.
- `cargo clippy --lib --test windows --test tree -- -D warnings`: passed after correcting the root conditional.
- `cargo clippy --lib --test syntax -- -D warnings`: passed.
- `cargo clippy --lib --test windows --test tree --test syntax -- -D warnings`: passed.
- `cargo run --quiet --example source-check -- --source src/tree/entry.rs --source tests/windows/preferences.rs --source tests/tree/preferences.rs`: passed after separating the ordering key to retain the complexity bound.
- `cargo run --quiet --example source-check -- --source src/buffer.rs --source src/buffer/syntax.rs --source src/buffer/policy.rs --source tests/syntax/preferences.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/tree/preferences.rs --source tests/windows/preferences.rs --source tests/syntax/preferences.rs`: passed.

The controller records final evidence on the clean PR head after its
PR-reference follow-up commit. Required commands precede focused shared
rendering, scrolling and publication regressions:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test windows`
- `cargo test --test tree`
- `cargo test --test syntax`
- `python3 scripts/check-source.py`
- `cargo test --test panels`
- `cargo test --test display`
- `cargo test --test recovery`
- `git diff --check`

## Risks and Follow-ups

Executable startup configuration loading remains assigned to Slice 20; supplied
library snapshots exercise these policies now. Tree observations retain the
existing direct-child/expanded-branch scope and native filename comparison;
no natural sorting, recursive discovery or new grammar is introduced.
