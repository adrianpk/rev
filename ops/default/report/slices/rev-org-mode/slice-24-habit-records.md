# Slice 24: maintain Org habit schedules and history

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-24-habit-records`
PR: pending

## Purpose

Deliver habit schedules and source history under the
[approved plan](../../../plan/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history)
and [tracker](../../../tracker/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history).

## Delivered Behavior

T24.1 recognizes unfinished tasks with local STYLE habit and a single active,
repeating SCHEDULED timestamp. Daily, weekly, monthly and yearly repeats retain
calendar arithmetic and the distinct +, ++ and .+ modes. An explicit maximum
must exceed the minimum. Missing schedules, hourly periods and finished reset
states reject the complete task transition.

Completion reuses ordinary task publication: reset state, schedule, selected
history, LAST_REPEAT and progress remain one undo unit. Explicit repeat logging
preferences and file/local declarations retain their authority.

## Implementation Notes

`OrgDay` supplies checked civil rollover without observing an ambient clock.
`org.time.extend_today_until` defaults to midnight and admits local hours 0..23.
Date-only habit recurrence uses that Org day; timed recurrence and actual log
stamps retain the captured real civil time. The operation clock and zone remain
captured once by the existing completion owner.

Behavior references: [Org habit tracking](https://orgmode.org/manual/Tracking-your-habits.html),
[Org habit implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-habit.el)
and [Org task recurrence](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org.el).

## Contracts Added or Changed

- ORG20: compatible habit recognition, calendar ranges and recorded history.
- ORG07: checked repetition and coherent canonical source publication.

## Files of Interest

- `src/org/time/habit.rs`: calendar ranges and explicit Org day.
- `src/org/visibility/tags/tasks/habits.rs`: native task/source admission.
- `src/session/outline/tasks/repeat.rs`: completion-time recurrence.

## Validation

T24.1 development checks passed:

- `cargo check`.
- `cargo test --test org_time --test org_editing habit --no-fail-fast`: five
  selected schedule/day/source cases passed, zero skips.
- `python3 scripts/check-config-reference.py`: 122 schema entries, 46 RGB
  roles, 181 bindings and 15408 exact packaged template bytes agree.

## Risks and Follow-ups

Habit graph rendering belongs to the next recorded slice. The delivery-set full
gate has not run.

- `cargo test --test configuration org_time`: three selected cases passed.
- `cargo clippy --lib --test org_time --test org_editing --test configuration -- -D warnings`.
- Changed-source quality and `git diff --check` passed at unchanged limits.

Source checks identified two over-complex admission functions. Schedule parsing
and unfinished reset validation now have separate owners; assertions and limits
are unchanged.


T24.1 commit: `5adee7cd157a`.

T24.2 adds `HabitRecord`, a revision-qualified disposable observation. Only the
selected task's own section contributes, including ordinary task drawers;
literal examples, child entries and nonfinished states do not. Source-local
workflow declarations define finished keywords. Completion order and duplicate
same-day events remain intact; LAST_REPEAT remains independent metadata.
Missing observations remain Unrecorded, and malformed actual history contributes
an unresolved count rather than an invented completion or failed day.

Observation admits at most 262144 entry bytes, 4096 completion records and
4096 bytes per history line. Completion uses the same admission before source
publication. No independent store, clock read, periodic scan or history rewrite
is added. Compatible source lines survive ordinary completion byte-for-byte.

T24.2 development checks passed:

- `cargo test --test org_time --test org_editing habit --no-fail-fast`: eight
  selected cases passed, zero skips, including exact 4096/4097 record boundaries.
- `cargo clippy --lib --test org_time --test org_editing -- -D warnings`.
- Changed-source quality and `git diff --check` passed at original limits.

An initial fixture consumed its LAST_REPEAT value before querying the same
observation; it now borrows that value. No behavior assertion was removed.
