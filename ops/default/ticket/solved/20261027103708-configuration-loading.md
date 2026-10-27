---
id: TKT-20261027103708
title: Load declarative XDG TOML configuration
status: solved
kind: task
severity: unclassified
priority: normal
scope: ops
tags: configuration
source: chat
reported_at: 2026-10-27T10:37:08Z
ready_at: 2026-10-27T10:37:08Z
commits: [9b2760232ac118abf381e9d043ed312d8e16745d, be0c8abfa16cf3993e52fade6e8aa55dba0a58f5, 035ca7fb9b69fa02424557a00744e6d6c35a6002, 100d9a7d939b96a16e8309abba7af2c9705dc6e3, 2736cc112f04968c2780a496a18020397d373cb9]
started_at: 2026-10-27T18:27:50Z
reviewed_at: 2026-10-27T18:27:50Z
reconciled_at: 2026-10-27T18:27:50Z
closed_at: 2026-10-27T18:27:50Z
resolution: fixed
---

# Load declarative XDG TOML configuration

## Expected outcome

Implement the accepted [configuration contract](../../spec/editor-configuration.md)
and [model catalog](../../spec/editor-configuration-model.md) for CFG01-CFG03, CFG07-CFG08, CFG14-CFG15, CFG21.
The approved [master plan](../../plan/rev-configuration-ui.md) defines exact task boundaries.

## Acceptance criteria

- Slice 1: Model typed configuration; pass its recorded focused checks and report its observable outcome.
- Slice 2: Validate TOML and diagnostics; pass its recorded focused checks and report its observable outcome.
- Slice 3: Load bounded include graphs; pass its recorded focused checks and report its observable outcome.
- Slice 4: Resolve layered settings; pass its recorded focused checks and report its observable outcome.
- Slice 20: Activate startup configuration; pass its recorded focused checks and report its observable outcome.
- Preserve canonical text, undo, saved-state identity and safe file publication.
- Record implementation commits, slice PRs and exact-candidate validation;
  move to solved only after every assigned slice integrates successfully.

## Delivery

Approved as part of the single configuration and interaction delivery. Slice
branches and reports are authoritative; this ticket does not request a separate
parallel branch or a second tracker. No future-inventory feature is included.

## Verified delivery evidence

- Slice 1: [#50](https://forge.adrianpk.com/adrianpk/rev/pulls/50); integrated `9b2760232ac118abf381e9d043ed312d8e16745d`. [Report](../../report/slices/rev-configuration-ui/slice-1-config-types.md) records the delivered behavior and exact-head checks.
- Slice 2: [#51](https://forge.adrianpk.com/adrianpk/rev/pulls/51); integrated `be0c8abfa16cf3993e52fade6e8aa55dba0a58f5`. [Report](../../report/slices/rev-configuration-ui/slice-2-toml-validation.md) records the delivered behavior and exact-head checks.
- Slice 3: [#52](https://forge.adrianpk.com/adrianpk/rev/pulls/52); integrated `035ca7fb9b69fa02424557a00744e6d6c35a6002`. [Report](../../report/slices/rev-configuration-ui/slice-3-config-includes.md) records the delivered behavior and exact-head checks.
- Slice 4: [#53](https://forge.adrianpk.com/adrianpk/rev/pulls/53); integrated `100d9a7d939b96a16e8309abba7af2c9705dc6e3`. [Report](../../report/slices/rev-configuration-ui/slice-4-config-resolution.md) records the delivered behavior and exact-head checks.
- Slice 20: [#69](https://forge.adrianpk.com/adrianpk/rev/pulls/69); integrated `2736cc112f04968c2780a496a18020397d373cb9`. [Report](../../report/slices/rev-configuration-ui/slice-20-startup-configuration.md) records the delivered behavior and exact-head checks.
