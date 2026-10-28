---
id: TKT-20261027190629
title: Add mouse and trackpad scrolling, selection and pane focus
status: in_progress
ready_at: 2026-10-27T23:05:50Z
started_at: 2026-10-27T23:05:50Z
branch: feat/rev-input-14-drag-autoscroll
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/94
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: mouse, scrolling, selection
source: chat
reported_at: 2026-10-27T19:06:29Z
commits: [0e3fa967691c1be21295efe00b22a2b3f0e520d9, 5ea8db194ddf83b11f38623d0af256a22e65d3a0, b9e7ec37a13d42cc3a7e594b608c638ad8af7b8d, 0aafffa4001ed294d38904b0a11c392c521bb71d, f3958d9fabf4eee353d205f5ba725151c56228aa, 89329d0f0199ea0ddfd71bb50a764c62e51160bc, 7d2c71c78fd591c6b8bc12bd55790424ec99c9d7, 9f4f1c199d829c2099ca0ab29c39233ddb8ac1dd, 265527a3237817182e791afde228bb31cef4d1a9, 9ce11b32d0a97e6930ed073e362949c8a07f4f69, 812ac03c0b0a7b3bb46c63d3b0a8f760c1c5d885, 17ff0de8cbff3f9a122f8d9d014a810eb04759fc, c769de9e0cfc48308b59cd8dc48cdbaa65b61d74, e0fca7a6e88b34e4eea84813cdd4d424e34465e9, 148a326f351d78bda33867623cd697712caaa0ee]
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


## Primary slice progress

Slice 10 integrated into dev through canonical PR
[#90](https://forge.adrianpk.com/adrianpk/rev/pulls/90) at
`7b3e61401c9da9d56caf54ea2cb8cd9739676932`. Its
[report](../../report/slices/rev-input-navigation/slice-10-pointer-lifetime.md)
records native admission, reversible terminal ownership, failure/retry, keyboard
continuity and thirteen owned PTYs. Exact-candidate canonical and focused checks
passed. This ticket remains in progress until all primary slices 10-15 integrate.

Slice 11 integrated into dev through canonical PR
[#91](https://forge.adrianpk.com/adrianpk/rev/pulls/91) at
`be336d69ef607bec7fb4a5ebdfed1f6aa5bdcb4a`. Its
[report](../../report/slices/rev-input-navigation/slice-11-pointer-hit-focus.md)
records current tree/pane focus, checked boundaries, prompt isolation and native
region/history preservation. Canonical evidence `IBT4WLILH3CAXCVW2IBSGQN2V6`
and focused evidence `RCVKDXA4MARYXGAGBUIJICY374` passed on that candidate.
Slice 12 is active; this ticket remains in progress until all primary slices
10-15 integrate.

Slice 12 integrated into dev through canonical PR
[#92](https://forge.adrianpk.com/adrianpk/rev/pulls/92) at
`ba2dc0dc6eb279a4a574b59220c4325a29ced1d4`. Its
[report](../../report/slices/rev-input-navigation/slice-12-pointer-scroll.md)
records pointed-pane/tree scrolling without focus changes, native bounds and
source/history/idle outcomes. Canonical `BYQCHD5JLJJXDW5YPZWVZT45Q6` and
focused `GJ65LOWS4MV3U32T6I2ULSHDZQ` evidence passed on that candidate.
Slice 13 is active; the ticket stays in progress through primary slices 10-15.

Slice 13 integrated into dev through canonical PR
[#93](https://forge.adrianpk.com/adrianpk/rev/pulls/93) at
`1203b49f2f4fc9ef8a8bfa1e1605623c289aff3e`. Its
[report](../../report/slices/rev-input-navigation/slice-13-pointer-selection.md)
records canonical drag regions, cancellation, Unicode and native editing/history.
Canonical `HJL3RO2I34C6RF7LELHPNF4DQ4` and focused
`4NCW4Q36NYOUM2XHZ7VYTQEXGF` evidence passed on that candidate.
Slice 14 is active; this ticket stays in progress through primary slices 10-15.

Slice 14 is implemented in canonical PR [#94](https://forge.adrianpk.com/adrianpk/rev/pulls/94), targeting dev. Its
[report](../../report/slices/rev-input-navigation/slice-14-drag-autoscroll.md)
records bounded source-edge region extension, native input-priority deadlines,
controlled-clock bounds, source/Unicode validation and two owned SGR PTYs.
Required and focused exact-head evidence follows PR-reference commits through
the controller. Integration is pending; the ticket stays in progress through
all assigned primary integrations.
