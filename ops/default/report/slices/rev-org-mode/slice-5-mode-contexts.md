# Slice 5: Activate native Org mode and contexts

Status: drafting
Delivery set: rev-org-mode
Plan: [Native Org Delivery](../../../plan/rev-org-mode.md)
Tracker: [Native Org Tracker](../../../tracker/rev-org-mode.md)
Branch: `feat/rev-org-05-mode-contexts`
PR: pending

## Purpose

Activate native Org through existing canonical buffers and validated input owners.

## Delivered Behavior

Visiting an admitted `.org` basename selects native Org mode through the existing
literal association resolver. User association rules retain precedence. Compiled
`org-mode` and `text-mode` commands select idempotent per-buffer behavior shared
by its views. Typed `org.enabled` defaults to true and rejects disabled entry.

## Implementation Notes

Mode choice is runtime buffer metadata. It survives settings reapplication and
file reassociation without changing canonical text, revisions, read-only state,
saved state or history. Text mode releases retained Org indexes. Mode selection
performs no source analysis and creates no worker or polling deadline.

## Contracts Added or Changed

- ORG01: native activation by admitted file identity or compiled command.
- ORG17: compiled command identities and typed availability configuration.

## Files of Interest

- `src/buffer/mode.rs`: per-buffer explicit choice and automatic activation.
- `src/session/mode.rs`: shared command handlers.
- `src/config.rs`, `src/config/associations.rs`: typed availability and suffix rule.
- `tests/org_source/mode.rs`: source preservation, choices and availability.

## Validation

- `cargo check --lib`
- `cargo test --test org_source mode::`: three tests passed.

## Risks and Follow-ups

T5.2 context routing and T5.3 combined editing validation remain in progress.
