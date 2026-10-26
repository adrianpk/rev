# Slice 19: Map shared-buffer edits across views

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-19-shared-views`
PR: `#19`
Integrated: Rebase + Fast-forward into dev at `8295f8b32759eed0c88a811019eceb3fe6c107c8`.

## Purpose

Publish shared text edits with independently valid points, visible line anchors
and a coherent buffer mark, including history invoked after view changes.

## Delivered Behavior

- Insertion, deletion and replacement map every live point of the selected
  canonical buffer with right affinity and valid grapheme adjustment. The
  initiating view receives its explicit intended cursor; other views keep
  independently mapped current positions.
- Each viewport follows its source line start with left insertion affinity.
  Edits before a visible line preserve its source context; edits through it
  collapse the anchor to the replacement boundary. Horizontal scroll remains
  independent and ordinary drawing reveals each mapped point.
- Undo and redo map current points and viewport anchors through the actual
  inverse or forward changes. They restore the recorded cursor to the original
  view while it still displays that buffer, otherwise to the invoking view.
  Closed views remain closed and later splits never inherit their identities.
- The shared mark retains existing left affinity, grapheme adjustment and
  reversible selection restoration. Each operation enters the canonical
  buffer's history once; changing view still ends typing continuity.
- Reconciliation finishes before drawing, including suspended presentation.
  Rejected edits leave all points, scroll, mark, text and history unchanged.
- Canceling a modified buffer's close preserves all of its views. Explicit
  discard replaces every reference while retaining the layout and unrelated
  buffer views; revisiting the unchanged file loads its disk content normally.

## Implementation Notes

- T19.1: `b0f19612089f873c8dd2cd7592479ff2f0020739` adds viewport mapping,
  stable history origin selection and checked native shared replacement.
- T19.2: `9849102407d640f212ea27808b29b3c9e63183db` adds native and PTY
  evidence; `e9b129320fe6a00225d7be3a6323a195940a5534` makes the invalid
  reversed-range fixture explicit for the required Clippy check.
- `Windows` captures live membership, qualified points and source top positions
  synchronously. The edit owner maps tops against the private candidate using
  the same accepted change ranges used for point and mark mapping.
- All fallible conversion and history admission complete before publication.
  After the buffer publishes, only the captured live registry records receive
  their prepared points and logical top rows. No structural mutation intervenes.
- History retains an optional stable view identity alongside its existing cursor
  snapshot. Replay resolves that identity only within the current target-buffer
  membership; it never restores another view's historical position or layout.
- Viewport anchors map from current source positions during history navigation,
  so a removed visible region does not incorrectly restore an obsolete scroll
  snapshot. The existing rope remains canonical without per-key document copying.

## Contracts Added or Changed

- `Prepared::map_top` resolves left-affine source line positions in candidate
  text before publication; source stamps and candidate coordinate bounds are checked.
- Private window publication reconciles all captured views only after successful
  position preparation and the sole checked buffer publication path.
- Transactions retain their originating view identity without owning or reviving
  views. Typing groups cannot inherit a different originating identity.
- `Session::replace` exposes the same checked scalar-range replacement used by
  editing commands, including shared points, mark, history and viewport mapping.
- `CHANGELOG.md` describes independent shared cursor/viewport mapping and history
  restoration to the original live view.

## Files of Interest

- `src/buffer/edit/views.rs`: candidate top mapping and history origin metadata.
- `src/buffer/edit/mapping.rs`: shared accepted-change position mapping.
- `src/window/points.rs`: live capture, history target resolution and publication.
- `src/session/edit.rs`, `src/session/history.rs`: common native edit/replay paths.
- `tests/windows.rs`: mapping, history identity, lifecycle and rejection evidence.
- `scripts/check-pty.py`: scrolled shared edits, closed-origin undo and shared kill.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; 189 tests.
- `python3 scripts/check-source.py`: passed; safe Rust, five parameters,
  nesting at most three and cyclomatic paths at most ten.
- `cargo test --test windows`: passed; twenty-nine tests, zero skips. Added
  coverage includes edits before/across another cursor, removed visible lines,
  exact-top affinity, combining text, undo/redo from another view, closed/switched
  history origins, modified shared-buffer close, suspended mapping and rejection.
- `python3 scripts/check-pty.py --suite terminal`: passed; thirty-six cases,
  zero skips. New journeys verify source-context retention after a newline in
  another pane, independent hardware cursor and history restoration, undo after
  deleting the original view, and canceled/discarded shared modified buffers.
  Owned fixtures stop after their checks finish; terminal restoration is checked.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
