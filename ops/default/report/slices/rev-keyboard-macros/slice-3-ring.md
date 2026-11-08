# Slice 3: Ring, append and counters

Status: reviewing
Delivery set: rev-keyboard-macros
Plan: [Keyboard Macros Delivery](../../../plan/rev-keyboard-macros.md)
Tracker: [Keyboard Macros Tracker](../../../tracker/rev-keyboard-macros.md)
Branch: `feat/rev-keyboard-macros-03-ring`
PR: [#321](https://forge.adrianpk.com/adrianpk/rev/pulls/321)

## Purpose

Deliver T3.1-T3.3: a global bounded ring, temporary repeat controls, checked
append and per-definition integer counters through native editing and prompts.

T3.1-T3.3: [native ring and counter implementation](https://forge.adrianpk.com/adrianpk/rev/commit/68ec75071860e531f96046b4b99b63cbec8541f9).

## Delivered Behavior

- The shared session ring retains 1..32 immutable definitions (default eight).
  Rotate older/newer, swap, delete or execute the second without changing the
  selected head. Revisions and counter metadata replace values without mutating
  previously pinned bodies.
- C-x e enables bare e repeats; ring actions enable C-k/C-l/C-n/C-p/C-d/C-t.
  Unrelated input leaves the temporary map and keeps its normal dispatch.
  C-g cancels automatic work and repeat ownership while retaining real edits.
- Supplied start prefixes append. Plain C-u runs once before capture, double
  plain C-u skips execution, and the configuration can disable pre-execution.
  Failed preliminary runs prevent capture; canceled, empty or oversized capture
  retains the previous body. Each automatic iteration keeps its native undo unit.
- Nonpositive replay counts execute forward until native failure, cancellation
  or the finite outer budget. Successful saturated movement remains success.
- Each fresh recording starts current, previous and iteration-start at zero.
  Inserts, numeric increments, assignment, addition, iteration reset and global
  last-inserted restore preserve distinct prefix forms. Signed 64-bit overflow
  stops before the overflowing operation.
- Native prompts accept literal counter values and bounded d/o/x/X formats.
  Accepted operations replay semantically without reopening prompts. Formats
  retain literals, escaped percents, flags and bounded width/precision; invalid
  input preserves the previous format. Outside format changes affect only new
  recordings, while in-body changes keep their position in the definition.

## Implementation Notes

The ring owns immutable definition values with stable session identities and
checked body revisions. Bodies share immutable storage; counter metadata updates
cannot mutate an admitted runner or pinned snapshot. The 32-entry,
4,096-step and 256-KiB encoded-definition limits bound this slice's native
allocations below the 48-MiB retained macro ceiling, including a working recorder
and runner. Named-snapshot admission remains owned by Slice 5.

Append pins its base, optionally completes a native run, then seeds bounded
capture from that body. Publication replaces the selected identity only after
successful additional capture. Native failures retain partial source history
and publish the last successful run counter state without publishing a body.

Counter steps retain admitted operations and typed insertion arguments. The
standard-library formatter accepts exactly one integer conversion, checks
256-byte formats, width/precision up to 128 and 512-byte results, and handles
signed-minimum magnitude without overflow. Counter insertion uses native checked
paste/edit admission and the runner's existing history and generated-text budget.
Explicit redisplay yields a frame at its step boundary. Consent/layout hooks
change history groups only while a macro owns them; ordinary typing retains its
native grouping across idle input polling.

## Contracts Added or Changed

- Eleven canonical macro commands, descriptions and shared default gestures.
- `macros.ring_limit = 8` (integer 1..32) and
  `macros.execute_before_append = true` (boolean), merged independently in the
  immutable configuration snapshot pipeline.
- The packaged sample, configuration/default tables, command/binding catalogs
  and description oracle contain the delivered surfaces.
- The existing Unreleased product entry now includes ring, append and counters.

## Files of Interest

- `src/macros.rs`, `src/macros/counter.rs`: immutable definitions, ring state,
  checked counters and bounded formats.
- `src/session/macros/`: native controls, append continuation, temporary maps,
  semantic capture and cooperative playback.
- `src/config/macros.rs`, `src/input/keymap/defaults.rs`: configuration and
  shared gestures for ordinary and modal profiles.
- `tests/macros/ring.rs`, `tests/macros/counters.rs`,
  `src/session/macros/tests.rs`: native workflows, boundaries and pinned values.

## Validation

Development commands below passed while preparing this slice. The controller
validator must record the unchanged ordered selection on the final clean PR head
after the PR-reference commit. These results do not establish the integrated
delivery-set gate or real PTY acceptance.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings` — PASS.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- ring counters --test-threads=1` — PASS: 17 native ring/counter cases, including modal states and idle history.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1` — PASS: 52 selected command cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test editing -- --test-threads=1` — PASS: all 114 editing cases.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` — PASS: 644 command entries, 293 configuration entries, 92 pages and 639 links/anchors.
- `git diff --check` — PASS.

Focused development checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test macros -- recording runner capture --test-threads=1` — PASS: 26 cases (25 prior macro cases plus semantic counter prompt capture).
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::macros::tests -- --test-threads=1` — PASS: 8 cases, including pinned revisions, append step bounds and exact-budget completion.

## Risks and Follow-ups

Snapshot tests pin real recorded definitions; they do not claim named command
admission. Queries, region execution, nested calls, naming, persistence and
definition/menu editing retain their later recorded slices. Full PTY and
large-buffer acceptance remain Slice 8.
