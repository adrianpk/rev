---
id: TKT-20261028153707
title: Show the file-tree root expanded on first display
status: reviewing
kind: task
severity: unclassified
priority: unclassified
scope: ui
tags: tree, navigation, directories
source: chat
reported_at: 2026-10-28T15:37:07Z
ready_at: 2026-10-30T23:24:55Z
started_at: 2026-10-31T03:02:07Z
branch: feat/rev-wrap-navigation-05-directory-targets
reviewed_at: 2026-10-31T03:20:52Z
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/178
commits: [c5a8568c938772c69018c18fa5f2bb935b003632, 18ce4a7f1138511ef060b79fbde16755a8a9c494, 121c167275df9e2f19f5d5470859bdb13bb86b49]
---

# Show the file-tree root expanded on first display

## Reported behavior

When the user shows the file tree with C-x t t, only its collapsed root is
presented. An extra expansion action is needed before immediate files and
subdirectories can be browsed. The requested default is to show that first
level already expanded.

This improves the existing tree interaction independently of the pending
[file/directory opening ticket](20261028094803-open-directory-targets-from-cli-and-find-file.md).
That ticket must use the same initial expansion behavior when accepting a
directory from the CLI or find-file.

## Requested outcome

- On first display of a newly initialized tree root, expand the root and show
  its immediate children without a separate expansion command. This includes
  C-x t t and startup where the tree is initially visible.
- A newly selected directory root that is presented immediately receives the
  same initialization. It does not require opening a document first.
- First level means root plus immediate entries. Leave child directories
  collapsed; do not recursively enumerate their descendants.
- Hiding/showing an already initialized tree preserves its selected entry,
  viewport and deliberate expansion/collapse state. Initial expansion must
  not repeatedly override the user's subsequent navigation choices.
- Retain existing ordering, hidden-file preferences, alias/link handling and
  native keyboard/pointer interaction. Empty directories remain valid expanded
  roots with no children.
- Keep buffers, unsaved text and history intact. Directory observation belongs
  to initialization/show/root-change handling, not drawing or cursor movement.

## Current source observations

The captured baseline deliberately constructs a collapsed root in src/tree.rs.
src/tree/navigation.rs already expands one directory's immediate children on
explicit expansion, without reading already expanded entries. Showing/focusing
the tree does not currently initialize that expansion. This is a requested UX
change to the initial policy, not evidence of a newly reproduced runtime fault.

## Acceptance direction

- Start with ordinary default-hidden tree settings and show it using C-x t t:
  immediate entries are visible immediately, with child directories collapsed.
- Exercise initially visible configuration and the directory-opening entry
  points linked above; use the same initialization owner rather than duplicate
  CLI-only expansion behavior.
- Verify empty, large and unreadable roots, hidden-file preferences, Unicode
  and spaces, and an ordinary file alongside nested directories.
- Demonstrate that initial display does not read deeper directories or require
  recursive indexing. Keep work/storage within the admitted listing policy.
- Expand/collapse a child, hide/show the tree and retain that deliberate state.
  A deliberate later root collapse is not silently undone on every toggle.
- Report an unavailable/unreadable listing truthfully without presenting an
  unobserved directory as an empty success or losing document state.
- Update the initial-tree documentation and focused checks with the implemented
  policy; preserve pure drawing and ordinary focus/cancellation semantics.

## Delivery boundary

The shared [File and Directory Navigation specification](../../spec/editor-file-navigation.md)
owns NAV03 and the related directory-opening interaction. This ticket is ready:
first-display behavior and acceptance cases are defined. The accepted
navigation policies preserve this independent initialization behavior. Implementation belongs to
the selected joint delivery; no runtime change is claimed by this transition.

## Implemented resolution

Slice 5 implements the accepted directory/root and first-presentation policies.
The [Slice 5 report](../../report/slices/rev-wrap-navigation/slice-5-directory-targets.md)
records canonical source preservation, root/configuration precedence, bounded
first-level observation and native CLI/PTY evidence. Canonical [PR #178](https://forge.adrianpk.com/adrianpk/rev/pulls/178)
targets `dev`. Controller receipt `5BMRLUDPOYEPF7E4V3XQYLLXMZ` passed 336 native
tests, documentation checks and nine CLI/PTY cases at
`121c167275df9e2f19f5d5470859bdb13bb86b49`. The ticket remains under review
until verified merge; final PR-reference validation is recorded by the controller.
