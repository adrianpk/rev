---
id: TKT-20261027103712
title: Configure bounded history rings and prompts
status: solved
kind: task
severity: unclassified
priority: normal
scope: domain
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:12Z
ready_at: 2026-10-27T10:37:12Z
commits: [726dbfbedf3192c25a45188ad143ef96aceb4940]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Configure bounded history rings and prompts

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG17, CFG20.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 16: Apply bounded history and prompt preferences; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 16: [#65](https://forge.adrianpk.com/adrianpk/rev/pulls/65); integrated `726dbfbedf3192c25a45188ad143ef96aceb4940`. [Report](../../report/slices/rev-configuration-ui/slice-16-bounded-preferences.md) records the delivered behavior and exact-head checks.
