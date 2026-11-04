# Slice 7: Separate Fetch and Reviewed Push

Status: drafting
Delivery set: rev-git-integration
Plan: `ops/default/plan/rev-git-integration.md`
Tracker: `ops/default/tracker/rev-git-integration.md`
Branch: `feat/rev-git-integration-07-transport`
PR: pending

## Purpose

Deliver GIT09 and GIT16-GIT19 through approved T7.1-T7.3 in the
[plan](../../../plan/rev-git-integration.md) and
[tracker](../../../tracker/rev-git-integration.md).

## Delivered Behavior

T7.1 implements explicit fetch:

- Select an exact configured remote row in Refs and invoke & / M f or
  `git-fetch`. M b reopens Refs. Missing selection reports its requirement.
- Installed Git receives that configured name and its observed transport and
  fetch refspec policy. Complete current refs, HEAD and private effective
  configuration are rechecked before launch. A remote without a configured URL
  cannot fall back to a repository path.
- Success refreshes compatible Status and Refs, including actual tracking
  objects. Fetch does not run pull, merge or rebase, even with pull policy set.
  Modified visiting source, index, current branch and disk edits remain owned.
- Processes retains safe argv, actual exit and outcome. Raw transport/helper
  and configuration output is withheld to protect embedded credentials.
  Started work retains ordinary cancellation and view-exit ownership.

## Implementation Notes

The existing Git owner serializes shared repository storage and owns finite
worker/output lifetimes. Fetch uses `git.transport_timeout_seconds`; private
configuration and parsed metadata share the existing output, 4096-byte field
and 32 MiB session admission. The complete effective configuration is compared
without storing its values in public observations or operation records.
After a possible effect, checked local facts replace old authority. Failure or
unavailable verification never triggers an automatic retry. Revalidation does
not claim an exclusive transaction against other Git clients.

## Contracts Added or Changed

`git-fetch`, the independent Refs bindings and complete active sample are
usable in both profiles. Local ref checking and result refresh are shared by
the existing ref and transport worker children. No dependency was added.

## Files of Interest

- `src/git/refs/transport.rs`, `src/git/work/refs/transport.rs`: exact configured
  name admission, private configuration checks, fetch and result verification.
- `src/session/git/refs.rs`: explicit remote selection and native result feedback.
- `tests/git_integration/transport.rs`, `transport_presentation.rs`: real local
  bare remotes, both object formats and both native profiles.

## Validation

T7.1 development checks passed:

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev --test git_integration -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration transport:: -- --test-threads=1` (3 tests, 5.96 seconds)
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands --test configuration reference -- --test-threads=1` (21 tests)
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check` (511 commands, 267 configuration entries, 76 pages and 485 links)
- `git diff --check`

The initial compile required adding the new operation's process label; all
focused tests passed after that correction. Fixture deadlines, output limits
and acceptance assertions were preserved. Final exact-HEAD evidence remains
pending until T7.2-T7.3 and the canonical PR-reference follow-up are complete.

## Risks and Follow-ups

T7.2 reviewed upstream push and T7.3 transport failure/uncertainty acceptance
remain pending. Combined PTY/resource acceptance remains Slice 13 and the full
User Guide remains Slice 14.
