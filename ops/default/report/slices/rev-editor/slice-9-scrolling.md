# Slice 9: Preserve vertical motion and scrolling

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-09-scrolling`
PR: `#9`

## Purpose

Navigate logical lines and pages while preserving the intended display column,
revealing point, and keeping scroll and recenter state within the active view.

## Delivered Behavior

- `C-n` and `C-p` move between logical lines while retaining the intended
  terminal-cell column. Short lines clamp to their ends; a later long line
  restores the goal. Destinations inside tabs or wide graphemes snap to their
  starts without discarding the intended column. Down and Up are aliases.
- `C-v` and `M-v` move point and viewport forward/backward by a page. PageDown
  and PageUp use the same handlers. A page advances by the usable text height
  minus two context rows, with at least one logical line of progress.
- `C-l` cycles point's line through center, top and bottom of the viewport.
  Recenter preserves point and the intended column. Other navigation starts
  the cycle again; nonvertical commands and direct insertion also clear the
  previous goal column.
- Line and page destinations clamp to existing logical rows, preserve Unicode
  and CRLF boundaries, and reveal point horizontally and vertically. Pages and
  recenter are safe no-ops while geometry cannot show an editing pane; logical
  line motion remains available and resize recovery reveals the retained goal.
- Navigation preserves canonical text, revision, mark, newline metadata,
  existing edit history, dirty state and read-only state. Releases are ignored
  and supported key repeats dispatch once.

## Implementation Notes

- T9.1: `60160f6f034d6554dc36315d706f4c221c30f544` adds view-owned goal
  columns, checked navigation, page/recenter behavior, bindings and initial tests.
- T9.2: `bc16af2c0d9744525c638aff98ae336ed56bf43f` expands display and real
  executable coverage. `2c4364566b343db6b51bc5080f27e032dcbe11b3` synchronizes
  the PTY fixture's first resize with an actual event-reader response.
- The session prepares a candidate view, computes checked cell-to-source
  destinations and reveals point before publishing point, goal and scroll
  together. No canonical buffer transaction is needed for navigation.
- Paging shifts the viewport by the actual clamped row distance, preserving
  point's screen row where geometry and document boundaries permit. Recenter
  clamps at buffer start and may display blank rows below the final line.
- The new PTY journey uses actual input/response synchronization before its
  first resize, then checks status and cursor output after each key sequence.
- Source checks pass safe Rust, five parameters, nesting at most three and
  cyclomatic paths at most ten.

## Contracts Added or Changed

- `View` owns the optional goal column and private recenter cycle alongside
  qualified point and logical-row/cell scroll offsets.
- `Scroll` identifies five working commands: `next-line`, `previous-line`,
  `scroll-up-command`, `scroll-down-command` and `recenter-top-bottom`.
- `Command::named`, Emacs bindings and reported arrow/Page aliases reach the
  same session handlers. Binding lookup continues to own repeat admission.
- `Session` resets navigation continuity for nonvertical commands and direct
  insertion, while resize and recenter preserve the intended column.
- `CHANGELOG.md` records usable vertical navigation, paging and recentering.

## Files of Interest

- `src/view.rs`, `src/view/scroll.rs`: per-view goals and viewport navigation.
- `src/command/scroll.rs`, `src/command.rs`: implemented command identities.
- `src/input/bindings.rs`: Emacs and arrow/Page aliases.
- `src/session.rs`, `src/session/motion.rs`: dispatch and candidate publication.
- `tests/display.rs`: eighteen projection and navigation integration tests.
- `scripts/check-pty.py`: actual vertical/page/recenter and resize journey.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; seventy-six tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test display`: passed; eighteen tests, zero skips. Added cases
  cover short-line goal restoration, tabs/wide/emoji destinations, CRLF,
  page/recenter sequences, continuity resets, tiny geometry and resize recovery,
  binding repeats/releases, bounds and read-only dirty-state preservation.
- `python3 scripts/check-pty.py --suite terminal`: passed; eleven cases, zero
  skips. The new journey checks actual Emacs and arrow/Page keys, retained
  columns, recenter cursor positions, one-row paging, resize recovery and dirty
  quit protection. Owned fixtures stop after their checks finish.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
