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

The completed slice's ordered exact-HEAD validation belongs to the controller
receipt after its PR-reference follow-up.

## Risks and Follow-ups

T13.2 combined workflows/matrix and T13.3 resource/lifetime acceptance remain
in progress. No full delivery-set gate is claimed.
