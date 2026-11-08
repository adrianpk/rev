# Slice 1: Recording and replay

Status: reviewing
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-01-recording`
PR: pending

## Purpose

Deliver T1.1-T1.3: a usable native recording/replay increment with bounded
ownership, shared controls, undo boundaries and focused integration evidence.

## Delivered Behavior

- F3 and C-x ( start recording real source editing. F4 and C-x ) finish without
  replay; F4 subsequently replays the last definition. C-x e finishes and runs
  once more. Positive prefixes count playback iterations; a positive C-x ) count
  includes the initial recording pass. C-x C-k C-s shares the start handler.
- Capture retains logical normalized keys, including compatible Escape/Meta,
  and exact UTF-8 paste. Prefix inputs remain attached to their native effects.
  Releases, presentation events and recording controls are excluded. Def remains
  visible in the active status area. Configured bindings retain precedence.
- Both profiles use native input and edit owners. Modal entry states are checked
  before effects and between iterations. Missing/stale source ownership and
  native refusals stop playback with typed iteration/step evidence.
- Every outer iteration forms one native undo unit in each affected buffer.
  Adjacent typing and other iterations remain separate. Cancellation retains
  partial effects, closes their unit and releases automatic wake ownership.
- Empty, canceled, unsupported and over-limit capture preserves the previous
  valid definition. Unsupported prompt, external-job, pointer and context
  transfers stop before their effect rather than recording a false success.

## Implementation Notes

The cohesive macro owner retains one bounded recorder, one immutable current
definition and one pinned runner. Native input exposes its last logical chord
after normalization; the session stages pending prefixes until a real operation
is accepted. Playback re-enters ordinary native input without terminal escape
synthesis, a second edit dispatcher, dependency or background worker.

The existing host deadline services at most 256 logical inputs and a four-ms
cooperative target per turn. Deferred native layout completes before another
macro step. Source/view continuation checks reject external changes. Limits are
4,096 captured steps, 256 KiB admitted definition representation, 1,000,000 run
steps/positive iterations, 64 MiB generated text and 60 seconds of automatic work.
The small fixed set of bounded retained definitions stays below the aggregate
48 MiB macro-memory ceiling; there is no ring, name table or recent-input store.

Buffer history owns explicit automatic iteration identities. Undo/redo replays
each original checked primitive mapping within that unit, preserving disjoint
edits and source anchors. History pruning counts whole logical units. A shared
native publication budget rejects excess generated text before publication.

## Contracts Added or Changed

- Five canonical native macro commands, descriptions and both-profile defaults.
- Logical function-key spelling in the native keymap vocabulary.
- Session macro status and typed stop observations, independent of echo parsing.
- Native automatic history units and generated-text admission across buffers.
- Current command/binding references, the five approved macro description rows
  in the shared oracle, and one usable Unreleased product outcome.

## Files of Interest

- `src/macros.rs`, `src/session/macros.rs`, `src/session/macros/runner.rs`: state,
  controls, capture, admission and cooperative playback.
- `src/input.rs`, `src/input/meta.rs`, `src/input/keymap/`: logical input and maps.
- `src/buffer/edit/automatic.rs`, `src/buffer/edit/history.rs`: native undo units
  and shared publication limits.
- `tests/macros.rs`, `tests/macros/`: fourteen native recording/runner cases.
- `docs/reference/commands.md`, `docs/reference/bindings.md`, `CHANGELOG.md`:
  available gestures, catalog descriptions, boundaries and release notes.

## Validation

All seven ordered slice checks passed at clean HEAD
`c27600d50ae15d6897272b2ef273697d3b98c363`, recorded by controller validation
receipt `MGFGZPZUIDPKGVFAFR4ZMWFEWJ`. The integrated delivery-set gate and real
PTY acceptance were not run.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording runner --test-threads=1` — PASS: 14 recording/runner cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — PASS: 52 command cases, including palette::catalog_reference.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1` — PASS: 114 editing cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — PASS: 633 command entries, 291 configuration entries, 92 reachable pages and 639 local links/anchors.
- `git diff --check` — PASS.

The unchanged seven-command slice selection also applies after PR-reference
commits. Fresh controller evidence identifies that final clean HEAD and ordered
execution; a passing slice receipt does not establish delivery-set acceptance.

## Risks and Follow-ups

Receipt `NSK6367K3IZTF5IQM2YKDPQ7ZO` exposed four existing missing description
oracle entries. On 2026-11-08 the owner explicitly authorized restoring only
prefix-argument, universal-argument, digit-argument and negative-argument to match
their compiled descriptions. Repair `c27600d50ae15d6897272b2ef273697d3b98c363`
applies that bounded exception to the plan's catalog-debt exclusion. No runtime,
test, assertion, gate or controller configuration changed in the repair. The
unchanged seven-command selection now passes, including the catalog assertion.

Later approved slices own named-command and accepted-prompt capture, completion,
ring/repeat maps, append/counters, nonpositive repetition, queries/regions, named
snapshots, explicit persistence, definition/menu editing and complete guidance.
Those surfaces are explicitly unavailable in this increment. Real PTY and large
buffer acceptance remain assigned to Slice 8.
