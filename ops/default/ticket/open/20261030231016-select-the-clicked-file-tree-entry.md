---
id: TKT-20261030231016
title: Select the clicked file-tree entry
status: open
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: mouse, tree, navigation
source: chat
reported_at: 2026-10-30T23:10:16Z
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

## Decisions still open

The owner suggested configurability as a possibility. Decide whether single
click only selects or also opens a file/expands a directory, whether repeated
clicks have another action, and whether a preference is warranted. Neither
one-click activation nor a new configuration default is approved by this ticket.

## Acceptance direction

- Click a different visible row and observe matching focus and selection.
- Use the actual event/render path after scrolling, resize, clipped labels,
  root expansion and listing refresh; do not infer identity from an old index.
- Define handling for directory/root rows, empty area and disappeared entries.
- Preserve keyboard navigation, unsaved buffers and canonical opening behavior.
- Apply only the separately agreed activation policy after selection.

## Specification and boundary

[File and Directory Navigation NAV05](../../spec/editor-file-navigation.md)
owns the shared identity and interaction contract. This ticket stays open until
activation details are settled; capturing it does not implement the behavior.
