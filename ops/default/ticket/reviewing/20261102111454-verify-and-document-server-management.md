---
id: TKT-20261102111454
title: Acceptance and user guidance
status: reviewing
kind: task
severity: unclassified
priority: normal
scope: docs
tags: lsp, installation
source: chat
reported_at: 2026-11-02T11:14:54Z
ready_at: 2026-11-02T11:14:54Z
started_at: 2026-11-02T16:39:21Z
branch: feat/rev-language-server-installation-07-acceptance
reviewed_at: 2026-11-02T17:03:59Z
pr: 228
commits: [2dd2314c1bf4c3ee71b6a6c008e7c217465a5d89, 91608bcd5e60e5f16f6ed08f4aa74f72ae1e3195, 8cd7b6ef855bd41bc5291390d03fc37353ef193b, c9e081bcfd6c919d40d9b610683fbd984d83d5e0]
---

# Acceptance and user guidance

## Outcome

Deliver one representative native/PTY management journey in both startup profiles, real Rust/Go integration, managed acquisition and grammar-independent semantic routing.

## Current behavior and inputs

The optional stdio LSP client and exclusive semantic/native completion policy
are delivered. Guided management, managed installation and lifecycle are delivered through
Slice 6. Preserve their verified contracts and ownership boundaries.
Use the [approved specification](../../spec/editor-language-server-installation.md)
and [model](../../spec/editor-language-server-installation-model.md).

## Acceptance and scope

- Deliver one representative native/PTY management journey in both startup profiles, real Rust/Go integration, managed acquisition and grammar-independent semantic routing.
- Document ordinary setup, custom languages, source selection, recipes and lifecycle; update complete configuration examples, command/reference catalogs, Unreleased and canonical Markdown-to-Spore guidance.
- Finish slice tickets before the configured integrated gate, record resource/ownership evidence and close only after controller validation of the exact candidate.
- Own requirements INS01-INS06; preserve any compatible language independently of the bundled grammar and recipe lists.
- Keep ordinary editing and its configuration independent of optional acquisition tools.

## Delivery and dependencies

Owning delivery: [Installation plan](../../plan/rev-language-server-installation.md),
Slice 7; [tracker](../../tracker/rev-language-server-installation.md).
Prerequisite: Slice 6 integrated; earlier contracts remain in force.
One slice PR uses the exact plan branch and title. The controller owns progress.

## Planned validation

Run the owning slice's exact configured commands and its acceptance cases.
Record actual commands, candidate and limitations in the slice report.
Both-profile debug native/PTY management, actual Rust/Go semantics and owned
shutdown passed. The guided chapter, full configuration example and Spore
navigation are implemented; 68 pages/413 links, all live inventories, seven
documentation regressions and all-example compilation passed. All seven configured
commands passed at `c9e081bcfd6c919d40d9b610683fbd984d83d5e0`, receipt
`NPV2LWJOXJ3LBCYVU2CYJJ3WKI` (39 passed; three opt-in checks ignored).
Canonical PR: https://forge.adrianpk.com/adrianpk/rev/pulls/228
The PR-reference head receives fresh configured validation; closure remains
post-merge.
See the [Slice 7 report](../../report/slices/rev-language-server-installation/slice-7-acceptance.md).
