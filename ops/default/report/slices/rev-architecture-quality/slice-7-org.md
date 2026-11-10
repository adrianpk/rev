# Slice 7: Org workflow decomposition

Status: delivered
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-07-org`
PR: #21 - https://forge.adrianpk.com/rev/rev/pulls/21
Integrated: `954fed70cf96a00ce4711d6a13b0cad1b469630e`

## Purpose

Resolve selected ownership and source-policy findings in native Org document
workflows and their existing session adapters.

## Delivered Behavior

Native Org separates literal-block pairing from content classification,
startup exclusion ranges from keyword selection, all-drawer
updates from visibility dispatch, and structural-request admission from Org
command routing. All four baseline source-policy findings are corrected.

Agenda/calendar and tangling snapshots now use explicit shared-reference clones.
Private calendar projection borrows the retained snapshot and clones it only for
the resulting observation. Redundant owned clones are removed from structural
item ranges, agenda directory iteration and heading-motion errors.

Native time preparation replaces narrowing casts with checked conversions after existing
civil/date, month-cell, recurrence and POSIX admission. Timestamp decimal parsing
now produces each native field width directly. Clock observations retain ordered
minute endpoints; duration formatting isolates its existing floored-tick admission.
All 79 native time cases pass, and the time owner has no selected Clippy findings.

Private document preparation now borrows inspected source ranges, list targets,
calendar times, archive/link selectors, attachment context, table layout, formula
graph inputs, export/include/macro inputs, footnote labels and feed inputs.
Custom agenda publication retains the supplied view directly, removing one
clone. Native Org has no remaining selected pass-by-value findings.

Numeric preparation checks scalar/index conversions in document/list cursors,
calendar/habit projections, clock state, source-program caps and ZIP fields.
Calendar spans retain a private u16 day count and a separate week-alignment
owner; formula aggregates use an exactly representable bounded integer count.
Native Org has no selected Clippy findings.

Session preparation borrows synchronous capture/refile plans, draft text,
attachment listings, link navigation, table/list edits and publication outcomes.
Outline preparation borrows observations unless the resulting prompt, report or
visibility state retains them. Table export moves its checked destination into
the save owner instead of cloning it. Pending prompt retirement and asynchronous
work retain their existing ownership boundaries. The complete scoped quality
command now passes with no blocking Clippy or native source-policy findings.

The approved lifecycle correction distinguishes stable fixtures from cases that
own pending initial observation. Addressed agenda and saved-clock views remain
stable before capturing command authority, and native styles follow syntax
policy while structural observation remains available for folding. All 791
native cases in the complete frozen Slice 7 command pass.

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
- Private preparation consumes only retained output or mutable work. Attachment
  jobs still own their captured context; selected attachment checks still bracket
  resolution. Include evidence, graph work charges, feed status updates, export
  diagnostics and cancellation checks keep their existing order and limits.
- Custom agenda blocks build rows before moving the supplied view into the
  published result; baseline configuration retains its independent clone.
- One narrow, reasoned Clippy expectation preserves the public owned
  `ClockScan::new` request signature. Its fields are Copy values and a borrowed
  policy; no public request or caller is reshaped.
- Calendar arithmetic preserves public day counts and checked span shifts.
  Habit windows remain bounded to 366 days per side and 4096 source completions;
  configured Org day/week fields retain their admitted ranges, with
  unrepresentable public calendar inputs rejected through existing Result paths.
- Formula counters fit the existing ten-million-step budget; precision retains
  its existing 0..15 decimal and 1..15 significant-digit policies. Checked
  formatting conversions preserve fixed/scientific spelling and finite results.
- Three narrow clock-display expectations preserve existing floating-point
  minute rounding/range checks. Two statement-level expectations cover the
  already integral admitted round exponent and the fixed 256-entry CRC table.
- Document/list cursor projection retains signed net character deltas and
  validation order. Document cursor arithmetic rejects exhausted positions
  instead of wrapping into an unsigned index.
- Clock-state reads still stop at 65537 bytes and reject above 65536. Source
  program caps remain 600 seconds and 16 MiB, each intersected with the existing
  configured ceiling. ZIP output retains its five-member stored ZIP32 profile.
- Synchronous session preparation keeps each plan alive in its existing caller;
  checked visits, edits, publication order, stale-source admission and uncertain
  save retention are unchanged. Capture return needs only its owning view ID.
- Habit, feed and execution prompt selectors contain only Copy identity/command
  fields. Resource-bearing prompts remain owned. Four narrow, reasoned Clippy
  expectations preserve the existing one-shot Pending dispatch signatures for
  capture, estimate, tag and source-header responses and their snapshot retirement.
- Existing in-module capture and refile fixtures pass borrowed plans/outcomes;
  their inputs, assertions, timeouts and publication fault injection are unchanged.
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
- `src/org/agenda/custom.rs`, `query/observe.rs` and `scan.rs`: view retention,
  snapshot borrowing and the unchanged public clock-request boundary.
- `src/org/attachment/plan.rs`, `src/session/org/attachment.rs` and the two
  existing in-module fixtures: borrowed preparation within the owned job.
- `src/org/export/`, `feed/`, `table/` and `formula/eval/graph.rs`: private
  preparation input borrowing and retained result ownership.

- `src/org/document.rs` and `list/`: checked net scalar cursor and indentation deltas.
- `src/org/formula/eval/aggregate.rs` and `numeric/`: exact admitted counter and
  precision widths, unchanged formatting and finite-result policies.
- `src/org/io/clock.rs`, `execution.rs` and `export/odt/zip.rs`: unchanged read,
  process/output and binary package ceilings with checked native widths.
- `src/session/agenda_actions/resolve.rs`, `habits.rs` and
  `outline/tasks/repeat.rs`: admitted file-length and Org day-boundary fields.

- `src/session/capture/`, `refile/`, `attachment/prompt.rs`, `links/visit.rs`,
  `tables/`, `tables.rs`, `feeds/` and `exports.rs`: borrowed synchronous inputs
  and retained checked publication owners.
- `src/session/outline/`, `outline.rs`, `habits/`, `execution.rs` and
  `source_edit.rs`: observation lifetimes, Copy prompt selectors and owned
  one-shot prompt retirement boundaries.

- `src/buffer/org.rs`, `src/session/agenda_actions/execute.rs` and
  `src/session/outline/clocks/state/restore.rs`: admitted native styles and
  stable explicitly addressed views.

## Validation

All four frozen commands passed through `looper validate` on clean published
HEAD `954fed70cf96a00ce4711d6a13b0cad1b469630e` before canonical PR #21 merged
into `dev`. Validation ID: `XCFEU34ADA4ANKN2JMAFMFHJCG`. The canonical PR
contract check also passed. The earlier corrected implementation at
`e4888d6414e6767a59b22fc4443aa346b1dafcf5` passed the same working-tree checks.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/org.rs --scope src/org --scope src/session/agenda --scope src/session/agenda.rs --scope src/session/agenda_actions --scope src/session/agenda_actions.rs --scope src/session/agenda_custom --scope src/session/agenda_custom.rs --scope src/session/agenda_queries --scope src/session/agenda_queries.rs --scope src/session/agenda_sources.rs --scope src/session/capture --scope src/session/capture.rs --scope src/session/refile --scope src/session/refile.rs --scope src/session/attachment --scope src/session/attachment.rs --scope src/session/document.rs --scope src/session/outline --scope src/session/outline.rs --scope src/session/tables --scope src/session/tables.rs --scope src/session/tangling --scope src/session/tangling.rs --scope src/session/execution --scope src/session/execution.rs --scope src/session/feeds --scope src/session/feeds.rs --scope src/session/previews.rs --scope src/session/exports --scope src/session/exports.rs --scope src/session/publishing --scope src/session/publishing.rs --scope src/session/source_edit --scope src/session/source_edit.rs --scope src/session/links --scope src/session/links.rs --scope src/session/habits --scope src/session/habits.rs --scope src/session/org --scope src/session/org.rs`. All-target compilation and formatting pass; all 431 selected sources satisfy the source guards and selected Clippy reports zero blockers. The 377 diagnostics outside this slice remain deferred.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`. All 791 cases pass across eight targets (97 agenda, 235 editing, 16 headings, 18 indentation, 192 io, 81 source, 73 tables, 79 time); zero failures, ignores or filtered cases.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Passed: `git diff --check`.

Historical batches and their focused corrective checks are retained below.
Their failed outcomes precede the complete passing corrected-slice result above.

Maintainer-approved lifecycle correction:

- Stable Org fixtures now finish initial observation before capturing view
  snapshots or issuing stable-context commands. Explicit coloring, cancellation,
  retired-insertion and pending-visibility cases use an unobserved constructor
  or retain their initial live work. No assertion, input, timeout or resource
  ceiling is weakened.
- Agenda actions and stored-clock recovery suppress document startup folding
  for their explicitly addressed views before capturing request authority.
  The original view, source/file checks and runtime freshness comparisons remain.
- Native styles honor syntax admission while initial visibility can still use
  cached structural observation.
- Passed working-tree cohort: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda -- --test-threads=1`. All 97 cases pass, including the 28 failures in the retained controller batch.
- Failed focused batch with partial passes: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_editing --test org_io --test org_tables --no-fail-fast -- colors:: retired_insertions pending_controls retired_requests startup_lifetime_visibility document_readonly readonly_properties readonly_tags timestamp_readonly attachments_ execute_admission clock_recovery_mismatch link odt_failed_output refile_ clock_restart tex_job_ text_export_failed_output readonly_tables recalculation_preservation --test-threads=1`. Editing passes 12 cases and tables pass two; IO passes 38, with one read-only canonical-view setup failure corrected next.
- Passed corrective rerun: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_io -- --exact link_edges::links_canonical_readonly --test-threads=1`. The remaining IO case passes after preparing its unchanged-view baseline.
- Passed working-tree cohort: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_headings -- --test-threads=1`. All 16 cases pass, including the retained canceled-observation failure.
- Passed additional owner check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/buffer/org.rs --scope src/session/agenda_actions/execute.rs --scope src/session/outline/clocks/state/restore.rs`. Formatting, compilation, selected Clippy admission and native source limits pass; there are zero blocking diagnostics in these owners and 377 selected-profile diagnostics deferred outside the selection.

