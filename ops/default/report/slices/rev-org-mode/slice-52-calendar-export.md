# Slice 52: export Org planning data as iCalendar

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-52-calendar-export`
PR: `#153`

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

T52.3 verifies actual calendar bytes through an independent Python content-line
and Gregorian-date reader with hand-authored semantic fixtures. Thirteen workflows
cover Unicode folding/escaping, protocol-looking body data, leap/overnight bounds,
repeatable rendering, stable explicit/fallback UIDs, source-state changes, dirty
BOM/CRLF publication, prompt cancellation, stale observations, aliases, read-only
subtree/region selection, exact final-byte budgets and the 4096-component bound.
The existing RGB/NO_COLOR journey exports an owned unsaved appointment, checks its
actual calendar, then restores the established undo/source assertions.

## Implementation Notes

Wire emission folds at 75 UTF-8 octets with CRLF and one continuation space,
without splitting scalars. Text escapes backslashes, commas, semicolons and
newlines; unsupported controls reject output. Cancellation, deadlines and the
final output byte limit cover rendering. Components/headings are limited to
4096, explicit zones to 64, and ID values to 1024 bytes. No new Cargo dependency,
service, asset reading or execution authority is introduced.

Task commits: T52.1 `b11351ded602a5a5ba0b465de6c23fb8d3ad4eea`, T52.2 `d144051b63bab6f993220fb470f9f66c1cbda9a4`, T52.3 `3a15ab3d7a2b752154e1ac5ff50068ad14d52fac`.

Canonical PR: [#153](https://forge.adrianpk.com/adrianpk/rev/pulls/153).

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

- `cargo test --test org_io calendar_`: thirteen calendar, independent-reader and guarded-publication workflows passed, zero failures or ignored tests.
- `cargo clippy --lib --test org_io --test commands -- -D warnings`: passed with the completed workflow fixtures.
- Changed-source quality passed with unchanged limits. The selected-scope assertion now checks SUMMARY separately from the protocol name VTODO, retaining the task-keyword suppression assertion and all source/view/history checks.

Verified integration: canonical PR [#153](https://forge.adrianpk.com/adrianpk/rev/pulls/153)
merged into `dev` at `ed455d3630328723e8f1cd002e43dd9fd0a08843`.
Required exact-head receipt `FVWINGZQNWSKMB7IWS6MNKS4IT` passed all five
ordered checks and 221 tests. Focused receipt `WNC26EXZKIFKQBZCKXTGL4XEZ2`
passed 280 command/configuration tests, configuration references and two owned
RGB/NO_COLOR sessions. Zero failures, ignored tests or skipped sessions.

## Risks and Follow-ups

None recorded.
