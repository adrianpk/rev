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

Org editing, agenda, capture, calendar and source-edit roles route through typed,
validated context maps. Editing roles inherit the admitted editor map before
local overrides. Non-text roles consume unbound text and paste. Transient prompts
and tree focus take precedence; workspace transitions retire pending chords,
Escape timing and held-key continuation.

## Implementation Notes

Mode choice is runtime buffer metadata. It survives settings reapplication and
file reassociation without changing canonical text, revisions, read-only state,
saved state or history. Text mode releases retained Org indexes. Mode selection
performs no source analysis and creates no worker or polling deadline.

Host-owned Org roles retain their identity beneath a transient prompt. The input
API selects routing without manufacturing agenda, calendar or draft state.
Feature owners activate their role when delivered; only implemented native
commands enter the catalog or bindings. All eight context maps validate together,
including inherited editor prefix conflicts and reserved cancellation.

## Contracts Added or Changed

- ORG01: native activation by admitted file identity or compiled command.
- ORG17: compiled identities, typed availability, context inheritance and prompt precedence.

## Files of Interest

- `src/buffer/mode.rs`: per-buffer explicit choice and automatic activation.
- `src/session/mode.rs`: shared command handlers.
- `src/config.rs`, `src/config/associations.rs`: typed availability and suffix rule.
- `src/input/org.rs` and `src/input/keymap/`: role routing, admission and inheritance.
- `tests/org_source/mode.rs`: source preservation, choices and availability.
- `tests/commands/org_contexts.rs`, `tests/configuration/org_keymaps.rs`: role
  isolation, prompt precedence, transitions, provenance and rejection.

## Validation

- `cargo check --lib`
- `cargo test --test org_source mode::`: three tests passed.
- `cargo test --test commands org_contexts::`: three tests passed.
- `cargo test --test configuration org_`: two tests passed.

## Risks and Follow-ups

T5.3 combined editing and catalog validation remains in progress.
