# Slice 20: Handle tiny and changing terminal geometry

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-20-resize`
PR: `#20`
Integrated: Rebase + Fast-forward into dev at `95cca8e1ffe5c510cf030ae2d415d21898d5bae0`.

## Purpose

Fit nested editing branches down to their complete minimum dimensions, safely
suspend undersized presentation and restore valid independent views after resize.

## Delivered Behavior

- Layout recomputes each branch's recursive minimum before placing its children.
  It prefers equal halves while reserving enough space for both complete
  subtrees, including right dividers, pane status rows and the global echo row.
- Three same-direction nested panes remain usable at 11 columns by 3 rows for
  right splits or 3 columns by 7 rows for below splits, regardless of which
  branch contains the nested split. Each pane retains three text columns and
  at least one text row plus its own status.
- An insufficient report suspends the editing projection without deleting or
  partially replacing the logical layout. Zero width, zero height and tiny
  dimensions retain view identities, valid points, independent navigation state,
  canonical text, mark and undo/redo history.
- Restoring usable dimensions derives fresh rectangles, reveals each point in
  its own pane and repaints bounded rows and the selected cursor. Rejected
  splits preserve the existing tree and show a clipped echo message safely.
- Active argument, inventory and confirmation input remain owned through tiny
  or suspended surfaces. A typed file argument survives zero and one-cell
  geometry and still selects its destination after expansion.

## Implementation Notes

- T20.1: `18647c936a7172319c5c96b2c7af598cf8cb2a44` adds recursive minimum
  allocation and native minimum-size/transient-surface evidence.
- T20.2: `555985b025019931a37c6d5903d5f07ce1f3ec0c` adds geometry grids,
  native dimension endpoints, repeated resize/restore and real PTY evidence.
- A leaf requires a 3-by-2 pane extent. Below branches add child heights and
  retain the larger width; right branches add child widths plus one divider
  and retain the larger height. Recursive additions are checked.
- Allocation subtracts the second child's minimum before clamping the preferred
  half to the valid interval. Cross-axis admission and checked offsets complete
  before either child rectangle is placed. Failure discards the derived geometry.
- Layout remains the structural owner. Resize changes presentation dimensions,
  then ordinary frame composition derives geometry before drawing; it never
  replaces views or publishes a text/history mutation.
- Native tests cover 1,584 small size/layout combinations, non-overlapping
  complete pane coverage, maximum individual axes and repeated suspended frames.
  A maximum two-axis report checks rectangle arithmetic without allocating a
  full raster. Undo/redo are exercised after restoration, not only counted.
- PTY fixtures verify every emitted cursor address against the current report,
  zero-dimension silence apart from cursor hiding, minimum usable nested layouts,
  extreme skinny axes, rejected splits and retained prompt arguments. Each report
  completes before the next so intermediate frames have an unambiguous geometry.

## Contracts Added or Changed

- Private `Minimum` calculation describes complete subtree requirements with
  checked native-dimension arithmetic.
- `Area::divide_to` reserves both recursive minima while retaining the preferred
  equal division when it fits; normal leaf split admission keeps its existing
  minimum pane requirements.
- `Layout` derives all child rectangles using those minima or suspends the
  complete projection. The returned geometry cannot contain a partial subtree.
- Existing frame, view and focused-input owners retain their clipping and
  restoration contracts, now verified across nested resize transitions.
- `CHANGELOG.md` describes minimum-aware nested resizing and safe suspension.

## Files of Interest

- `src/window/geometry.rs`: recursive minimum extents and admitted allocation.
- `src/window/layout.rs`: subtree requirements and complete geometry placement.
- `tests/windows/resize.rs`: native grids, extreme axes and state restoration.
- `tests/windows.rs`: focused window-suite module registration.
- `scripts/check-pty.py`: terminal bounds, nested resizing and prompt continuity.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 194 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test windows`: passed; thirty-four tests, zero skips. Added
  coverage verifies branch minima, all tiny transient surfaces, small geometry
  grids, non-overlap and complete coverage, native dimension endpoints, repeated
  resize/restore, rejected splits and retained independent views/undo/redo.
- `python3 scripts/check-pty.py --suite terminal`: passed; thirty-nine cases,
  zero skips. New journeys verify both nested directions at exact minima,
  zero/tiny/extreme-axis reports, bounded hardware cursor addresses, retained
  dirty text and undo, split refusal and a surviving file prompt argument.
  Owned fixtures stop after their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
