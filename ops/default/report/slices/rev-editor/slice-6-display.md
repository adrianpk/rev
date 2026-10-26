# Slice 6: Render one rope-backed viewport

Status: delivered
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-06-display`
PR: `#6`
Integrated: Rebase + Fast-forward into dev at `f87c76b69ab2794f0f4a117a957bef143064f4b8`.

## Purpose

Render the initial canonical rope through an independent view with bounded
terminal-cell geometry, a visible cursor, status, and a separate echo area.

## Delivered Behavior

- Starts the terminal session with the actual empty buffer and its selected
  viewport. Point, top logical row, and horizontal cell offset belong to the
  view rather than the text store.
- Projects visible logical rows using the existing safe grapheme glyphs.
  Wide text, combining clusters, emoji, tabs, and displayed controls retain
  their cell widths. Partially clipped glyphs become spaces, preserving the
  following text's alignment without printing half a wide cluster.
- Shows the buffer name, dirty and read-only indications, one-based logical
  line and terminal-cell column, and `Text` mode. The selected status begins
  with `>`. New buffers have stable generated labels; the initial buffer is
  named `*scratch*`.
- Reserves the bottom row for echo/error messages. Controls in document and
  echo text become safe visible glyphs, so an error does not overwrite document
  rows or inject terminal commands.
- Reveals point in both scroll axes when geometry is usable. Tiny dimensions
  suspend the document/cursor presentation; zero dimensions emit no row moves.
  Resizing preserves the buffer and point and resumes a valid viewport.
- Writes full-width bounded rows with explicit row positioning and one flush
  per frame. Redraws follow startup, resize, or input feedback; the session
  waits for events between them.

## Implementation Notes

- T6.1: `9b1d62b17dc98a039243e96693e4ceed2275e94d` adds view state, cell
  clipping, derived frames, buffer status metadata, and runtime drawing.
- T6.2: `5ed738ed6e12f4f4c739aca025f279cd30d4a423` completes nine display
  tests, places viewport geometry with the view owner, and adds a real PTY
  journey through minimal and zero dimensions back to a usable frame.
- Rendering borrows rope lines and retains only derived visible row strings.
  It creates no second authoritative document or whole-document String copy.
- The session builds a fresh frame at every redraw. It retains no projection
  cache across text, read-only metadata, scroll, or geometry changes. A frame
  records its source stamp, and view points must match the current buffer.
- Normal geometry needs three columns and three rows: at least one document
  row, one status row, and one echo row. Suspended frames hide the cursor and
  clear their remaining rows safely. No arbitrary surface cap is introduced.
- Dirty indication reflects published edits relative to the initial clean
  buffer state; it reads history metadata rather than scanning document bytes.
- The terminal PTY suite follows the display suite because this slice replaces
  the executable's frame output and resize handling. Its owned sessions restore
  terminal state and close after their checks.
- Source checks pass the safe-Rust, five-parameter, three-level nesting, and
  ten-path cyclomatic limits.

## Contracts Added or Changed

- `View` owns a qualified point and independent scroll offsets. `reveal` changes
  only scroll; invalid source stamps or grapheme positions return typed errors.
- `Size` describes native terminal geometry, including zero dimensions.
  `Position` identifies a surface cursor rather than a logical document cell.
- `Frame` exposes immutable derived rows and an optional admitted cursor.
  Each row has exactly the frame's terminal-cell width and a document, status,
  or echo role. Its writer emits only those rows and flushes once.
- `Buffer::name` and `Buffer::is_modified` provide truthful initial status data.
  Read-only state remains owned by the buffer's checked edit boundary.
- `Glyphs` can internally project bounded UI text with the same safe display
  policy used for rope fragments. The runtime session owns its buffers, view,
  input state, geometry, and echo message.

## Files of Interest

- `src/view.rs`: qualified point, scroll, geometry, and reveal behavior.
- `src/display.rs`, `src/display/clip.rs`: derived rows, clipping, and output.
- `src/session.rs`, `src/lib.rs`: event-driven initial viewport and resize flow.
- `src/buffer.rs`: initial labels and dirty-status metadata.
- `src/coordinates/display.rs`: safe UI text glyph projection.
- `tests/display.rs`: nine public-boundary display tests.
- `scripts/check-pty.py`: actual executable viewport/resize journey.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; forty-five tests, including nine display tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test display`: passed; nine tests, zero skips. Cases cover the
  initial frame, independent scroll/reveal, mixed-width clipping, safe errors,
  all zero/tiny dimension combinations, native dimension extremes, fresh text
  and status metadata, invalid qualified views, and bounded output/flushes.
- `python3 scripts/check-pty.py --suite terminal`: passed; eight cases, zero
  skips. The added journey observes the initial status/cursor and resizes to
  `3x3`, `1x1`, `0x0`, and back to `80x24` before ordinary exit and restoration.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
