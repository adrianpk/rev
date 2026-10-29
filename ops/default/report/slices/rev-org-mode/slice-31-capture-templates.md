# Slice 31: define native Org capture targets and templates

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-31-capture-templates`
PR: pending

## Purpose

Deliver typed capture admission and non-writing target discovery under the
[approved plan](../../../plan/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates)
and [tracker](../../../tracker/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates).

## Delivered Behavior

T31.1 admits typed native entry/item/checkitem/table-line/plain templates and
file/headline/outline/date-tree target data. Empty arrays select the reference
Task fallback, with an explicit missing destination. Separate assigning-source
roots qualify template files and default files without IO or path creation.

## Implementation Notes

Selection retains literal text and bounded data. No template language is
executed. The public capture choice is distinct from an editable source or
future draft. Exact keys, array replacement and per-leaf provenance follow the
existing configuration owner.

## Contracts Added or Changed

- ORG10: typed template/target selection and explicit destination entry.
- ORG17: admitted capture leaves and exhaustive defaults/sample coherence.

## Files of Interest

- `src/config/capture.rs`: native template kinds and admission.
- `src/org/capture.rs`: exact selection and explicit file entry.
- `tests/configuration/org_capture.rs`: defaults, rejection and source roots.
- `docs/reference/org-capture.md`: native data contract.

## Validation

- `cargo test --test configuration org_capture -- --nocapture`: three cases
  passed with zero failures or ignored tests.
- `cargo clippy --lib --test configuration -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter/nesting/complexity limits.
- `python3 scripts/check-config-reference.py`: passed after adding capture's
  typed leaves to the existing exhaustive checker; no schema assertion weakened.
- Final exact-head controller validation follows task commits and the canonical
  PR-reference follow-up.

## Risks and Follow-ups

T31.2 and T31.3 remain active. Editable expansion and publication belong to the
recorded successor slices. The delivery-set full gate has not run.
