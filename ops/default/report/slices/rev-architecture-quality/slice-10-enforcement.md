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
Gate repair 2 branch: `fix/rev-architecture-quality-git-cleanup-2`
Gate repair 2 PR: [#26](https://forge.adrianpk.com/rev/rev/pulls/26)
Gate repair 2 implementation: `0e8cf144aba9360b585df103fc85043309fecd2f`
Gate repair 2 integrated: `ab813819e0e53f246660ac8fd702609e743b3dfb` through canonical PR #26.
Gate repair 2 state: delivered; accepted in the final controller gate.
Gate repair 3 branch: `fix/rev-architecture-quality-recovery-paths-3`
Gate repair 3 PR: [#27](https://forge.adrianpk.com/rev/rev/pulls/27)
Gate repair 3 implementation: `82160a02e730e2f8f202ad28a8f06884735e6ac2`
Gate repair 3 integrated: `fdc7ab4f70216a7a2035a234dd7b8568f7c2b46c` through canonical PR #27.
Gate repair 3 state: delivered; accepted in the final controller gate.

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

Review reconciliation confirmed by final controller acceptance:

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
- `tests/recovery/paths.rs`: denied-entry identity and parent-row recovery checks.

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

The unchanged controller gate passed; final results and evidence limits are in
Delivery-set closure below. Repository-local hook activation is verified.
No main/release/publication operation is included. Foreign vendored compiler
warnings remain visible and are
outside first-party lint admission. New native behavior and test coverage are not
claimed from static lint success.
The gate repair is integrated with exact clean PR-HEAD evidence recorded through
Looper. The final controller rerun is accepted in Delivery-set closure below.

### Gate repair 2 directory acceptance

The controller rerun at `3165ff2e69575daaf09ddca6b5ed48aef4ccf506`
(`MAUHSFFUQ2SVVJ2PKWGVFU2FGP`) passed all 190 library tests and all 242 Git
integration cases. The native all-targets command then failed three of the 46
`path_completion` cases: `acceptance::directory_return`,
`failures::symlink_directory` and `stale::changed_kind`. Successful directory
acceptance replaced the document view or buffer, contrary to stale fixture
expectations. The unchanged focused suite reproduces all three failures in
12.67 seconds (43 passed, three failed).

The injected repair 2 branch remains
`fix/rev-architecture-quality-git-cleanup-2`, with its pinned Git cleanup title.
The initial injection excluded the failing fixtures, so the first checkpoint
recorded the scope blocker. The current injection adds exactly the three fixture
files. [Directory acceptance](../../../spec/editor-file-navigation.md) and
[editing contexts](../../../../../docs/reference/contexts.md) already require
fresh temporary work after checked departure. No runtime correction is needed.

Source-retention assertions now check the still-open prompt. Post-acceptance
checks require a new clean unnamed buffer, and held Return must retain that new
view/stamp and expanded root. Symlink referent checks, current-kind revalidation,
nested file visits and unchanged original/unrelated disk contents remain covered.
All 14 existing case identities and input paths, contents and events remain;
assertion sites increase from 94 to 110. The corrected target passes all 46 cases
with no ignores in 11.36 seconds. All 44 focused context cases also pass in 58.07
seconds, including both profiles' save/discard/cancel journeys.
Runtime files, freshness/consent guards, deadlines, resource limits, budgets and
the frozen commands are unchanged. The worker has not run the full gate. The
original slice and repair 1 remain delivered; delivery-set closure is pending.

Repair 2 uses exactly the five frozen repair commands above in its canonical PR
body and clean-HEAD record, ending with one `git diff --check`. Additional focused
checks below run separately; their evidence supplements that canonical record:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests/path_completion/acceptance.rs --scope tests/path_completion/failures.rs --scope tests/path_completion/stale.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test path_completion -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test contexts -- editor:: --test-threads=1`

The earlier eight-command sequence passed through Looper on the clean published implementation
HEAD `0e8cf144aba9360b585df103fc85043309fecd2f`, validation
`7KU3SWMAIABRTRXN2VTZO35CV3`. This includes zero findings in both quality scopes,
14 process tests, nine Git conflict-resolution cases (102.52 seconds), 46 path
completion cases (12.53 seconds), 44 focused context cases (61.81 seconds),
documentation and the final diff check. The installed staged hook also checks all
1,838 first-party files and 73 Cargo roots with zero Clippy/source findings.
Canonical PR #26 records the immutable report-introduction link and successful
commands. The reference HEAD `b84ddf700365459182f1c03dc9c58c0cd68147ce` also passed
that sequence in `TA2N2ESAVVQERXBOS6NHQJSWOW`. Controller review rejected its
command order because focused checks preceded the required fifth command.
The subsequent nine-command sequence passed technically on
`b1dd1ebd686845648f4003f1d390d1218701389e` in `SCTGSZ252MHZRTG4ZQQDU6PV26`,
but controller review rejected its duplicate diff command. Canonical validation
now contains exactly the five required commands, each nonempty and unique, in
their injected order. Focused regression evidence remains separate from that
list. The final canonical five passed through Looper on clean published and
integrated HEAD `ab813819e0e53f246660ac8fd702609e743b3dfb` in validation
`MPIGY2O6ICXU2PFYLBBPFEUHXD`. They cover all 14 process tests, nine Git conflict
cases (81.19 seconds), the 16-file quality scope, documentation and diff checks.
The canonical PR body matches those five commands exactly, with no duplicates.
Separate focused checks on the same HEAD pass quality for all three fixture
files, all 46 path-completion cases (11.16 seconds) and all 44 context cases
(54.37 seconds). Forgejo confirms PR #26 is merged into dev at that exact HEAD.
Both gate repairs and all ten slice reports are delivered. Runtime files,
assertions and every gate requirement are unchanged; the controller full gate
remains pending. Fresh preparation evidence is recorded against the clean
document-closure HEAD before returning it for that gate.

### Gate repair 3 denied tree entries

The retained controller gate at `2c42309240f35b3d94550c6a0efed0eea8c24899`
(`H4HN2DZKILYOMQO3HOE2K3OPT3`) passes 25 recovery cases and fails
`paths::failed_paths`. The unmodified focused case reproduces selection of
`/tmp`, the parent navigation row, rather than the fixture's denied directory.
Changing only selection to the existing bounded path helper reaches the real
permission-denial assertions, then reproduces the stale row count (six versus
five). The root and four filesystem entries now include the existing parent
navigation row; no source entry or resource capacity is added by this repair.

The fixture selects the denied directory by identity, checks the parent row's
kind and path, and retains both exact visible-row counts at six. Each refresh
also retains the parent-row check. Both original case identities and every
permission, input, history, syntax, byte-preservation, save and undo/redo journey
remain. Assertion sites increase from 29 to 32. No runtime, freshness/consent,
resource/time, protocol or gate changes are included. The same Slice 10 report
and pinned repair identity remain in use; the worker does not run the full gate.

The implementation recovery command passes all 26 cases, with no ignores, in
22.18 seconds. All five unchanged ordered checks pass through Looper on clean
published implementation HEAD `82160a02e730e2f8f202ad28a8f06884735e6ac2` in
`QC7FU7PXI3ZXPFLLCBRTZSUFEF`: zero Clippy/source findings in the one-file scope,
all 26 recovery cases (23.86 seconds), all seven native parent-navigation cases
(8.89 seconds), documentation and diff checks. The installed staged hook also
passes all 1,838 first-party files and 73 Cargo roots with zero Clippy/source
findings. Canonical PR #27 uses the pinned title/branch, immutable
report-introduction link and five unique successful commands. All five commands
also pass on the clean published and integrated final HEAD
`fdc7ab4f70216a7a2035a234dd7b8568f7c2b46c` in validation
`3AUW5YLBWRCVKOZELSH4QOCW2D`, including all 26 recovery cases (20.86 seconds)
and all seven parent-navigation cases (8.06 seconds), with no ignores.
Forgejo confirms PR #27 is merged into dev at that exact HEAD. All three gate
repairs and all ten reports are delivered. Fresh prerequisite evidence was
recorded against the clean document-closure HEAD; the controller's unchanged
full gate subsequently passed on that candidate as recorded below.

Frozen repair commands, in canonical order:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests/recovery/paths.rs`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test recovery -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test tree -- parent:: --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

## Delivery-set closure

Initial closure on 2026-11-12. All ten slice PRs (#15-#24) and three bounded gate repairs
(#25-#27) are integrated into `dev`. The controller passed the unchanged frozen
integrated gate on clean published candidate `5625b0dbd5648ca81c888a19ad1faedc5241524d`
in validation `6XRS3YAETTKWUXPPWZ7L323GHI`. The successful record is keyed by that
candidate; its log basename is
`5625b0dbd5648ca81c888a19ad1faedc5241524d.6XRS3YAETTKWUXPPWZ7L323GHI.log`.
Earlier failed/preparation batches above remain historical evidence.

Every stage in the plan's [frozen integrated gate](../../../plan/rev-architecture-quality.md#frozen-integrated-gate)
passed in order through the controller's single chained command:

| Stage | Result |
| --- | --- |
| `make lint` | Formatting, strict all-target/all-feature Clippy and native source policy pass: 1,838 first-party Rust files, both packages, all 73 roots, zero blocking or deferred diagnostics and zero source findings. |
| `make test-hooks` | All 28 real Git fixtures pass. Both partial-staging directions, index/export tampering, conflicts, symlink/cache ownership and independent linked-worktree concurrency retain content integrity. Measured cold/warm fixture checks take 31s/2s. |
| `cargo test --locked --workspace --all-targets -- --test-threads=1` | 3,078 top-level tests plus seven nested child runs pass; zero failures. All 190 editor library, 242 Git, 46 path-completion and 26 recovery cases pass. Three existing upstream-acquisition cases remain ignored. |
| `cargo test --locked --workspace --doc` | The public SDK doctest passes; editor has zero doctests. |
| `cargo run --locked --quiet -p rev --example documentation-check` | 684 commands, 299 configuration entries, 106 reachable Markdown pages and 743 local links/anchors pass. |
| `bash scripts/build-user-guide.sh` | Local Spore User Guide builds under `target/userguide`. |
| External `scripts/check-rev.sh` | Independent Go tests and non-modal/modal interoperability pass against API `rev.plugins/0.1`, editor `0.6.0` at the exact candidate and client `3365b84649919492ade7e80c63139b459b5ad923`. Source/history and local binding assertions pass. |
| `git diff --check` | Passes on the clean integrated candidate. |

All compiler/check stages used the unchanged one-job, disabled-incremental,
zero-dev/test-debuginfo environment from the plan. The installed executable
`.githooks/pre-commit` and repository-local `core.hooksPath=.githooks` are verified;
the gate includes actual installed linked-worktree commits and separate caches.
No owned test fixtures remain active. Existing caches and unrelated files remain.
F1-F7 are resolved in the [review](../../../report/rev-architecture-nit-review.md#review-limits-and-remediation-closure),
AQL01-AQL06 and the plan/tracker are Completed, and all ten reports are delivered.
Only configured Markdown paths change during this closure; runtime, hook/lint
policy and frozen commands stay identical to the tested candidate.

Acceptance limits remain explicit. The three opt-in live acquisition tests
(`live::managed_go`, `live::managed_npm`, `live::optional_packages`) are not run.
The foreign `crossterm` unused-parentheses warning remains outside first-party
policy. At this initial gate, [Slice 9](slice-9-fixtures.md#risks-and-follow-ups)
retained standalone expectation/deadline failures; those are now discharged by
the expanded acceptance below. Compiling/testing all Cargo targets alone does
not execute every example's custom main journey. No manual editor session,
allocation/latency benchmark, main promotion, release or publication is claimed.

### Standalone acceptance closure

The owner reopened the initial closure to correct all five recorded standalone
families. Canonical PR #28 and bounded merge-settlement fixture repair #29 are
integrated into `dev`. The controller passed the five complete entrypoints
followed by all eight unchanged parent stages on clean published candidate
`422d6f66945d30f7ae640726cf1a63d17586bb1d`, validation
`VSWWUTEOGG2SGZW266X2FB7AMA`. The
[correction closure](../rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md#delivery-set-closure)
records the immutable receipt/log, full stage results and unchanged same-UID
reaper checksum. The initial receipt and earlier failures above remain historical.

All five standalone native/PTY/resource journeys pass without deadline or
assertion changes. Fresh full lint covers 1,838 files and 73 roots with zero
blocking/deferred findings; all 28 hook cases pass (cold/warm 29s/2s). The
3,078 top-level native tests and seven nested child runs pass with zero failures,
including all 242 Git cases in 413.65s. SDK docs, Markdown/Spore, independent Go
tests, both-profile interoperability and whitespace pass. The three existing
opt-in acquisition ignores and foreign warning remain unchanged.

Parent T9.4-T9.8 and T10.4 are complete. Both delivery plan/tracker sets, the
specification and review are Completed; all reports are delivered and the
concern index is updated. Only configured Markdown paths differ from the tested
candidate. Repository-local hooks remain active, no owned Rev fixtures remain
active, and unrelated processes, files and per-worktree caches are preserved.
