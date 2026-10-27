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

- PASS: `cargo test --test commands goto::` (initial two dispatch cases).
- Boundary, view ownership, terminal and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
