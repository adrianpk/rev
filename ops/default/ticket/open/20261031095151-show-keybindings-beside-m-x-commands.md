---
id: TKT-20261031095151
title: Show keybindings beside M-x commands
status: open
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: commands, keybindings
source: chat
reported_at: 2026-10-31T09:51:51Z
commits: []
---

# Show keybindings beside M-x commands

## Requested behavior

Show an assigned keyboard shortcut beside each command in the M-x candidate
list when one exists. The owner wants to discover commands through M-x and
learn their shortcuts for subsequent use, following the Emacs interaction model.

## Expected outcome

- Keep the command name visible and add its shortcut as secondary information.
- Derive hints from effective bindings for the context from which M-x was
  opened, including configured overrides. Do not advertise an overridden or
  inapplicable binding as an available shortcut.
- Keep commands without a binding available through M-x, without inventing
  shortcuts or requiring users to assign them.
- Preserve command-name matching, candidate navigation, completion and
  invocation. A shortcut annotation is display information, not prompt text.
- Keep names and hints readable at ordinary terminal widths; handle narrow
  panes and commands with multiple bindings without confusing the two fields.

## Validation direction

Check a command with a default binding, one with a configured replacement,
an unbound command and context-specific bindings. Confirm that displayed hints
agree with actual dispatch in the originating context. Verify filtering,
selection, completion, invocation and narrow-width rendering remain usable.

## Delivery boundary

This is a bounded presentation ticket. Reuse the existing command registry,
effective keymaps and candidate-list rendering. It does not introduce new
shortcuts, change dispatch or authorize implementation during capture.
