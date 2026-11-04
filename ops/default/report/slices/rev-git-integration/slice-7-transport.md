# Slice 7: Separate Fetch and Reviewed Push

Status: reviewing
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

T7.2 implements reviewed upstream push:

- p / `git-push` prepares the selected local branch. * retains one exact push
  source independently of navigation, index and History marks; - clears it.
  M p on a selected remote constrains that marked branch's configured upstream.
- Preparation resolves one configured remote and destination and checks Git's
  actual upstream relation. A scrollable native review shows the full source
  ref/object, configured remote name and full destination. Only y accepts.
  Return does not submit; n or C-g cancels. The accepted preparation is used once.
- Mirror, follow-tags, recursive publication, multiple push URLs, conflicting
  push remotes/refspecs and ambiguous tracking reject before mutation. The
  explicit full-object refspec and no-expansion argv prevent default matching,
  force, deletion or extra tag publication. Git retains hooks, signing and
  receiver protections.
- Verification queries the private effective push endpoint, including when it
  differs from the fetch URL. Actual child failure, checked destination state
  and unavailable publication verification are distinct; no automatic retry runs.

T7.3 establishes bounded failure and uncertainty behavior:

- A real edit to the selected fetch refspec's force policy invalidates the
  captured configuration before a changing child, retaining existing tracking.
- An actual successful push followed by temporary unavailability of its owned
  bare endpoint produces an indeterminate publication outcome. The retained
  child exit remains successful; restoring that fixture proves the real object
  was published once. No success response or remote state is fabricated.
- Owned remote helpers invoke real Git credential fill with an owned failing
  credential helper. Noninteractive failure receives a fixed safe diagnosis;
  endpoint, helper stderr and credential sentinels remain absent from records.
  These fixtures prove helper/credential ownership, not live network authentication.
- Quiet deadline, excessive-output and explicitly cancelled helpers use the
  existing finite transport, output, group cleanup and five-second fixture
  bounds. Records preserve possible effects and cancellation/timeout/output
  distinctions. All owned children finish cleanup; no retry runs automatically.

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

`git-fetch`, `git-push`, the independent Refs bindings and complete active sample are
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

T7.2 development formatter, strict Clippy, 21 reference tests and documentation
checks pass (512 commands, unchanged configuration/page/link counts). Exact
single-branch publication in both object formats, conflicting-policy rejection
and stale confirmation tests pass. The first four-test push run passed three
cases but exceeded the unchanged five-second fixture limit during receiver
rejection cleanup. No process or assertion was changed; the focused command
below passed in 2.36 seconds. The first native test assumed a remote row after
refresh; its no-mark assertion now explicitly selects that remote. Both
profiles then passed native review, cancellation, publication and source retention.
Initial compile/lint corrections were limited visibility and one needless borrow.

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration transport::push::receiver_rejection -- --test-threads=1` passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration transport::presentation::native_push -- --test-threads=1` passed.

T7.3 focused development command:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test git_integration transport::acceptance:: -- --test-threads=1` passed all four tests in 9.95 seconds.

The first configuration-change fixture also changed public upstream facts and
therefore reached the earlier stale-ref rejection. It now changes only fetch
force policy, retaining the same tracking identity so the configuration check
is exercised directly. Its missing local capture binding was corrected before
the successful run. The no-changing-child and unchanged-tracking assertions,
fixture deadlines and all resource ceilings remain enforced. Shared ref
invalidation diagnostics now name the request rather than tag-only parameters.
The final candidate will run the required ordered prefix, the shared 107-test
Git baseline, command/configuration references, documentation and whitespace
through the controller after the PR-reference follow-up. Expected warm cost
is several minutes; this is component regression, not the selected full gate.

## Risks and Follow-ups

Canonical PR-reference publication and successful exact-HEAD checks remain
pending. Combined PTY/resource acceptance remains Slice 13 and the full
User Guide remains Slice 14.
