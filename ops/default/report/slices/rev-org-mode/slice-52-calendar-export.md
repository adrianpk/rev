# Slice 52: export Org planning data as iCalendar

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-52-calendar-export`
PR: pending

## Purpose

Deliver checked calendar output under the
[approved plan](../../../plan/rev-org-mode.md#slice-52-export-org-planning-data-as-icalendar)
and [tracker](../../../tracker/rev-org-mode.md#slice-52-export-org-planning-data-as-icalendar).

## Delivered Behavior

T52.1 emits native RFC 5545 event/task components from immutable export
snapshots. Active timestamps and non-task planning become events; task keywords
select tasks with checked status, scheduled start and deadline. Date-only ranges
use exclusive ends, timed values resolve to UTC, and absent event end clocks use
the reference two-hour duration. Summaries, descriptions, locations, categories,
visibility and explicit IDs remain data. Literal source blocks are never scanned
as appointments or executed.

UIDs derive from checked explicit IDs, or the canonical source association and
heading text without task state/tags. Role suffixes distinguish tasks, planning
and ordered timestamps. Duplicate identities and digest collisions reject output.
Native property resolution and the complete task workflow retain their owners.
Named per-entry zones are captured during owned preparation, not rendering.

T52.2 exposes `org-export-to-icalendar` through the ordinary guarded file
workflow. Interval-compatible `+N` rules convert to RRULE. Catch-up/restart modes,
habit bounds, independent range endpoints, unsafe month/leap-day normalization,
variable-zone timed recurrence and conflicting task anchors remain complete source
with explicit limits. No future dates, alarms or completion instants are invented.
Per-entry UTC/IANA zones are captured once; fold/gap resolution rejects rather than
selecting an unobserved offset. Finished task declarations retain their true status.

## Implementation Notes

Wire emission folds at 75 UTF-8 octets with CRLF and one continuation space,
without splitting scalars. Text escapes backslashes, commas, semicolons and
newlines; unsupported controls reject output. Cancellation, deadlines and the
final output byte limit cover rendering. Components/headings are limited to
4096, explicit zones to 64, and ID values to 1024 bytes. No new Cargo dependency,
service, asset reading or execution authority is introduced.

## Contracts Added or Changed

- ORG25: native iCalendar output from checked current source.
- ORG07: retained zone observations and explicit recurrence conversion limits.

## Files of Interest

- `src/org/export/icalendar.rs`: native calendar ownership and UID checks.
- `src/org/export/icalendar/observe.rs`: native heading/property observations.
- `src/org/export/icalendar/wire.rs`: bounded escaped content-line emission.
- `tests/org_io/calendar_exports.rs`: public calendar workflows.

## Validation

- `cargo test --test org_io calendar_`: two native renderer/recurrence workflows passed, zero failures or ignored tests.
- Changed-source quality passed with unchanged native limits after small owner helpers replaced excessive function complexity.

- `cargo test --test org_io calendar_`: six renderer, zone, recurrence-limit and rejection workflows passed, zero failures or ignored tests.
- `cargo test --test commands visibility_keys`: compiled command catalog round-trip passed.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed.
- Changed-source quality passed with unchanged limits after the explicit range-end conversion was separated from ordinary event defaults.

## Risks and Follow-ups

None recorded.
