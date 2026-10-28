# Slice 19: reschedule repeated Org tasks correctly

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-19-repeaters`
PR: `#120`

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

T19.2 connects native task completion to one transaction: advance owned active
repeaters, reset to the first unfinished or selected REPEAT_TO_STATE keyword,
remove stale CLOSED/non-repeating SCHEDULED, record selected history/LAST_REPEAT
and derive parent progress from the reset state. Configured repeat logging and
STARTUP/LOGGING overrides retain cancellable notes and ordinary drawer ownership.
One captured clock/zone survives all offset, missing-time correction and note
stages. Stale drafts and read-only source cannot publish partial changes.

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
The calculation layer observes no ambient time and writes no source. Native
completion supplies one captured observation and resolves every next endpoint
before publication. A selected entry admits 262144 UTF-8 bytes and 64 repeating
ranges; literals and child entries do not contribute. Planning edits share their
existing bounded source owner, property edits use the existing property owner,
and all edits publish through the canonical buffer/history transaction.

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
- `src/org/visibility/tags/tasks/repeat*`, `src/session/outline/tasks/repeat*`: qualified observation, atomic planning and retained completion stages.
- `tests/org_editing/repeat*`, `scripts/configuration_ui/org_repeats.py`: native and terminal acceptance.

## Validation

T19.2 commit: `553f20ea028f8af8d9d19ee7946c933e19a1b0c1`.

T19.1 development checks passed:

- `cargo check --quiet`.
- `cargo test --test org_time timestamps -- --nocapture`: eight existing timestamp tests passed, zero skips.
- `cargo test --test org_time repeater -- --nocapture`: nine recurrence tests passed, zero skips.
- `cargo clippy --lib -- -D warnings`.
- Focused changed-source quality passed with the existing safe-Rust, parameter,
  nesting and complexity thresholds after splitting calculation responsibilities.

T19.1 commit `b0479ab5f842c5117530f18f52cf28c3754df8f3` also passed clean-head
controller receipt `BJXI7CB4LF5H5CV54ZPCNAEUGB`: all 59 time tests, 23 timestamp
editing tests and `git diff --check`.

T19.2 development checks passed:

- `cargo test --test org_editing --test org_time repeat -- --nocapture`: 12
  editing and nine time tests passed, including eleven new completion/offset/
  correction/logging cases and the retained repeated-minute regression; zero skips.
- `cargo test --test org_editing --test configuration task -- --nocapture`:
  13 shared editing and two configuration tests passed, zero skips.
- `cargo clippy --lib -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 115 schema/catalog entries,
  46 RGB roles, 165 default bindings and 14574 packaged template bytes agree.
- Focused changed-source quality passed at the existing thresholds.

Development checks exposed complexity boundaries, an iterator lint and enum
reference formatting drift. Responsibilities were split at calculation,
observation and planning ownership boundaries; the original checks pass after
correction. No check was weakened.

T19.3 commit: `73ecd1db9ecd79cb797a1332baf9d77a346575cf`.

T19.3 acceptance checks passed:

- `cargo test --test org_editing repeat_edges -- --nocapture`: five native
  read-only, atomic-rejection, range-bound, captured-clock and shared-save tests
  passed, zero skips.
- `cargo clippy --test org_editing -- -D warnings`.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  four owned editing/read-only RGB/NO_COLOR sessions passed, zero skips.
- Focused changed-test source quality passed at the existing thresholds.

The terminal extension retains exact source snapshots, Unicode notes, parent
progress, delay removal, LAST_REPEAT/state history, BOM/CRLF save, one-operation
undo/redo and normal cleanup. Its first setup used free date input with suffixes;
the native prompt correctly rejected that form. The driver now uses the admitted
complete timestamp spelling. Assertions, fixture ownership and deadlines remain
unchanged.

[Canonical PR #120](https://forge.adrianpk.com/adrianpk/rev/pulls/120) records
final required checks against its clean head through controller
validation in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_editing`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused final evidence additionally covers full configuration/command/display
regressions, configuration-reference agreement and the four owned task PTYs.

Final clean head `aa5fad3ef7870ae1b29ecc31661f09a440f3ba20` integrated through canonical PR #120.
Required controller receipt `EYXTNPTDAGE6TVSCEV3CHCCKLT` passed all five checks
and 224 tests. Focused receipt `WOSCCVGU5ZFLCL3WOYW7EX2I22` passed 295
configuration/command/display tests, configuration-reference agreement and four
owned RGB/NO_COLOR task PTYs, with zero skips. Acta readiness
`01M61ZRZT18P9GT0MV9QY8VF33` and the canonical merge were verified.

## Risks and Follow-ups

The full delivery-set gate has not run.
