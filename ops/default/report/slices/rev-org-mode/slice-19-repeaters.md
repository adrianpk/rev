# Slice 19: reschedule repeated Org tasks correctly

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-19-repeaters`
PR: pending

## Purpose

Implement checked recurrence and coherent repeated-task completion under the
[approved plan](../../../plan/rev-org-mode.md#slice-19-reschedule-repeated-org-tasks-correctly)
and [tracker](../../../tracker/rev-org-mode.md#slice-19-reschedule-repeated-org-tasks-correctly).

## Delivered Behavior

T19.1 provides checked, source-preserving recurrence values and calculations.
`+` advances one stored interval, `++` advances whole intervals until future,
and `.+` uses completion date or its minute for hourly recurrence. Calendar
months/years retain excess-day normalization independently of work durations.
Inactive timestamps do not repeat. Hourly recurrence requires a source clock.

Warning/delay values expose their checked calendar boundaries. Ordinary `-`
delays remain in subsequent occurrences; first-only `--` delays are removed
when repeating. Habit minimum/maximum intervals retain calendar units and
reject an inverted window at use. Unknown or malformed source remains in the
document and receives a diagnostic when interpreted.

These are calculation-layer APIs. Native task-state completion, logging and
LAST_REPEAT integration remain T19.2 work; no completed command is claimed yet.

## Implementation Notes

The existing timestamp suffix owner delegates validation to typed recurrence
values. Formatting retains untouched weekday, clock and suffix spelling.
Clock ranges keep their civil length across midnight. Connected date-range
endpoints advance independently; an unmarked endpoint remains exact and an
inverted complete result rejects before publication.

Fixed-unit catch-up uses arithmetic rather than a backlog-sized loop. Calendar
catch-up normalizes month-end anchors before jumping across safe months. Its
normalization loop is bounded to 4800 steps, one full Gregorian month cycle;
source years remain 0001..9999 and interval counts are positive checked u32.
No operation observes ambient time, resolves a zone, writes source or starts
background work. Callers supply their captured completion observation and must
resolve local-time candidates before publication.

Behavior follows the [repeated-task reference](https://orgmode.org/manual/Repeated-tasks.html),
[scheduling delays](https://orgmode.org/manual/Deadlines-and-Scheduling.html)
and [habit interval ranges](https://orgmode.org/manual/Tracking-your-habits.html).
An owned Emacs 31.1/Org 9.8.7 reference process confirmed month-end normalization,
independent range endpoints, inactive/literal-source retention and first-only
scheduled-delay removal. A preliminary clock replacement did not control all
reference time helpers; its clock-dependent output is not validation evidence.
Native recurrence invokes no reference editor or external date process.

## Contracts Added or Changed

- ORG07: typed recurrence modes, explicit warning/delay values and checked calendar advancement.
- ORG20: checked minimum/maximum habit interval values without invented history.

## Files of Interest

- `src/org/time/repeat*`: suffix values, calendar calculations and bounded catch-up.
- `src/org/time/timestamp*`: source-preserving suffix admission and formatting.
- `tests/org_time/repeaters.rs`: independent mode, boundary, range and preservation cases.

## Validation

T19.1 development checks passed:

- `cargo check --quiet`.
- `cargo test --test org_time timestamps -- --nocapture`: eight existing timestamp tests passed, zero skips.
- `cargo test --test org_time repeater -- --nocapture`: nine recurrence tests passed, zero skips.
- `cargo clippy --lib -- -D warnings`.
- Focused changed-source quality passed with the existing safe-Rust, parameter,
  nesting and complexity thresholds after splitting calculation responsibilities.

No final canonical PR validation or native task-completion evidence is claimed.

## Risks and Follow-ups

T19.2 task completion/logging/LAST_REPEAT publication and T19.3 native acceptance
remain pending. The full delivery-set gate has not run.
