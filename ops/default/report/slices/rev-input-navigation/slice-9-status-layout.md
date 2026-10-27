# Slice 9: Align pane status into left and right groups

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-09-status-layout`
PR: pending

## Purpose

Present existing pane-local status facts in two measured groups while retaining
one status row, truthful source observations and existing palette ownership.

## Delivered Behavior

- Identity, clean/modified state, RW/RO access and logical position form the left
  group. Syntax mode or Text aligns with the owning status rectangle's right edge.
- Flexible space separates complete groups. Status width follows the original
  pane rectangle independently of number fields or document scrolling.
- Narrow composition omits language first, compacts position to `Lline:column`,
  then clips identity. When complete numeric/access facts cannot fit, it omits
  those facts rather than emitting partial or misleading tokens.
- Current active/inactive styling and the separate prompt/echo row remain.

## Implementation Notes

A private disposable facts projection reads the canonical buffer and that view's
logical cell. Existing `Glyphs` and clipping own safe terminal-cell measurement.
No settings, segment registry, retained status state or extra row is introduced.
Fact capture, complete-group layout and narrow degradation have separate small
functions under the unchanged source-quality limits.

The smallest status retains complete state/access indicators as geometry permits;
zero/unusable frame geometry keeps the existing suspended presentation. Label
measurement and rendering use identical safe glyph semantics, including wide,
combining and control-bearing filenames. No orphan position separators remain.

## Contracts Added or Changed

R05 and STA01 preserve pane-local status observations and palette ownership.
STA02 uses measured groups and deterministic complete-field degradation.

## Files of Interest

- `src/display/status.rs`: measured pane-local groups and safe clipping.
- `src/display.rs`: one-row status painting inside original pane bounds.
- `tests/display/status.rs`: native actual-row group assertions.

## Validation

- PASS: `cargo test --test display status::` checks actual complete groups,
  Unicode/tab logical position, exact flexible spacing, width and echo isolation.
- PASS: `python3 scripts/check-source.py` retains all unchanged source limits.

- PASS: `cargo test --test display status::` (five groups) additionally pins
  fifteen narrow thresholds, zero/unusable suspension, all four dirty/access
  combinations, Rust/Text alignment, sanitized Unicode labels across 59 widths,
  independent shared points and active/inactive palette cells after focus/gutter
  changes. Rendering retains source state, history and prompt bounds.

## Risks and Follow-ups

None recorded.
