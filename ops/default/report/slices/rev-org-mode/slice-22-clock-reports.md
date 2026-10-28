# Slice 22: compare Org clock totals and effort

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-22-clock-reports`
PR: pending

## Purpose

Deliver native clock totals and explicit clock-table updates under the
[approved plan](../../../plan/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort)
and [tracker](../../../tracker/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort).

## Delivered Behavior

T22.1 adds `org-clock-display` through M-x. Task, subtree and file queries use
current canonical source, including unsaved edits, without changing source or
history. Optional civil date bounds are half-open and resolved in the captured
operation zone. Actual intervals clip to the selected range; task/subtree totals
compare their root effort, while file totals compare top-level estimates.
Open and unresolved intervals remain separate counts. Overlapping intervals
sum separately and disclose their overlap count, avoiding silent deduplication.

## Implementation Notes

Native revision-qualified structure supplies source ownership and literal
exclusions. A report admits 262144 UTF-8 source bytes, 4096 CLOCK candidates and
the existing 4096 heading/drawer bound. Each CLOCK line admits 4096 bytes.
Malformed, ambiguous and nonexistent endpoint observations cannot invent time;
recorded duration text cannot prove endpoint or offset equivalence. Resolvable
endpoints supply actual UTC-minute durations, including offset transitions.
Read-only inspection and cancellation preserve ordinary editor ownership.

## Contracts Added or Changed

- ORG19: source-derived closed/open interval observation.
- ORG21: bounded actual/effort comparison without source mutation.
- ORG23: derived report rows retain revision-qualified source locators.

## Files of Interest

- `src/org/visibility/reports.rs`: checked source selection and effort ownership.
- `src/org/time/clock/observe.rs`: typed interval admission and zone resolution.
- `src/org/agenda/clocks.rs`: derived query, row and summary contracts.
- `src/session/outline/reports.rs`: native prompt and read-only comparison.

## Validation

T22.1 development checks passed:

- `cargo test --test org_time --test org_editing clock --no-fail-fast`: 26 editing
  and nine time tests passed, zero skips.
- `cargo test --test commands org_visibility`: compiled command and key parity.
- `cargo clippy --lib --test org_time --test org_editing --test commands -- -D warnings`.
- Changed-source quality at the unchanged safe-Rust, parameter, nesting and
  cyclomatic limits.

Development checks caught fixture initialization and case-sensitive `UTC`
selection, native rendering accessors and source-quality boundaries. Fixtures
now initialize the ordinary native mode before dispatch and use the documented
zone name; observation and parsing responsibilities were split without changing
assertions or resource limits.

## Risks and Follow-ups

The delivery-set full gate has not run. Explicit dynamic report publication and
broader report contracts remain in T22.2 and T22.3.

T22.1 commit: `4d45e7957e9c`.

T22.2 adds native opaque dynamic-block observations and compiled
`org-clock-report` / `org-dblock-update` handlers. C-c C-x C-c inserts/updates;
C-c C-x C-u and contextual C-c C-c explicitly refresh a selected body.
C-c C-x C-d invokes the read-only comparison. Existing radio-checkbox bindings
retain their identities. Header, closing delimiter and other blocks remain exact.
One normal transaction replaces the body; identical output adds no history.

The finite parameter profile admits file/nil/subtree scope, maxlevel 1..256,
paired explicit civil date/time bounds, and documented day/week/month/year
blocks. The default range is the captured current year; untilnow selects all
closed records. Unsupported/duplicate/conflicting options reject before any
source change. No code, formatter or source-discovery hook is interpreted.
Tables roll descendant minutes into visible ancestors, count intervals once in
the grand total, distinguish effort/open/unresolved values and disclose overlaps.
Output admits 1048576 UTF-8 bytes.

The [Org clock-table reference](https://orgmode.org/manual/The-clock-table.html)
defines compatible dynamic block delimiters, scope, maxlevel and explicit bounds.
The implemented finite options and preserved unsupported source are documented
in the command/key reference.

T22.2 development checks passed:

- `cargo test --test org_time --test org_editing clock --no-fail-fast`: 28 editing
  and ten time tests passed, zero skips.
- `cargo test --test org_source --test commands`: 60 structure/source and 156
  command cases passed, zero skips.
- `cargo clippy --lib --test org_time --test org_source --test org_editing --test commands -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 120 catalog/schema entries,
  46 RGB roles, 174 default bindings and 14994 template bytes agree.
- Changed-source quality passed at the original limits.

Checks caught native coordinate/date API mismatches and parameter-parser
complexity. The implementation now uses the existing scalar line boundaries and
calendar operations, and separates option/boundary/calendar responsibilities.
The CRLF fixture also verifies a closing delimiter with trailing spaces.
