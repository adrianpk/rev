---
id: TKT-20261027191322
title: Add Emacs-style goto-line navigation
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: navigation, keybindings
source: chat
reported_at: 2026-10-27T19:13:22Z
ready_at: 2026-10-27T20:34:14Z
started_at: 2026-10-27T20:34:14Z
branch: feat/rev-input-02-goto-line
reviewed_at: 2026-10-27T20:42:27Z
commits: [645d5b45c8344e6a0033c0bf6679430235db8343, 4cf7b662a1b8200282800df1e38b0338e6f5969b]
---

# Add Emacs-style goto-line navigation

## Requested outcome

Provide `goto-line` with the default Emacs bindings `M-g g` and `M-g M-g`.
`M-g` is a prefix, not the complete command. Retain `C-g` as `keyboard-quit`.

The command prompts for a line number and moves point to the beginning of
that logical line in the active buffer. Lines are numbered from one, regardless
of the gutter's eventual absolute or relative display mode.

## Specification

[Line Navigation](../../spec/editor-line-navigation.md) fixes bindings, numeric
bounds, cancellation and view ownership for this delivery.

## Acceptance direction

- Expose `goto-line` through the compiled Rust command catalog and M-x, and
  integrate both default key sequences with the existing keymap owner.
- Use the ordinary minibuffer input and cancellation behavior. Cancelling
  with `C-g` must leave the original point and viewport intact.
- Reveal the destination in the selected view while preserving unrelated
  views, document bytes, dirty state and undo history.
- Verify first, middle and last lines, empty buffers and Unicode documents.
  Validate malformed or overflowing input without panics or unintended edits.
- Match the existing Emacs `goto-line` behavior for out-of-range requests;
  the specification records the observed start/end clamping behavior.

## Reference

- [GNU Emacs: Moving Point](https://www.gnu.org/software/emacs/manual/html_node/emacs/Moving-Point.html)

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.

## Implementation evidence

The [Slice 2 report](../../report/slices/rev-input-navigation/slice-2-goto-line.md)
records both inherited aliases and M-x, bounded signed numeric input, native
logical clamping and checked source/view ownership. Eight native regressions
and two owned PTY journeys pass, including invalid-input correction, Unicode/CRLF,
empty/final-empty lines, read-only/shared views, cancellation and unchanged state.
The ticket remains reviewing until canonical integration is verified.
