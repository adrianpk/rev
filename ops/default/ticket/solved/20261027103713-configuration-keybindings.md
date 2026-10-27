---
id: TKT-20261027103713
title: Configure existing commands by input context
status: solved
kind: task
severity: unclassified
priority: normal
scope: ui
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:13Z
ready_at: 2026-10-27T10:37:13Z
commits: [09c26d8951fbbdf63eedaf245288cf9aa1a9fc0f, acebc81731bb936cb14d5b13f6060546d167ca7c, c26ba1b857f90c8baf2cb6cf2b3e75c40879a3ec]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Configure existing commands by input context

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG06, CFG12-CFG13, modified R22.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 17: Validate context keymaps; pass its recorded focused checks and report its observable outcome.
- Slice 18: Dispatch configurable editor bindings; pass its recorded focused checks and report its observable outcome.
- Slice 19: Dispatch transient and tree bindings; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 17: [#66](https://forge.adrianpk.com/adrianpk/rev/pulls/66); integrated `09c26d8951fbbdf63eedaf245288cf9aa1a9fc0f`. [Report](../../report/slices/rev-configuration-ui/slice-17-keymap-contracts.md) records the delivered behavior and exact-head checks.
- Slice 18: [#67](https://forge.adrianpk.com/adrianpk/rev/pulls/67); integrated `acebc81731bb936cb14d5b13f6060546d167ca7c`. [Report](../../report/slices/rev-configuration-ui/slice-18-editor-keymaps.md) records the delivered behavior and exact-head checks.
- Slice 19: [#68](https://forge.adrianpk.com/adrianpk/rev/pulls/68); integrated `c26ba1b857f90c8baf2cb6cf2b3e75c40879a3ec`. [Report](../../report/slices/rev-configuration-ui/slice-19-context-keymaps.md) records the delivered behavior and exact-head checks.
