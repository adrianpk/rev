# Slice 17: resolve Org local times and zone changes

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-17-time-zones`
PR: pending

## Purpose

Observe native system/named time zones and resolve local times without guessing.

## Delivered Behavior

Native immutable Zone observations admit system localtime, captured local TZ,
explicit UTC and validated relative IANA names. Local TZ also admits absolute
TZif paths and bounded POSIX rules. Explicit selection takes precedence over TZ.

## Implementation Notes

TZif v1/v2/v3/v4 parsing checks headers, transitions, type indices, designations,
indicator consistency and footer agreement. Unknown -00 records and exhausted
coverage return errors. POSIX rules cover both hemispheres, month/week/weekday,
Julian and ordinal dates, explicit offsets, negative DST, all-year DST and signed
rule times. No implementation-defined implicit DST rule is invented.

Bounds are 1 MiB per regular file, 65536 transitions, 256 types, 4096 designation
bytes, 1024 POSIX bytes and 64 abbreviation bytes. Known offsets remain within
24 hours. Leap-second TZif files reject under the existing Unix-time profile.
Future recurrence evaluates at most eight events and retains no expanded years.
Named paths reject empty, . and .. components. Files are preflighted before
opening, checked again through the descriptor, and read with a bounded take.

The parser follows [RFC 9636](https://www.rfc-editor.org/rfc/rfc9636.html) and
local selection uses documented [TZ formats](https://sourceware.org/glibc/manual/latest/html_node/TZ-Variable.html).
Independent fixture encoders use the published byte format; no external reader
or copied implementation is part of native operation.

## Contracts Added or Changed

- ORG07: native explicit zone offsets and known/unknown coverage.
- ORG18: finite immutable observations without a service, dependency or idle loop.

## Files of Interest

- `src/org/time/zone/`: native TZif/POSIX parsing and explicit observation inputs.
- `tests/org_time/zones.rs`, `zone_observation.rs`: owned format/selection fixtures.

## Validation

T17.1 development checks passed:

- `cargo check --quiet`
- `cargo test --test org_time zone -- --nocapture`: five tests passed, zero skips.
- Focused changed-source quality passed at the original safe-Rust, parameter,
  nesting and complexity thresholds.

Fixtures cover exact transition boundaries, future footer recurrence, empty
coverage, unknown types, both hemispheres, negative/all-year DST, leap rules,
quoted names, fractional offsets and real owned system/named zone files.

## Risks and Follow-ups

Operation snapshots, explicit fold/gap resolution and configuration integration
continue in T17.2. Timestamp entry remains with the recorded Slice 18.
