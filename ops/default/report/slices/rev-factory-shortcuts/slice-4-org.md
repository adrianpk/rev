# Slice 4: Org Document Defaults

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `feat/rev-factory-shortcuts-04-org`
PR: pending

## Purpose

Deliver accepted P05-P07 through the existing Org editing owners.

## Delivered Behavior

C-c C-x E cycles declared EFFORT_ALL choices. C-c C-x C-c opens a read-only
property-column projection, replacing its clock-report factory assignment.
C-c C-v j and C-c C-v C-j (normalized Return) insert checked Babel header
arguments. The old h factory leaf is removed. Clock reports and explicit old
user assignments remain available; C-c C-x C-r retains radio-checkbox behavior.

## Implementation Notes

Only the five accepted Org leaf edits change production code. The Org local
factory and exhaustive sample each contain 83 entries. Native handlers,
configuration schemas, source formats and authority checks remain unchanged.
Historical audit results and the accepted mapping manifest are preserved.

Ten focused regressions extend the existing frozen factory-shortcut filter.
They reuse two prepared profiles and owned source files; observation retains its
five-second deadline. The additional Org-only warm native run took 23.72 seconds;
the complete shared shortcut suite took 75.88 seconds.
The cases protect source identity, disk bytes, refusal/cancellation and history
through actual input and projections rather than a duplicate binding catalog.

## Contracts Added or Changed

P05-P07 implement FSA03-FSA09 within existing heading, source-block and profile
admission. The selected legacy Org shortcut rows are reconciled without changing
its original completion status, candidate or gate evidence.

## Files of Interest

- [Factory map](../../../../../src/input/keymap/defaults.rs) and
  [sample configuration](../../../../../docs/reference/config.toml).
- [Org regressions](../../../../../tests/commands/factory_shortcuts/org.rs)
  and [boundaries](../../../../../tests/commands/factory_shortcuts/org/guards.rs).
- [Org bindings](../../../../../docs/reference/org/bindings.md),
  [source headers](../../../../../docs/reference/org-source-blocks.md) and
  [columns](../../../../../docs/reference/org-columns.md).
- [Task guidance](../../../../../docs/tutorials/user-guide/07-org-tasks-and-agenda.md),
  [source procedure](../../../../../docs/how-to/extract-org-code.md) and
  [selected legacy rows](../../../spec/editor-org-mode.md).

## Validation

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: PASS; workspace formatting, strict Clippy and native safety/parameter/nesting/complexity policy across 1,907 files, two packages and 75 Cargo roots.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts -- --test-threads=1`: PASS; 20 tests, including ten added Org regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands org_contexts -- --test-threads=1`: PASS; four existing role, prompt, transition and cancellation regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration keymaps -- --test-threads=1`: PASS; 11 keymap inheritance, alias, override and admission regressions.
- `git diff --check`: PASS.

Focused documentation command:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS; 110 reachable
pages, 780 local links/anchors, 707 names in each combined command inventory,
322 configuration entries, 10 command-reference and 12 configuration-reference
regressions. The latter check all 28 sample maps against compiled defaults.
Selected specification/report local references also pass.

The five-edit sample frontier matches P05-P07 exactly. The accepted manifest
retains SHA-256 `e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.
Effort/columns cover all seven editing states; header insertion additionally
checks plain j, raw Control-J and Return. Source/disk identity, one-group edits,
undo, cancelled/refused input, read-only projection, old overrides, hints and
existing list/table/heading/radio behavior pass.

The new worktree compiled cold dependencies in 2m 57s. The raw required lint
command took 122.68 seconds; native shortcut/context/configuration runs took
75.88/2.16/14.96 seconds excluding compilation. Build settings remain one job,
no incremental output and no dev/test debuginfo. Temporary fixtures use the
existing owned lifecycle. The controller records the five exact commands once
against the final clean HEAD after PR references are committed. The integrated
full gate remains controller-owned.


## Risks and Follow-ups

Slice 5 owns the accepted Agenda changes. Physical terminal acceptance remains
with Slice 7. These document shortcuts add no missing help, TODO-heading branch
or functional adapter.
