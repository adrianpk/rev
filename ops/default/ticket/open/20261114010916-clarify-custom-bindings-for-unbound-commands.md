---
id: TKT-20261114010916
title: Review factory shortcuts for unbound commands
status: solved
kind: follow_up
severity: unclassified
priority: unclassified
scope: ui
tags: keybindings, commands, defaults
source: chat
reported_at: 2026-11-14T01:09:16Z
commits: ["c279fd03ccc383a9ae606665bf0cd7ae2cfccc49", "2ae7f75ff1a067a8b553dfc665b6022b66610e57", "c1c0aeaf217f63beb64bd25733efe661c09d2c4e", "f1f186918dc24d1478f92bf47ee55f6a9bd5dea9", "9ca41e679978354e095bd09bfe9983aa735e8d5e", "37d98aa6e3a8e091f56c69262fab4dce87e15918", "979ba5ebb64310d0a1d1e3801d53689fdf525cec", "72b8d40d816ee1cf2eb1aec333658fcb6a9c503e"]
resolution: fixed
closed_at: 2026-11-14T23:15:37Z
branch: dev
pr: https://forge.adrianpk.com/rev/rev/pulls/53
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

The original capture authorized review only. Runtime delivery was subsequently
approved through the linked seven-slice specification and plan.

## Resolution

The approved [delivery plan](../../plan/rev-factory-shortcuts.md) and
[specification](../../spec/editor-factory-shortcut-alignment.md) are complete.
All 707 native commands and six input actions have reviewed dispositions;
[mapping recommendations](../../report/rev-factory-shortcuts/mapping-contract.md)
record accepted routes, conflicts, named-only access and deliberate differences.
The accepted 29 records / 49 base-context edits are implemented through PRs
#47-#53, preserving explicit user overrides and protected recovery routes.

The controller full gate passed at `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`,
including independent complete-map checks, source effects, both profiles,
three bounded Linux PTYs, hook acceptance, reference checks and Spore projection.
The [completed tracker](../../tracker/rev-factory-shortcuts.md) records the exact
command and evidence. This solved ticket stays at the controller-configured
closure path so its existing source references remain valid. The separate
functional-parity specification retains its own scope.
