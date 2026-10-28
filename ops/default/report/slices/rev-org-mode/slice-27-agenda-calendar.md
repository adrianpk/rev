# Slice 27: display native day and week Org agendas

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-27-agenda-calendar`
PR: pending

## Purpose

Deliver read-only day/week calendars under the
[approved plan](../../../plan/rev-org-mode.md#slice-27-display-native-day-and-week-org-agendas)
and [tracker](../../../tracker/rev-org-mode.md#slice-27-display-native-day-and-week-org-agendas).

## Delivered Behavior

Calendar observations show active dates/ranges, scheduled and deadline entries,
unfinished overdue reminders and approaching deadline warnings on the captured
effective today. Weekly views start on Monday by default. Time grids and source
locators are derived data and never become another editable document.

## Implementation Notes

One query retains its instant, zone rules and effective date. Day/week movement
reprojects bounded immutable facts without reading a clock or advancing source.
Appointments can project repeating occurrences through the existing checked
calendar recurrence. Planning reminders use the stored nearest task occurrence,
explicit warning/delay suffixes and a fourteen-day default deadline warning.
Local workflows control completed-state reminders. COMMENT/archive subtrees,
metadata drawers and literal blocks do not become agenda entries.

The existing structural recognizer now exposes unqualified immutable facts for
owned disk observations without fabricating a canonical buffer stamp. Ordinary
buffer-qualified structural APIs retain their existing identity checks. The
same cancellation flag can propagate a finite compute deadline.

Typed preferences add day/week span, weekday 1..7 and 20000 retained facts/rows,
with a 100000 maximum. Source analysis keeps its existing byte/node/depth bounds.
Source and row limits remain explicitly incomplete. Date ranges clip to at most
seven visible days; recurrence projection caps at 4096 occurrences per fact.

Behavior references: [weekly/daily agendas](https://orgmode.org/manual/Weekly_002fdaily-agenda.html),
[planning semantics](https://orgmode.org/manual/Deadlines-and-Scheduling.html),
and [literal escaping](https://orgmode.org/manual/Literal-Examples.html).

The calendar owns one native pane and shares the existing cancellable job slot
for refresh, cached projection and bounded disk return. Compiled local controls
select spans/dates, scroll rows, refresh and visit source. Canonical buffer rows
check the exact revision. Disk return compares reread text as well as metadata,
then publishes normal BOM/CRLF-aware ingress. Changed view ownership rejects
pending source return. Peers retain their ordinary editing context. Refresh
alone recaptures time and source, including unsaved canonical text. Closing a
calendar restores its source view; deleting its owner retires the projection.

Date-range continuation days retain only their actual endpoint times. Hourly
repeats reuse the existing range-duration arithmetic. Both endpoint local times
are checked for gaps/folds without choosing offsets. Cancelled source analysis
remains incomplete when cached facts are projected onto another date.

## Contracts Added or Changed

- ORG08: canonical source facts, finite calendar rows and visible incompleteness.
- ORG09: captured calendar context and source-qualified row locations.

## Files of Interest

- `src/org/agenda/calendar/`: source facts, planning rules and bounded projection.
- `src/org/structure.rs`: shared structural observation without false identity.
- `tests/org_agenda/calendar_rows.rs`: dates, planning, literal scope and limits.
- `src/session/agenda/`: native view controls and pane ownership.
- `src/org/agenda/navigation.rs`: bounded checked disk source return.
- `tests/org_agenda/calendar_native.rs`: native controls and real file return.

## Validation

- `cargo test --test org_agenda --test configuration calendar_`: four focused
  cases passed, zero failures or ignored cases.
- `cargo clippy --lib --test org_agenda --test configuration -- -D warnings`:
  passed.
- Changed-source quality passes at the original safe-Rust, parameter, nesting
  and complexity limits after splitting projection responsibilities.
- `python3 scripts/check-config-reference.py`: 144 entries, 54 RGB roles,
  192 bindings and 16924 packaged bytes agree. The enum catalogue now recognizes
  the actual closed agenda-span type and its exact documented choices.

- `cargo test --test org_agenda native_calendar -- --nocapture`: two native
  calendar journeys passed. Stale-disk coverage uses an unopened target;
  an already open buffer correctly takes precedence over external disk edits.
- `cargo test --test org_agenda --test org_io agenda_ -- --nocapture`: fifteen
  existing source/agenda journeys passed.
- `cargo clippy --lib --test org_agenda --test org_io --test commands -- -D warnings`:
  passed after boxing the owned scan request and splitting control/visit helpers.
- The complete configuration reference now agrees on 201 bindings and 17182
  packaged bytes. Changed-source quality passes without exceptions.

The literal-block fixture initially contained an unescaped heading. It now uses
Org's comma escaping so the same exclusion assertion tests a valid literal
block. Malformed structure remains diagnosed. No assertion or limit was weakened.

- `cargo test --test org_agenda calendar_ -- --nocapture`: ten focused cases
  passed, zero ignored cases. They cover actual native refresh/source return,
  BOM/CRLF, shared canonical edits, stale revisions, narrow Unicode panes,
  captured clock counts, range endpoints, hourly range duration and zone gaps/folds.
- `cargo clippy --lib --test org_agenda --test org_time -- -D warnings`: passed.
  Changed-source quality passes at unchanged limits.

The range, hourly-duration and cancellation-completeness assertions first
reproduced defects, then passed with the corrections. Fixtures and acceptance
bounds remain unchanged.

Final controller evidence binds the clean canonical PR head. Required order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_agenda --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

The delivery-set full gate has not run.
