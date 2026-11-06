# Slice 1: Session ranking

Status: reviewing
Delivery set: rev-command-ranking
Plan: [Command Ranking Delivery](../../../plan/rev-command-ranking.md)
Tracker: [Command Ranking Tracker](../../../tracker/rev-command-ranking.md)
Branch: `feat/rev-command-ranking-01-session`
PR: pending

## Purpose

Deliver UI02 and CMR01-CMR03 through session-local usage ranking in the shared
M-x and modal colon selector.

## Delivered Behavior

- Empty queries show four distinct recent commands, four other frequent
  commands and the alphabetical remainder, with no duplicates or unused
  frequency slots. Frequency uses the last 256 named invocations.
- Exact names and configured prefix/substring admission take precedence over
  usage. Every text-match rank applies the same four/four policy. Equal counts
  prefer the latest invocation and then the canonical name.
- Accepted native and registered AI names add one event through the shared
  selector path, including configured acceptance aliases. Handler refusal or
  later argument cancellation does not erase an accepted invocation.
- Opening, typing, navigation, Tab, history recall, unknown names, cancellation,
  direct bindings and internal calls add no events. Each open selector captures
  usage and admitted names so geometry changes preserve its chosen identity.
- Existing prompt, visible-row and distinct session-history bounds retain
  their meanings. Session usage starts empty without reading or writing files.

## Implementation Notes

T1.1 and T1.2 are implemented in
`3adc46aa4c2e815229c86995aa143ca10023e09e`. T1.3 adds native modal/display
regressions, reference guidance, one Unreleased outcome and this report.

`command::usage::Usage` owns a bounded oldest-first event queue. Counts and last
positions are derived from that queue; no clock, permanent frequency counter,
dependency or background worker is added. `Palette` captures a clone when it
opens, filters admitted identities and ranks each text-match partition.
`Input::command_key` observes accepted names alongside the existing distinct
M-p/M-n history, before returning the command to its handler.

## Contracts Added or Changed

- Four recent slots, four frequent slots and a 256-invocation window are
  compiled policy values, independent of configurable distinct-name history.
- Text relevance precedes usage; exact typed names take precedence in every
  configured matching policy. No persisted state or new configuration leaf is
  introduced in this slice.
- Tab completes without invoking; Return retains exact/selected acceptance;
  cancellation, held-input suppression and argument ownership remain intact.

## Files of Interest

- `src/command/usage.rs`: bounded event owner and deterministic ranking.
- `src/input.rs`, `src/input/palette.rs`: session ownership, selector snapshot
  and shared named acceptance.
- `tests/commands/ranking.rs`, `tests/commands/palette.rs`: ranking matrix,
  invocation exclusions, alias acceptance and complete catalog handler parity.
- `tests/modal/contexts.rs`, `tests/display/ranking.rs`: shared entry gestures,
  argument cancellation, rendering and selected identity across resize.
- `docs/reference/commands.md`, `docs/reference/prompts.md`, `CHANGELOG.md`:
  delivered session behavior and unchanged controls.

## Validation

All five required commands passed locally, in order:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test modal --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The suites passed 328 tests: commands 168, display 42 and modal 118. Six new tests
cover ranking groups, eviction at 256 events, match policies and exact names,
invocation exclusions and acceptance aliases, admitted AI names, both modal
entry gestures and selected identity across resize. Documentation validation
passed 85 reachable pages and 573 local links/anchors.

The four focused ranking tests passed with
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands ranking -- --test-threads=1`.
The initial command-suite run passed 167 tests and exposed one catalog parity
fixture that compared unequal usage: named `execute-extended-command` now
promotes itself in the reopened selector. Equalizing that fixture's prior usage
retains its complete-frame comparison. Its focused rerun passed with
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands palette::catalog_dispatch -- --exact --test-threads=1`.
No test or assertion was disabled or weakened.

Observed successful command times were 2.89s formatting, 0.16s cached Clippy,
300.26s tests, 1.16s documentation and 0.01s whitespace. The first worktree test
build took 1m42s; the initial Clippy run took 35.02s. Canonical clean-HEAD
evidence belongs to Looper receipts for the final PR-reference commit.

These are slice checks; integrated acceptance belongs to the controller's final
gate. Fixtures are suite-owned; no persistent service is required.

## Risks and Follow-ups

Usage is session-local. Bounded persistence and publisher failure behavior remain
Slice 2; restart/PTY acceptance, complete guidance and joint closure remain
Slice 3. The separate keybinding-hint ticket keeps its scope.
