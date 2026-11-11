# Slice 10: Staged enforcement and acceptance

Status: delivered
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `chore/rev-architecture-quality-10-enforcement`
PR: [#24](https://forge.adrianpk.com/rev/rev/pulls/24)
Integrated: `67dd8de1e9b51868f8f38379fc0079defd2a8d87` through canonical PR #24.
Gate repair branch: `fix/rev-architecture-quality-git-cleanup-1`
Gate repair PR: [#25](https://forge.adrianpk.com/rev/rev/pulls/25)
Gate repair implementation: `6f8e282f6688405d8f0bd001673e74934d657a58`
Gate repair integrated: `ab30c95e4a164a54bcc3db606e2d4f4340335ea9` through canonical PR #25.

## Purpose

Enforce the completed native workspace quality policy at the staged commit
boundary, preserving editor behavior, acceptance coverage and user content.

## Delivered Behavior

Both packages inherit the blocking compiler, Clippy ownership/representation and
unsafe-code policy. The SDK also denies missing public error/panic documentation.
Exact leak/forget/raw-ownership methods are prohibited with individual reasons;
borrowed pointer identity comparisons remain available. Local reasoned
expectations remain checked for fulfillment, with no new suppression baseline.

`make lint` checks every workspace target/feature and all first-party source.
`make test-hooks` exercises real Git and the real Cargo/native checker in a small
two-package parser workspace. `make install-hooks` provides explicit, idempotent
repository-local activation while preserving conflicting hooks.

The hook validates an index copy and exact Git blob export, never an unstaged
repair. It rejects conflicts, symlinked inputs, changed exports and index changes.
Source, manifest, lockfile, policy and tooling commits check complete workspace
owners. Docs/data-only commits use fast whitespace and index-integrity checks.
Private per-worktree caches use stable paths, owner markers and exclusive locks;
normal build targets, index bytes and tracked/untracked/ignored worktree content
are preserved. Linked worktrees use their own hook files and independent caches.

## Implementation Notes

Final adoption exposed two remaining implicit completion cancellation clones,
nine newly deprecated atomic update calls and the documented SDK hello example's
source complexity. Explicit `Arc::clone` retains the same shared cancellation
lifetime. Atomic updates use `try_update`, stable before the existing Rust 1.98
minimum; the former method delegates to it. Orderings, closures, admission limits
and error paths are unchanged. The hello example separates client admission,
frame flushing and callback receiving with unchanged protocol and IO behavior.
Its existing native workflow passes under both editor profiles.

An alternate-target measurement also exposed discovery of generated Rust under
the checkout's old default target directory. Discovery now excludes each package's
ordinary Cargo output alongside the configured target. Explicit Cargo roots in
an excluded build tree still fail admission. A real metadata fixture protects
this boundary. All nine native checker tests pass, including the retained scoped
admission, parser, local expectation and compiler-result checks.

The export copies blob bytes directly: checkout smudge filters, line-ending
conversion and export-ignore rules cannot repair or hide staged source. Native
checks verify those bytes and executable modes afterward. An exclusive cache
owner updates its own export, preserving matching source timestamps for Cargo
reuse and removing only obsolete exported paths. Cancellation retains the cache until the next owner. The fixture
runner waits for its own paused checks before removing only its own temporary
repositories. It does not stop unrelated processes or change this checkout's hook
configuration. Fixture status/diff observations use another index copy so they
cannot refresh the metadata measured by the raw index-preservation assertion.

### Gate repair: Owned Git cleanup

The original Slice 10 delivery remains integrated through canonical PR #24.
The bounded gate repair is integrated through canonical PR #25.

Controller validation `LYMBVLYJ4VGE3RDKMUABFBAYJQ` on integrated candidate
`4d92ea260259a8a453810e193582f3b0f508595c` passed lint/hooks but failed the native
Git suite: 241 cases passed and `conflicts::resolution::sides_then_stage` remained
in Cleanup at the existing five-second per-operation fixture deadline. The
isolated baseline passes in 36.39 seconds, so it does not reproduce on every run.

The capture cleanup path discarded complete no-live-member observations and
rescanned the process table. A later observation failure could retain an already
stopped group. The cohesive cleanup owner now carries that shutdown result through
TERM grace, KILL and final reaping. An exited leader goes directly through KILL
observation. The leader remains unreaped until the last signal/observation, and
incomplete or failed observations still retain cleanup authority. The two-second
cleanup budget, 500-ms TERM grace, 250-ms drain, output limits, cancellation and
Git freshness/effects/publication guards remain unchanged.

The deterministic `shutdown_observed` regression fails before the correction
with a redundant-observation error and passes afterward. Real child regressions
cover exited leaders whose descendants inherit or redirect output, plus actual
TERM-to-KILL escalation. Existing cancellation, timeout, private workspace,
process-state and reaping tests remain.
No Git fixture assertion, timeout, input or repository-state check is changed.

## Contracts Added or Changed

- Added inherited workspace lint tables, explicit method restrictions and strict
  canonical flags; retained the five/three/ten source limits and SDK protocols.
- Added repository architecture, ownership, convention and validation guidance.
- Added exact staged candidate and private-cache commit enforcement. Authorized
  prepare_gate setup has installed the hooks; the final gate remains controller-owned.

Review reconciliation before controller acceptance:

| Finding | Current evidence |
| --- | --- |
| F1 | Complete two-package Cargo discovery; every target root is checked, including SDK children, tests, examples and documented Rust. Alternate-cache generated output is excluded by ownership. |
| F2 | Strict all-target/all-feature Clippy covers both inherited packages without selected diagnostics deferred. |
| F3 | All first-party source passes unchanged safety, parameter, nesting, complexity and local-exception rules. Existing test assertions, resource ceilings and protocol inputs remain unchanged. |
| F4 | `Invocation`/queued `Call` own arguments; `Pending` retains correlation, context, permit, deadline and completion authority without arguments. No new payload or latency claim is inferred. |
| F5 | Keymap refresh compares its owned observation by reference and moves it into the cache after publication; an unchanged hit does not duplicate the observation vector. |
| F6 | `completion::state::State` owns invocation identity, editing authority, source, popup, worker, resolver and bounded recency. Dismissal cancels resolution/worker authority; captured-source and late-result guards remain. |
| F7 | Canonical commands, real staged-index/partial-staging/concurrency fixtures and explicit conflict-preserving installation are present. Repository-local activation and linked-worktree hook resolution are verified during prepare_gate. |

The specification's criteria map remains the review contract. Mechanical checks
cover its selected ownership, conversion, lifetime, source and exception rules.
Cohesive boundaries, synchronization invariants, intentional sharing, captured
lifetimes and the absence of unnecessary allocation still require manual review
and native regressions. A green linter does not prove those properties.

## Files of Interest

- `Cargo.toml`, `crates/rev-plugin-sdk/Cargo.toml`, `clippy.toml`: inherited policy.
- `examples/source_check/`: discovery, diagnostics and source-policy owner.
- `.githooks/pre-commit`, `scripts/test-hooks.sh`, `scripts/install-hooks.sh`:
  staged boundary, real Git acceptance and explicit installation.
- `Makefile`, `AGENTS.md`, `.agents/`: canonical commands and repository guidance.
- `src/completion/job.rs`, atomic admission/identity owners and
  `docs/reference/plugin-hello.rs`: bounded final-adoption corrections.
- `src/process/capture.rs`, `src/process/capture/cleanup.rs` and
  `src/process/capture/tests.rs`: bounded cleanup observations and regressions.

## Validation

Implementation checks pass for complete workspace lint, real hook acceptance,
all nine source-checker tests, the existing hello workflow, documentation catalogs
and the Spore User Guide build. All five frozen commands passed on the clean
published and integrated HEAD `67dd8de1e9b51868f8f38379fc0079defd2a8d87` in Looper
validation `RH3QDRJEPZIQ77JIV5ERDRERDT`, using the exact ordered commands below.
The unchanged full gate remains a controller acceptance step after preparation.

After integration, authorized `make install-hooks` setup activated repository-local
`core.hooksPath=.githooks`. Primary and linked-worktree hook resolution are verified;
the default hook directory contained only samples, with no conflicting hooks or
another repository's configuration changed. Fresh prerequisite evidence for the
clean preparation HEAD is recorded separately through Looper.

All 28 hook fixture cases pass. Hook acceptance covers both partial-staging directions, a real installed commit,
docs-only admission, native bounds, other-member coverage, staged manifest/lint/
tooling repair attempts, disallowed ownership escapes, stale expectations,
smudge/export-ignore attempts, Rust deletion/rename, conflicts, concurrent owners,
changed index, changed exported bytes/modes, unsafe/foreign cache paths, default/configured hook conflicts and
installed independent linked-worktree commits during another owner's check.

Measured implementation checks on this host:

| Check | Cold | Unchanged warm |
| --- | --- | --- |
| Complete workspace lint | 304 seconds | 49 seconds |
| Actual worktree staged hook | 587 seconds | 60 seconds |
| Two-package parser fixture hook | 52 seconds | 4 seconds |

Cold measurements include compilation with empty exclusive caches. The lint cold
run preceded the default-output discovery correction; subsequent full checks and
the actual staged hook use the corrected 1,836-file inventory. The warm hook uses
the timestamp-preserving export and checks executable modes. These are observed
implementation costs under current host load, not latency guarantees or final
clean-HEAD acceptance evidence. Per-worktree caches are retained; no normal build
cache is shared or cleaned.

Frozen commands, in canonical order:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint` - passed; final PR-HEAD evidence is recorded by Looper.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make test-hooks` - passed; final PR-HEAD evidence is recorded by Looper.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` - passed; final PR-HEAD evidence is recorded by Looper.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh` - passed; final PR-HEAD evidence is recorded by Looper.

Additional focused checks:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --example source-check` - nine passed.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins guidance::hello_workflow -- --test-threads=1` - one passed, covering both profiles.
- `bash .githooks/pre-commit` - complete actual worktree staged candidate passed both cold and unchanged warm checks, with exact index bytes preserved.
- `git diff --check` - passed.

Gate repair checks, in canonical order, passed through Looper on clean published
and integrated HEAD `ab30c95e4a164a54bcc3db606e2d4f4340335ea9` in validation
`CFTQW7EQ4MUNXXPT5GUEYWMLFY` after the repair PR references were committed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/process.rs --scope src/process --scope src/git/work/command.rs --scope src/git/work/conflicts --scope tests/git_integration/support.rs --scope tests/git_integration/conflicts_resolution.rs` - passed for all 16 selected files; zero Clippy/source findings.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- process:: --test-threads=1` - all 14 pass during implementation.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- conflicts::resolution --test-threads=1` - all nine pass in 104.56 seconds.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` - passed.
- `git diff --check` - passed.


## Risks and Follow-ups

The controller must execute the unchanged full gate before delivery-set closure.
Local hook activation is verified; the delivery set remains in progress.
No main/release/publication
operation is included. Foreign vendored compiler warnings remain visible and are
outside first-party lint admission. New native behavior and test coverage are not
claimed from static lint success.
The gate repair is integrated with exact clean PR-HEAD evidence recorded through
Looper. The controller must rerun the unchanged full gate before set closure.

### Gate repair 2 scope blocker

The controller rerun at `3165ff2e69575daaf09ddca6b5ed48aef4ccf506`
(`MAUHSFFUQ2SVVJ2PKWGVFU2FGP`) passed all 190 library tests and all 242 Git
integration cases. The native all-targets command then failed three of the 46
`path_completion` cases: `acceptance::directory_return`,
`failures::symlink_directory` and `stale::changed_kind`. Directory selection
replaced the document view or buffer, contrary to the existing assertions.

The injected repair 2 branch remains
`fix/rev-architecture-quality-git-cleanup-2`, with its pinned Git cleanup title.
Its allowed runtime and test paths contain only process cleanup and Git conflict
owners. The failing fixtures and session directory-opening owner fall outside
that scope. The tracker records the failing command and assertion locations;
an explicit scope decision is required before correcting those owners. No
runtime files, assertions, deadlines, budgets or gate requirements were changed,
and no additional full gate was run by the worker. The original slice and repair
1 remain delivered; delivery-set closure remains pending.
