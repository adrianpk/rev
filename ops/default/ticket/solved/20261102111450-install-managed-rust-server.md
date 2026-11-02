---
id: TKT-20261102111450
title: Managed Rust installation
status: solved
kind: task
severity: unclassified
priority: normal
scope: ops
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:50Z
ready_at: 2026-11-02T11:14:50Z
in_progress_at: 2026-11-02T12:29:53Z
branch: feat/rev-language-server-installation-03-managed-rust
reviewing_at: 2026-11-02T14:04:26Z
pr: 224
reviewed_at: 2026-11-02T14:16:56Z
closed_at: 2026-11-02T14:16:56Z
resolution: fixed
commits: [377a474ab07a2486c69edeb8ea47268ba6a63941, df770c9dfe475b86e4faa935723ff92a4e48ff74, ee39a681179a7c3ecf6316d20004c900f1b0e529]
---

# Managed Rust installation

## Outcome

Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.

## Current behavior and inputs

The optional stdio LSP client, guided existing-server setup and exclusive
semantic/native completion policy are delivered. Managed acquisition is prepared
in canonical PR #224.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Implement versioned XDG payloads, schema-1 installation facts, OS-owned mutation lock and one bounded asynchronous management worker.
- Acquire an exact official Rust server release with integrity verification and finite structured helper processes; initialize/shutdown in an owned empty root before publication.
- Publish ready inventory before conflict-checked selection; cover missing prerequisites, corrupt downloads, invalid registry, failed probe and installation while LSP is disabled.
- Own requirements INS01, INS02, INS03, INS05, INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 3; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 2 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
All three tasks are implemented and focused development checks passed; see the
tracker for exact commands and the [Slice 3 report](../../report/slices/rev-language-server-installation/slice-3-managed-rust.md).
Controlled payloads verify acquisition/protocol mechanics and both native profiles;
no upstream Rust semantic integration is claimed. All four configured commands
passed at `ee39a681179a7c3ecf6316d20004c900f1b0e529` with receipt
`WJ6BNUUPPJYWBZTRYTZD2YCSCP`. Canonical PR: #224. The final PR-reference
head requires its own controller receipt before integration.

Final controller receipt `IKLQJ3752CS64MESVHEZXAIP5A` passed all configured
commands at `11bc8142ffe1b0371b050cea91a1ea06df96d19f`, merged by PR #224.
