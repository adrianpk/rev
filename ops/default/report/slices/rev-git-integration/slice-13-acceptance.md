# Slice 13: Combined Native PTY and Resource Acceptance

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-13-acceptance`
PR: pending

## Purpose

Verify GIT01-GIT20 together through T13.1-T13.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T13.1 adds one native Rust entry and Bash driver, sharing the integration
  target's real installed-Git repository/configuration/identity fixtures.
- SHA-1 and SHA-256 observations distinguish actual HEAD, index and working
  bytes; both profiles enter Status and restore the native source owner.
- Both-profile Linux PTY probes decode actual C-x g and q, measure quiet Git
  Status, and verify source bytes, application exit and terminal restoration.
- T13.2 adds representative native journeys through partial-hunk and whole-path
  staging, retained commit drafts and explicit submission, marked fetch/push to
  an owned local bare remote, exact conflict-side replacement and reviewed abort.
  Assertions check actual blobs, index/worktree bytes, refs and sequence files.
- Frozen routing expectations cover 289 contextual bindings in each profile,
  including complete family prefixes and their distinct contextual meanings.
  Mapping assertions complement the existing per-variant semantic fixtures.
- The expanded host PTY journeys use actual mouse events, diff expansion,
  staging, literal composer text, cancellation/retained draft, C-c C-c submission,
  18-by-72 resize, host commands, a live retained terminal, declined application
  exit, explicit child exit and source/TTY restoration.

## Implementation Notes

The shared Linux PTY owner retains its eight-second lifetime and 1 MiB stream
cap. Git navigation legitimately hides the cursor. The initial probe reached
clean Status but lacked frame geometry; the wrapper now announces its actual
24-by-100 geometry through the existing hidden-frame parser. No deadline,
stream cap or observable assertion was weakened. Fixtures use owned temporary
roots and isolated homes/configuration; no user repository or remote is used.

## Contracts Added or Changed

The acceptance executable and Bash entry add validation only. Runtime behavior,
startup preferences, dependencies and vendor code remain unchanged.

## Files of Interest

- `examples/git-integration-acceptance.rs`: one native acceptance entry.
- `examples/git_integration_acceptance/`: native journeys and PTY adapters.
- `scripts/check-git-integration.sh`: finite isolated fixture orchestration.
- `tests/git_integration/support.rs`: existing shared real Git fixture.
- `examples/completion_acceptance/terminal.rs`: existing bounded Linux PTY owner.

## Validation

T13.1 development checks passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --example git-integration-acceptance -- -D warnings`: strict acceptance-target Clippy.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example git-integration-acceptance -- --native`: both formats and both profiles.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked --bin rev --example git-integration-acceptance`: native executable and editor build.
- `target/debug/examples/git-integration-acceptance --pty "$PWD/target/debug/rev" "$PWD/scripts/check-git-integration.sh"`: both profiles, zero CPU ticks during each 350 ms quiet interval, 11,909 captured bytes each; startup 169,341/239,922 microseconds, peak RSS 29,148/30,376 kB on this development build. These are local observations, not release measurements.

T13.2 development checks passed with the same exact native, build, PTY and
strict acceptance-target Clippy commands above. The final expanded PTY probes
captured 179,520/179,510 bytes, with startup 194,264/294,350 microseconds and
zero CPU ticks during each 350 ms quiet interval. Native journeys pass in both
profiles/object formats. Each native wait retains its five-second bound; the
Bash native phase admits 60 seconds for the combined multi-operation journey,
while each host PTY keeps its original eight-second and 1 MiB limits.

Fixture corrections bind the native file-tree root to the isolated repository
before entering Git from a terminal buffer and assert the exact repository
identity before changing actions. Terminal-rendered cells are decoded through
the existing row parser, and owned child exit requires the actual Buffer closed
receipt before application shutdown. These corrections preserve the intended
observable assertions and process/stream limits.

The completed slice's ordered exact-HEAD validation belongs to the controller
receipt after its PR-reference follow-up.

## Risks and Follow-ups

T13.3 resource/lifetime acceptance remains in progress. No full delivery-set gate is claimed.

An exploratory source-quality scan across 86 Git runtime/acceptance files
reported parameter, nesting and complexity violations in existing runtime
owners and several acceptance functions. Strict acceptance-target Clippy passes.
The broad source scan is retained as a failed development diagnostic; its
findings have not triggered runtime rewrites. T13.3 still needs its explicit
source-check coverage, binary/startup/RSS/service measurements and owned
process/FD lifetime evidence before slice readiness.
