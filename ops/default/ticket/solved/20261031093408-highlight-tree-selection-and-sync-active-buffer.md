---
id: TKT-20261031093408
title: Highlight tree selection and synchronize the active buffer
status: solved
resolution: fixed
closed_at: 2026-11-07T17:56:49Z
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: tree, selection, buffers
source: chat
reported_at: 2026-10-31T09:34:08Z
ready_at: 2026-11-07T15:19:21Z
started_at: 2026-11-07T17:17:52Z
branch: feat/rev-tree-selection-01-follow
pr: 308
reviewed_at: 2026-11-07T17:40:15Z
commits: [0b5e987a20a02bbef1d7c53a1ae91749ab42b813, 88087201423c6ea4e331d0328fd295ae94942607, 750da5ece50597b114c3fff02150793219bb2ab2, b3e77ecb57a3dfd3d141a911ae319ccf72ab2ef8, 27e222f65a81bbdb0d3aece0e7279877faf997b2]
---

# Highlight tree selection and synchronize the active buffer

## Reported behavior

Tree clicking now works. When the clicked file is already displayed, however,
the absence of a persistent visible selection indication makes the click appear
ineffective. The owner requests a later presentation follow-up: highlight the
selected entry and synchronize that indication with the displayed buffer.

## Requested outcome

- Keep the selected tree entry visibly identifiable after activation transfers
  focus to an editing pane, including activation of an already displayed file.
- Reflect the displayed file when buffer navigation or editing-pane focus
  changes, rather than updating only after a tree click.
- Preserve keyboard tree navigation, file/directory activation, the existing
  selection-only click preference and canonical buffer/path identity.
- Reuse native tree, view and palette ownership. Highlighting must not modify
  source text, create history, open files or replace an explicitly chosen root.

## Baseline source observations

Before this delivery, src/display/tree.rs admitted selected-row decoration
only when panel.active() was true. Existing palette.tree_selected and palette.tree_selected_text roles
already exist. The requested change therefore includes visible selection while
the editing pane has focus, rather than assuming the editor has no selection
style anywhere. This is static inspection supporting the owner's report, not
a separate terminal reproduction.

## Approved specification

Owner decision on 2026-11-06: synchronization must be configurable and enabled
by default. [File Tree Selection and Active Buffer](../../spec/editor-tree-selection.md)
and its model record the field and complete focus/reveal rules. The 2026-11-07
revision reconciles the behavior with current Project/Workspace roots. The owner
approved the specification and requested a plan, slicing, tracker and Looper.

- Follow the active editing pane; tree/prompt focus retains its editing origin.
  Use one visible highlight, with separate manual browsing and active-file
  identities so keyboard navigation is not reset by following.
- Keep selected-row feedback after focus leaves the tree. Reuse existing tree
  selection colors and NO_COLOR reverse video in both tree presentations.
- Match only reliable loaded entries. Scroll to a loaded offscreen file on an
  actual follow transition; preserve collapsed directories and explicit roots.
  Unnamed, out-of-root, filtered or unavailable targets have no active-file mark.
- In overlapping Workspace routes, retain an eligible matching route or use
  declared root order, without duplicating the buffer or highlighting two rows.

The [plan](../../plan/rev-tree-selection.md) and
[tracker](../../tracker/rev-tree-selection.md) define one implementation slice.
Slice 1 and canonical integration are complete. The controller full gate passed
`27e222f65a81bbdb0d3aece0e7279877faf997b2` before documentation closure.

## Acceptance direction

Check repeated clicks on the current file, opening another file, keyboard buffer
switching, split panes and mouse focus changes. Selection must remain legible
through focus transfer, scrolling, resize and listing refresh. Preserve stable
entry identity and the distinction between selecting and activating. Keep drawing
free of filesystem IO and avoid a background scan for each buffer change.

## Relationship and delivery boundary

This follows the solved [clicked-entry ticket](../solved/20261030231016-select-the-clicked-file-tree-entry.md)
and [File and Directory Navigation](../../spec/editor-file-navigation.md).
Their delivered behavior and historical evidence remain intact. This ticket
captures later work; it does not reopen rev-wrap-navigation or start implementation.

## Implementation evidence

The [Slice 1 report](../../report/slices/rev-tree-selection/slice-1-follow.md)
records delivered behavior, native validation, task commits, canonical integration
and successful final-gate evidence. The ticket is solved with resolution fixed.

Canonical [PR #308](https://forge.adrianpk.com/adrianpk/rev/pulls/308) merged into
dev at `b3e77ecb57a3dfd3d141a911ae319ccf72ab2ef8`. The report introduction is `750da5ece50597b114c3fff02150793219bb2ab2`.

## Resolution

Fixed: selection remains visible after activation and follows the active editing
file by default, with configurable manual selection and preserved source/root
ownership. The unchanged controller gate passed candidate `27e222f65a81bbdb0d3aece0e7279877faf997b2`
in log `KGGQSFRVEFDYE7B4A3KIT2FGIO`: 98 selected Rust tests, native
catalogs, documentation links and the local Spore guide build. The
[delivery closure](../../report/slices/rev-tree-selection/slice-1-follow.md#delivery-set-closure)
records the exact command and evidence scope.
