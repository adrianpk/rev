# Slice 3: Resolve Unicode and display coordinates

Status: reviewing
Delivery set: rev-editor
Plan: [rev Editor Delivery](../../../plan/rev-editor.md)
Tracker: [rev Editor Tracker](../../../tracker/rev-editor.md)
Branch: `feat/rev-03-coordinates`
PR: pending

## Purpose

Convert explicit text and display coordinate domains and navigate complete
Unicode graphemes across rope chunks.

## Delivered Behavior

- Converts typed scalar positions, absolute UTF-8 byte offsets, logical
  row/byte columns, and terminal-cell positions. Exact source boundaries round
  trip; invalid offsets and noncanonical row aliases return typed errors.
- Moves forward/backward over extended graphemes, including combining marks,
  emoji modifiers and ZWJ sequences, regional-indicator pairs, Indic conjuncts,
  and CRLF. Movement clamps at buffer boundaries.
- Projects lines into safe glyphs with explicit scalar and cell spans. Wide
  text, four-cell tab stops, and combining sequences use their display widths.
  Interior wide/tab cells map to the source start; columns past EOL clamp.
- Displays C0 controls as caret text, DEL as `^?`, and C1 controls as hexadecimal
  text. Standalone zero-width clusters receive a dotted-circle presentation.
  Projected document text emits no terminal control characters.
- Uses LF logical line indexing and preserves CRLF as one grapheme separator.
  Lone CR and other controls remain visible within their line.

## Implementation Notes

- T3.1: `b21fb6f837c6d959ee37716a88e96ac5da2ff400` introduces coordinate
  conversions, context-fed navigation, glyph projection, and the recorded line
  policy with initial behavior tests.
- T3.2: `7a04433a8c7134b8c060379b1f4d1ca6b0a528b4` completes eight coordinate
  tests and bounded seam overlap for retained-cursor regional context.
- `Coordinates` borrows a qualified buffer slice and retains its revision stamp.
  It does not create another canonical text store. Byte conversions check exact
  scalar boundaries; display conversions reject positions inside graphemes.
- The cursor supplies requested preceding context and adjacent chunks to the
  pinned Unicode segmentation library. Forward chunk transitions include one
  preceding scalar so a retained cursor does not recount regional context at
  an exact chunk start. The derived overlap is bounded to a chunk and one scalar.
- Glyph text borrows contiguous source storage. Only a cross-chunk grapheme,
  bounded cursor overlap, or transformed presentation needs an owned fragment.
- The reproduced vertical-tab line-count failure established why Ropey's
  default Unicode/CR line features conflict with visible controls. P10 now
  records LF indexing with CRLF retention, and Cargo enables only Ropey's
  `simd` feature. Dependency versions and text bytes remain unchanged.
- New coordinate functions have measured cyclomatic complexity at most 8,
  logical nesting at most 2, and at most five parameters including receivers.

## Contracts Added or Changed

- `Scalar`, `Byte`, `LineByte`, and `Cell` identify separate coordinate domains.
- `Coordinates` exposes checked conversions, grapheme boundary queries and
  motion, and lazy per-line glyph iteration.
- `Glyph` carries safe presentation text, a scalar source span, and a cell span.
  Line separators are excluded from presentation without changing source bytes.
- Reverse cell conversion deliberately selects the start of a touched cluster
  and clamps beyond line end. Raw line-byte conversion preserves positions
  inside CRLF; visual conversion and motion keep that separator whole.
- The Cargo Ropey feature selection and plan P10 specify the same LF/CRLF and
  visible-control policy.

## Files of Interest

- `src/coordinates.rs`: coordinate types, conversions, line bodies, and motion.
- `src/coordinates/cursor.rs`: chunk/context adapter and bounded seam overlap.
- `src/coordinates/display.rs`: safe glyphs, Unicode widths, and lazy projection.
- `tests/coordinates.rs`: eight behavior tests and real chunk-seam fixtures.
- `Cargo.toml`, `ops/default/plan/rev-editor.md`: matching line-policy decisions.

## Validation

Successful commands, in canonical order. Looper records final clean PR-head
evidence after the report and PR-reference follow-up commits.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed; sixteen tests, including eight coordinate tests.
- `python3 scripts/check-source.py`: passed.
- `cargo test --test coordinates`: passed; eight tests, zero skips. Fixtures
  exercise round trips, motion, width/tab policies, controls, empty text,
  invalid positions, and verified non-boundary rope seams for combining,
  emoji, regional parity, and Indic contexts against flat segmentation.
- `git diff --check`: passed.

## Risks and Follow-ups

None recorded.
