# Slice 2: Mode Mapping Contract

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `docs/rev-factory-shortcuts-02-mode-contract`
PR: pending; approved owner decision checkpoint precedes submission
Report introduction: [immutable proposal report](https://forge.adrianpk.com/rev/rev/src/commit/da5056f6e6feebc52d9114f88036097ea8e132a7/ops/default/report/slices/rev-factory-shortcuts/slice-2-mode-contract.md)

## Purpose

Complete the contextual semantic audit and publish an exact, reviewable mapping
proposal for D01-D03 without changing runtime defaults or selecting product
decisions on the owner's behalf.

## Delivered Behavior

The [matrix](../../rev-factory-shortcuts/semantic-matrix.md) now assigns an
evidence-backed semantic disposition to every frozen public command: 333 core/
profile rows, 238 Org, 119 Git and 17 Markdown names, plus six prompt actions.
The 707-command, 2,349-entry, 28-context catalog and native source tree remain
unchanged. This is completed audit work; the slice is awaiting the mapping
decision and is not delivered or eligible for a runtime successor.

The [mapping contract](../../rev-factory-shortcuts/mapping-contract.md) and
[proposal JSON](../../rev-factory-shortcuts/mapping-proposal.json) specify
26 records and 46 base-context edits. P01-P08 retain the reviewed core/Org
proposals. P09-P13 move conflicting Agenda controls into a native v family.
P14-P19 move History cherry-pick to A, remove misleading History P/F/f defaults
and add direct Refs f/P aliases while preserving existing Refs aliases.
Named access and explicit old overrides remain available for displaced actions.

D01/D02 alternatives, exact legacy supersessions and intentional differences
are explicit. The proposed choice preserves all seven recorded slice identities,
PR titles and exact gate commands. Accepting a different Git choice may require
an explicit controller/configuration decision; the worker has not changed it.

## Implementation Notes

T2.1 finishes the mode review against native source tree
`442765af9faac4351c208b2bfc5ca02c1b896331` at the Slice 1 closure baseline.
The [contextual review](../../rev-factory-shortcuts/mode-review.md) links native
operation owners and records point/profile/projection boundaries. Per-command
rows distinguish direct routes, dispatcher access, aliases and named-only
operations. It does not infer reachability from raw target counts.

Reference extraction uses GNU Emacs 31.1, Org 9.8.7, Markdown mode 2.8-alpha at
`f8d1be7cd5dfd64c0e4f88c29a3f84408cde0b47` and Magit 4.5.0 at
`c800f79c2061621fde847f6a53129eca0e8da728`. It runs isolated `-Q` package maps,
records 374 command-family queries and 1,080 contextual lookups, and retains
package source hashes. Absent functions, static prefixes and internal
reference dispatchers are supporting observations rather than parity claims.

Owned native probes compile the current library in this worktree using
read-only prebuilt dependencies. They verify:

- All 46 proposed base-context edits under both profiles: 92 normalized traces,
  prefix neighbors and protected C-g across all 28 compiled maps. Removals are
  exact private snapshot leaf removals; no configuration unbinding mechanism
  or shipped factory change is implied.
- M-m input dispatch in Text, Org and Markdown across nonmodal and six modal
  states: 21 sessions retain source and editing state.
- Declared effort cycling creates one history group; property columns retain
  source text/stamp/history. Checkbox insertion succeeds at a list and refuses
  a heading, unlike the reference heading TODO branch.
- Babel header cancellation retains source; acceptance edits the opening line.
  Source-edit Save retains its draft; abort restores unchanged Org source.
- Markdown plain/list/M-RET/table Return branches, including indirect helper
  equivalence and source alignment before table-row movement. Observed package
  list Return differs intentionally from the native list convention.
- Daily Agenda projection is view-only. After explicitly selecting the fixture
  date, changing the selected second entry's effort leaves the first unchanged.
  Native daily view uses the current range start, not an assumed today anchor.
- Calendar next-day selection from a captured instant commits the selected
  date, while cancellation retains source/history. Capture Save retains its
  draft; abort restores origin without creating its target file.
- History fetch/push and Refs fetch without a selected configured remote refuse.
  The owned local Git repository's HEAD, saved state and index remain unchanged;
  no transport starts.

T2.2 presents the exact proposal and concrete decision alternatives. D03
records missing heading/source-save behavior separately from retained native
Markdown/tree/calendar/Git conventions; it commissions none of that behavior.
T2.3 reconciles the later runtime tasks, unchanged gate selection and estimated
build/check costs. Both tasks remain incomplete until owner acceptance and
approval evidence are committed.

## Contracts Added or Changed

FSA01/FSA02/FSA05-FSA07/FSA09 gain complete contextual audit evidence and an
exact proposed mapping boundary. The owning shortcut specification stays Draft;
no approval stamp is added to the mapping contract. Runtime source, factory
maps, personal configuration, durable models, resource limits, frozen commands
and historical delivery evidence are unchanged.

Selected supersessions of Org Familiar interaction and Git Binding review
matrix rows are identified for the corresponding runtime slices after
acceptance. Native Git transient differences and named displaced commands are
part of that proposal, rather than undocumented exceptions.

## Files of Interest

- [Contextual review](../../rev-factory-shortcuts/mode-review.md): native owners,
  family decisions, indirect access and actual behavior boundaries.
- [Mapping contract](../../rev-factory-shortcuts/mapping-contract.md) and
  [proposal](../../rev-factory-shortcuts/mapping-proposal.json): exact changes,
  D01-D03 alternatives, displaced access and legacy supersessions.
- [Mode audit runner](../../rev-factory-shortcuts/mode-audit.py),
  [mode reference](../../rev-factory-shortcuts/reference-modes.json) and
  [asserted native results](../../rev-factory-shortcuts/mode-probe-results.json):
  reproducible isolated effects, normalized routes and reference identities.
- [Semantic matrix](../../rev-factory-shortcuts/semantic-matrix.md): all 707
  command identities, six actions and 28 contexts with completed dispositions.

## Validation

Pre-decision focused checks:

- `python3 ops/default/report/rev-factory-shortcuts/mode-audit.py --package-dir "$REV_AUDIT_PACKAGES" --dependency-dir "$REV_AUDIT_DEPS"`: PASS. Variables select installed isolated reference package sources and compatible prebuilt dependencies; the runner owns all editor library output, sessions and Git fixtures. It also checks proposal hashes, unique matrix coverage and local document references without asserting owner approval.
- `python3 ops/default/report/rev-factory-shortcuts/audit.py --dependency-dir "$REV_AUDIT_DEPS"`: PASS; frozen core/catalog evidence reproduces with unchanged native source and defaults.
- `git diff --check`: PASS for the proposal candidate.

The measured focused mode comparison took 10.414 seconds with the owned library
cache warm. This measures the audit command, not the planned final gate.

The injected approval-dependent coverage command and exact clean-HEAD
controller receipt are pending. Their required order and assertions are
unchanged. The contract intentionally has no mapping approval stamp before the
owner decision, so this report does not claim that required gate passed.

The private native compile timeout is 180 seconds, probe process timeout
120 seconds, isolated reference timeout 60 seconds and native observation
settle deadline five seconds. No limit was raised. These are audit probes,
not physical terminal acceptance, broad Cargo checks, changing Git journeys or
the integrated delivery-set gate. Fixtures are removed by the runner on exit.

## Risks and Follow-ups

The [approved plan's Controller decision checkpoint](../../../plan/rev-factory-shortcuts.md)
requires this clean committed proposal before a Slice 2 PR. Obtain explicit
D01-D03 acceptance through that checkpoint, record the accepted proposal,
reconcile approval metadata and run the unchanged configured commands against
the exact clean HEAD. Only then prepare the canonical Slice 2 PR and its
immutable report-introduction link. Runtime Slice 3 remains ineligible.

Later slices must verify changed factory input-to-effect behavior, removed
leaves, named displaced access, overrides, prefix cancellation, effective hints,
selected source identity, undo/count and consent. Slice 7 owns the selected PTYs
and exact integrated gate. The contract estimates 3-8 minutes warm or 10-20
minutes cold from separately measured prior checks; actual gate phase costs
remain to be measured. No delivery or behavioral acceptance is claimed here.
