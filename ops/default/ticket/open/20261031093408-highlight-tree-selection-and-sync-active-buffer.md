---
id: TKT-20261031093408
title: Highlight tree selection and synchronize the active buffer
status: open
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: tree, selection, buffers
source: chat
reported_at: 2026-10-31T09:34:08Z
commits: []
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

## Current source observations

src/display/tree.rs admits the selected-row decoration only when panel.active()
is true. Existing palette.tree_selected and palette.tree_selected_text roles
already exist. The requested change therefore includes visible selection while
the editing pane has focus, rather than assuming the editor has no selection
style anywhere. This is static inspection supporting the owner's report, not
a separate terminal reproduction.

## Decisions before development

- With different files in multiple panes, define which view drives the active
  file indication. The active or most recently active editing pane is a candidate.
- Distinguish keyboard tree selection from the active-file indication so that
  navigating unopened entries is not immediately reset by synchronization.
  Decide whether separate visual states or an explicit transition policy is needed.
- Define behavior for unnamed buffers, files outside the chosen root, aliases,
  collapsed ancestors and unavailable entries. Do not imply a different file
  is active or introduce automatic root changes to make a marker appear.
- Select palette-role reuse and any focus-dependent contrast before adding new
  configuration. Exact colors and automatic expansion/reveal are not selected.

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
