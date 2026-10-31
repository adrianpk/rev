---
id: TKT-20261028094803
title: Open directories from startup and find-file
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: files, directories, tree
source: chat
reported_at: 2026-10-28T09:48:03Z
ready_at: 2026-10-30T23:41:32Z
started_at: 2026-10-31T03:02:07Z
branch: feat/rev-wrap-navigation-05-directory-targets
reviewed_at: 2026-10-31T03:20:52Z
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/178
commits: [c5a8568c938772c69018c18fa5f2bb935b003632, 18ce4a7f1138511ef060b79fbde16755a8a9c494, 121c167275df9e2f19f5d5470859bdb13bb86b49]
---

# Open directories from startup and find-file

## Requested outcome

Accept a directory as a complete opening target alongside a file. The user
must be able to choose the directory itself without being required to select
an ordinary document inside it.

Support these entry paths through coherent native behavior:

- `rev` starts normally; `C-x C-f` can then open either a file or a directory.
- `rev .` opens the current directory as the visible, expanded file-tree root.
- `rev ./project` and `rev /path/to/project` open the selected directory with
  its tree visible and first level expanded.
- `rev ./project/notes.txt` opens the file and associates the tree with its
  containing directory when no directory was explicitly chosen.
- `rev /path/to/project/notes.txt` has the same file-target behavior. Absolute
  versus relative spelling does not determine whether a target is a directory.
- `C-x C-f` accepts a selected or fully entered directory as the final target,
  while retaining file/directory completion and ordinary file opening.

Opening a directory should present its tree without requiring a document
selection. Existing documents, unsaved edits, history and views remain usable.
The directory itself is a navigation context, not ordinary editable file text.

## Initial tree visibility and expansion

An explicit directory target requests browsing: show that directory's tree
with its root expanded and immediate entries visible. This applies to CLI
directory targets and a directory accepted through C-x C-f, even when ordinary
startup would hide the tree. Child directories remain collapsed until selected
for expansion; do not recursively load a directory hierarchy.

A file target opens the document and does not itself force the browser to
appear. The ordinary default remains file-focused with a hidden tree; an
explicit configured visible-tree preference or an already visible browser
retains its normal effect. Opening another file must not forcibly hide an
existing browsing context. Visibility choices do not rewrite configuration.

Manual tree display in the existing editor must also show the first level
without an extra root-expansion action. The independent
[initial tree expansion ticket](20261028153707-expand-file-tree-root-on-first-display.md)
owns that behavior. Integrate it with directory opening rather than making
CLI directories a special case. Respect temporary narrow-terminal geometry;
retain the requested browsing state without corrupting editing layout.

## Tree root precedence

An explicit directory choice becomes the session's tree root and takes
precedence over an automatic file-parent choice. Opening a file beneath that
chosen root must not unexpectedly narrow the tree to the file's immediate
subdirectory. A later explicit directory choice replaces the chosen root.
Without an explicit directory choice, opening a file associates the tree with
that file's parent directory.

Directory selection should provide a usable browsing context for subsequent
file opening, including an initially empty editor. Reconcile the behavior with
the existing configured tree-root policy and document its precedence rather
than silently removing a configurable behavior.

## Current source observations

Static inspection of the captured baseline shows:

- `src/session/path.rs`: directory acceptance currently descends within the
  find-file prompt instead of completing a directory-open operation.
- `src/session/startup.rs`: a supplied startup path is passed to ordinary file
  loading; no distinct directory-opening operation is selected there.
- `src/tree.rs`: startup root selection already distinguishes initial file
  parent and cwd according to the existing tree policy.

These are source observations, not a new runtime reproduction or proof that
all paths have been tested. They identify the existing owners to reconcile.

## Acceptance direction

- Verify the CLI and `C-x C-f` paths above with relative/absolute paths,
  spaces, Unicode, existing files and directories, and normal new-file behavior.
- Verify `rev .`, a relative directory and an absolute directory all present
  an expanded first-level tree, while relative/absolute file targets open the
  file without an implicit browser reveal. Respect explicit visibility settings.
- Accepting a directory through C-x C-f shows the same expanded browsing
  context; C-x t t initializes expansion correctly in ordinary file sessions.
- Make accepting a directory and navigating within it usable and unambiguous.
  Return accepts a directory as the final opening target; Tab completes/descends
  while retaining the prompt. Keep the two actions distinct.
- Distinguish automatic file-parent rooting from an explicit directory root.
  Verify nested files and later changes of the explicit root.
- Keep canonical buffer reuse and checked alias/symlink handling; directory
  selection must not introduce implicit recursive scanning.
- On cancellation, unavailable paths or unreadable directories, preserve the
  prior root, document views, dirty state and history, with useful feedback.
- Preserve bounded, lazy tree observations and both keyboard and pointer use.

## Delivery boundary

The shared [File and Directory Navigation specification](../../spec/editor-file-navigation.md)
fixes directory identity, root precedence and prompt acceptance. This ticket
is ready and owned by Slice 5 of [Wrap and Navigation](../../plan/rev-wrap-navigation.md).
Versioning readiness does not implement it or alter completed Org records.

## Implemented resolution

Slice 5 implements the accepted directory/root and first-presentation policies.
The [Slice 5 report](../../report/slices/rev-wrap-navigation/slice-5-directory-targets.md)
records canonical source preservation, root/configuration precedence, bounded
first-level observation and native CLI/PTY evidence. Canonical [PR #178](https://forge.adrianpk.com/adrianpk/rev/pulls/178)
targets `dev`. Controller receipt `5BMRLUDPOYEPF7E4V3XQYLLXMZ` passed 336 native
tests, documentation checks and nine CLI/PTY cases at
`121c167275df9e2f19f5d5470859bdb13bb86b49`. The ticket remains under review
until verified merge; final PR-reference validation is recorded by the controller.
