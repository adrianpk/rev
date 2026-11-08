# Slice 2: Named commands and prompt capture

Status: delivered
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-02-capture`
PR: [#320](https://forge.adrianpk.com/adrianpk/rev/pulls/320)
Integrated: `e92a1f3e0f5a700d1fe52c1860fffab5a097bea7` on `dev`

## Purpose

Deliver T2.1-T2.3: retain accepted native command identities, prompt values and
literal completion edits without replaying selector presentation or consent.

T2.1-T2.3: [native capture implementation](https://forge.adrianpk.com/adrianpk/rev/commit/838968c735dd6903f9632e96095aa0acb0667af9).

## Delivered Behavior

- M-x and modal colon record one canonical command name and its typed prefix.
  Query text, candidate movement and selector completion are excluded. Changed
  ranking or bindings cannot retarget a named invocation; ordinary key entry
  continues to resolve the current effective keymap.
- File paths, buffer prompt/list selections, goto, searches and replacement
  accept literal values qualified by native prompt roles. Prefixes remain with
  the invocation. An incompatible rebound prompt stops before consuming an
  argument. Missing command names and failed searches have typed stops.
- Manual completion captures checked literal edits, relative scalar ranges,
  expected source and the resulting cursor. Playback uses native edit admission
  without launching a completion provider or retaining a candidate ordinal.
- Save/overwrite, buffer-close and query-replace decisions remain native prompts.
  Playback waits for fresh answers with no automatic macro wake. Consent is not
  recorded, user wait time is excluded from active work time, and cancellation
  preserves the partial iteration as one native undo unit.
- Unsupported external jobs, terminal transfers and directory/context replacement
  stop before launch or transfer. Expanded commands do not add usage events.

## Implementation Notes

The bounded recorder suspends selector input and retains an incomplete command
until its native argument owners finish. Accepted names carry typed prefixes;
key-entered prompting commands retain their original logical trigger inputs.
Each native owner supplies an accepted role/value after validation. Replays
validate that role against the live pending owner rather than parsing echo text.

The pinned runner services command triggers and arguments within the existing
256-input/four-millisecond cooperative turns and shared outer budgets. Consent
releases automatic wake ownership, retains the native source check and history
group, and resumes through ordinary input. Native failures retire the owned
prompt and close the group without rolling back source edits.

Completion capture occurs after native edit preparation and before publication.
Its inert literal contract uses relative scalar offsets and expected text;
replay prepares and publishes through the same checked edit/history owner.
Capture limits include command arguments, triggers and escaped literal edits.
No dependency, provider fixture, new runtime worker or persistence surface is added.

## Contracts Added or Changed

- Native command steps with canonical identities, typed prefixes and accepted
  role-qualified arguments, preserving logical-key invocation semantics.
- Literal completion source/range preconditions and native cursor publication.
- Public typed layout/consent wait observation, plus missing-name and prompt-role
  stop reasons, independent of human-facing echo messages.
- Current macro reference boundaries and the existing Unreleased product entry.

## Files of Interest

- `src/macros.rs`, `src/macros/capture.rs`: bounded definition, recorder and
  accepted argument/completion values.
- `src/session/macros/`: capture, finite playback, consent and literal editing.
- `src/session/search/`, `src/session/path.rs`, `src/session/buffers/`,
  `src/session/replace.rs`, `src/session/completion.rs`: native acceptance owners.
- `tests/macros/capture.rs`, `src/session/macros/tests.rs`: native integration
  and focused admission evidence.

## Validation

All seven ordered checks passed at clean final PR head
`e92a1f3e0f5a700d1fe52c1860fffab5a097bea7`, recorded by controller receipt
`CTGMHPV3BOOPKLL2GBEWHP7ZFX`. Forgejo and Acta confirm PR #320 merged into `dev`.
These checks do not establish the integrated delivery-set gate or real PTY acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- capture --test-threads=1` — PASS: 11 capture cases, including parameterized profiles and buffer selectors.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — PASS: 52 selected command cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test completion -- manual_edit cancel stale --test-threads=1` — PASS: 4 selected completion cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — PASS: 633 command entries, 291 configuration entries, 92 reachable pages and 639 links/anchors.
- `git diff --check` — PASS.

Focused development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording runner capture --test-threads=1` — PASS: all 25 native macro cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::macros::tests -- --test-threads=1` — PASS: 5 native admission cases for changed ranking/bindings, unsupported job admission, missing names, mismatched prompt roles and search continuation failure.

## Risks and Follow-ups

Built-in command names cannot disappear in a live session. The missing-name
fixture changes only the retained name of a real recording to exercise resolver
refusal; it does not claim dynamic named-macro admission, which belongs to Slice 5.

Slices 3-8 now deliver the subsequent ring/repeat maps, append/counters, macro
query pauses, regions/nested execution, named definitions, explicit persistence,
definition/menu editing and complete PTY/large-buffer acceptance. The
[Slice 8 closure](slice-8-acceptance.md#delivery-set-closure) records the passing
integrated gate and evidence limits. This report retains its original slice
receipt and validation scope.
