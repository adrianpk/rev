# Slice 13: Combined Native PTY and Resource Acceptance

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-13-acceptance`
PR: [#270](https://forge.adrianpk.com/adrianpk/rev/pulls/270)

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

- T13.3 verifies four simultaneous real repository refreshes, explicit path
  truncation, retained observations/output and refusal before overcommitting
  process-record bytes. Quiet Git creates no timer or readiness wakes.
- A real 262,144-byte Git blob exercises a 4,096-byte job cap and three-record
  retention. Malformed porcelain injection preserves the last confirmed
  observation with failed freshness instead of publishing a false clean state.
- Both profiles retain an owned Git leader and descendant on q/declined exit;
  explicit termination completes within the existing cleanup bounds, preserving
  actual HEAD and saved bytes. Descriptor and worker counts return to baseline.
- The driver compares the optimized executable against fresh archived source
  at the approved pre-Git baseline, measures startup/idle/RSS and reports phase
  costs. Manifest and lockfile comparison establishes zero dependency additions.

## Implementation Notes

The shared Linux PTY owner retains its eight-second lifetime and 1 MiB stream
cap. Git navigation legitimately hides the cursor. The initial probe reached
clean Status but lacked frame geometry; the wrapper now announces its actual
24-by-100 geometry through the existing hidden-frame parser. No deadline,
stream cap or observable assertion was weakened. Fixtures use owned temporary
roots and isolated homes/configuration; no user repository or remote is used.

The four-child fixture uses the approved 2,097,152-byte process-record default.
An initial 262,144-byte aggregate could admit only one output reservation; that
smaller policy now has a separate refusal assertion preserving the live job.
Four children retain the same per-job output, context and 32 MiB session bounds.
Repeated direct-output requests capture a fresh canonical Status before each
new preparation; stale post-failure evidence is not reused.

Only corrupt-porcelain refusal uses an explicit test double: installed Git
cannot reproducibly produce malformed porcelain. Discovery still uses real
Git; the injected status record verifies parser/freshness handling and does not
claim real Git corruption. All repository effects, transport and large-output
producers use actual installed Git and owned local repositories.

Strict source quality covers the acceptance entry, every acceptance module,
and the reused fixture, metrics and Linux PTY owners. The native/PTY code is
split at workflow boundaries to satisfy the unchanged parameter, nesting and
complexity checks. Runtime Git sources and all hard resource limits are unchanged.

## Contracts Added or Changed

The acceptance executable and Bash entry add validation only. Runtime behavior,
startup preferences, dependencies and vendor code remain unchanged.

## Files of Interest

- `examples/git-integration-acceptance.rs`: one native acceptance entry.
- `examples/git_integration_acceptance/`: native journeys and PTY adapters.
- `scripts/check-git-integration.sh`: finite isolated fixture orchestration.
- `examples/git_integration_acceptance/resources.rs`: real concurrent/output
  admission, retention, malformed input, RSS, descriptor and worker assertions.
- `examples/git_integration_acceptance/lifetime.rs`: guarded owned-process exit.
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

T13.3 development checks passed:

- `cargo fmt --all -- --check`.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --example git-integration-acceptance -- -D warnings`.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-git-integration.sh --build`: optimized candidate/baseline builds, strict source quality and zero dependency additions.
- `target/release/examples/git-integration-acceptance --resources`.
- `target/release/examples/git-integration-acceptance --lifetime`.
- `target/release/examples/git-integration-acceptance --pty "$PWD/target/release/rev" "$PWD/scripts/check-git-integration.sh"`.
- `target/release/examples/git-integration-acceptance --startup "$PWD/target/git-baseline-772051f7a70b4be66d86ccf4fdea27f9b6c2da68/release/rev" "$PWD/scripts/check-git-integration.sh"`.

Observed release-build measurements (local development evidence):

| Measurement | Result |
| --- | --- |
| Cold candidate/baseline build | 408 / 355 seconds |
| Baseline/candidate executable | 22,454,440 / 24,444,576 bytes |
| Git delivery executable delta | +1,990,136 bytes |
| Four repositories | 2,048 input paths, 128 retained paths, 100,762 charged bytes |
| Fair service | All four contexts complete; 53 poll turns, longest observed poll 20 microseconds |
| Retained stream capacity | 1,106 bytes in three records after repeated output limits |
| Resource phase / peak RSS | 5,504 milliseconds / 12,692 kB |
| Completed resource and exit fixtures | Four descriptors and one thread, matching each baseline |
| Accepted owned-process exit | 553,853 / 545,200 microseconds by profile |
| Candidate startup | 62,493 / 81,480 microseconds; 4 / 6 CPU ticks at first frame |
| Candidate peak RSS after Git Status | 15,196 / 16,176 kB |
| Baseline startup | 47,315 / 59,085 microseconds; 2 / 4 CPU ticks at first frame |
| Quiet baseline, ordinary candidate and Git | Zero CPU ticks in each 350 ms interval |
| Expanded candidate PTY output | 229,526 / 229,516 bytes |

Both binaries use the same installed compiler, release profile, locked offline
dependencies and one build job, with separate target directories. Baseline
source is freshly archived from `772051f7a70b4be66d86ccf4fdea27f9b6c2da68`.
RSS includes allocator/editor overhead and is distinct from charged Git capacity.
Observed timings are measurements, not new responsiveness guarantees.

The completed slice's exact clean PR head and ordered successful commands are
recorded by the controller validation receipt after the PR-reference follow-up.
The full driver prints final per-phase costs into that receipt's retained log.

## Risks and Follow-ups

The earlier exploratory scan of integrated runtime owners remains a failed
complexity diagnostic. The required driver keeps explicit source coverage of
this acceptance entry and its shared fixtures; runtime rewrites are outside
this validation change. No delivery-set gate or deployment is claimed here.

## Integration

[PR #270](https://forge.adrianpk.com/adrianpk/rev/pulls/270) merged through
controller-owned Rebase + Fast-forward into dev at
`d15ab4469f3a05604117276340d87d0f834c871f`. Controller receipt
`YQZ3ALMQDPBYYUWCT7PUZ2CREN` passed all five ordered checks at that exact head,
including the combined driver and strict acceptance-target Clippy. Its retained
log records the final phase measurements. This closes Slice 13 of 14; the
selected delivery-set gate remains controller-owned after Slice 14 integration.
