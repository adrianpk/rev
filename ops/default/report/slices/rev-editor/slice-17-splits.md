# Slice 17: Create nested editing splits

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-17-splits`
PR: `#17`
Integrated: `db4f7813fdf3ee905b4447a083a6d2ff3901dfbe`

## Purpose

Create nested editing panes with independent view state, one canonical buffer
and validated geometry that preserves existing layout when a split cannot fit.

## Delivered Behavior

- `C-x 2` / `split-window-below` create upper/lower panes; `C-x 3` /
  `split-window-right` create left/right panes. Both directions nest in either
  order and preserve the original selected view and input focus.
- Each split allocates a distinct stable view identity and copies the original
  buffer reference, point, intended column and scroll state. Subsequent movement
  and scrolling belong to each selected pane and use its assigned geometry.
- Each pane has its own clipped document and status row; right divisions have
  a separator and the session retains one global echo. Only the selected pane
  supplies the status marker and translated hardware cursor. Unicode, tabs,
  combining sequences and wide glyphs respect pane boundaries.
- Admission accounts for three text columns, one text row plus status, right
  separators and the global echo before changing the tree. A rejected split
  preserves layout, view states and selected identity and displays an error.
- When resized geometry cannot fit the tree, presentation suspends safely while
  retaining all leaves and buffers. Restoring usable dimensions renders them again.
- Existing edits and undo/redo publish qualified points for all views of the
  selected canonical buffer. Buffer closure repairs every displayed reference
  while retaining layout and view identities.

## Implementation Notes

- T17.1: `d8aa6ff34885e2499e81ab5faa57c544cff9b68b` introduces view registry,
  split layout, geometry admission, pane composition and common command bindings.
- T17.2: `e6a2e82e6566d590564de3d3ac6028160f953028` adds native and real-terminal
  evidence for independent navigation, geometry, identity and publication behavior.
- `Windows` owns one identity-to-view registry, a selected identity and a small
  recursive below/right tree. Views contain buffer references rather than cloned
  editable text. A candidate tree is published only after complete split admission.
- Every admitted division halves its available axis within u16 terminal geometry;
  minimum pane dimensions bound nesting depth below 32. No background worker,
  docking engine or new dependency is introduced.
- Geometry owns pane/status rectangles and right dividers. Composition clips in
  display-cell space and derives visible output afresh; mixed rows are explicitly
  distinguished from uniform document/status rows.
- Native view selection validates live identity and buffer-qualified point, ends
  typing continuity and respects prompt focus. Split commands retain original
  focus and do not reset its navigation state.
- Transaction preparation/publication receives the live selected-buffer point set,
  including the correct initiating index. Other points use the existing mapper;
  viewport row origins clamp to live line bounds after publication.

## Contracts Added or Changed

- `ViewId` is a never-reused process-local identity distinct from `BufferId`.
  `Windows` exposes borrowed views, immutable layout and derived geometry.
- `Windows::split` checks all existing geometry and proposed children before
  allocation/publication; `Layout` leaves name exactly one live registry view.
- `Session::select_view` provides checked native selection and `windows` exposes
  read-only state. Both split commands join the common command registry and keys.
- `Frame::compose` combines pane projections, dividers, statuses, global echo and
  one selected cursor within the assigned terminal surface.
- Session mutation/history and buffer lifecycle orchestration retain valid view
  references now that several leaves can display one buffer.
- `CHANGELOG.md` describes the usable split directions, nesting and fit refusal.

## Files of Interest

- `src/window.rs`, `src/window/`: live view identities, layout, geometry and point sets.
- `src/display/compose.rs`: cell-bounded pane composition and cursor translation.
- `src/session/windows.rs`, `src/session.rs`: split commands, selection and rendering.
- `src/session/edit.rs`, `src/session/history.rs`: canonical publication of live points.
- `src/session/buffers.rs`, `src/session/file.rs`: pane-local visits and reference repair.
- `src/command/split.rs`, `src/input/bindings.rs`: canonical split names and key sequences.
- `tests/windows.rs`, `scripts/check-pty.py`: native invariants and actual executable journeys.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 169 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test windows`: passed; ten tests, zero skips. Cases cover both
  nested orders, minimum/exact/rejected geometry, distinct identities, copied
  initial state, independent point/scroll, Unicode seams, selected cursor/status,
  shared edit/undo, replaced references, suspended layout, nesting bounds and focus admission.
- `python3 scripts/check-pty.py --suite terminal`: passed; thirty cases,
  zero skips. New journeys exercise both nested orders through real split keys,
  Unicode pane output, retained focus, independent sibling status, shared editing
  and undo, and a refused non-fitting split. Owned fixtures stop after their
  checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
