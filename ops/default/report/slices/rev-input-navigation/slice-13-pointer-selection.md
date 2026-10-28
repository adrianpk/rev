# Slice 13: Select canonical text by dragging

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-13-pointer-selection`
PR: pending

## Purpose

Use primary press/drag/release to select canonical document text through the
existing checked point movement and linear mark/region owners.

## Delivered Behavior

- A text press focuses its pane, places point and arms one revision-qualified
  source anchor. A click or same-cell motion creates no new region.
- A real text drag activates the ordinary linear mark and extends point through
  the source view's checked text geometry. Release retains the canonical region
  and ends capture; a later orphan motion cannot change it.
- Capture remains with its originating source across gutters, status, padding,
  separators, tree and neighboring panes. Non-text cells retain the last valid
  endpoint; returning to source text resumes the same anchor. Wheel reports do
  not transfer scrolling while a gesture is captured.
- Keyboard input/commands, resize, reported focus loss, source edits/rebinds and
  view changes retire capture. C-g retains its ordinary region cancellation.
  The native mode-owner cancellation hook ends capture while retaining region.
- Released regions feed native copy/kill/yank and exact undo/redo. Read-only
  text admits selection/copy and retains its region when an edit is refused.
- Source bytes, saved state and undo history remain unchanged by selection.

## Implementation Notes

Session retains one minimal capture and a bit distinguishing an armed click from
an active selection. The source ViewId and buffer stamp are rechecked against
current composition before each extension. No rendered text, UI label, event
queue, clipboard write or timer is retained.

The native host can call `Session::cancel_pointer_capture` before disabling mode;
current ownership-loss routes use that same hook. Actual startup/runtime mode
controls remain assigned to Slice 15. Capture memory is session-owned and is
released with the session on terminal exit/error. Current source stamps and live
composition are checked for every drag/release, so delayed reports cannot revive
an invalidated gesture.

## Contracts Added or Changed

PTR03 reuses represented text boundaries. PTR04 selects the native linear region
and retains it after release for ordinary editor operations.

## Files of Interest

- `src/session/pointer/selection.rs`: captured source, checked extension and release.
- `src/session/pointer.rs`: native primary gesture dispatch.
- `tests/pointer/selection.rs`, `tests/pointer/capture.rs`,
  `tests/pointer/selection_edit.rs`: canonical regions,
  source ownership, mode-owner cancellation, Unicode/source boundaries and
  native copy/kill/yank/history regressions.
- `scripts/configuration_ui/pointer_selection.py`: actual SGR press/drag/release,
  CSI focus loss, saved bytes and owned PTY idle/restoration checks.
- `tests/editing.rs`, `tests/editing/regions.rs`: gutter-aware historical editing
  cursor, source rendering and region-color expectations.

## Validation

Development checks passed:

- `cargo test --test pointer selection::`: forward/reverse canonical regions,
  final release endpoint, orphan motion, click/same-cell behavior, empty region
  after a real drag and unchanged file/history state passed.
- `cargo test --test pointer`: 37 groups passed, including both gutter sides,
  retained capture across non-text cells/other panes, release outside source,
  wheel isolation, nine cancellation owners and successful buffer rebind.
- `cargo test --test pointer selection_edit::`: three groups passed for native
  copy/kill/yank and read-only refusal; tab widths 1/4/8/32 on either gutter side
  with horizontal offsets; Unicode grapheme, clipped-edge, CRLF and EOF bounds.
- `python3 scripts/check-pty.py --suite pointer-selection`: two owned sessions
  passed for actual SGR gestures, native copy/kill/yank/undo/redo, exact saves
  with BOM and mixed newlines, no implicit clipboard/ring copy, different-buffer
  ownership, reported focus loss, resize/C-g cancellation and later plain click.
- `cargo test --test editing direct_input`: corrected native cursor/text-origin
  fixture passed. The same selection ranges/colors passed in the editing run.
- `cargo clippy --test pointer --test editing -- -D warnings`: focused native lint
  passed.
- `python3 scripts/check-source.py`: unchanged safe-Rust and complexity limits.

PTY fixtures retain the existing eight-second deadline and 1-MiB output bound,
owned HOME/XDG roots, exact bounded filesystem snapshots, raw-input assertion,
terminal restoration, child reaping and descriptor closure. Active capture
emits no output or process CPU ticks over the existing 120 ms idle sample.
The warm two-session fixture takes under one second locally, excluding build.
Every observed stage rejects OSC 52 clipboard writes. No runtime resource limit,
input batch ceiling or timer has changed.

The unchanged dev baseline `560a743cd7342452d008f35bc6b42a52b111531a` failed
three historical editing fixtures: direct input, multiline region rendering and
shared-view focus. Their terminal coordinates omitted the delivered six-cell
factory gutter. Corrected cursor/text origins and selection-color cells
retain the exact original source ranges, bytes, revisions, history lengths,
focus and cancellation assertions. The first corrected editing run passed 95
cases and exposed the remaining direct-input text-prefix expectation; its
focused rerun passed after including the same gutter field. Required final
validation covers all 96 cases on the final candidate.

Final canonical and focused clean-head evidence is recorded through the controller
after task and PR-reference commits. Required ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test editing --test commands --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final commands:

- `cargo test --lib --test coordinates --test windows --test tree`
- `python3 scripts/check-pty.py --suite pointer-focus`
- `python3 scripts/check-pty.py --suite pointer-scroll`
- `python3 scripts/check-pty.py --suite pointer-selection`
- `git diff --check`


## Risks and Follow-ups

Bounded drag-edge autoscroll and ordinary reporting/settings activation remain
assigned to the recorded later slices. This layer adds no idle work.
