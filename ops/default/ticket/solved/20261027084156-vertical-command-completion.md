---
id: TKT-20261027084156
title: Show M-x command candidates in a vertical list
status: solved
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: minibuffer, completion
source: chat
reported_at: 2026-10-27T08:41:56Z
ready_at: 2026-10-27T10:37:08Z
commits: [f3b62e02f067e64bf515a28c8f0e5aaf4b18ba58, 0bb5aadf82d19849857d0dea63c62dd58f7ee511]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Show M-x command candidates in a vertical list

## Observed behavior

M-x shows up to three command candidates horizontally beside the query.
Long command names compete for the width of one terminal row, and the
presentation does not provide a visible vertical selection list.

The current behavior is described in
[Commands](../../../../docs/reference/commands.md) and implemented by
`src/input/palette.rs` and `src/display/prompt.rs`.

## Expected outcome

Present M-x candidates as a compact vertical list, with one command per row,
incremental filtering, a clearly selected row, and keyboard selection.
Use Helm/Ivy-style interaction and Odra's vertical command completion as
presentation references, while retaining rev's small built-in command model.

Odra's reference uses a bounded bottom panel with the query first and rows
below it. The relevant source references are its Candidate Session Reference,
`src/odra/candidate_ui.odin`, and `src/odra/interaction_workflow.odin`.
Only the interaction and layout are references; rev does not need Odra's
source-neutral navigation framework for this ticket.

## Acceptance criteria

- M-x displays matching command names on separate rows instead of an inline
  horizontal hint. Typing updates the candidate list.
- The query stays visible, selection is clear, and keyboard movement keeps
  the selected row in view without replacing the typed query merely to move.
- Return invokes the selected eligible command exactly once through the
  existing Rust command owner. C-g cancels and restores editing focus without
  changing document text or buffer history.
- Candidate rows fit the available terminal geometry. Resizing preserves
  usable input and selection; tiny terminals do not produce invalid cursor
  coordinates or lose editing state.
- Existing argument prompts, command matching, unknown-command handling,
  and command history remain usable. Any intentional interaction changes
  are agreed and documented before implementation.
- Validate filtering, selection and dispatch at native boundaries, plus one
  representative terminal journey covering invocation, cancellation and resize.



## Approved delivery

Ready for the single [configuration and interaction plan](../../plan/rev-configuration-ui.md).
[Editor Interaction](../../spec/editor-interaction.md) fixes UI02, including
geometry, navigation, matching, path acceptance and bounds. Assigned slices:
21, 22. Use their recorded branches, task commits and
reports; no separate ticket branch or independent controller is required.

## Verified delivery evidence

- Slice 21: [#70](https://forge.adrianpk.com/adrianpk/rev/pulls/70); integrated `f3b62e02f067e64bf515a28c8f0e5aaf4b18ba58`. [Report](../../report/slices/rev-configuration-ui/slice-21-vertical-command-list.md) records the delivered behavior and exact-head checks.
- Slice 22: [#71](https://forge.adrianpk.com/adrianpk/rev/pulls/71); integrated `0bb5aadf82d19849857d0dea63c62dd58f7ee511`. [Report](../../report/slices/rev-configuration-ui/slice-22-command-selection.md) records the delivered behavior and exact-head checks.
