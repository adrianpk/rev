# Slice 16: model Org calendar dates and durations

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-16-civil-time`
PR: pending

## Purpose

Provide checked native calendar values and separate work-duration arithmetic.

## Delivered Behavior

The public org_time target exercises Gregorian dates in years 0001..9999,
wall times without leap seconds, and ordered half-open civil intervals. Invalid
entered dates and times reject without normalization or source mutation.
Existing bounded Org duration admission now belongs to the shared time owner;
the public org::Duration contract and stored source spelling remain intact.

## Implementation Notes

Dates use a checked serial-day identity and constant-time Gregorian conversion.
Civil values carry no implicit host zone. Ordered civil bounds do not claim an
elapsed duration across offset changes. Work-unit minute scales remain separate.
There is no dependency, idle job, source parser or editable shadow document.

## Contracts Added or Changed

- ORG07: checked Gregorian civil values underlying timestamp operations.
- ORG18: bounded arithmetic and explicit failures.
- ORG21: shared Org work-duration admission with configured unit scales.

## Files of Interest

- `src/org/time/`: checked civil and work-duration owners.
- `tests/org_time.rs`: public native integration target.

## Validation

T16.1 development checks passed:

- `cargo test --test org_time`: five tests passed, zero skips.
- Focused changed-source quality passed: original parameter/nesting/complexity
  and safe-Rust constraints remain enforced.

## Risks and Follow-ups

Calendar arithmetic, shared formatting and operation-clock integration continue
in T16.2. Zone resolution belongs to the recorded Slice 17.
