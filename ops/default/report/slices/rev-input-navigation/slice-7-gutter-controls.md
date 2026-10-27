# Slice 7: Wire gutter settings and native commands

Status: delivered
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-07-gutter-controls`
PR: [#87](https://forge.adrianpk.com/adrianpk/rev/pulls/87)
Integrated: `aee53c1ba94b57d331dc7f28429c408972db49a7`

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

The exhaustive reference remains tied to the typed partial schema. Its scanner
now reads the shared native enum definitions, admitting their default attribute
without omitting any variants, and checks the nested gutter palette. Drift cases
retain prior assertions and add missing gutter leaves/colors and enum choices.
All 111 factory binding entries remain unchanged. The existing line-number
Unreleased entry now describes the operational controls as one product outcome.

## Contracts Added or Changed

GUT10 extends the existing palette snapshot with admitted number-field leaves.
GUT12 and CFG05 use the same validated startup graph and native view policy.

## Files of Interest

- `src/config.rs`, `src/config/{merge,project,validate,palette}.rs`: typed startup leaves.
- `src/view.rs`, `src/session/startup.rs`: initial policy from the admitted snapshot.
- `tests/configuration/gutter.rs`: presence, precedence, diagnostics and whole-candidate rejection.
- `src/command/gutter.rs`, `src/session/gutter.rs`: real registry identities and handlers.
- `tests/gutter/controls.rs`: native M-x, policy cells, reset/provenance and context admission.

- `docs/reference/{config.toml,defaults.md,gutter.md}`: complete settings and controls.
- `scripts/configuration_ui/gutter_controls.py`: owned factory/configured terminal journeys.
- `tests/configuration/{catalog,resolution_contract}.rs`: all 29 palette defaults and projections.

## Validation

- PASS: `cargo test --test configuration gutter::` covers partial values, all
  resolved leaves/colors, source provenance and invalid source/candidate cases.
- PASS: `cargo test --test gutter controls::` verifies all concrete transitions,
  per-view inheritance/divergence, no text/history effects, frozen reset, M-x,
  configurable editor targets and prompt/tree refusal.
- PASS: `python3 scripts/check-source.py` retains all unchanged source limits.

- PASS: `cargo test --test commands catalog_reference` matches every registry name
  to both public catalogs, including the eight concrete gutter identities.
- PASS: `cargo test --test configuration --test commands` (229 native tests,
  zero skipped) retains startup isolation, complete sample defaults, published
  integer/enum admission, all 29 color projections, keymaps and command dispatch.
- PASS: `python3 scripts/check-config-reference.py` (70 catalog entries, 29 RGB
  roles, 111 bindings and three optional examples; valid links).
- PASS: `python3 -m unittest discover -s scripts -p test_config_reference.py`
  rejects omitted leaves/colors, missing native enum choices and all prior drift.
- PASS: `python3 scripts/check-pty.py --suite gutter-controls` (two owned sessions,
  zero skipped). No-config M-x exercises all eight commands, all current forms,
  hidden cycling, side changes, inspect/reset, C-g, monochrome emphasis and exact
  Unicode/BOM/CRLF edit/save/undo bytes. Configured startup verifies include/root
  precedence, custom width/padding, four RGB roles and frozen reset after a source
  edit. Both children finish and restore terminal state and owned descriptors;
  existing eight-second/one-MiB PTY limits remain unchanged.

PASS at the integrated candidate: canonical evidence `S4U4BQHGATEAZSWB26ORGOE6AE` records all five checks and 283 native tests:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test gutter --test configuration --test commands --test display`
- `python3 scripts/check-source.py`
- `git diff --check`

PASS on the same candidate: focused evidence `56QBBUMAP2AVIWFLDJZBEZDRX6` records reference/drift checks and two owned PTYs:

- `python3 scripts/check-config-reference.py`
- `python3 -m unittest discover -s scripts -p test_config_reference.py`
- `python3 scripts/check-pty.py --suite gutter-controls`
- `git diff --check`

## Risks and Follow-ups

None recorded.
