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
same native handler as M-x. The native loop applies that flag through the existing terminal reporting owner
before the initial frame and after each bounded input batch. It acquires only
DEC 1002/1006, rejects pointer input immediately when disabled, and propagates
reporting failures through the ordinary full terminal cleanup path. Repeated
state synchronization emits no redundant mode writes. Documentation describes
the terminal-dependent bypass gesture, native disable path and bounded active
drag behavior without promising a universal physical modifier.

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

`python3 scripts/check-pty.py --suite pointer-lifetime` passed 13 owned PTYs,
including actual factory activation with one reporting pair, unchanged keyboard
editing, decoder coverage, partial acquisition/rollback, disable failure/retry,
normal/error/unwind restoration and passive filesystem behavior. The historical
factory assertion now checks the delivered enabled default and exact restoration
counts/order; its typing, history, prefix and resource assertions are retained.
The exhaustive reference guard passed all 72 catalog leaves and 111 bindings.

Final ordered and focused evidence will be recorded through the controller on
the clean candidate after task and PR-reference commits.

## Risks and Follow-ups

The pointer ticket remains active until this final primary slice integrates.
