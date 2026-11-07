---
id: TKT-20261101224648
title: Add quit alias to command completion
status: solved
resolution: fixed
closed_at: 2026-11-07T21:59:14Z
reviewed_at: 2026-11-07T21:59:14Z
started_at: 2026-11-07T21:50:54Z
branch: dev
ready_at: 2026-11-07T21:50:54Z
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: commands, completion, modal
source: chat
reported_at: 2026-11-01T22:46:48Z
commits: [ff19b4cd74313aadaab9455c346661c013250a32]
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

## Capture-time source observations

`src/command.rs` named the exit command `save-buffers-kill-terminal`. Its
name lookup and candidate matching use the fixed command catalog. The colon
selector uses the same command interface as M-x; the alias must participate in
that interface rather than exist only as a display label.

## Validation direction

Verify discovery, candidate completion and invocation of `quit`, including
prefixes that also match other commands. Compare both exit names and `C-x C-c`
for clean sessions and modified buffers. Check saving, discarding and canceling
an exit, including Escape cancellation in the modal profile. Preserve existing
command lookup, completion, keybindings and catalog consistency.

## Delivery

The owner selected this bounded ticket for implementation on 2026-11-07.
It is delivered directly on dev with the existing command catalog and exit owner.

## Resolution

Implemented in `ff19b4cd74313aadaab9455c346661c013250a32`. The quit spelling stays visible in the
selector and its usage history, while dispatch, configured bindings, descriptions
and shortcut inspection resolve to the existing guarded exit action.

## Validation

- Native command tests cover q/qu/qui completion, both editing profiles, shared
  descriptions, effective shortcuts, configured alias bindings and absent Ex spellings.
- Native recovery tests compare both names and C-x C-c for clean exit, save,
  discard and cancellation, including the normal modal Escape deadline.
- Existing exit recovery, selector selection, command catalog and description
  guards pass. Org startup and repeated drawer-command regressions also pass.
- Two real terminal sessions complete qui to quit, show the inherited shortcut,
  exit successfully and restore terminal state without changing source files.
- Formatting, Clippy, the 623-name documentation catalog and local Markdown-derived
  Spore guide build pass.
