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
- `src/view.rs`: policy lifetime within the existing view.
- `src/panel/composition.rs`: checked live editing-panel projection boundary.
- `tests/gutter.rs`: native integration-test target.

## Validation

- PASS: `cargo test --test gutter` verifies factory policy, source identity,
  logical line metadata, complete field/text geometry and separate status/prompt
  bounds using the actual view and panel owners.
- Override/lifecycle, decimal-boundary and final clean-HEAD checks are in progress.

## Risks and Follow-ups

None recorded.
