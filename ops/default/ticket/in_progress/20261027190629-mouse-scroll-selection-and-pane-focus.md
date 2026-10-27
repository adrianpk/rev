---
id: TKT-20261027190629
title: Add mouse and trackpad scrolling, selection and pane focus
status: in_progress
ready_at: 2026-10-27T23:05:50Z
started_at: 2026-10-27T23:05:50Z
branch: feat/rev-input-10-pointer-lifetime
pr: pending
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: mouse, scrolling, selection
source: chat
reported_at: 2026-10-27T19:06:29Z
commits: []
---

# Add mouse and trackpad scrolling, selection and pane focus

## Requested outcome

Support terminal-delivered pointer input for three editor interactions:

- Scroll document views and the file tree with a mouse wheel or trackpad.
- Click the file tree or any visible editing pane to focus it.
- Select document text by dragging, if supported by the admitted terminal
  input protocol, using the editor's existing region behavior.

Keep keyboard navigation fully available. This is native support in the Rust
binary, without an addon, plugin system or device-specific integration.

## Behavioral reference

The user requested affinity with the pointer behavior of their other editor.
Retain these relevant interaction decisions without importing its architecture:

- Clicking an inactive pane uses the same focus transition as keyboard pane
  navigation. A text click resolves the corresponding buffer position.
- Map coordinates through current panel geometry and the displayed document,
  including scrolling, tabs, Unicode graphemes and wide glyphs.
- A text drag uses the canonical buffer region, excludes borders, status rows,
  gutters and neighboring panes, and remains owned by its originating view.
- Releasing a drag leaves the selected region available to ordinary editing
  commands. A click without dragging only places point.

## Acceptance direction

- Exercise wheel/trackpad scrolling in both directions and pane focus across
  a visible tree and at least two split document views.
- Verify pointer selection integrates with the existing region commands and
  preserves document bytes, dirty state and undo history until an edit occurs.
- Check hits after resize, viewport movement and pane changes; obsolete or
  out-of-bounds coordinates must not move point into unrelated content.
- End transient drag capture on release, cancellation or loss of its target.
  Motion and scroll processing must stay bounded and must not introduce the
  prolonged movement reported in the separate arrow-navigation ticket.
- Enable only the terminal reporting modes needed for the selected gestures
  and restore those modes through normal and failure cleanup paths.
- Verify the supported terminal path with actual input. A trackpad is usable
  to the extent that its terminal exposes scroll and pointer events.

## Selected scroll and focus behavior

The wheel or trackpad scrolls the panel beneath the pointer without changing
keyboard focus. An ordinary primary-button click changes pane focus. The
product owner accepted the conventional behavior after checking existing
editors; the Emacs wheel owner uses this default and restores the selected
window after scrolling another window.

Reference: [Emacs wheel behavior](https://github.com/emacs-mirror/emacs/blob/master/lisp/mwheel.el).

## Selected pointer contract

[Pointer Interaction](../../spec/editor-pointer-interaction.md) fixes native
controls, hovered-pane scrolling, checked focus/point placement, linear region
selection and bounded active-drag autoscroll. Terminal bypass is documented as
terminal-owned; a native mouse-disable command remains available. No pane resize,
context-menu or extra gesture framework is added by this delivery.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.
