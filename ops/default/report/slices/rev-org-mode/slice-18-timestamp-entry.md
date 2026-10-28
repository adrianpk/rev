# Slice 18: enter and edit Org timestamps

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-18-timestamp-entry`
PR: pending

## Purpose

Enter and edit checked Org civil timestamps through native controls.

## Delivered Behavior

T18.1 delivers active/inactive timestamp, date/time range, schedule and deadline
entry. The native transient prompt projects three calendar months at 88 columns
or one on narrower surfaces, supports keyboard and visible-day mouse selection,
and preserves the originating source view through cancellation and resize.
Calendar inspection retains a selected date for explicit later insertion.

T18.2 adds native shifted-arrow field/day adjustments, explicit timestamp
commands, weekday normalization and successive-command range entry. Repeated
adjustments retain the selected semantic field across changed weekday/hour
widths. Context actions retain task, priority, tag and checkbox ownership when
point has no timestamp; literal source contexts reject structural fallthrough.

Date input accepts ISO/short dates, English months/weekdays, ISO week dates,
relative dates/hours and clock ranges. Empty input retains source spelling.
Changed dates receive a correct weekday; untouched weekday, clock, whitespace
and suffix spelling remain exact. Planning edits preserve neighboring facts,
drawers, line endings, BOM and EOF termination. Accepted edits publish through
the existing canonical transaction/history owner.

## Implementation Notes

One bounded source observation retains revision-qualified ranges, including both
ends of a connected date range. Every draft rechecks its selected view and source
before publication. Schedule/deadline use the existing structural heading and
planning ownership. They cannot fall through from unavailable literal contexts
to an ancestor heading.

Each operation captures its clock and complete zone once. Repeated local times
require an observed offset choice; missing times return to date entry. Offset
choices validate the operation while source remains an Org civil timestamp.
No ambient clock or zone is reread during prompt interpretation. Editing a
range end validates all retained endpoints before publication. Successive range
entry requires the same command, view, revision and point; source/view actions
revoke that authority.

Source timestamps admit 256 UTF-8 bytes each, complete ranges 514, free date
input 256, and source observation one logical line of at most 4096 bytes. The
calendar projects at most three six-week grids, clips small surfaces, and owns
no editable document or idle worker. Month/year calendar selection clamps to a
valid day; timestamp civil arithmetic retains its separate excess-day semantics.

The prompt behavior follows the [Org timestamp reference](https://orgmode.org/manual/Creating-Timestamps.html)
and [date/time prompt](https://orgmode.org/manual/The-date_002ftime-prompt.html).
An owned Emacs 31.1/Org 9.8.7 reference process and installed reference source
confirmed strict relative weekday occurrences and the short-year future window.
Native operation invokes no reference editor, external date process or FFI.

## Contracts Added or Changed

- ORG07: native checked timestamp/calendar entry with explicit local-time resolution.
- ORG17: working named/default/remapped controls and transient prompt ownership.
- Calendar prompts inherit admitted minibuffer editing; their local map admits
  calendar/navigation actions. A host calendar role cannot edit document text.

## Files of Interest

- `src/org/time/timestamp*`, `entry*`, `month.rs`: bounded source/date/calendar values.
- `src/org/timestamp*`, `visibility/tags/timestamps.rs`: qualified edit targets.
- `src/session/outline/timestamps*`: native prompts, retained zone and publication.
- `src/input*`, `src/display/calendar.rs`: focused routing and clipped projection.
- `tests/org_time/timestamps.rs`, `tests/org_editing/timestamps.rs`: public value and session workflows.
- `docs/reference/`: commands, bindings, configuration and operational reference.

## Validation

T18.1 development checks passed:

- `cargo check --quiet`.
- `cargo clippy --lib -- -D warnings`.
- `cargo test --test org_editing --test org_time timestamps -- --nocapture`:
  seven native session and eight timestamp/date-entry tests passed, zero skips.
- `cargo test --test commands --test configuration org -- --nocapture`:
  seven command and thirteen configuration tests passed, zero skips.
- `python3 scripts/check-config-reference.py`: 114 schema/catalog entries,
  46 active RGB roles, 163 default bindings and 14438 packaged template bytes agree.
- Focused changed-source quality passed with the existing safe-Rust, parameter,
  nesting and complexity thresholds.

Development failures exposed invalid-clock suffix fallback, a wrong relative
weekday/default clock, planning insertion after CRLF, stale map counts and lint
issues. Their focused checks passed after correction. T18.1 commit `3da11bf31957afb061b8a9b8c487a74ce4b801a1` also passed
clean-head controller receipt `Z7ZXSK5BB3YHKS4ZJ5ED5K7JAA` for its focused
editing/time, command/configuration and configuration-reference checks plus
`git diff --check`.

T18.2 development checks passed:

- `cargo clippy --lib -- -D warnings`.
- `cargo test --test org_editing --test org_time timestamp -- --nocapture`:
  15 native editing and 12 time tests passed, zero skips.
- `python3 scripts/check-config-reference.py`: 114 schema/catalog entries,
  46 RGB roles, 165 default bindings and 14486 packaged template bytes agree.
- Focused changed-source quality passed at the existing thresholds.

No final controller validation or terminal fixture evidence is claimed yet.

## Risks and Follow-ups

T18.3 broader native/display/terminal acceptance and final canonical PR validation
remain pending. The full delivery-set gate has not run.
