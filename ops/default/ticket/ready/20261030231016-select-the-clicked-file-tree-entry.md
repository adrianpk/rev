---
id: TKT-20261030231016
title: Select the clicked file-tree entry
status: ready
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: mouse, tree, navigation
source: chat
reported_at: 2026-10-30T23:10:16Z
ready_at: 2026-10-30T23:41:32Z
commits: []
---

# Select the clicked file-tree entry

## Requested outcome

Clicking a visible entry in the file tree should focus the tree and select the
entry clicked, instead of retaining a different previously selected row. The
owner raised this during the Org walkthrough; it was deferred for capture.

The delivered pointer contract supports focusing the tree. Current source in
src/session/pointer.rs routes Hit::Tree to focus_tree without row identity.
This describes a follow-up interaction, not proof of a regression.

## Accepted activation policy

Following the owner's maximum-Emacs-similarity criterion, use Dired-style
activation adapted to the native tree. A primary click selects the checked
rendered entry and activates it: files open through the existing owner in the
remembered editing pane; directories expand/collapse without changing the root.
mouse.tree_activate_on_click defaults true; false keeps selection/focus only.
The setting uses startup TOML validation, sample coverage and compiled defaults.
No double-click timing mechanism is introduced. Global mouse disablement wins.
Blank area focuses without activating an entry; invalid/stale hits never act on
an unrelated row. Preserve native root activation and file failure behavior.

## Acceptance direction

- Click a different visible row and observe matching focus and selection.
- Use the actual event/render path after scrolling, resize, clipped labels,
  root expansion and listing refresh; do not infer identity from an old index.
- Define handling for directory/root rows, empty area and disappeared entries.
- Preserve keyboard navigation, unsaved buffers and canonical opening behavior.
- Cover default activation, the selection-only preference and global mouse disablement.

## Specification and boundary

[File and Directory Navigation NAV05](../../spec/editor-file-navigation.md)
owns the shared identity and interaction contract. This ticket is ready and
owned by Slice 7 of [Wrap and Navigation](../../plan/rev-wrap-navigation.md).
The status transition does not claim an implemented click interaction.
