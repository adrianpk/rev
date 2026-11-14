# Slice 3: Editor Defaults

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `feat/rev-factory-shortcuts-03-editor`
PR: pending

## Purpose

Deliver accepted P01-P04 through existing native commands and context admission.

## Delivered Behavior

M-m moves to the first non-whitespace scalar on the current logical line;
blank lines use their end. C-x + balances existing pane divisions while retaining
focus and the configured policy for future splits. C-x p p opens the registered
Project picker, retaining its cancellation and dirty-source decisions. These
three shortcuts are inherited by every modal state. C-M-_ redoes native history
groups in nonmodal, Insert and Replace editing; Normal C-r remains redo and
nonmodal C-r remains backward search.

## Implementation Notes

Four leaves in the existing editor factory map flow through the established
context and modal inheritance. No command, handler, configuration schema or
persistent format changes. The exhaustive reference includes the corresponding
24 base-context leaves; local overlays and independent user assignments retain
priority. The accepted manifest remains unchanged at SHA-256
`e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.
Native Project-picker cancellation retains source/cursor/history; C-g retires
a Visual selection to Normal through the existing quit owner.

## Contracts Added or Changed

P01-P04 implement FSA03/FSA04 with focused regression coverage for FSA07-FSA09.
The first two slices remain closed after PR #48 integration. Subsequent shortcut
slices and separately specified functional parity work retain their boundaries.

## Files of Interest

- [Factory defaults](../../../../../src/input/keymap/defaults.rs): four native aliases.
- [Factory shortcut tests](../../../../../tests/commands/factory_shortcuts.rs): shared owned sessions and real input effects.
- [Configuration reference](../../../../../docs/reference/config.toml) and
  [binding reference](../../../../../docs/reference/bindings.md) and
  [configuration counts](../../../../../docs/reference/defaults.md): exact factories and state boundaries.
- [Editing guide](../../../../../docs/tutorials/user-guide/02-edit-text.md),
  [panes guide](../../../../../docs/tutorials/user-guide/03-buffers-and-panes.md),
  [modal guide](../../../../../docs/tutorials/user-guide/11-modal-editing.md) and
  [Project guide](../../../../../docs/tutorials/user-guide/20-projects-and-workspaces.md).

## Validation

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: PASS; full workspace formatting, strict Clippy and native source policy across 1,905 files, two packages and 75 Cargo roots.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts -- --test-threads=1`: PASS; 10 cohesive input/effect tests, including 189 first-text language/state/source cases, count consumption, read-only motion, redo groups/boundaries, pane focus/split policy, effective hints, overrides/local overlays and owned Project cancellation/restoration.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration keymaps -- --test-threads=1`: PASS; 11 configuration keymap regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal keymaps -- --test-threads=1`: PASS; six modal keymap regressions.
- `git diff --check`: PASS.

Focused documentation command:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS; 110 reachable
pages, 780 local links/anchors, 707 names in each combined command inventory,
322 configuration entries, 10 command-reference and 12 configuration-reference
regressions. The latter include exact agreement between all 28 sample maps and
compiled local/effective defaults. Changed report/document local links also pass.

All commands use the dedicated worktree cache with one build job, disabled
incremental output and no dev/test debuginfo. After the cold dependency build,
the focused native tests took 49.17 seconds, configuration keymaps 10.40 seconds
and modal keymaps 8.53 seconds, excluding compilation. Prepared snapshots are
reused for first-text cases without reducing coverage or assertions. Temporary
source/catalog fixtures are owned and removed through their existing fixture
lifecycle. Project observation retains its five-second deadline.

Local checks passed before PR creation. The final controller validation receipt
will attach the five required commands, once in canonical order, to the clean
HEAD containing PR references. No delivery-set gate has run for this slice.


## Risks and Follow-ups

Physical terminal compatibility belongs to Slice 7. The controller owns merge,
successor startup and the unchanged integrated full gate. No personal settings
or external Project catalog are used by the tests.
