---
id: TKT-20261030231016
title: Select the clicked file-tree entry
status: reviewing
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: mouse, tree, navigation
source: chat
reported_at: 2026-10-30T23:10:16Z
ready_at: 2026-10-30T23:41:32Z
started_at: 2026-10-31T03:48:50Z
branch: feat/rev-wrap-navigation-07-tree-click
reviewed_at: 2026-10-31T03:56:13Z
commits: [65023e2ee5bb5ea94ea2ac34d5c70adcf159ca41]
---

# Select the clicked file-tree entry

## Requested outcome

Clicking a visible entry in the file tree should focus the tree and select the
entry clicked, instead of retaining a different previously selected row. The
owner raised this during the Org walkthrough; it was deferred for capture.

The delivered pointer contract supports focusing the tree. At capture, source in
src/session/pointer.rs routed Hit::Tree to focus_tree without row identity.
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
owns the shared identity and interaction contract. Slice 7 of
[Wrap and Navigation](../../plan/rev-wrap-navigation.md) implements this behavior;
the ticket is under review pending canonical integration.

## Implemented resolution

Primary clicks use current composed content bounds and opaque root/viewport/
slot/path/type identity before selecting and invoking the native tree action.
Startup `mouse.tree_activate_on_click` defaults true; false selects/focuses
without IO. Global disablement suppresses both operations. Missing, changed-type
or denied activations retain usable selection and dirty canonical documents.

The [Slice 7 report](../../report/slices/rev-wrap-navigation/slice-7-tree-click.md)
records native event/render coverage for clipping, resize/scroll/refresh, split
ownership, aliases, failures, configuration layering and frozen startup policy.
The existing PTY probe now supplies a focused two-session SGR tree-click mode
with exact file preservation and terminal restoration. Canonical PR references
and clean-head controller validation precede handoff.
