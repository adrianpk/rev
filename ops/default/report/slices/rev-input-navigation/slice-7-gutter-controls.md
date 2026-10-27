# Slice 7: Wire gutter settings and native commands

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-07-gutter-controls`
PR: pending

## Purpose

Expose the accepted native numbering policy through startup TOML and real
compiled commands, preserving the existing settings and view lifetime owners.

## Delivered Behavior

- Seven optional gutter leaves and four semantic palette colors resolve through
  the existing built-in/include/root layers and immutable startup snapshot.
- Initial executable views receive admitted startup gutter policy. Splits copy
  view values and provenance; rebind retains the view's chosen policy.
- Eight compiled commands support mode/visibility/current-line/side changes,
  inspection and reset. M-x and configured editor bindings reach the same handlers.
- Explicit mode selection shows numbers; mode cycling retains visibility. Reset
  clears only the addressed view's overrides and restores its startup policy.
- Invalid type, enum, numeric bound, color or unknown leaf rejects the complete
  startup candidate, including an invalid included value overridden by its root.

## Implementation Notes

Configuration reuses native mode/current-line/side enums and the admitted format;
there is no second preference store. Optional leaves retain absence, false and
zero through ordinary overlay and source provenance. Native host constructors
remain explicit; executable startup uses `View::with_settings`.

Commands run through ordinary command continuity and redraw; they create no text
edits, implicit persistence, TOML reload or factory binding conflicts. Inspect
reports each mutable leaf's startup/view source and fixed format values. Prompt
and tree ownership retain their existing refusal/cancellation behavior.

## Contracts Added or Changed

GUT10 extends the existing palette snapshot with admitted number-field leaves.
GUT12 and CFG05 use the same validated startup graph and native view policy.

## Files of Interest

- `src/config.rs`, `src/config/{merge,project,validate,palette}.rs`: typed startup leaves.
- `src/view.rs`, `src/session/startup.rs`: initial policy from the admitted snapshot.
- `tests/configuration/gutter.rs`: presence, precedence, diagnostics and whole-candidate rejection.

- `src/command/gutter.rs`, `src/session/gutter.rs`: real registry identities and handlers.
- `tests/gutter/controls.rs`: native M-x, policy cells, reset/provenance and context admission.

## Validation

- PASS: `cargo test --test configuration gutter::` covers partial values, all
  resolved leaves/colors, source provenance and invalid source/candidate cases.

- PASS: `cargo test --test gutter controls::` verifies all concrete transitions,
  per-view inheritance/divergence, no text/history effects, frozen reset, M-x,
  configurable editor targets and prompt/tree refusal.
- PASS: `python3 scripts/check-source.py` retains all unchanged source limits.

## Risks and Follow-ups

None recorded.
