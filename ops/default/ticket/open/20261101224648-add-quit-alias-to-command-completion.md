---
id: TKT-20261101224648
title: Add quit alias to command completion
status: open
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: commands, completion, modal
source: chat
reported_at: 2026-11-01T22:46:48Z
commits: []
---

# Add quit alias to command completion

## Requested behavior

Expose `quit` as an alias for the existing `save-buffers-kill-terminal` command.
Users familiar with Vim should be able to type a short prefix in the colon
selector and use ordinary completion to reach `:quit`.

The command name is `quit`. Typing `:q` supplies a partial query; it does not
introduce a separate `q` command. The request is a convenient rev command name,
without adding Vim Ex spellings or force-quit behavior.

## Expected outcome

- Include `quit` in the existing shared command lookup and completion used by
  colon and M-x. Prefixes such as `q`, `qu` and `qui` can find the alias through
  normal candidate navigation and completion.
- Invoking `quit` uses the same session-exit coordinator as
  `save-buffers-kill-terminal` and `C-x C-c`.
- Preserve the existing command name and shortcut. Keep save, discard and
  cancellation decisions for modified buffers unchanged.
- Retain normal matching and completion behavior for other commands. This
  ticket does not introduce a separate modal command parser.

## Current source observations

`src/command.rs` names the exit command `save-buffers-kill-terminal`. Its current
name lookup and candidate matching use the fixed command catalog. The colon
selector uses the same command interface as M-x; the alias must participate in
that interface rather than exist only as a display label.

## Validation direction

Verify discovery, candidate completion and invocation of `quit`, including
prefixes that also match other commands. Compare both exit names and `C-x C-c`
for clean sessions and modified buffers. Check saving, discarding and canceling
an exit, including Escape cancellation in the modal profile. Preserve existing
command lookup, completion, keybindings and catalog consistency.

## Delivery boundary

Capture this as later work. Ticket creation does not implement the alias or
start a delivery loop.
