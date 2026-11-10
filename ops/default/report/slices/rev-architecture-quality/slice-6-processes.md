# Slice 6: External process workflows

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-06-processes`
PR: pending

## Purpose

Resolve the selected ownership, conversion, guard-lifetime and source-bound
findings in native AI/Pi, LSP, shell, terminal and process owners and their session
adapters.

## Delivered Behavior

Implementation pending.

## Implementation Notes

Preserve generation-bound worker IO, explicit mailbox ownership, finite child
cleanup and cancellation, environment redaction and terminal restoration.

## Contracts Added or Changed

No contract change is authorized. Preserve package APIs, plugin/SDK protocols,
external configuration and existing resource and source-policy limits.

## Files of Interest

- `src/{ai,lsp,process,shell,terminal}.rs` and their child modules.
- `src/session/{ai,lsp,shell,terminals}.rs` and their child modules.

## Validation

Pending: the four frozen ordered Slice 6 commands in the approved plan and
controller instruction. Final evidence must belong to the exact clean PR head.

## Risks and Follow-ups

Implementation and focused acceptance remain pending. The controller owns merge,
full delivery-set validation and successor startup.
