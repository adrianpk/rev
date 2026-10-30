---
id: TKT-20261028224018
title: Define comfortable long-line wrapping
status: open
kind: unclassified
severity: unclassified
priority: unclassified
scope: ui
tags: wrapping, display, navigation
source: chat
reported_at: 2026-10-28T22:40:18Z
commits: []
---

# Define comfortable long-line wrapping

## Reported behavior

Writing a long line currently feels awkward. The owner requests a comparison
with VS Code, Neovim and Emacs to clarify word wrap and soft wrap before
settling rev's behavior.

The delivered editor deliberately has no soft wrap. Long logical lines use a
horizontal viewport that follows point. This is recorded in
[display defaults](../../../../docs/reference/defaults.md), the baseline
[editor specification](../../spec/rev-editor.md), and its GUT03 amendment.
There is no existing wrap toggle to enable in the delivered configuration.

## Terms and reference behavior

- Soft wrap changes presentation only: one logical source line occupies
  several display rows without inserting newlines. Word-aware soft wrap
  prefers word boundaries; wrapping exactly at the viewport edge can instead
  divide a word. These are separate display choices, not two file formats.
- Hard wrapping or filling inserts actual newline characters. It changes
  source, save output and undo history and is distinct from visual wrapping.
- VS Code defaults editor.wordWrap to off. Setting it to on wraps at the
  viewport width, and Alt+Z toggles the display behavior. Its additional
  wordWrapColumn and bounded settings do not establish requirements for rev.
- Neovim defaults wrap to on but linebreak to off: screen-edge continuation
  can split words. Enabling linebreak selects permitted word-break points;
  breakindent can indent continuation rows. textwidth is a separate setting
  that can insert real newlines.
- Emacs supports ordinary screen-edge continuation and optional truncation.
  M-x visual-line-mode wraps at word boundaries and changes some line commands
  to operate on visual rows. Auto Fill is separate and inserts real newlines.

## Approved direction

Approved by the product owner on 2026-10-28: native word-aware soft wrapping,
with a visual result comparable to Emacs Visual Line mode, plus an explicit
no-wrap alternative. Wrapping changes presentation without inserting real
newlines. This approval does not adopt Emacs's changes to line-editing commands.
Factory defaults, per-language policy, configuration spelling and keybindings
remain open decisions.

Use rev's existing configuration and native command model where appropriate.
Do not introduce an addon, external formatter or plugin requirement. An
automatic hard-wrap/fill feature is a separate product decision; it must not
be added implicitly to solve this display problem.

## Decisions required before implementation

- Factory behavior and whether Org, Markdown, plain text and source code use
  different startup preferences; scope of runtime overrides across views.
- Viewport wrapping versus an optional configured column; continuation
  indentation/markers, unbroken tokens, tabs, wide graphemes and tiny panes.
- Logical versus visual behavior of Up/Down, C-n/C-p, C-a/C-e, C-k and their
  native command identities. Do not silently import all Emacs Visual Line
  editing semantics merely because its display is a reference.
- Interaction with shared buffers, splits/resizes, horizontal scrolling,
  pointer hit testing, selection, rectangles, searches and goto-line.
- Existing absolute/relative gutter numbers retain logical source-line
  identities. A continuation row must not invent another document line.
- Org folded projections, list/heading indentation and source-preserving
  table editing must remain coherent. Coordinate later modal editing rather
  than defining competing motion or selection owners.
- Bound layout and input work on very long logical lines; preserve the
  delivered navigation responsiveness and normal cancellation.

## Validation direction and delivery boundary

Use source-preserving wrapping examples, resize/split cases, Unicode/tab
boundaries, logical numbering and representative keyboard/pointer journeys.
Enabling or disabling visual wrap alone must not change document bytes, dirty
state or undo history. An explicit future fill operation would require its own
source-edit contract.

This ticket records the approved visual direction with unresolved editing and
configuration details. The [Word-Aware Soft Wrapping specification](../../spec/editor-soft-wrap.md)
and its model now capture the coordinated proposal. Reconcile the delivered
no-wrap contracts and close the listed decisions before implementation planning.
This ticket stays open; no slices or new delivery cycle are created. Scheduling
is not assigned.

## References

- [VS Code word wrap](https://code.visualstudio.com/docs/editing/codebasics#_word-wrap).
- [Neovim wrap, linebreak, breakindent and textwidth](https://neovim.io/doc/user/options/).
- [Emacs continuation lines](https://www.gnu.org/software/emacs/manual/html_node/emacs/Continuation-Lines.html).
- [Emacs Visual Line mode](https://www.gnu.org/software/emacs/manual/html_node/emacs/Visual-Line-Mode.html).
