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
Signed minute/hour/day/week shifts use checked civil arithmetic. Month/year
shifts normalize excess days forward, matching a clean, owned Org timestamp
adjustment observation: 2024-01-31 + one month becomes 2024-03-02;
2024-02-29 + one year becomes 2025-03-01. Entered invalid dates still reject.
No unit scale participates in a calendar shift.

OperationTime captures an injected fallible SystemTime clock once and retains
whole Unix seconds, flooring subsecond instants even before the epoch. Known
offset projection is explicit; it does not resolve a zone. Elapsed subtraction
uses instants, not wall-time subtraction. Task logging now uses this shared
capture and checked formatter with its existing bounded offset observation.
Selected timestamp output uses minute precision and English weekdays.

Work durations support checked totals and H:MM[:SS] display with unbounded
hours. Sub-precision fractions truncate, matching the owned Org observation;
values too large for displayed u64 ticks reject. Original entered spelling
remains unchanged. There is no dependency, idle job or editable shadow document.

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

T16.2 development checks passed:

- `cargo test --test org_time`: ten tests passed, zero skips.
- Focused changed-source quality passed with all original thresholds.
- Owned `emacs -Q --batch` observations established month/year rollover and
  minute/second duration display semantics; no reference implementation is copied.

## Risks and Follow-ups

Boundary and injected-clock verification continue in T16.3. Zone resolution
belongs to the recorded Slice 17.
