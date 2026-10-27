# Slice 2: Add Emacs-style line navigation

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-02-goto-line`
PR: pending

## Purpose

Navigate to a one-based logical line through the existing native command,
keymap and argument owners while preserving canonical text and view ownership.

## Delivered Behavior

- `M-g g`, `M-g M-g` and `M-x goto-line` open the same line-number prompt.
  `M-g` alone remains a prefix and C-g retains cancellation.
- The existing configured prompt limit bounds input. Signed 64-bit decimal
  integers are admitted; surrounding whitespace is ignored. Invalid or
  overflowing input retains the draft for correction without moving point.
- An existing positive line goes to its start. Nonpositive values clamp to
  buffer start; requests beyond the last logical line clamp to exact EOF.
- Cancellation retains the complete selected view, including intended column
  and recenter state. Successful navigation reveals only that view and uses
  the ordinary shared mark, including in read-only buffers.
- Acceptance rejects an intervening source revision without moving the newer
  mapped view or altering its document state.

## Implementation Notes

The compiled registry and inherited editor map own the command and both aliases.
The pending prompt captures ViewId and source stamp. Acceptance verifies both
before deriving a checked line-byte-zero coordinate from the canonical rope.
A candidate view resets navigation goals and reveals the destination before
publication. Prompt entry itself preserves the retained view and navigation
sequence. No text transaction is published.

## Contracts Added or Changed

NAV01 adds the compiled command, bindings and bounded numeric argument.
NAV02 retains ordinary per-view movement, C-g and shared-buffer history owners.

## Files of Interest

- `src/command.rs`: compiled goto-line identity.
- `src/input/keymap/defaults.rs`: inherited M-g sequences.
- `src/session/goto.rs`: argument validation, ownership and checked destination.
- `tests/commands/goto.rs`: native binding and actual prompt dispatch.

## Validation

- PASS: `cargo test --test commands goto::` (six native dispatch, boundary,
  cancellation, shared/read-only and stale-source regressions).
- Logical-bound cases cover empty/single-line, LF/CRLF, Unicode, final empty
  lines and both signed extremes, comparing exact positions and document state.
- Terminal, malformed-input and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
