# Slice 5: Named snapshots and key bindings

Status: reviewing
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-05-names`
PR: pending

## Purpose

Deliver T5.1-T5.3: independent session command snapshots, native key binding
capture and bounded pinned named calls through the existing editor owners.

## Delivered Behavior

- C-x C-k n names the current recording for this session. Native M-x and modal
  colon expose its canonical identity, normal ranking, optional description and
  effective key hint. Ring append, eviction and playback retain independent
  named bodies and working counters.
- Compiled commands, prompt actions and admitted AI commands keep their names.
  Replacing another session name requires a fresh decision. Missing or stale
  targets stop before child effects; admitted bodies remain pinned through a
  pause even when their session name is replaced.
- C-x C-k b displays and reads a normalized native sequence. Single digits and
  uppercase letters expand to reserved C-x C-k slots. An existing matching name
  supplies the target; otherwise the key owns an anonymous independent snapshot.
  Rebinding a name moves its one session key. Occupied source leaves or prefixes
  require consent; native cancellation and prefix arguments remain reserved.
- The source-only binding layer preserves configured maps beneath it. Prompt,
  component and raw-terminal owners keep their input. Logical key capture is
  bounded to eight chords, ignores repeats and rejects unsupported notation.
- Named calls captured through selectors or physical bindings retain the stable
  name once. Nested calls share the outer history unit and execution budgets,
  return to pinned native continuations and reject name cycles or more than
  sixteen active frames. Session names and keys survive context replacement.
- At most 256 names and 256 session bindings fit within the shared 48 MiB
  retained-memory ceiling. Naming, binding, replay and exit create no library
  file. Only actual user selector acceptance contributes command usage.

## Implementation Notes

A bounded Copy macro-name value joins the existing command identity owner.
The native palette merges admitted session identities with compiled and AI names;
optional descriptions remain identity-qualified and display as inert single-line
text. Runtime lookup revalidates the same collection instead of accepting a row
number or retaining a mutable ring index.

The macro owner snapshots definition metadata and shares immutable step storage.
Each call pins its definition in a bounded parent continuation. Native editing,
source checks, manual pauses, counters and cooperative scheduling keep their
existing owners. Child iterations share the caller's automatic history group
and outer work, iteration, text and active-time allowances.

The input owner reads native chords in a focused prompt and resolves the session
source layer before ordinary maps, with native argument and modal grammar guards.
Replacement consent rechecks source point and catalog generation. Hints inspect
that same effective resolver. A later AI save also checks the session namespace
before publishing its existing concern-owned configuration edit.

Retained-memory admission counts distinct immutable bodies once, definition and
catalog metadata, recorder drafts, queued arguments, pinned call/manual frames
and live original-line target sets. Cached body sizes keep admission independent
of body text length. Refusal retains valid published definitions and real source
effects. No dependency, idle worker or new configuration leaf is introduced.

## Contracts Added or Changed

- `kmacro-name-last-macro` and `kmacro-bind-to-key`, their descriptions and two
  shared global gestures in ordinary and modal maps.
- Typed dynamic macro identities and internal anonymous binding dispatch,
  optional native palette metadata and session source-layer hints.
- Cycle, depth and retained-memory stopping outcomes, plus session catalog
  ownership across context replacement.
- Canonical descriptions, command/binding/default references, complete sample
  configuration and the existing Unreleased macro outcome.

## Files of Interest

- `src/macros/names.rs`, `src/session/macros/names.rs`: snapshot, namespace,
  resource admission and checked naming/binding consent.
- `src/session/macros/calls.rs`, `src/session/macros/runner.rs`: pinned native
  continuations, shared budgets and canonical call lookup.
- `src/input/macro_bindings.rs`, `src/input/palette.rs`, `src/input/hints.rs`:
  native key capture, scoped dispatch, descriptions, ranking and effective hints.
- `tests/macros/names.rs`, `src/session/macros/tests.rs`: native workflows,
  context/namespace boundaries, retained memory and call admission.

## Validation

The development selection passed. The controller validator must record the
unchanged ordered selection on the final clean PR head after PR references.
These focused checks do not establish the integrated delivery-set gate or PTY
acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — pass.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- names --test-threads=1` — pass, 14 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — pass, 52 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal -- prefix_arguments repeat count operator navigation --test-threads=1` — pass, 45 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — pass, 650 commands and 293 configuration entries; 92 pages and 639 links/anchors.
- `git diff --check` — pass.

Additional development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- --test-threads=1` — pass, 70 cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib -- session::macros::tests queries_clock regions_admission --test-threads=1` — pass, 16 native admission, budget and ownership cases.

## Risks and Follow-ups

Persistent library encoding, explicit save/reload/delete and inert TOML export
remain Slice 6. Definition/menu editing remains Slice 7. Full integrated, PTY,
restart and large-buffer acceptance remains Slice 8.
