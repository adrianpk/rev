# Slice 24: maintain Org habit schedules and history

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-24-habit-records`
PR: `#125`

## Purpose

Deliver habit schedules and source history under the
[approved plan](../../../plan/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history)
and [tracker](../../../tracker/rev-org-mode.md#slice-24-maintain-org-habit-schedules-and-history).

## Delivered Behavior

Unfinished tasks with local STYLE habit and one active repeating SCHEDULED
use the ordinary task completion controls. Daily, weekly, monthly and yearly
ranges retain calendar arithmetic and distinct +, ++ and .+ semantics. An
explicit maximum must exceed the minimum. Missing schedules, hourly periods,
finished reset states and invalid rescheduled calendar ranges reject the whole
transition before publication.

Completion preserves compatible source history and publishes reset state,
schedule, selected log, LAST_REPEAT and progress in one undo unit. Existing file
workflows, STARTUP/LOGGING choices, note cancellation and log drawer ownership
remain effective. Explicitly disabled logging leaves old observations intact.

The native history observation retains actual completion order and duplicate
same-day events. LAST_REPEAT is separate metadata, not invented completion
history. Absent days remain Unrecorded; malformed actual records are unresolved,
not fabricated completions or failed days. Literal examples, child entries and
nonfinished states do not contribute to the selected heading's history.

## Implementation Notes

`HabitRecord` is a revision-qualified disposable snapshot of current canonical
source, including unsaved/read-only source. It has no edit or file authority.
Observation admits 262144 entry bytes, 4096 actual completion records and 4096
bytes per history line. Incomplete/cancelled structure and stale revisions reject.

`OrgDay` supplies checked civil rollover without observing an ambient clock.
Typed/layered `org.time.extend_today_until` defaults to midnight and admits local
hours 0..23. Date-only habit recurrence uses that Org day; timed recurrence and
log stamps retain actual captured local time. The existing operation captures
clock and zone once. The actual scheduled source span identifies the candidate
whose calendar bounds must remain valid after repetition.

No dependency, independent habit store, periodic scan or history rewrite is
added. Task/source admission, canonical edits, file saves and shared-view mapping
remain with their existing owners.

Behavior references: [Org habit tracking](https://orgmode.org/manual/Tracking-your-habits.html),
[Org habit implementation](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org-habit.el)
and [Org task recurrence](https://raw.githubusercontent.com/emacs-mirror/emacs/master/lisp/org/org.el).

## Contracts Added or Changed

- ORG20: compatible habit recognition, calendar ranges and recorded history.
- ORG07: checked repetition and coherent canonical source publication.

## Files of Interest

- `src/org/time/habit.rs`: calendar ranges and explicit Org day.
- `src/org/visibility/tags/tasks/habits/`: canonical task/history admission.
- `src/org/agenda/habits.rs`: immutable source observations and day evidence.
- `tests/org_editing/habit_workflows.rs`: native completion, file and undo checks.

## Validation

T24.1: `5adee7cd157a5e1fddb603ad7ac0edc5fb31e19f`.
T24.2: `3a273633634370622a9eca19ed2c5bd43165d275`.
T24.3: `922b9101d7f49beb1ec167b030c17e7043c13110`.

Development checks passed:

- `cargo check`.
- `cargo test --test org_time --test org_editing habit --no-fail-fast`: fifteen
  selected cases passed, zero skips. Covers recurrence types, civil boundary,
  local workflows/logging, history ownership, duplicate events, stale/incomplete
  observations and exact entry/record/line bounds.
- `cargo test --test configuration org_time`: three selected cases passed;
  typed day-boundary layering, provenance, factory value and rejection included.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 122 schema entries, 46 RGB roles,
  181 bindings and 15408 exact packaged template bytes agree.
- Changed-source quality and `git diff --check` passed at unchanged limits.

Native completion checks include note cancellation, read-only rejection before
clock capture, exact saved BOM/CRLF source, preserved old note bytes, independent
peer anchors and complete undo/redo. Calendar ranges are checked again before
publishing the next occurrence. No terminal suite is required for this slice.

Development compilation caught fixture calls that did not use the native edit
API and a moved timestamp observation. Fixtures now use real prepare/publish,
borrowed observations and the existing read-only buffer setup. Source checks
identified complexity above the original limit; parsing, reset validation and
repeat logging now have their own cohesive owners. No assertion or bound was
weakened.

Final controller receipts bind the clean canonical PR head. Required order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

Habit graph rendering belongs to the next recorded slice. The delivery-set full
gate has not run.

[Canonical PR #125](https://forge.adrianpk.com/adrianpk/rev/pulls/125) targets dev.
Final required/focused controller receipts bind the clean PR-reference head;
verified integration remains pending.
