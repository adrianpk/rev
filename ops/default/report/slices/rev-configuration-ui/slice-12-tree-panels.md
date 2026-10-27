# Slice 12: Compose the tree through panels

Status: reviewing
Delivery set: rev-configuration-ui
Plan: [Configuration and Interaction Delivery](../../../plan/rev-configuration-ui.md)
Tracker: [Configuration and Interaction Tracker](../../../tracker/rev-configuration-ui.md)
Branch: `refactor/rev-config-ui-12-tree-panels`
PR: pending

## Purpose

Compose tree, editing and reserved auxiliary surfaces from common native panel
bounds while retaining their existing semantic and input owners.

## Delivered Behavior

- Tree content, footer, selected row and cursor render inside reserved panel
  cells. A shared Unicode separator connects to nested editing boundaries;
  string-prefix painting and later overlay/cursor shifts are removed.
- Editing panes start at their native workspace origin. Syntax, regions and
  search clip and translate once through panel bounds. Tree focus suppresses
  editing activity while keeping syntax and independent view state intact.
- Supplied workspace compositions reserve full-width auxiliary content and its
  shared separator above the prompt, using only surplus rows after complete
  editing minima. The tree footer ends above that reservation.
- Insufficient editing geometry suspends all editing leaves together while
  retaining available tree/prompt presentation. Narrow hiding repairs focus;
  later resize/reopen retains the tree root/selection and pending command draft.
- Tabbed/wide/combining text clips by cells at pane seams. Hiding, resizing and
  drawing retain canonical text, independent points, history and saved identity.

## Implementation Notes

- T12.1: `1dd17fc3427736749d8533ce807963f66036628f` adds native tree
  composition, derives editing view sizes and overlay placement from it and
  replaces the prefix-based tree presenter. Tree row reveal uses its actual
  content height; status is separate from entry labels.
- T12.2: `bb0ded2f9b942e9b9d81ed5575a86227bda1e050` extends workspace
  composition with bounded auxiliary reservations. `Frame::compose_workspace`
  renders one supplied layout, preserving Tree focus and the full-width prompt.
- T12.3 checks tree plus nested panes, rendered Unicode seams, pending input,
  narrow focus repair, reopen, dirty-state undo and 324 native workspace cases.
- Geometry remains disposable metadata derived from Windows/Layout and Tree.
  No second layout tree, directory scan on draw, retained frame cache or input
  registry is introduced. Native maximal geometry uses compact spans.
- The existing Unreleased presentation note and tree reference now include
  the tree's shared separator behavior.

## Contracts Added or Changed

- `Composition::workspace` derives editing/tree panels, global prompt and
  optional full-width auxiliary content from one native surface. It never
  changes semantic focus or admits only part of an editing split subtree.
- `Composition::editing_size` supplies the existing view calculations from its
  editing viewport; `Frame::compose_in` and `compose_workspace` reuse those
  native bounds rather than recomputing prefix offsets.
- Tree entry labels, footer and focus remain Tree-owned. Panel content bounds
  admit selection/cursor cells, and narrow resize retains the existing explicit
  semantic focus repair. Drawing does not read the filesystem.

## Files of Interest

- `src/panel/composition.rs`: workspace, tree and auxiliary bounds.
- `src/display/tree.rs`, `compose.rs`: native tree/shared frame rendering.
- `src/session.rs`, `session/tree.rs`, `session/search.rs`: common projection.
- `src/tree.rs`: content-height reveal and separate entry/footer presentation.
- `tests/panels/rendering.rs`, `workspace.rs`: native composition and lifecycle.
- `tests/panels/geometry.rs`: complete tree/auxiliary cell partition checks.
- `CHANGELOG.md`, `docs/reference/file-tree.md`: visible separator behavior.

## Validation

Development checks:

- `cargo test --test tree presentation`: passed.
- `cargo test --test panels workspace_tree`: passed.
- `cargo test --test tree overlay_coordinates`: passed.
- `cargo test --test tree geometry_lifecycle`: passed.
- `cargo test --test tree nested_layout`: passed.
- `cargo test --test panels workspace::`: passed; two reservation cases.
- `cargo test --test panels workspace_`: passed; three workspace/grid/lifecycle cases.
- `cargo clippy --lib --test panels --test tree --test windows -- -D warnings`: passed.
- `cargo clippy --test panels -- -D warnings`: passed after final test additions.
- `cargo run --quiet --example source-check -- --source src/panel/composition.rs --source src/display/tree.rs --source src/display/compose.rs --source src/session.rs --source src/session/tree.rs --source src/session/search.rs --source src/tree.rs --source tests/panels/rendering.rs`: passed.
- `cargo run --quiet --example source-check -- --source src/panel/composition.rs --source src/display/tree.rs --source src/session.rs --source src/session/tree.rs --source tests/panels/workspace.rs`: passed.
- `cargo run --quiet --example source-check -- --source tests/panels/rendering.rs --source tests/panels/workspace.rs`: passed after final test additions.

The controller stores final results for the exact clean PR head after its
PR-reference follow-up commit. Commands, in canonical order, with focused
rendering/overlay regressions after the required prefix:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test panels`
- `cargo test --test tree`
- `cargo test --test windows`
- `python3 scripts/check-source.py`
- `cargo test --test display`
- `cargo test --test search`
- `cargo test --test syntax`
- `git diff --check`

## Risks and Follow-ups

Command/path candidate presentation and navigation remain assigned to their
later slices. The running session requests zero auxiliary rows until those
consumers are activated; supplied compositions exercise the reservation now.
