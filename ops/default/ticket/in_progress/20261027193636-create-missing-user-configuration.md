---
id: TKT-20261027193636
title: Create the complete user configuration when missing
status: in_progress
ready_at: 2026-10-28T00:45:40Z
started_at: 2026-10-28T00:45:40Z
branch: feat/rev-input-16-bootstrap-template
pr: https://forge.adrianpk.com/adrianpk/rev/pulls/96
kind: task
severity: unclassified
priority: unclassified
scope: domain
tags: configuration, startup
source: chat
reported_at: 2026-10-27T19:36:36Z
commits: [43119ca646bfc23b105f0126d056d9acd7674a17, 554cc86bf72d7ce5851b9a67947a5a8cd7fd2107, cb3b791c7f258c4aca9aafb8378f3d099a795883]
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
The completed configuration contract currently forbids this startup write;
this is an explicit new capability, not an already implemented setting.

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

Slice 16 is implemented in canonical PR [#96](https://forge.adrianpk.com/adrianpk/rev/pulls/96), targeting dev. Its
[report](../../report/slices/rev-input-navigation/slice-16-bootstrap-template.md)
records the bounded compiled resource, complete default-equivalent values,
standalone native verification and pure loading guards. Required and focused
exact-head evidence follows PR-reference commits through the controller.
Integration is pending. This ticket stays in progress through primary slices
16-17; absent-file publication remains with the recorded next slice.
