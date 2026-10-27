# Slice 9: Align pane status into left and right groups

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-09-status-layout`
PR: [#89](https://forge.adrianpk.com/adrianpk/rev/pulls/89)

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
functions under the unchanged source-quality limits. Projection is bounded by
status width and the existing label/glyph traversal, with no timer, external query
or background owner.

The smallest status retains complete state/access indicators as geometry permits;
zero/unusable frame geometry keeps the existing suspended presentation. Label
measurement and rendering use identical safe glyph semantics, including wide,
combining and control-bearing filenames. No orphan position separators remain.

Historical terminal markers now include the presentation separator and match only
variable spaces between a complete position and its exact mode. Raw terminal
bytes remain unchanged for color, geometry, cursor, lifetime and source checks.
The matcher rejects wrong facts and gaps crossing a divider, ANSI command or row.
The acceptance failure journey explicitly checks mode omission in its 27-cell
panes and restoration of all three modes at width 120. Its original decisions,
source/save assertions, eight-second deadline and 1-MiB output limit remain.

## Contracts Added or Changed

R05 and STA01 preserve pane-local status observations and palette ownership.
STA02 uses measured groups and deterministic complete-field degradation.

## Files of Interest

- `src/display/status.rs`: measured pane-local groups and safe clipping.
- `src/display.rs`: one-row status painting inside original pane bounds.
- `tests/display/status.rs`: native actual-row group assertions.
- `scripts/check-status.py`, `scripts/check-pty.py`: owned terminal geometry and semantic status fixtures.
- `docs/reference/status.md`: user-facing facts, coordinates and degradation.

## Validation

Development checks passed:

- `cargo test --test display status::`: five groups cover fifteen narrow
  thresholds, all dirty/access combinations, Rust/Text, safe Unicode labels,
  independent shared points and active/inactive palette cells.
- `cargo test --test gutter --test panels --test windows`: 92 native cases;
  narrow gutter status omits Text and a wider projection restores it.
- `python3 scripts/check-status.py`: exact per-pane right edges, Unicode,
  independent positions, edit/undo, gutter side, completion/cancel and
  narrow/zero restoration in RGB and NO_COLOR; owned children/descriptors close.
- `python3 scripts/check-pty.py --suite acceptance`: four historical journeys
  preserve document/block/workspace/failure assertions and exact saved bytes.
- `python3 scripts/check-config-reference.py`: 70 entries, 29 RGB roles,
  111 bindings and published links remain valid.
- `python3 scripts/check-source.py`: safe Rust and unchanged complexity limits.

Final clean-head canonical validation is recorded by the controller after PR
reference commits, in this exact order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test display --test panels --test windows --test commands`
- `python3 scripts/check-pty.py --suite acceptance`
- `python3 scripts/check-source.py`
- `git diff --check`

The canonical PR and controller evidence identify the final candidate; these
implementation observations do not substitute for that exact-head record.

## Risks and Follow-ups

Broader development diagnostics found existing gutter-coordinate failures in
`editing::direct_input`, `editing::regions::multiline_surface` and
`editing::regions::shared_focus`. All three reproduce on the unchanged dev base
`4c1bdbf1ac426255d07e8628b6bfd2be4b7732d0` using
`cargo test --test editing direct_input` and `cargo test --test editing regions::`.
The optional configuration/UI workspace journey also retains its older
string-splitting assumption about document gutter separators. These independent
coordinate fixtures remain for the approved delivery's integrated validation;
this slice preserves their assertions and does not claim those diagnostics pass.
