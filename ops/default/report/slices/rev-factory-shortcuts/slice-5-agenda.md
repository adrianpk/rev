# Slice 5: Agenda Defaults

Status: reviewing
Delivery set: rev-factory-shortcuts
Plan: [Factory Shortcut Alignment Delivery](../../../plan/rev-factory-shortcuts.md)
Tracker: [Factory Shortcut Alignment Tracker](../../../tracker/rev-factory-shortcuts.md)
Branch: `feat/rev-factory-shortcuts-05-agenda`
PR: pending

## Purpose

Deliver accepted P08-P13 through existing native Agenda owners.

## Delivered Behavior

Daily d retains v d. Tags, search, show-all-today habits, graph-window input
and sort use v M, v /, v A, v G and v S. Their old M/s/A/G/S factory leaves
are removed. Names and explicit old user assignments remain available.
Calendar admission owns daily selection; habit graphs retain their read-only
owner and never reinterpret d as habit-date input. Use the named graph-date
command to select its date.

## Implementation Notes

The Agenda factory and exhaustive sample retain 47 entries. Their 11-leaf
delta matches accepted P08-P13 exactly. Query, habit and rendered sort hints
follow the new routes. Native handlers, source formats, consent and resource
limits remain unchanged. The frozen catalog, semantic matrix, startup template
and accepted manifest are untouched; manifest SHA-256 remains
`e578368195b2e46acb0c7e7e1a8f3b289ddf0595667f64f890a8cb418d8dd6cf`.

Eight regressions extend the existing factory-shortcut filter. They reuse
prepared profiles and the existing owned Org fixture and helpers, controlled
clock and five-second observation deadline. Cases exercise actual effects,
qualified selected-heading identity, one-edit undo, source/disk preservation,
retired leaves, every relocated family, named/old overrides, invalid/cancelled
prompts and prefix/role transitions. A prompt-accept override cannot turn any
changed suffix into confirmation consent. Three existing native input cases
follow the relocated keys, retaining all prior assertions.

## Contracts Added or Changed

P08-P13 implement FSA03 and FSA05-FSA09 within the native projection owners.
Only the selected legacy Agenda shortcut rows are reconciled; original Org
completion, candidate and gate metadata retain their historical scope.

## Files of Interest

- [Factory defaults](../../../../../src/input/keymap/defaults.rs) and
  [configuration sample](../../../../../docs/reference/config.toml).
- [Agenda regressions](../../../../../tests/commands/factory_shortcuts/agenda.rs),
  [habit compatibility](../../../../../tests/commands/factory_shortcuts/agenda/habits.rs)
  and [prompt ownership](../../../../../tests/commands/factory_shortcuts/agenda/guards.rs).
- [Agenda reference](../../../../../docs/reference/org-agenda.md),
  [bindings](../../../../../docs/reference/org/bindings.md) and
  [task guidance](../../../../../docs/tutorials/user-guide/07-org-tasks-and-agenda.md).
- [Selected legacy rows](../../../spec/editor-org-mode.md) and
  [accepted mapping contract](../../rev-factory-shortcuts/mapping-contract.md).

## Validation

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`: PASS; workspace formatting, strict Clippy and native source policy across 1,910 files, two packages and 75 Cargo roots.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts -- --test-threads=1`: PASS; 28 tests, including eight new Agenda regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands org_contexts -- --test-threads=1`: PASS; four existing role, prompt and transition regressions.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration keymaps -- --test-threads=1`: PASS; 11 inheritance, admission, normalized-key and override regressions.
- `git diff --check`: PASS.

Focused checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands factory_shortcuts::agenda -- --test-threads=1`: PASS; eight tests; 12.60 seconds of warm test execution.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_agenda controls -- --test-threads=1`: PASS; four existing native calendar/query/habit/sort controls; 3.35 seconds of test execution.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-documentation.sh`: PASS; 110 reachable pages, 780 local links/anchors, 707 canonical names in each combined inventory, 322 configuration entries, 10 command-reference and 12 configuration-reference regressions; all 28 sample maps match compiled defaults.

The initial focused run reported six passes and two failures from incorrect
observations about C-g closing a projection and graph-title clipping at column
four. Corrected observations preserve both native behaviors; the final focused
and required runs above pass. Selected artifact references and incidental
identifier checks also pass.

The ordinary target compiled cold dependencies in 2m 37s. Pre-commit lint took
120.86 seconds; test execution took 158.64/11.97/47.34 seconds for the shared
shortcuts, Org contexts and keymaps. The eight added Agenda cases took 12.60
seconds in their focused warm run. Documentation validation took 29.73 seconds.
These are observed costs. Checks retain one build job, disabled incremental
output, no dev/test debuginfo and the injected single-thread test settings.

The controller records all five exact commands once against the final clean
HEAD after canonical PR references are committed. The integrated full gate
and physical terminal acceptance remain with the controller and Slice 7.

## Risks and Follow-ups

Slice 6 owns the accepted Git routes. Physical terminal acceptance remains
with Slice 7. Existing Agenda operation admission remains authoritative.
