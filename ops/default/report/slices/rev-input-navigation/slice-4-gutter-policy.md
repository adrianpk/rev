# Slice 4: Define native gutter view policy

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `refactor/rev-input-04-gutter-policy`
PR: [#84](https://forge.adrianpk.com/adrianpk/rev/pulls/84)
Integrated: `db5006cb038a644b4786d015f80d83afeb33daaf`

## Purpose

Prepare checked native view policy and disposable gutter geometry within the
existing editing view, logical line index and panel owners.

## Delivered Behavior

- Every editing view owns a small native gutter policy with factory values and
  explicit leaf provenance, separate from canonical text and history.
- Visibility, mode, current-line form and side have independent explicit
  overrides. Reset clears provenance and restores the view's startup values;
  repeated updates report no change.
- Splits inherit effective values and override metadata by value. Buffer switch,
  file ingress and shared-buffer closure retain each view's policy; deleting a
  view releases its own state and rejects its former identity.
- A composition derives a gutter projection only for a live editing ViewId,
  checking its source revision before using the canonical logical line index.
- Decimal-boundary edits, undo/redo, view movement and resize derive fresh
  geometry. Wrong buffers/revisions, invalid points and closed identities cannot
  produce a projection. Narrow content hides the complete number field while
  retaining the requested view policy.
- Number width follows complete-buffer line metadata and admitted minimum
  digits. Checked terminal-cell geometry reserves padding and one separator
  while retaining the panel's protected three text columns.

## Implementation Notes

`View` owns the policy by value; `Composition` supplies editing-only content
bounds. The projection records the existing ViewId, source stamp, logical point
row, line count, effective policy and checked number/text bounds. It retains no
buffer reference, number-row array, document copy, timer or background worker.
Digit calculation uses checked integer metadata, with at most 20 admitted digits.
The minimum is admitted in 1..=20 and each padding in 0..=4; none can truncate a
complete source line index. Width is independent of the relative origin.
Each projection uses source/point line-index lookup plus bounded field arithmetic;
retained policy memory scales only with the existing view count.
Projection callers derive a fresh snapshot after source, view or layout changes.

`Windows::change_gutter` admits a live ViewId before changing one native leaf;
its result tells the existing caller whether values/provenance need redraw.
`View::rebind` resets ordinary new-buffer navigation while retaining policy.
The existing buffer-switch, file-visit and close owners all use that boundary.
No policy references a buffer or introduces an auxiliary lifetime registry.

This is the approved preparatory layer. Rendering, operational gutter TOML and
compiled M-x controls remain assigned to their following slices. Existing
settings samples, command registry and displayed text remain unchanged.

## Contracts Added or Changed

GUT01 establishes view-owned derived presentation, GUT04 establishes stable
checked field sizing, and GUT11 assigns policy ownership to existing view
lifetime. No document/history ownership changes.

## Files of Interest

- `src/gutter.rs`, `src/gutter/policy.rs`, `src/gutter/geometry.rs`: native records
  and checked projection geometry.
- `src/view.rs`, `src/window.rs` and `src/session/{buffers,file}.rs`: policy
  updates and lifetime through the existing split/rebind/close paths.
- `src/panel/composition.rs`: checked live editing-panel projection boundary.
- `tests/gutter.rs` and `tests/gutter/`: native policy, actual session lifecycle,
  source/identity invalidation and checked integer/cell-bound regressions.

## Validation

- PASS: `cargo test --test gutter` (12 native groups, zero skipped).
- The real view/panel boundary verifies factory values, canonical source
  identity, logical metadata, complete field/text bounds and separate
  status/prompt reservations without modifying text or history.
- Independent-leaf cases cover startup-equal overrides, provenance, reset,
  idempotent updates, both split directions and foreign/closed ViewIds.
  Actual session cases cover buffer switch, shared-buffer closure and file
  ingress, retaining each view's policy and the ordinary text/history owners.
- An absent file path is admitted as a new document by ordinary file ingress.
  The rejected-ingress fixture uses an existing directory, preserving the
  failure assertion and established new-file behavior.
- Checked-format cases reject unsupported minima/padding. Decimal cases cover
  9/10, 99/100, 9999/10000, 99999/100000 and `usize::MAX` without creating one
  source row or label per possible index.
- The partition grid covers 4,428 projections across side, visibility, padding,
  source count, zero/tiny/usable dimensions and ordinary/maximal u16 origins.
  Every admitted number/text partition covers content exactly and retains the
  protected three text columns; hidden fields restore the original text bounds.
- A real native edit crosses 9999/10000 lines and undo/redo restores coherent
  widths with fresh stamps. Relative-origin movement keeps width stable; reset
  changes presentation only. Stale/wrong source, invalid points and closed
  identity checks reject projection. Resize suspension retains the whole view.
- Focused source analysis passes unchanged safe-Rust, parameter, nesting and
  complexity limits for the new records, changed owners and native tests.

PASS: canonical controller evidence `AVX4W4B2M4OCHU3UXSQUUIYFON` validates
the integrated commit with this exact ordered list (72 native tests):

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test panels --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

PASS: focused controller evidence `PZDKOF6JKENREIFMCNYTWMY3KP` records
this list on the same integrated commit (217 native and 13 acceptance tests):

- `cargo test --test commands --test files`
- `python3 scripts/check-config-reference.py`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `git diff --check`

## Risks and Follow-ups

None recorded.
