---
id: TKT-20261027085234
title: Complete filesystem paths when opening a file
status: solved
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: completion, file-opening, minibuffer
source: chat
reported_at: 2026-10-27T08:52:34Z
ready_at: 2026-10-27T10:37:08Z
commits: [39cb2a1d1049409ec31ba8fccb7fb86a68cc2904, 9f365b86d4097f6c78a6777f848c651bcbe2bce3, e86c5451203cac1f64279a4cc6a6e29762ed8989]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Complete filesystem paths when opening a file

## Observed behavior

C-x C-f requires the user to type the path without filesystem completion.
Tab keeps the draft and displays `Enter a path; C-g cancels`, so discovering
directory entries and avoiding filename mistakes requires another tool.

See [Focused prompts](../../../../docs/reference/prompts.md) and
`src/session/prompt.rs`.

## Expected outcome

Provide filesystem completion in the find-file interaction. Users can complete
path components, inspect matching files and directories, navigate into a
directory, and choose a file without remembering its complete path.

## Acceptance criteria

- C-x C-f and M-x find-file use the same filesystem-aware completion behavior.
- Tab completes an unambiguous component or shared prefix. Ambiguous matches
  appear as selectable candidates rather than requiring an exact typed name.
- Files and directories are distinguishable. Completing or accepting a
  directory allows continued navigation; accepting a file uses the normal
  open/revisit handler.
- Candidate presentation is consistent with the separately requested vertical
  completion UI. Keyboard navigation keeps the query and selection usable.
- Relative and absolute paths, spaces, and Unicode filenames work without
  losing path components or confusing display cells with stored text.
- Users can still explicitly open a new, nonexistent file path. Completion
  does not restrict find-file to existing candidates or create a file merely
  by browsing or accepting a path.
- C-g cancels without changing document text, undo state or the selected
  buffer. Reopening an existing visit preserves its unsaved buffer contents.
- Unreadable or changed directories, missing entries, and no-match states
  produce useful feedback. Listing is bounded and does not recursively scan
  the filesystem to complete one path component.
- Resize and tiny geometry preserve prompt state and valid cursor placement.
- Validate path matching and directory traversal at native filesystem
  boundaries, plus one representative terminal journey for completion,
  file acceptance and cancellation.

## Related work

- [Vertical M-x completion](20261027084156-vertical-command-completion.md).
- [Panel presentation abstraction](20261027085102-panel-presentation-abstraction.md).


## Approved delivery

Ready for the single [configuration and interaction plan](../../plan/rev-configuration-ui.md).
[Editor Interaction](../../spec/editor-interaction.md) fixes UI03-UI04, including
geometry, navigation, matching, path acceptance and bounds. Assigned slices:
23, 24, 25. Use their recorded branches, task commits and
reports; no separate ticket branch or independent controller is required.

## Verified delivery evidence

- Slice 23: [#72](https://forge.adrianpk.com/adrianpk/rev/pulls/72); integrated `39cb2a1d1049409ec31ba8fccb7fb86a68cc2904`. [Report](../../report/slices/rev-configuration-ui/slice-23-path-candidates.md) records the delivered behavior and exact-head checks.
- Slice 24: [#73](https://forge.adrianpk.com/adrianpk/rev/pulls/73); integrated `9f365b86d4097f6c78a6777f848c651bcbe2bce3`. [Report](../../report/slices/rev-configuration-ui/slice-24-path-navigation.md) records the delivered behavior and exact-head checks.
- Slice 25: [#74](https://forge.adrianpk.com/adrianpk/rev/pulls/74); integrated `e86c5451203cac1f64279a4cc6a6e29762ed8989`. [Report](../../report/slices/rev-configuration-ui/slice-25-find-file-integration.md) records the delivered behavior and exact-head checks.
