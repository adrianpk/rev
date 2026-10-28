---
id: TKT-20261027193636
title: Create the complete user configuration when missing
status: solved
reviewed_at: 2026-10-28T01:18:47Z
closed_at: 2026-10-28T01:18:47Z
resolution: fixed
ready_at: 2026-10-28T00:45:40Z
started_at: 2026-10-28T00:45:40Z
branch: feat/rev-input-17-bootstrap-publication
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/97
kind: task
severity: unclassified
priority: unclassified
scope: domain
tags: configuration, startup
source: chat
reported_at: 2026-10-27T19:36:36Z
commits: [43119ca646bfc23b105f0126d056d9acd7674a17, 554cc86bf72d7ce5851b9a67947a5a8cd7fd2107, cb3b791c7f258c4aca9aafb8378f3d099a795883, bd888985abd2db85433b5d41130c740c9ad62eee, 81348a8a665727d9f26277ccb6c95f02f62ea0b5, 7be5674e876dd3a9cdefb0ff7ce6b4587120a726]
---

# Create the complete user configuration when missing

## Requested outcome

On normal editor startup, create the user configuration at the existing
resolved XDG path only when no entry exists there. Use the complete supported
reference sample, including explicit factory palette values and commented
optional examples. Existing user content must remain untouched.

## Specification

[Configuration Bootstrap](../../spec/editor-configuration-bootstrap.md) defines
the required CFG01/CFG14/CFG15 replacements and generated-template guarantees.
It replaces the former prohibition on startup creation while retaining read-only
configuration resolution/loading and startup-only settings application.

## Acceptance direction

- Honor the existing XDG resolution and default config.toml location.
- Create only the missing configuration parent directories and file.
- Package the complete template with the binary; do not require a checkout,
  installed documentation file or network access at runtime.
- Generated settings must resolve to the same factory defaults as no file.
- Preserve every existing destination, including empty or invalid user files.
- Verify no-clobber publication, concurrent first starts and creation failure.
- Keep a visible diagnostic plus usable defaults if provisioning fails,
  consistent with the existing startup recovery policy.
- Keep normal loading read-only and configuration application startup-only.

## Delivery boundary

Selected for the same [combined twenty-slice delivery](../../plan/rev-input-navigation.md)
and [tracker](../../tracker/rev-input-navigation.md) as the other six current
work items. Follow its assigned primary slices, lifecycle and controller-owned
integration/acceptance; do not create a separate successor run for this ticket.


## Primary slice progress

Slice 16 integrated through canonical PR [#96](https://forge.adrianpk.com/adrianpk/rev/pulls/96)
at `120101a17b85a7775a6afccda618cd59709428fe`. Its
[report](../../report/slices/rev-input-navigation/slice-16-bootstrap-template.md)
records the bounded compiled resource, native default equivalence and read-only
loading guards. Canonical evidence `BFCV7FDE46KJARMTSXDNHKRZJF` and focused evidence
`EE7VWXG2W7Z6PXYSFJLI5IXVPA` passed on that candidate. Slice 17 subsequently integrated the absent-file
publication and race/failure guarantees. Both primary slices are complete.

Slice 17 integrated through canonical PR [#97](https://forge.adrianpk.com/adrianpk/rev/pulls/97)
at `b682ccb3bfd1a2df5752354354fc09d2cc606a58`. Its
[report](../../report/slices/rev-input-navigation/slice-17-bootstrap-publication.md)
records native absent-root publication, no-clobber preservation, complete
fallback diagnostics and isolated filesystem/process fault and race checks.
Canonical evidence `2SKMXAMP6SD6BZWF3F5Q6Q5HS3` and focused evidence
`DZVKE2NXRHA4SDAMCWOALZV2SU` passed on that candidate. The ticket is solved after
both assigned primary integrations; combined terminal and resource acceptance
continues under the existing delivery plan.
