---
id: TKT-20261031095151
title: Show keybindings beside M-x commands
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: commands, keybindings
source: chat
reported_at: 2026-10-31T09:51:51Z
ready_at: 2026-11-06T18:48:39Z
started_at: 2026-11-06T18:58:38Z
branch: test/rev-command-keybinding-hints-02-acceptance
reviewed_at: 2026-11-06T19:45:54Z
pr: 295
commits: [c18cdc0b5bb3feb05074e4d2bcba07c0afcfba6b, 844ec6c5fb9d0c6f71d1de2ca6f98538423d8d12, 2a66698097f1a66ec1efbf18dc61c9f33c4990f7, 266f6384283bbdc9c791d1c2c9250d510bfd3023, 543c5324ca45fbdf83c31fe272a3516b0c781f94]
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

## Specification trail

The owner requested formal specification before implementation on 2026-11-06.
[Command Keybinding Hints](../../spec/editor-command-keybinding-hints.md)
records the originating-context rules, primary-hint policy, rendering boundaries
and acceptance scenarios. The owner approved continuation on 2026-11-06.
Slice 1 is integrated through PR #294. Slice 2 acceptance and guidance are
reviewing through the approved plan, tracker and Looper delivery. The integrated
controller gate remains pending.

## Slice 1 evidence

The [Slice 1 report](../../report/slices/rev-command-keybinding-hints/slice-1-effective.md)
records effective originating-context hints, canonical identity preservation,
cell-safe presentation, native regressions and all five successful slice checks.
The required suites passed 336 tests; eight focused hint tests also passed.
The source commit is c18cdc0b5bb3feb05074e4d2bcba07c0afcfba6b. This review does
not close the ticket or claim the delivery-set gate.

Canonical [PR #294](https://forge.adrianpk.com/adrianpk/rev/pulls/294) targets `dev` and
links the immutable report introduction. The ticket remains reviewing through
Slice 2 and the successful integrated controller gate.


## Slice 2 evidence

The [Slice 2 report](../../report/slices/rev-command-keybinding-hints/slice-2-acceptance.md)
records the native matrix, both-profile bounded PTY journeys, canonical usage
regressions and contextual Markdown/Spore guidance. Acceptance is implemented in
`2a66698097f1a66ec1efbf18dc61c9f33c4990f7`; guidance and UI02 reconciliation
are in `266f6384283bbdc9c791d1c2c9250d510bfd3023`. All 15 required usage tests
pass; each focused PTY journey restores the terminal and records only one named
identity. The ticket remains reviewing until the controller's combined gate.

Canonical [PR #295](https://forge.adrianpk.com/adrianpk/rev/pulls/295) targets `dev` and
pins the Slice 2 report introduction `543c5324ca45fbdf83c31fe272a3516b0c781f94`.
The current metadata identifies this active review; PR #294 and its original
branch remain recorded in the Slice 1 report and tracker.
