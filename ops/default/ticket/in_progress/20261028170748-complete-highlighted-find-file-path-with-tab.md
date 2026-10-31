---
id: TKT-20261028170748
title: Complete the highlighted find-file path with Tab
status: in_progress
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: completion, paths, minibuffer
source: chat
reported_at: 2026-10-28T17:07:48Z
ready_at: 2026-10-30T23:24:55Z
started_at: 2026-10-31T03:31:51Z
branch: feat/rev-wrap-navigation-06-path-tab
commits: []
---

# Complete the highlighted find-file path with Tab

## Reported behavior

C-x C-f finds matching paths while the user types, but the candidate list does
not provide the expected completion workflow. Tab should insert the first
suggestion without requiring more manual path typing. After navigating the
list, the user should be able to complete that selected suggestion and continue
through directory components in the same prompt.

The request concerns candidate completion ergonomics, not absent matching.
Do not replace the matching engine or change its ranking merely to address
this interaction. The owner requests the familiar selected-candidate behavior
used by Odra's filesystem path navigator.

## Requested outcome

- A nonempty candidate list has a visibly selected first result by default.
  Typing still edits the user's draft; displaying a suggestion alone does not
  insert it automatically.
- Tab completes the currently selected candidate. Without manual candidate
  navigation, that is the first visible suggestion, including when several
  matches share a prefix. It must not stop at an unchanged shared prefix while
  leaving the visible suggestion unusable.
- Up/Down and C-p/C-n move the highlighted selection without accepting a path.
  The next Tab completes that choice rather than reverting to the first row
  or interpreting it as another incomplete text query.
- Completing a directory adds its separator, keeps find-file active, refreshes
  the immediate child candidates and permits further typing, navigation and
  Tab completion. Each explicit step traverses only the selected component.
- Completing a file fills the draft without opening it. Return remains the
  explicit acceptance action; completion must not invoke a file open, accept
  a final directory target, save a buffer or create a file implicitly.
- Retain literal path components, absolute/relative spelling, spaces, Unicode,
  query cursor handling and existing no-match/new-file behavior. Candidate
  labels and their visual markers are not inserted as path text.
- Preserve the chosen candidate's identity across admitted refreshes. If it
  disappears, report that outcome and preserve the draft instead of completing
  an unrelated entry at the same row index.

## Existing contract and source observations

[Editor Interaction UI03](../../spec/editor-interaction.md) currently permits
Tab to complete a unique name or the longest shared component prefix. This
ticket deliberately requests selected-candidate completion by default; prepare
the corresponding contract amendment during implementation planning. The
[earlier path-completion delivery](../solved/20261027085234-file-path-completion.md)
retains its historical solved state and validation evidence.

The inspected baseline separates an implicit first row from an explicitly
chosen candidate in src/file/completion/navigation.rs. The adjacent
navigation/complete.rs implementation supports a unique/explicit selection but otherwise
computes a shared prefix, which may produce no draft update. Existing native
tests assert that policy and also cover explicit selection. That source does
not establish that the owner's reported full keyboard interaction works.
Reproduce it through the real prompt/event route rather than changing only a
low-level helper or claiming selection support is absent everywhere.

Odra's filesystem-path-navigation contract selects the first result by default
and makes Tab complete the selected file/directory without opening a file.
Use that behavioral reference without importing its implementation stack.

## Acceptance direction

- C-x C-f and M-x find-file use the same native completion interaction.
- Given alpha/ and alpine/ matching an al draft, Tab completes the selected
  first directory and exposes its children, rather than stopping at alp.
- Move to the other result with keyboard navigation, press Tab and continue
  completing a nested path under that selected directory.
- Complete a file and verify that the prompt stays open until Return. Tab,
  selection movement and cancellation must not change the active document,
  dirty contents, history or filesystem.
- Verify query edits update candidates/selection coherently without silently
  inserting the current first suggestion; resizing/clipping must not alter
  which candidate Tab will complete.
- Cover relative/absolute paths, spaces, Unicode, directories, files, no
  matches, valid new paths, changed/unreadable directories and disappeared
  selected candidates under existing bounded observation rules.
- Include one representative real-terminal journey: type a prefix, Tab into
  a directory, navigate to a different child, complete it, then accept/cancel.
  Native helper assertions alone are insufficient evidence of this UX.
- Coordinate directory completion versus final directory acceptance with the
  [directory-opening ticket](../solved/20261028094803-open-directory-targets-from-cli-and-find-file.md).
  Preserve the distinction between Tab browsing/completion and Return acceptance.

## Delivery boundary

The shared [File and Directory Navigation specification](../../spec/editor-file-navigation.md)
owns NAV04 and the distinction between completion and opening. This ticket is
ready: selected-candidate Tab behavior is defined independently of the final
directory-acceptance gesture. Runtime changes and UI03 reconciliation belong to
the selected joint delivery; historical delivery records remain intact.
