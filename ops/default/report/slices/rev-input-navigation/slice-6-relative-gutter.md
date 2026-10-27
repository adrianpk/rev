# Slice 6: Render independent relative numbering

Status: reviewing
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
- Shared panes keep their own relative origins across focus changes. Splits
  inherit override provenance by value, then may diverge without moving point.
- File visits, buffer switches and buffer/window closure retain live policies
  and recompute labels for current source. Edits and undo/redo map both views.
- Movement and scrolling retain width derived from the complete buffer;
  viewport padding beyond EOF remains unnumbered.

## Implementation Notes

`NumberRow` derives a single checked real row using the existing effective view
policy. Distance uses checked-range integer metadata and `abs_diff`; current-line
selection is independent of the calculation mode. The existing renderer resolves
the current logical row from the pane's own view, including inactive panes.
Hidden or suppressed fields return before row materialization.
The row loop remains limited by the admitted number area height; each integer
label is at most 20 digits and uses the existing line index/count. A 100,000-line
fixture renders six text rows with correct EOF padding, then hides/restores
without changing point or mode. This is structural/native evidence; timing and
memory workload acceptance belongs to the assigned later performance slice. No retained number
arrays, caches, background work or settings store are introduced.

## Contracts Added or Changed

GUT02 defines view-local logical distances and current-line forms. GUT04 retains
stable full-source digit width. GUT11 retains independent view policy and point.

## Files of Interest

- `src/gutter/row.rs`: policy-driven labels and semantic row identity.
- `src/display/gutter.rs`: bounded visible-row rendering with effective policy.
- `tests/gutter/relative.rs`: actual field cells, semantic roles and integer bounds.
- `tests/gutter/shared.rs`: composed shared panes and native editing/lifecycle paths.
- `tests/gutter/rendering.rs`: absolute and relative decimal-boundary history.
- `docs/reference/gutter.md`: factory behavior and supported native policy.

## Validation

- PASS: `cargo test --test gutter relative::` covers the six mode/current-form
  combinations and integer limits. Actual output cells retain semantic styling,
  document text, cursor origin and exact Unicode/CRLF source bytes.
- PASS: `cargo test --test gutter shared::` covers both split directions with
  points on lines 20/60, inactive current labels, focus changes, mapped edits and
  undo/redo, window deletion, independent modes/forms, hide/restore/reset and
  native file/buffer rebind/close. Unicode/BOM/CRLF bytes remain exact.
- PASS: `python3 scripts/check-source.py` retains safe Rust and all existing
  parameter, nesting and complexity limits.

- PASS: `cargo test --test gutter` (25 native groups, zero skipped). Both modes
  cross the 9999/10000 boundary through real edit/undo/redo; current and adjacent
  labels, document origin and hardware cursor remain coherent. Relative cases
  also cover empty, one-line, Unicode, CRLF, final-empty and EOF padding rows.
- PASS: `python3 scripts/check-config-reference.py` retains 59 admitted catalog
  entries, 25 configurable RGB roles and 111 bindings with valid reference links.
  Native relative policy does not advertise unsupported terminal controls.

The controller records final results on the clean PR head for this canonical list:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test display --test windows`
- `python3 scripts/check-source.py`
- `git diff --check`

Separate focused final evidence uses the same head and this ordered list:

- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-pty.py --suite absolute-gutter`
- `git diff --check`

## Risks and Follow-ups

None recorded.
