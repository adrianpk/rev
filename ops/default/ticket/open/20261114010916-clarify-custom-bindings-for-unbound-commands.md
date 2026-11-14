---
id: TKT-20261114010916
title: Review factory shortcuts for unbound commands
status: open
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: keybindings, commands, defaults
source: chat
reported_at: 2026-11-14T01:09:16Z
commits: []
---

# Review factory shortcuts for unbound commands

## Goal

Analyze commands without factory shortcuts and identify which should receive
conventional bindings consistent with the existing keymaps and command families.

## Scope and acceptance

- Review the native command catalog, including mode-specific commands, and
  identify useful missing factory shortcuts.
- Propose conventional chords that fit existing prefix families and editing
  behavior; avoid assigning arbitrary shortcuts just to cover every command.
- Check conflicts, context and profile ownership, terminal input, protected
  recovery keys and explicit user overrides.
- Produce a recommendation table with command, context, proposed shortcut and
  rationale or conflict. Identify commands that should remain unbound.

References: `docs/reference/bindings.md`, `src/input/keymap/defaults.rs` and
`src/input/keymap/map.rs`.

Capture only; no configuration or runtime change is authorized in this task.
