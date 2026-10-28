# Slice 15: Activate native mouse defaults and runtime control

Status: drafting
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `feat/rev-input-15-pointer-controls`
PR: pending

## Purpose

Activate the complete native focus, wheel and canonical drag-selection path with
usable startup defaults and a session-only runtime control.

## Delivered Behavior

Typed `mouse.enabled = true` and `mouse.scroll_lines = 3` enter the existing
schema, include precedence, provenance and immutable startup snapshot. Distances
are integers in 1..=100. `M-x mouse-mode` toggles session admission, including
from tree focus, without rewriting the source. Disabled input rejects pointer
reports and removes capture/deadlines while retaining the canonical region.
Wheel/tree distance uses the configured value; drag-edge steps also clamp to
the current text-area height.

## Implementation Notes

The session owns one effective enabled flag. The frozen snapshot owns the
startup preference and step. The command catalog and context admission use the
same native handler as M-x. Terminal reporting activation and complete lifecycle
verification follow in the remaining recorded tasks.

## Contracts Added or Changed

PTR01 and PTR06 supply the native defaults and compiled toggle. CFG05 retains
the startup snapshot and no-write runtime override boundary.

## Files of Interest

- `src/config.rs`, `src/config/{merge,project,validate}.rs`: typed control leaves.
- `src/command.rs`, `src/session/pointer.rs`: fixed command and session admission.
- `src/session/pointer/{scroll,autoscroll}.rs`: configured bounded distances.
- `tests/configuration/mouse.rs`, `tests/pointer/controls.rs`: real startup and handler outcomes.
- `docs/reference/config.toml`, `docs/reference/defaults.md`: exhaustive sample and catalog.

## Validation

Development `cargo test --test pointer --test configuration` passed 53 pointer
and 85 configuration groups. Coverage includes false/absent layering and source
origins, both integer endpoints and whole-candidate rejection, real M-x in editor
and tree focus, configured wheel/drag steps, disable during live capture,
retained native region and frozen settings after a root-file change.
`python3 scripts/check-config-reference.py` passed sample/model/catalog equality.

Final ordered and focused evidence will be recorded through the controller on
the clean candidate after task and PR-reference commits.

## Risks and Follow-ups

The pointer ticket remains active until this final primary slice integrates.
