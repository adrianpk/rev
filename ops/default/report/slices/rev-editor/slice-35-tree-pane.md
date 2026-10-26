# Slice 35: Render and focus the left file tree

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-35-tree-pane`
PR: `#35`
Integrated: Rebase + Fast-forward into dev at `242b1f8c155d6a8af9d750075a50dac87eb2180b`.

## Purpose

Present a narrow left file-tree pane with a selected collapsed root, explicit
workspace focus and reliable return to the retained editing window.

## Delivered Behavior

- C-x t t and M-x toggle-file-tree show or hide the pane. C-x t f and
  M-x focus-file-tree show and focus it. The root row has a selection marker
  and reverse video; the footer identifies tree focus.
- The initial file's resolved parent supplies the startup root. Without an
  initial file, first opening selects the current directory. Hiding, resizing
  and later file visits preserve the root. Root presentation needs no Git
  metadata and performs no directory traversal.
- Tree focus rejects document editing and paste. Tab or C-g returns to the
  retained editing window without replacing its buffer, point, mark or history.
  Guarded exit remains available. Other-window cycles only editing leaves.
- M-x uses the full-width echo prompt while the tree retains workspace focus.
  Canceling or entering an unknown command returns to the tree. Existing editor
  argument prompts retain input ownership and reject tree toggles.
- The pane uses at most 24 columns and one third of the frame, plus a divider.
  Nested editing panes use the remaining area. Selection, search and cursor
  coordinates shift with the pane; the echo row keeps the full terminal width.
- Below 32 columns or without usable text/status geometry, the pane auto-hides
  and releases focus. Restoring geometry preserves the show preference and root
  while retaining editor focus. Explicit hiding restores full editing width.
- Unicode and control characters in root labels use the existing safe drawing
  and clipping policy. Showing and hiding preserves dirty text and undo state.

## Implementation Notes

- T35.1: `7662b67f95c256a78986ed3a7a923ea587e9da7f` adds the workspace
  tree owner, collapsed-root presentation, bound/named commands, focus admission,
  editing-area allocation, three native cases and product reference. All seven
  ordered checks passed, including the 70 existing terminal cases.
- T35.2: `929c8c881156181e20bd6a06c7fd965d7989259c` adds seven native
  lifecycle cases and two real PTY journeys. All seven ordered checks passed,
  including 365 native tests, ten focused tree tests and 72 terminal cases.
- The tree remains separate from the editing window registry. The selected
  editing view supplies the return target; transient command input retains its
  own owner. Successful file ingress returns focus to editing without moving
  the retained tree root.
- Frame composition reserves tree width before laying out editing leaves,
  shifts editing overlays once, and composes prompts in the global echo row.
  Auto-hide clears only workspace focus while retaining visibility preference.
- Native cases exercise full u16 skinny extents, resize restoration, prompt
  refusal/cancel, command admission, retained root, nested splits, selection
  and search offsets, and repeat/release isolation.
- PTY journeys use bounded complete-frame waits and owned temporary fixtures.
  They verify startup root selection in a plain directory, dirty-text refusal,
  named commands, exact undo, Unicode/control labels, narrow/zero geometry,
  restored editor focus and terminal-attribute restoration on exit.

## Contracts Added or Changed

- The command catalog exposes toggle-file-tree and focus-file-tree with the
  nested C-x t t and C-x t f bindings; named and bound routes share handlers.
- Workspace tree focus surrounds transient M-x focus. It blocks document
  commands and paste while preserving the last selected editing view.
- The left allocation precedes editing layout and overlay projection; the
  global echo/prompt row retains the full terminal width.
- Auto-hide releases focus while retaining root/show preference. Widening
  restores presentation with editor focus rather than reclaiming input.
- The file-tree reference, command catalog and changelog document the delivered
  presentation, focus and geometry behavior.

## Files of Interest

- `src/tree.rs`: retained root, visibility, focus and width policy.
- `src/session/tree.rs`: workspace admission and editing-area allocation.
- `src/display/tree.rs`: safe root/footer drawing and overlay projection.
- `src/command/tree.rs`, `src/input/tree.rs`: named and bound command routes.
- `src/session/windows.rs`, `src/session/search.rs`: editing geometry integration.
- `tests/tree.rs`: ten native tree presentation and lifecycle cases.
- `scripts/check-pty.py`: root/focus, dirty-text and resize/control-label journeys.
- `docs/reference/file-tree.md`, `docs/reference/commands.md`, `CHANGELOG.md`:
  reader-facing tree presentation and focus contracts.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 365 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test tree`: passed; ten tests, zero skips. Cases cover root
  presentation, named/bound commands, dirty buffers, editing-only cycling,
  prompt ownership, native geometry, overlays, retained root and nested splits.
- `python3 scripts/check-pty.py --suite terminal`: passed; 72 cases, zero skips.
  New journeys verify root selection, focus admission, dirty-text preservation,
  exact undo, Unicode/control drawing, resize restoration and terminal cleanup.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
