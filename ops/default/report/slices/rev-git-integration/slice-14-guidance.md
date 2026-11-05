# Slice 14: Git Learning Path and Reference

Status: delivered
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `fix/rev-git-integration-validation-1`
PR: [#272](https://forge.adrianpk.com/adrianpk/rev/pulls/272)

## Purpose

Make GIT01-GIT20 discoverable through T14.1-T14.3 in the
[approved plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

- T14.1 adds an independent three-chapter Git learning path, task guides,
  a control/configuration reference and source/process ownership explanation.
- The local exercise shows saved/index differences, compact Status/diff
  feedback, explicit commit submission, history and local tags without Org.
- Task guides cover exact staging, groups, amend/fixup, explicit fetch and
  integration, reviewed single-branch push, ordered revisions and conflicts.

- T14.2 aligns the Spore book with the independent Git path and extends the
  existing documentation checker to verify its index, chapter links and order.
- Prepared-family procedures describe literal input, separate review/execute,
  non-overwriting output, checked application and unknown arbitrary effects.

- T14.3 consolidates Unreleased Git outcomes, maps every required family/variant
  to existing semantic assertions and records additive baseline/index links.
- Closure fields record the final controller-tested candidate and gate receipt;
  historical editor and slice receipts keep their original meaning.

## Implementation Notes

Canonical Markdown remains under the existing Diataxis quadrants. The Git
reference connects the existing complete command, key and preference catalogs
instead of maintaining a second exhaustive inventory. Git guidance is labelled
as current Unreleased behavior alongside the released everyday editing guide.

Verified controller integration of Slice 13 closes its existing report/tracker
at PR #270 and activates the recorded final documentation slice.

## Contracts Added or Changed

User documentation only; native Git, source and process contracts are unchanged.

## Files of Interest

- `docs/tutorials/user-guide/git/index.md` and chapters 17-19: independent exercise.
- `docs/how-to/review-and-commit.md`, `integrate-git-changes.md`: task procedures.
- `docs/reference/git.md`: exact catalog/settings and finite-lifetime entry.
- `docs/explanation/git-ownership.md`: source, observation and job authority.

- `publication/userguide/index.sptree`, `examples/documentation-check.rs`: shared navigation contract.
- `docs/how-to/prepare-git-commands.md`: prepared family and direct routes.

- `ops/default/spec/editor-git-integration-evidence.md`: requirements/variant map and provenance.
- `CHANGELOG.md`: user-facing Unreleased outcomes.

## Validation

Exact ordered commands for the clean PR-reference head:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
- `cargo fmt --all -- --check`
- `git diff --check`

Coverage-map links and named native assertion owners resolve. Versioned
changelog history is byte-for-byte unchanged. These are documentation
consistency checks; runtime scenarios were not rerun for this slice.

The controller retains the exact head, ordered successful command list and
log in its validation receipt after the PR-reference follow-up. This report
does not substitute for that receipt.

## Risks and Follow-ups

The controller passed the final gate at `343850fb6914bf801dfa248b8cdd0e1c34ae1fef`;
closure evidence is recorded below. The configured modal filter selected zero
tests; both-profile native/PTY coverage is reported separately.

## Integration

[PR #271](https://forge.adrianpk.com/adrianpk/rev/pulls/271) merged through
controller-owned Rebase + Fast-forward into dev at
`71524f02b11264a59a8049d4c10bd64e627017d4`. Controller receipt
`56IS2Q3SWAQ5436XOUMSJIQ3HI` passed the four ordered commands listed above at
that exact head: 85 reachable pages, 550 local links/anchors, 593 commands,
271 settings, successful Spore projection/rendering, formatter and whitespace.
This closes Slice 14 of 14. The subsequent selected full gate and
delivery-set closure are recorded below.

## Gate Repair 1: Retained Shell Configuration

The retained full gate at `9ddc7972dfbcdd7effb18dfa6d3e7be887f603fb`
failed `status::configuration_catalog`: the idle Git owner admitted 65,574
bytes while the test still expected only its fixed 65,536-byte metadata.
The other 239 Git tests passed in that retained run. Shell policy strings,
argument-vector storage and the startup shell selection became retained Git
state with the approved direct-command routes; their existing charge is valid.

The exact focused reproduction below failed with the same values at the
unchanged base. The repair retains an equality assertion and accounts for
those capacities in default and explicitly configured shell cases. Both cases
also require an idle owner with no repository contexts or process records.
No runtime accounting, resource limit, gate requirement or startup execution
behavior changes. Only `tests/git_integration/status.rs` and this existing
report change; the original Slice 14 integration evidence above is preserved.

Focused reproduction (failed before repair):

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::configuration_catalog -- --exact --test-threads=1`

The repair's clean PR-reference head is validated through the controller with
this exact ordered list; its receipt retains the actual head and results:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::configuration_catalog -- --exact --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration status::byte_admission -- --exact --test-threads=1`
- `git diff --check`

[PR #272](https://forge.adrianpk.com/adrianpk/rev/pulls/272) merged through
controller-owned Rebase + Fast-forward into dev at
`2bd1f7182cec65757d670f230d446b57164b54f5`. Controller receipt
`275HLP6E5BALNYAJTW2D54M444` passed all five repair commands above at that exact
head, including both exact configuration/accounting tests with one case each
and 239 filtered out. The repair is delivered; the subsequent unchanged
full-gate result and delivery-set closure are recorded below.

## Delivery-set closure

Completed: 2026-11-05. Fourteen canonical PRs #258-#271 and authorized repair
#272 are integrated. The controller passed the unchanged selected full gate
at immutable candidate `343850fb6914bf801dfa248b8cdd0e1c34ae1fef` in receipt
`D6BU4EM7IWRUA3UHGDBE432UK3`. The specification/model and plan/tracker
are Completed; all fourteen reports are delivered. GIT01-GIT20 delivery is closed.
The tested candidate remains distinct from this documentation-only closure.

Exact successful controller command:

```sh
cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration --test configuration --test commands --test windows --test panels --test files --test external_changes --test shell_commands -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal git_ -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib process -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-git-integration.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check
```

Recorded native test results:

| Target | Passed |
| --- | ---: |
| commands | 163 |
| configuration | 153 |
| external_changes | 21 |
| files | 75 |
| git_integration | 240 |
| panels | 19 |
| shell_commands | 35 |
| windows | 41 |
| library process filter | 12 |
| Total top-level tests | 759 |
| Additional nested child runs | 3 |

The literal modal `git_` filter selected zero tests (117 filtered out). Its
successful command exit is not evidence that modal regression tests ran.
The separate real Git driver exercised both startup profiles, both object
formats and 289 contextual routes per profile, with actual index/commit,
local fetch/reviewed push, conflicts, a retained live terminal, pointer/resize,
composer submission/cancellation, source restoration and host TTY restoration.

The release driver passed strict source quality and zero dependency additions.
Its fresh pre-Git comparison baseline remains
`772051f7a70b4be66d86ccf4fdea27f9b6c2da68`. Baseline/candidate binaries were
22,454,440/24,444,576 bytes (+1,990,136). Recorded phase times were 261 seconds
for the candidate build, 332 for the baseline build, 12 native, 4 resources,
2 lifetime, 1 baseline startup and 6 PTY. These measured phase costs are not
a new timeout or an inferred activity duration.

Four actual repositories admitted 2,048 input paths with 128 retained paths,
101,294 charged bytes under the 32 MiB ceiling, 45 service turns and a longest
observed poll of 17 microseconds. A real 262,144-byte blob hit the 4,096-byte
job cap; retained stream capacity was 1,106 bytes across three records.
The explicitly injected malformed-porcelain boundary retained the prior
confirmed observation with failed freshness. Resource/cleanup fixtures returned
to four descriptors and one thread; resource peak RSS was 12,164 kB.
Both profiles measured zero CPU ticks over each 350 ms quiet interval.
Accepted owned-process exit reaped leader/descendant state in 534,513/535,301
microseconds. Logical admission, RSS and observed timing keep distinct meanings.

Documentation passed 593 commands, 271 settings, 85 reachable pages and 550
local links/anchors. All 85 Markdown pages projected and rendered through
Spore's strict/link checks into the local User Guide. Owned acceptance and
projection fixtures completed under their existing cleanup owners; no
persistent fixture service remains to stop. This record claims local guide
validation, with no public deployment or release inferred from it.

Only configured operational closure paths differ from the tested candidate;
runtime, tests, dependencies, public guide sources and gate requirements are
preserved. Earlier failed diagnostics and slice receipts retain their original
heads and do not become evidence for a different candidate.
