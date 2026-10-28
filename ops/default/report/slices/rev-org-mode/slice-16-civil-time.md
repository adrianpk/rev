# Slice 16: model Org calendar dates and durations

Status: reviewing
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

T16.3 development checks passed:

- `cargo test --test org_time`: all 19 tests passed, zero skips; 0.03 seconds
  of test execution includes a complete 146097-day Gregorian cycle.
- Focused changed-source quality passed at the existing thresholds.

An independent rejection fixture first reproduced a selected zero unit scale
being admitted by a public caller bypassing TOML validation. Selected work-unit
scales now require positive finite values even with a zero coefficient. The
same fixture passes for zero, negative, NaN and infinite policies. Existing
TOML policy validation is retained.

Boundary coverage includes invalid centuries/month ends, signed integer
extremes in every calendar unit, year-profile endpoints, exact parser byte/term
limits, nonfinite work totals and display overflow. Injected-clock fixtures
prove one capture, explicit failure, pre-epoch fractional flooring, known-offset
date changes, zero/negative elapsed intervals and elapsed/civil separation.
The prior private timestamp-conversion examples now exercise the shared public
formatter in this target.

The final required ordered checks will run through the controller after the
canonical PR-reference follow-up is committed and pushed. Focused same-HEAD
checks will additionally cover native editing and the existing four owned
RGB/NO_COLOR task terminal sessions because logging now uses the shared clock.
Receipt identities belong in Acta and the checkpoint without moving the
validated candidate afterward.

## Risks and Follow-ups

Work durations retain the existing floating-point minute profile, with display
truncation at selected precision; this is not exact subsecond accounting.
Civil shifts do not resolve zones or claim elapsed seconds. Zone observations,
folds/gaps and effective-zone capture belong to the recorded Slice 17.
