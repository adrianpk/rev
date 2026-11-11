# Slice 10: Staged enforcement and acceptance

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `chore/rev-architecture-quality-10-enforcement`
PR: pending

## Purpose

Enforce the completed native workspace quality policy at the staged commit
boundary, without changing editor behavior or user content.

## Delivered Behavior

Pending implementation of inherited lint policy, canonical commands and exact
index-snapshot commit checks.

## Implementation Notes

Slice 9 is integrated through canonical PR #23. Implementation uses the recorded
Slice 10 branch and a dedicated worktree. Hook installation, integrated gate and
final delivery-set closure remain controller-owned.

## Contracts Added or Changed

Pending; preserve source bounds, resource limits and all frozen checks.

## Files of Interest

- `Cargo.toml` and `crates/rev-plugin-sdk/Cargo.toml`: inherited policy.
- `Makefile`, `scripts/` and `.githooks/`: commit enforcement.
- `AGENTS.md` and `.agents/`: repository architecture and quality rules.

## Validation

Pending the five frozen ordered checks recorded in the approved plan.

## Risks and Follow-ups

The controller must verify local hook activation after integration and execute
the unchanged full gate before delivery-set closure.
