# Slice 4: Define native gutter view policy

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `refactor/rev-input-04-gutter-policy`
PR: pending

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
- Number width follows complete-buffer line metadata and admitted minimum
  digits. Checked terminal-cell geometry reserves padding and one separator
  while retaining the panel's protected three text columns.

## Implementation Notes

`View` owns the policy by value; `Composition` supplies editing-only content
bounds. The projection records the existing ViewId, source stamp, logical point
row, line count, effective policy and checked number/text bounds. It retains no
buffer reference, number-row array, document copy, timer or background worker.
Digit calculation uses checked integer metadata, with at most 20 admitted digits.
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
- `tests/gutter.rs` and `tests/gutter/{policy,lifecycle}.rs`: native policy and
  actual session lifecycle regressions.

## Validation

- PASS: `cargo test --test gutter` verifies factory policy, source identity,
  logical line metadata, complete field/text geometry and separate status/prompt
  bounds using the actual view and panel owners.
- PASS: the same gutter target now includes five groups covering independent
  leaf provenance/reset, both split directions, foreign/closed identities,
  actual buffer switching, shared-buffer closure and file ingress.
- An absent file path is admitted as a new document by ordinary file ingress.
  The rejected-ingress fixture uses an existing directory to verify the failure
  assertion while preserving that established new-file behavior.
- Focused source analysis passes unchanged safe-Rust, parameter, nesting and
  complexity limits for the changed owners and native tests.
- Decimal-boundary, invalidation and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