These historical focused results did not replace the unchanged eight-target
native command. The earlier working-tree batches below retain their original
results; the complete corrected-slice run above now passes every case.

- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/org.rs --scope src/org --scope src/session/agenda --scope src/session/agenda.rs --scope src/session/agenda_actions --scope src/session/agenda_actions.rs --scope src/session/agenda_custom --scope src/session/agenda_custom.rs --scope src/session/agenda_queries --scope src/session/agenda_queries.rs --scope src/session/agenda_sources.rs --scope src/session/capture --scope src/session/capture.rs --scope src/session/refile --scope src/session/refile.rs --scope src/session/attachment --scope src/session/attachment.rs --scope src/session/document.rs --scope src/session/outline --scope src/session/outline.rs --scope src/session/tables --scope src/session/tables.rs --scope src/session/tangling --scope src/session/tangling.rs --scope src/session/execution --scope src/session/execution.rs --scope src/session/feeds --scope src/session/feeds.rs --scope src/session/previews.rs --scope src/session/exports --scope src/session/exports.rs --scope src/session/publishing --scope src/session/publishing.rs --scope src/session/source_edit --scope src/session/source_edit.rs --scope src/session/links --scope src/session/links.rs --scope src/session/habits --scope src/session/habits.rs --scope src/session/org --scope src/session/org.rs`. All-target compilation and formatting pass; all 431 selected sources satisfy the source guards, down from four findings. Selected Clippy reports zero blocking diagnostics, down from 316; 377 diagnostics remain deferred outside this slice.
- Failed during the session cohort: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`. The first target, `org_agenda`, reports 71 passed and 26 failed; Cargo does not reach the other seven targets. Every current failure name occurs in the retained unchanged-source baseline (`be5daac794cfc3c5fa2a991dd4890cd3f7b95e25`, 68 passed and 29 failed). Three baseline habit failures do not recur in this run; the additional `habit_views::habit_no_color` failure from the numeric cohort also passes here. This variation does not certify those cases or the full frozen command.
- Passed: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`. 106 reachable pages, 743 local links/anchors, 684 commands and 299 configuration entries.
- Previously failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`. The first target, `org_editing`, reports 148 passed and 87 failed. The unchanged slice base reproduces the exact same 87 failure names and 148 passes; the other six targets are not reached.
- Previously passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_time -- --test-threads=1`. All 79 native cases pass with no ignored tests.
- Previously passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- attachment --test-threads=1`. All five native/session attachment cases pass; only the existing private preparation call arguments changed.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda -- custom_views:: --test-threads=1`. All seven custom-view cases pass, including checked calendar span shifts, block limits, source aliases and clock columns.
- Previously passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_source -- --test-threads=1`. All 81 cases pass, including source ranges, inline/structural observations, formulas, blocks, feeds and tangling.
- Previously failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_tables -- --test-threads=1`. 57 passed and 16 failed; no ignored or filtered cases. These historical failures were not compared against the retained checkpoint; the corrected full target now passes.
- Previously failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_io -- --test-threads=1`. 154 passed and 38 failed; no ignored or filtered cases. These historical failures were not compared against the retained checkpoint; the corrected full target now passes.
- Previously failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_headings -- --test-threads=1`. 15 passed and one failed (`lifecycle::cancelled_observation`); no ignored or filtered cases. This historical failure was not compared against the retained checkpoint; the corrected full target now passes.
- Previously passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_indentation -- --test-threads=1`. All 18 cases pass with no ignored or filtered cases.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- org:: --test-threads=1`. All 20 native Org unit cases pass, including document bounds, attachment/preview/refile ownership, formula deadline and table work limits.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_tables -- evaluation --test-threads=1`. All 14 numeric evaluation cases pass, including exact dependencies, mean overflow avoidance, fixed/scientific precision, long chains, invalid policy and non-finite inputs.
- Failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_editing -- document --test-threads=1`. One passed and 15 failed. Every failure name occurs in the retained unchanged-base editing results; assertions and fixture behavior remain unchanged.
- Failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_editing -- list --test-threads=1`. 12 passed and one failed (`list_edges::parsed_checkbox_contexts`). That failure name occurs in the retained unchanged-base editing results; assertions and fixture behavior remain unchanged.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_io -- odt_exports::odt_package --test-threads=1`. Both package cases pass, including independent ZIP/CRC/XML verification and exact output-byte boundaries.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda -- --exact habit_views::habit_no_color --test-threads=1`. The newly observed full-run `habit_no_color` failure does not reproduce in this single isolated rerun. This does not certify its full-run behavior.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda -- calendar_rows:: --test-threads=1`. All seven native calendar-row cases pass, including recurrence, range endpoints, limits, zones and archived-source exclusion.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- session::capture:: --test-threads=1`. The existing uncertain-publication retry case passes across retry, edited-draft and changed-source scenarios.
- Passed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- session::refile:: --test-threads=1`. All five existing refile/archive cases pass, including atomic rejection, partial saves and captured-save histories.
- Failed focused check: `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_tables -- interchange --test-threads=1`. Five cases pass and `interchange::interchange_rejections` fails: the expected view has uninitialized visibility, while the actual view has completed initial Org observation. This name also occurs in the earlier 16 table failures, which remain unclassified against the unchanged slice base. Roundtrip/export, conversion, limits, read-only and values cases pass.
- Passed: `git diff --check`.

The first-cohort baseline comparison ran the same frozen native command in
this owned worktree. All 13 implementation files were restored byte-for-byte afterward;
the primary checkout, tests, assertions and runtime limits were unchanged. The
focused editing comparison also restored those exact implementation bytes. Those
historical batches did not establish native-slice success; the complete corrected
run above now passes all eight targets.

## Risks and Follow-ups

No known Slice 7 check failures remain after the approved lifecycle correction.
The 377 selected-profile diagnostics outside this slice remain for subsequent
slices. Canonical PR #21 is integrated into `dev` with exact clean-HEAD evidence.
The controller owns the remaining delivery-set progression and full gate.
