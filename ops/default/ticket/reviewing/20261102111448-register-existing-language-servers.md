---
id: TKT-20261102111448
title: Existing server registration
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: persistence
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:48Z
ready_at: 2026-11-02T11:14:48Z
started_at: 2026-11-02T11:22:00Z
branch: feat/rev-language-server-installation-01-registration
reviewed_at: 2026-11-02T11:47:23Z
pr: pending
commits: [4c9e7120460c043776cef6bca0470bfeac1e2a9d, b91b6c51b2189f0bd50427555a38c2e36f4c4f81]
---

# Existing server registration

## Outcome

Add preferred-server/native-only and literal environment fields, whole-candidate validation, root provenance and a format-preserving conflict-checked settings writer.

## Current behavior and inputs

The optional stdio LSP client and exclusive semantic/native completion policy
are delivered. Guided management and managed installation are not yet delivered.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Add preferred-server/native-only and literal environment fields, whole-candidate validation, root provenance and a format-preserving conflict-checked settings writer.
- Expose guided existing/custom registration and scoped live application through the shared command owner; preserve runtime LSP enablement, exclusions, editor profile, includes and unrelated settings.
- Cover grammar-independent registration, source switching, configuration conflicts and symlink publication through native integration tests.
- Own requirements INS01, INS03, INS04, INS05; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 1; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisites are the delivered configuration, command, completion and LSP owners.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Validation

Native configuration/filesystem, both-profile prompt, source-switch and late-response
checks passed during implementation. The [Slice 1 report](../../report/slices/rev-language-server-installation/slice-1-registration.md)
records the checks. Controller validation of the exact clean PR head is required
before integration.
