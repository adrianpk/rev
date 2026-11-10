# Slice 7: Org workflow decomposition

Status: drafting
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-07-org`
PR: pending

## Purpose

Resolve selected ownership and source-policy findings in native Org document
workflows and their existing session adapters.

## Delivered Behavior

The first implementation cohort separates literal-block pairing from content
classification, startup exclusion ranges from keyword selection, all-drawer
updates from visibility dispatch, and structural-request admission from Org
command routing. All four baseline source-policy findings are corrected.

Agenda/calendar and tangling snapshots now use explicit shared-reference clones.
Private calendar projection borrows the retained snapshot and clones it only for
the resulting observation. Redundant owned clones are removed from structural
item ranges, agenda directory iteration and heading-motion errors.

The time cohort replaces narrowing casts with checked conversions after existing
civil/date, month-cell, recurrence and POSIX admission. Timestamp decimal parsing
now produces each native field width directly. Clock observations retain ordered
minute endpoints; duration formatting isolates its existing floored-tick admission.
All 79 native time cases pass, and the time owner has no selected Clippy findings.

Implementation remains in progress; selected Clippy ownership and conversion
findings still require correction before canonical PR preparation.

## Implementation Notes

- Literal-block processing retains the same two bounded passes, heading
  boundaries, cancellation/work ticks and incomplete-coverage behavior.
- Startup selection retains protected malformed/literal/drawer ranges, keyword
  ordering, case-insensitive values and every cancellation check.
- Visibility dispatch retains malformed-drawer rejection before mutation.
  Org routing retains editing/mode checks, immediate presentation commands,
  attachment/interaction admission and show-all/startup-fold precedence.
- Calendar projections retain immutable facts through result publication;
  agenda targets and tangle outputs keep their existing shared snapshot owners.
- Checked time conversions defend existing private invariants: years 0001..9999,
  one civil day, Gregorian month/day bounds, fixed name tables, admitted clock
  ranges and POSIX field/representative-year ranges. Error order and source
  spelling remain unchanged; decimal fields keep their existing width limits.
- Three narrow, reasoned Clippy expectations retain the existing floating-point
  work-duration contract at elapsed-seconds, clock-minutes and display-tick
  boundaries. Scaling still rejects at the exact exclusive 2^64 tick limit;
  finite/nonnegative admission, flooring and formatting are unchanged.
- Public requests, source text, navigation, consent, timestamps, formulas,
  source-block behavior and document/resource limits are preserved. Helpers
  remain under the selected Org and session concern roots.

## Contracts Added or Changed

None. Package, configuration, state, source-coordinate and wire APIs remain
compatible.

## Files of Interest

- `src/org/structure/rows.rs` and `lists.rs`: literal pairing/content phases and
  consumed structural row ranges.
- `src/org/visibility/startup.rs` and `state.rs`: startup exclusions and drawer
  mutation admission.
- `src/org/agenda/calendar.rs`, `calendar/` and `sources/walk.rs`: retained
  immutable observations and directory-path ownership.
- `src/org/tangle.rs`: explicit shared input ownership for prepared targets.
- `src/session/outline.rs` and `outline/operation.rs`: structural request
  admission and consumed motion errors.
- `src/session/agenda_actions.rs` and `agenda_actions/marks.rs`: explicit shared
  title ownership in captured targets.

- `src/org/time/date.rs`, `calendar.rs`, `civil.rs`, `month.rs` and
  `repeat/advance.rs`: checked civil/calendar conversion invariants.
- `src/org/time/timestamp/parse.rs`, `entry/` and `zone/posix/parse.rs`: native
  decimal fields and admitted rule values.
- `src/org/time/duration.rs`, `operation.rs` and `clock/`: existing duration
  representation, display admission and ordered recorded intervals.

## Validation

Working-tree cohort results follow; canonical PR preparation and exact clean-HEAD
Looper evidence remain pending. Agenda/editing failures below are retained
from the first cohort; this conversion cohort runs the complete native time
target without claiming a successful frozen eight-target run.

- Failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/org.rs --scope src/org --scope src/session/agenda --scope src/session/agenda.rs --scope src/session/agenda_actions --scope src/session/agenda_actions.rs --scope src/session/agenda_custom --scope src/session/agenda_custom.rs --scope src/session/agenda_queries --scope src/session/agenda_queries.rs --scope src/session/agenda_sources.rs --scope src/session/capture --scope src/session/capture.rs --scope src/session/refile --scope src/session/refile.rs --scope src/session/attachment --scope src/session/attachment.rs --scope src/session/document.rs --scope src/session/outline --scope src/session/outline.rs --scope src/session/tables --scope src/session/tables.rs --scope src/session/tangling --scope src/session/tangling.rs --scope src/session/execution --scope src/session/execution.rs --scope src/session/feeds --scope src/session/feeds.rs --scope src/session/previews.rs --scope src/session/exports --scope src/session/exports.rs --scope src/session/publishing --scope src/session/publishing.rs --scope src/session/source_edit --scope src/session/source_edit.rs --scope src/session/links --scope src/session/links.rs --scope src/session/habits --scope src/session/habits.rs --scope src/session/org --scope src/session/org.rs`. All-target compilation and formatting pass; all 431 selected sources satisfy the source guards, down from four findings. Selected Clippy reports 204 blocking diagnostics, down from 316; 377 diagnostics remain deferred outside this slice.
- Previously failed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`. The first target, `org_agenda`, reports 70 passed and 27 failed; Cargo does not reach the other seven targets. All 27 failures also reproduce with unchanged source from slice base `be5daac794cfc3c5fa2a991dd4890cd3f7b95e25`. That baseline run reports 68 passed and 29 failed, including two additional habit-view cases.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Previously failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`. The first target, `org_editing`, reports 148 passed and 87 failed. The unchanged slice base reproduces the exact same 87 failure names and 148 passes; the other six targets are not reached.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_time -- --test-threads=1`. All 79 native cases pass with no ignored tests.
- Passed: `git diff --check`.

The first-cohort baseline comparison ran the same frozen native command in
this owned worktree. All 13 implementation files were restored byte-for-byte afterward;
the primary checkout, tests, assertions and runtime limits were unchanged. The
focused editing comparison also restored those exact implementation bytes. No
successful native-slice result is claimed; six targets still await execution
beyond the retained agenda/editing failures.

## Risks and Follow-ups

The 204 selected Clippy diagnostics remain open. Agenda and editing failures
reproduce on the unchanged base and include source/view-freshness and report
publication rejections. Their resolution remains pending; no fixture or runtime
interaction correction is included in this cohort. Canonical PR preparation and
exact clean-HEAD validation require all original frozen checks to pass. The
controller owns merge, full delivery-set validation and successor startup.
