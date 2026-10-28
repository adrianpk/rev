# Slice 27: display native day and week Org agendas

Status: drafting
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

## Contracts Added or Changed

- ORG08: canonical source facts, finite calendar rows and visible incompleteness.
- ORG09: captured calendar context and source-qualified row locations.

## Files of Interest

- `src/org/agenda/calendar/`: source facts, planning rules and bounded projection.
- `src/org/structure.rs`: shared structural observation without false identity.
- `tests/org_agenda/calendar_rows.rs`: dates, planning, literal scope and limits.

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

The literal-block fixture initially contained an unescaped heading. It now uses
Org's comma escaping so the same exclusion assertion tests a valid literal
block. Malformed structure remains diagnosed. No assertion or limit was weakened.

## Risks and Follow-ups

Native projection, checked source return and real refresh follow in T27.2/T27.3.
The delivery-set full gate has not run.
