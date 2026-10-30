# Slice 60: complete the native Org control catalog

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-60-complete-controls`
PR: pending

## Purpose

Reconcile delivered native controls, the complete startup sample and embedded
template under ORG17 and ORG31. [Plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) own the approved task boundaries.

## Delivered Behavior

The control reference connects each shipped Org capability to its compiled
entry commands, focused context maps, typed preferences, palette roles and
feature references. It distinguishes accepted settings from fixed native policy.

## Implementation Notes

One compiled command catalog and the existing configuration loader remain the
owners. The reference sample is the executable's embedded template source.
Existing native suites establish feature behavior; catalog checks alone do not
prove terminal capability or optional tool success.

## Contracts Added or Changed

- ORG17: command, binding, preference and palette agreement.
- ORG31: complete documented controls and exact-candidate evidence.

## Files of Interest

- `docs/reference/org-controls.md`: capability and ownership map.
- `docs/reference/defaults.md`: exhaustive typed leaf and map catalog.
- `docs/reference/config.toml`: complete defaults and optional declarations.
- `scripts/check-config-reference.py`: packaged/reference consistency.

## Validation

Required ordered checks remain pending:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test commands --test org_editing --test org_agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused configuration reference and new catalog checks will use controller
receipts on a clean candidate. The full delivery gate remains controller-owned.

## Risks and Follow-ups

Optional programs and paths remain explicit user declarations. Example admission
does not establish that an external tool is installed or that a destination is
writable; those outcomes belong to the selected native action.
