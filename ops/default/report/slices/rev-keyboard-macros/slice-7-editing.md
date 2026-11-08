# Slice 7: Definition editing and macro menu

Status: delivered
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-07-editing`
PR: [#325](https://forge.adrianpk.com/adrianpk/rev/pulls/325)
Integrated: 88088e42aff78b0c4e31937f5668357548f1af12 on dev through canonical PR #325
Implementation: [17dcf0772016](https://forge.adrianpk.com/adrianpk/rev/commit/17dcf0772016131104782240f155684e76cf25f7)

## Purpose

Deliver T7.1-T7.3: checked native TOML drafts and recent-input projection,
step-by-step execution/editing, and a read-only stable-identity ring menu.

## Delivered Behavior

- C-x C-k Enter/C-e edits the ring head, with optional ring repeat keys after
  application. C-x C-k e selects a named or anonymous-bound snapshot. C-c C-c
  validates and replaces only the pinned target. Invalid, stale and oversized
  drafts stay editable. Native closing discards the draft and restores its source
  mode. C-c C-q inserts native key notation; C-c C-r projects whole step lines.
- C-x C-k l creates a new draft from at most 300 recent recordable logical inputs
  within 256 KiB. Accepted named commands and prompt values use the existing
  capture owner. Protected components, selector UI, terminal input and expanded
  replay do not enter the recent-input projection.
- C-x C-k Space asks before each logical step. Execute, remove, skip-once, text
  group, continue and truncate choices update one pending candidate. Native
  insert/replace/after/append reads preserve exactly-once effects; lowercase
  reads finish at a complete command on C-j, uppercase I/R reads one key sequence.
  Successful finish publishes once. Cancellation preserves the old body and
  real partial source edits in native history.
- list-keyboard-macros opens a read-only native component in both profiles. Its
  stable rows expose position, name, counter, format, rendered value and clipped
  body. Native actions edit properties, transpose, mark, flag, copy and delete.
  Region selection precedes marks, then the current row. Multi-copy and immediate
  deletion require fresh confirmation. Refresh and close retain source ownership.
- Session editing and ring deletion leave independent named/bound/saved snapshots
  intact. Named drafts retain saved baselines; library writes remain explicit.

## Implementation Notes

The session uses one checked target replacement path for ring, named, bound and
new definitions. A draft pins its original immutable definition, identity and
revision. The existing strict schema-1 codec validates the complete TOML record;
shared-memory admission counts draft targets, step candidates, recent input and
batched menu copies. Existing step/record and 48 MiB retention limits remain.

Definition text is a native editable transient buffer with its own undo owner.
The source view is retained and restored on apply/discard. The menu is a read-only
projection over stable native identities; property prompts and destructive
consent pin the original identity/revision. All targets are checked before a
batch mutation. Ring eviction/deletion cannot erase independent snapshots.

Step editing uses the existing runner, source/history owner, shared dispatch/text
budgets and nested-call/query handling. Questions pause the active clock without
an automatic wake. Lowercase input uses native capture while the original runner
is suspended; accepted edits already occurred and are not replayed twice. Only
successful final admission replaces the original body. No second dispatcher,
external language, dependency or implicit persistence path is introduced.

## Contracts Added or Changed

- Twenty-six native commands with the approved exact descriptions, global
  gestures and definition/menu actions.
- `keys.macro_definition` inherits ordinary editor text input in either profile;
  `keys.macro_menu` owns its read-only local controls. Both use effective native
  keymaps and hints. No macro preference or runtime bound is added.
- Typed step-choice input and read-only step-wait observation, alongside the
  existing query/consent observations and native recovery keys.
- Complete command/default sample references, a reachable editing reference and
  the existing Unreleased macro outcome.

## Files of Interest

- `src/macros/editing.rs`, `recent.rs`, `step_edit.rs`: checked target, bounded
  recent input and pending step candidate ownership.
- `src/session/macros/definition.rs`: native TOML buffer, helpers and target pins.
- `src/session/macros/step_edit.rs`: shared-runner questions and native reads.
- `src/session/macros/menu.rs`: stable rows, property prompts and checked batches.
- `src/input/step_edit.rs`, `keymap/defaults.rs`: native answers and local maps.
- `tests/macros/definition_editor.rs`, `step_edit.rs`, `menu.rs`, `lossage.rs`:
  native input/source/history and independent-snapshot fixtures.
- `docs/reference/macro-editing.md`: exact delivered editing and menu behavior.

## Validation

The unchanged eight-command selection passed at clean final PR head
88088e42aff78b0c4e31937f5668357548f1af12 with controller receipt
4JAKW7HCNV45UD2PQMFPMP6HKA. Forgejo and Acta confirm integration through PR #325.
These checks do not establish the delivery-set gate or PTY acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- definition_editor step_edit menu lossage --test-threads=1` — pass, 26 selected native workflow cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — pass, 52 selected command cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1` — pass, 114 editing cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1` — pass, 45 selected modal cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — pass, 680 commands, 295 configuration entries, 94 pages and 649 links/anchors.
- `git diff --check` — pass.

Additional development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- macros::library session::macros::tests queries_clock regions_admission --test-threads=1` — pass, 24 native admission, storage, stale-target, budget, source-owner and completed-layout cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- --test-threads=1` — pass, 106 macro integration cases; the 26 editing cases were rerun after the final layout-boundary correction.

## Risks and Follow-ups

Full integrated KMA01-KMA09 acceptance, real PTY journeys, large-buffer probes and
complete User Guide/how-to acceptance remain Slice 8. Source edits made while
step editing are real and remain after cancellation; ordinary native undo owns
them. Editing stays session-local until an explicit named save is confirmed.
