# Slice 22: compare Org clock totals and effort

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-22-clock-reports`
PR: `#123`

## Purpose

Deliver native clock totals and explicit clock-table updates under the
[approved plan](../../../plan/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort)
and [tracker](../../../tracker/rev-org-mode.md#slice-22-compare-org-clock-totals-and-effort).

## Delivered Behavior

`org-clock-display` compares actual task, subtree or file time with effort
without changing canonical source, including unsaved edits. Optional civil date
bounds are half-open and resolved in the captured operation zone. A task/subtree
compares its root effort; a file compares top-level estimates. Overlapping
intervals sum separately and disclose overlaps. Open and unresolved records stay
separate from measured time. Duration text cannot prove endpoint/offset identity.

`org-clock-report` inserts a compatible dynamic block on a blank line or empty
EOF, or updates a selected clocktable. `org-dblock-update` and contextual C-c C-c
explicitly refresh that body. Headers, closing delimiters, other blocks and
unrelated source stay exact. One ordinary transaction owns each change; identical
output adds no history. Shared views retain canonical positions and ordinary
undo/redo restores the complete manual body and view positions.

The finite clocktable parameter profile admits file/nil/subtree scope, maxlevel
1..256, paired civil date/time bounds and documented day/week/month/year blocks.
Default scope is file and default range is the captured current year; untilnow
selects all closed records. Unsupported, duplicate, conflicting and incomplete
options reject before publication. No code, formatter or discovery hook runs.
Tables roll deeper minutes into visible ancestors and count each interval once
in the grand total, retaining effort and record diagnostics.

## Implementation Notes

Native structure supplies literal exclusions and revision-qualified ownership.
Unknown dynamic blocks remain opaque source. Resolvable endpoints supply actual
UTC-minute durations across midnight and offset changes. Ambiguous/nonexistent
local endpoints and malformed records remain unresolved; report calculation
never invents an offset or rewrites recorded spelling.

A report admits 262144 UTF-8 source bytes, 4096 CLOCK candidates, 4096 bytes per
CLOCK line and the existing 4096 heading/drawer/dynamic-block bound. Output admits
1048576 UTF-8 bytes, including its wrapper and diagnostics. Exact boundaries
reject whole results instead of publishing a partial total. Native worst-case
pipe escaping also exercises output overflow from an admitted source.

`org::agenda::ClockScan` prepares an immutable bounded snapshot with checked
source locators. Interval calculation and table rendering share the existing
session-owned Org job slot; no extra concurrent worker or periodic writer is
introduced. Computation checks cancellation at line/render boundaries and yields
at 10ms compute intervals. Publication rechecks source revision, selected view,
point/focus and ordinary read-only/history admission. C-g, changed source and
moved views cannot publish an obsolete result. Completion retires its deadline;
idle reports add no polling.

The [Org clock-table reference](https://orgmode.org/manual/The-clock-table.html)
defines compatible delimiters, scope, maxlevel and explicit bounds. The command
reference documents the implemented data-only profile and preservation of
unsupported source.

## Contracts Added or Changed

- ORG19: source-derived closed/open interval observations and actual time.
- ORG21: bounded effort comparisons and explicit compatible clock tables.
- ORG23: derived observations retain source identity without edit authority.

## Files of Interest

- `src/org/agenda/`: bounded immutable scan, query/table data and rendering.
- `src/org/visibility/reports/`: native selection and exact publication targets.
- `src/org/time/clock/observe.rs`: typed source interval admission.
- `src/session/org.rs`, `src/session/outline/reports.rs`: shared job/publication.
- `tests/org_agenda/`: native provenance, boundaries and transaction contracts.
- `scripts/configuration_ui/org_reports.py`: actual terminal report/save/undo.

## Validation

T22.1 commit: `4d45e7957e9c`. Development checks passed:

- `cargo test --test org_time --test org_editing clock --no-fail-fast`: 26 editing
  and nine time tests passed, zero skips.
- `cargo test --test commands org_visibility`: two command/key parity cases.
- `cargo clippy --lib --test org_time --test org_editing --test commands -- -D warnings`.

T22.2 commit: `3ca4d3d4461a`. Development checks passed:

- `cargo test --test org_time --test org_editing clock --no-fail-fast`: 28 editing
  and ten time tests passed, zero skips.
- `cargo test --test org_source --test commands`: 60 source and 156 command cases,
  zero skips, including opaque dynamic blocks and exact delimiter ranges.
- `cargo clippy --lib --test org_time --test org_source --test org_editing --test commands -- -D warnings`.
- `python3 scripts/check-config-reference.py`: 120 catalog/schema entries,
  46 RGB roles, 174 bindings and 14994 packaged template bytes agree.

T22.3 commit: `e270b1e2859e2d02a039d30ff7e8c17f7ce365f1`. Development checks passed:

- `cargo test --test org_agenda`: ten native report/provenance/publication cases,
  zero skips; approximately 0.21 seconds warm. Cases include exact 4096/4097
  candidate and 262144/262145 byte admission, output overflow, shared-view undo,
  canceled work, read-only source and stale observation/publication rejection.
- `cargo test --test org_editing clock`: 24 existing lifecycle/editing cases,
  zero skips, selected because the shared native job owner changed.
- `cargo clippy --all-targets --all-features -- -D warnings`.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tasks`:
  eight owned RGB/NO_COLOR editing/read-only/restart PTYs passed, zero skips.
  The report journey checks inspection, explicit refresh, exact BOM/CRLF bytes,
  unchanged refresh, complete undo, terminal restoration and owned cleanup.
- Changed-source quality passed at unchanged safe-Rust, parameter, nesting and
  cyclomatic limits for each task.

Development checks caught native coordinate/date and rendered-frame API
mismatches, source-quality boundaries, fixture initialization and the
case-sensitive `UTC` selector. Owners now use existing native APIs; parsing and
formatting responsibilities are separate. The terminal fixture initially assumed
an unavailable whole-buffer chord and replacement-on-paste. Its driver now uses
ordinary beginning/mark/end/kill/paste commands and undoes both setup edits.
Exact source/history assertions, deadlines and resource limits remain unchanged.

[Canonical PR #123](https://forge.adrianpk.com/adrianpk/rev/pulls/123) targets dev.
Final controller receipts bind required and focused checks to the clean
PR-reference head. Required checks run in this order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_time --test org_agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

## Risks and Follow-ups

The delivery-set full gate has not run.

## Integration

Canonical PR #123 merged into dev at
`608e64a51443e11931cc4a3897a948fe8f1ec072`; live Forgejo and Git agree.
Required controller receipt `DNBOQFBOGGFETXKTGCZVKLOZDT` passed all five ordered
checks, including 75 tests. Focused receipt `KIMDANVL7LO66QYSIDUOONWPFO` passed
579 affected tests, reference agreement and eight owned PTYs. Both receipts
bind that exact clean head; no checks were skipped.
