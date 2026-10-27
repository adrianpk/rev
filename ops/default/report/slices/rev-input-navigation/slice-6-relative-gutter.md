# Slice 6: Render independent relative numbering

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-06-relative-gutter`
PR: pending

## Purpose

Render native relative logical labels from each view's point, with independently
selected current-line forms and stable full-buffer digit geometry.

## Delivered Behavior

- Noncurrent relative labels show absolute logical distance from that view's
  point. Absolute mode retains one-based source labels.
- Both modes honor absolute, zero and hidden current-line forms. The real
  current row retains semantic color and non-color emphasis, even when blank.
- Movement and scrolling retain width derived from the complete buffer;
  viewport padding beyond EOF remains unnumbered.

## Implementation Notes

`NumberRow` derives a single checked real row using the existing effective view
policy. Distance uses checked-range integer metadata and `abs_diff`; current-line
selection is independent of the calculation mode. The existing renderer resolves
the current logical row from the pane's own view, including inactive panes.
Hidden or suppressed fields return before row materialization. No retained number
arrays, caches, background work or settings store are introduced.

## Contracts Added or Changed

GUT02 defines view-local logical distances and current-line forms. GUT04 retains
stable full-source digit width. GUT11 retains independent view policy and point.

## Files of Interest

- `src/gutter/row.rs`: policy-driven labels and semantic row identity.
- `src/display/gutter.rs`: bounded visible-row rendering with effective policy.
- `tests/gutter/relative.rs`: actual field cells, semantic roles and integer bounds.

## Validation

- PASS: `cargo test --test gutter relative::` covers the six mode/current-form
  combinations and integer limits. Actual output cells retain semantic styling,
  document text, cursor origin and exact Unicode/CRLF source bytes.
- PASS: `python3 scripts/check-source.py` retains safe Rust and all existing
  parameter, nesting and complexity limits.

## Risks and Follow-ups

None recorded.
