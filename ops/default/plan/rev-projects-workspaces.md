# Files, Directories, Projects and Workspaces Delivery

Status: Completed
Completed: 2026-11-07
Validated candidate: `6bbc6138ac9227beca51e6b86f759bbf39e36c94`
Gate receipt: `ZS6AIYOX66WRNHE5ICPVA46GIP`
Approved: product owner, 2026-11-07; approved core specification/model and the established preparation commits, ordered slicing, push and single autonomous Looper delivery through dev
Date: 2026-11-07
Delivery set: rev-projects-workspaces
Specification: [Files, Directories, Projects and Workspaces](../spec/editor-projects-workspaces.md)
Model: [Project and Workspace Model](../spec/editor-projects-workspaces-model.md)
Concern: [Pending Work Index](../README.md)
Slice strategy: one complete state-storage foundation, then behavior-first increments, combined acceptance and guidance
Planning baseline: 1a2587840dd29a20600de15566d7a778f605f0cb
Tracker: [Projects and Workspaces Tracker](../tracker/rev-projects-workspaces.md)
Active slice: None - all eight slices delivered; full gate passed; delivery set closed
Full gate: `export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --lib --test contexts --test configuration --test commands --test modal --test display --test files --test tree --test recovery --test pointer -- --test-threads=1 && cargo test --locked --test git_integration source -- --test-threads=1 && cargo test --locked --test integrated_terminal lifecycle -- --test-threads=1 && cargo test --locked --test shell_commands lifecycle -- --test-threads=1 && cargo test --locked --test pi --test lsp -- --test-threads=1 && bash scripts/check-projects-workspaces.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`

## Outcome and strategy

Deliver the approved core in one retained Looper run: ordinary file/directory
work, explicit named Projects, independent multi-root Workspaces and the native
startup screen with the supplied banner. Opening a directory never registers a
Project and does not replace a registered Project's saved session.

A small storage foundation is necessary before exposing durable registration or
conversion: its aggregate commit point must be tested before UI callers can
publish records. Slice 1 is a complete, independently testable Rust domain/store
layer, not failing tests or stubs. Slices 2-6 add end-to-end native behavior;
Slices 7-8 complete representative acceptance and guidance. All eight stay
compilable and pass their recorded checks. There is no ninth delegated slice.

Basic named-artifact sessions restore files, buffer order, active file and
source positions. Whole-application startup restoration, pane layouts and tree
expansion recovery belong to the deferred specification. Reserve the agreed
startup.restore_session name/default only; do not implement or advertise an
inactive configuration key in this set.

## Authority and prerequisites

The owner's approval authorizes the preparation commits on dev, this ordered
slice map, the exact branches/reports and canonical PRs, controller-owned
Rebase + Fast-forward merges into dev, the frozen gate and documentation closure.
Looper owns progress and merges slice PRs without waiting for owner approval.
Main promotion, installation, releases, tags and mirrors are separate actions.

Rust/Cargo, Bash, existing dependencies, native/PTY helpers, Git, the Markdown
documentation checker and Spore are delivered prerequisites. Add no dependency,
runtime, provider requirement, external service or test framework. Use
changelog-normalize for delivered user-visible Unreleased outcomes.
Keep unrelated tree-selection, Org visibility, quit-alias and catalog-description
drafts/tickets outside this set.

The approved banner is src/session/startup/rev-logo.ans, SHA-256
0b427846a5a84cb525c812f25e4e28c08a2117254e21e21cd617b04e26628d62.
Preserve the source artwork. Its 100x13 cells and 35,650 bytes are renderer
input; raw escapes must not bypass native painting.

## Bounded technical recovery

Retain the same run, conversation, frozen checks and consumed budgets.
Use installed defaults of 200 worker turns and five stalled turns. Delegate
version 2 unattended-v1 recovery with two diagnoses per family, eight per run,
two failed repair attempts per family and two environmental retries.
Delegate at most two final-gate repair PRs, reporting through Slice 8, with
the existing exact-HEAD validation and canonical controller-owned merge path.

Repairs stay within this core's src/tests/examples/scripts/docs, CHANGELOG.md
and the two owning specifications. Exclude Cargo.toml, Cargo.lock, vendor and
the supplied banner. Derived grants require controller-verified delivery
provenance. No weaker assertions, changed gate, new product decisions, dependency
override, extra slice, budget reset or unrelated ticket work is delegated.
Ordinary provider retry behavior remains unchanged; there is no additional
long-tail wait, reserve, or host/service-control authority.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Context state and catalog | `feat/rev-projects-workspaces-01-state` | `feat(contexts): add typed contexts and atomic state storage` | `ops/default/report/slices/rev-projects-workspaces/slice-1-state.md` |
| Slice 2 | File and directory contexts | `feat/rev-projects-workspaces-02-directories` | `feat(contexts): preserve ordinary file and directory workflows` | `ops/default/report/slices/rev-projects-workspaces/slice-2-directories.md` |
| Slice 3 | Project registration and sessions | `feat/rev-projects-workspaces-03-projects` | `feat(projects): register projects and restore basic file sessions` | `ops/default/report/slices/rev-projects-workspaces/slice-3-projects.md` |
| Slice 4 | Workspace roots and trees | `feat/rev-projects-workspaces-04-workspaces` | `feat(workspaces): manage independent roots and workspace sessions` | `ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md` |
| Slice 5 | Artifact lifecycle and conversion | `feat/rev-projects-workspaces-05-management` | `feat(contexts): manage project and workspace lifecycles` | `ops/default/report/slices/rev-projects-workspaces/slice-5-management.md` |
| Slice 6 | Startup screen and shared consumers | `feat/rev-projects-workspaces-06-startup` | `feat(startup): add the native startup screen and context selection` | `ops/default/report/slices/rev-projects-workspaces/slice-6-startup.md` |
| Slice 7 | Combined native and terminal acceptance | `test/rev-projects-workspaces-07-acceptance` | `test(contexts): verify native startup and persistent context workflows` | `ops/default/report/slices/rev-projects-workspaces/slice-7-acceptance.md` |
| Slice 8 | Guidance and gate preparation | `docs/rev-projects-workspaces-08-guidance` | `docs(contexts): document files projects workspaces and startup` | `ops/default/report/slices/rev-projects-workspaces/slice-8-guidance.md` |

## Slice 1: Context state and catalog

Dependency: approved committed preparation on clean dev.
Requirements: PW01, PW09, PW15, PW17-PW19, PW22.

- T1.1: Define typed context, artifact, root and session identities and the approved path/name/source-coordinate model; separate live, prepared and durable state.
- T1.2: Implement the versioned aggregate contexts.toml codec and store with existing dependencies, checked revisions, bounded nonblocking publication, one atomic conversion commit point and preserved invalid input.
- T1.3: Add tests/contexts.rs with meaningful native storage/domain cases, real isolated files, aliases/non-UTF-8 paths, stale writers, capacity rejection and publication-fault preservation; deliver the slice report.

## Slice 2: File and directory contexts

Dependency: Slice 1 integrated into dev.
Requirements: PW01, PW04, PW06, PW18-PW20, PW22.

- T2.1: Admit standalone and directory contexts around existing file/tree owners. Preserve real source editing/saving, direct file/directory targets and explicit root intent; opening a directory never registers a Project or overwrites its saved session.
- T2.2: Add generate-new-buffer, open-directory and context-close through native commands and checked replacement. Preserve in-context find-file visits and loose buffers; reuse source save/discard/cancel and owned-job retirement without exiting the application.
- T2.3: Verify both profiles, standalone manual tree requests, directory cancellation, canonical buffer reuse, source stamps and stale asynchronous completions. Document each public operation actually delivered and update Unreleased through changelog-normalize.

## Slice 3: Project registration and sessions

Dependency: Slice 2 integrated into dev.
Requirements: PW05-PW06, PW15-PW16, PW18-PW20, PW22.

- T3.1: Deliver explicit project-create/project-promote and named project selection/listing, with unique roots/names, typed picker identities and no implicit path-based promotion.
- T3.2: Capture ordered file-backed buffers, active file, source cursor/view anchors and loose files; restore using native loaders and checked source coordinates. Protect dirty/unnamed sources and preserve missing-file/root diagnostics without creating replacement files.
- T3.3: Wire checkpoint ownership into accepted context departure and clean quit; selecting the same artifact is a no-op. Test source decisions, live promotion, restoration, clocks/terminal shutdown ownership and storage failures; document delivered commands and session behavior.

## Slice 4: Workspace roots and trees

Dependency: Slice 3 integrated into dev.
Requirements: PW09-PW11, PW15-PW16, PW18-PW22.

- T4.1: Deliver empty Workspace creation, named selection/listing and independent basic sessions. Add directories or Project-derived paths without importing Project sessions or consuming registrations.
- T4.2: Project ordered root identities into the existing decorated/minimal tree, retaining lazy discovery, Nerd Fonts policy, hit geometry and canonical buffers across aliases/overlap. Preserve unavailable roots and their saved entries.
- T4.3: Deliver checked root removal, remapping surviving overlapping file references and resolving only affected buffers before committing membership. Test empty/partial/unavailable roots, both profiles and native pointer/keyboard behavior; document delivered workflows.

## Slice 5: Artifact lifecycle and conversion

Dependency: Slice 4 integrated into dev.
Requirements: PW07-PW08, PW12-PW14, PW17-PW19, PW22.

- T5.1: Deliver rename/forget and explicit root relocation with stable IDs, root-relative remapping and unchanged source directories. Forgetting an active Project retains its live directory work; forgetting an active Workspace resolves sources and leaves no arbitrary replacement root.
- T5.2: Deliver atomic Project-to-Workspace conversion and independent duplication, including active/inactive sources, fresh Workspace/root IDs, retained live dirty text where permitted and cancellation before duplicate publication.
- T5.3: Test both sides of durable commit, restart recovery, stale writers, invalid/corrupt state, moved roots, root/name conflicts and source/session preservation. Keep revisions and all logical references consistent; document the lifecycle operations.

## Slice 6: Startup screen and shared consumers

Dependency: Slice 5 integrated into dev.
Requirements: PW01-PW04, PW20-PW22; accepted P01/P03/P18.

- T6.1: Show the native startup screen for no-target executable startup; explicit paths still open directly. Wire all five actions to the implemented domain operations, retaining cancel/return behavior, native selectors, ranking, hints and both editing profiles.
- T6.2: Render the supplied src/session/startup/rev-logo.ans as passive native styled cells on a matching black content area. Preserve its hash, RGB/Unicode artwork and buffer palettes; use a plain rev label for NO_COLOR or insufficient space and retain action identity during resize.
- T6.3: Verify per-file/root authority for Git, Pi, LSP, shell jobs, terminals, Org and global bookmarks/usage. A Workspace is not automatic cross-root AI context or service activation. Extend focused startup/command/modal/display cases and document the delivered entrypoints.

## Slice 7: Combined native and terminal acceptance

Dependency: Slice 6 integrated into dev.
Requirements: PW01-PW22 together.

- T7.1: Create examples/projects-workspaces-acceptance.rs and scripts/check-projects-workspaces.sh with the existing Rust/Bash surface and PTY helpers; keep domain/storage variants in native cases and real editor journeys representative.
- T7.2: Cover one bounded real journey per profile across startup, explicit targets, promotion, Project/Workspace selection, sessions, membership, conversion, duplicate/cancel and shutdown. Exercise banner fitting/NO_COLOR, tree variants and Unicode source anchors without multiplying the entire matrix through PTYs.
- T7.3: Probe ordinary and capacity-bound catalog/session work, responsiveness, retained allocations and idle scheduling; measure the approved profile rather than assuming it. Preserve source/configuration bytes, restore TTYs and reap only owned processes; deliver evidence and the report.

## Slice 8: Guidance and gate preparation

Dependency: Slice 7 integrated into dev.
Requirements: PW23 and joint PW01-PW22 reconciliation.

- T8.1: Complete the Diataxis guidance with short CLI examples, a guided file/directory/Project/Workspace path, focused lifecycle how-tos, exact command/state/limit reference and a concise context explanation. Use small textual UI examples where helpful.
- T8.2: Keep Markdown as the source for Spore and reconcile CLI/navigation/interaction baselines with approved context ownership. Retain historical evidence and unrelated pending work; document no inactive startup.restore_session option as current functionality.
- T8.3: Deliver all reports and exact task/commit/PR evidence, finalize one coherent Unreleased outcome and prepare the clean integrated candidate for the frozen controller gate. After it passes, close only the configured documentation paths and record actual evidence.


## Validation discipline

For every Cargo command except formatting, use the literal prefix:

    env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1

Reuse existing tests and native file-publication boundaries. Keep path/identity,
codec, overlap, conflict and failure variants parameterized at their cheapest
real boundary. Do not multiply all variants through real terminal journeys.
Never use actual user catalogs, repositories, credentials or remote services.

New public commands must be documented when they become usable so the command
inventory remains correct at each slice. Update Unreleased only for delivered
behavior, not incomplete internal storage types. Do not keep no-op aliases,
placeholder commands, weakened fixtures or skipped checks to stage the work.

## Required slice commands

The following lists are the exact ordered checks for clean-HEAD Looper evidence.
Record additional implementation-time focused checks separately in the report;
preserve these ordered receipt commands and the final git diff --check.

### Slice 1 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 2 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test files --test tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 3 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test recovery --test integrated_terminal -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 4 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test tree --test pointer -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 5 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test files --test recovery -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 6 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test commands --test modal --test display -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 7 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts -- --test-threads=1`
- `bash -n scripts/check-projects-workspaces.sh`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

### Slice 8 checks

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `bash -n scripts/check-projects-workspaces.sh`
- `bash -n scripts/build-user-guide.sh`
- `git diff --check`

## Combined acceptance and expected cost

Run the script's complete native/PTY/resource acceptance only in the final gate,
after all slices integrate. Build outside journey deadlines. Reuse the existing
stricter helper limits; each representative PTY journey has at most 30 seconds
and 1 MiB of observed output. Observe actual output/readiness rather than fixed
sleeps. Each profile covers the same native contract while policy variants stay
in isolated native tests.

Use actual temporary catalogs and source files for storage, aliases, incomplete
availability, dirty-buffer decisions and conversion fault injection. Include
clock/terminal/job ownership and stale result rejection when contexts change.
Verify no periodic context work remains scheduled at idle. Measure encoded and
decoded catalog copies at ordinary and admitted boundary sizes; do not infer
resource cost from element counts or claim the design estimate as a measurement.

The selected gate protects the new domain/store plus shared startup, file/tree,
command/modal/display, source decisions and service lifetime contracts. It uses
focused Git source, terminal and shell lifecycle cases and Pi/LSP regressions,
rather than chaining every earlier feature acceptance script or rerunning the
complete Git history/network matrix. Estimated warm-build cost is 8-15 minutes
under ordinary host load, plus compilation and the new measured acceptance
work. This is a planning estimate, not a timeout or a reported result.

## Gate preparation

All eight canonical PRs (#298-#305) are merged into dev; the last integration is
`cecd8205c7f61f503e9c4c1da963cefe6686600f`. All eight reports are delivered.
The final Slice 8 clean-HEAD receipt `COXYQZ74QJZGY2OD4BJK355A66` passed its
six required commands. This is slice evidence, not the combined delivery gate.

The gate prerequisites were admitted on the integrated source: all selected
Cargo test/example targets resolve, 118 packages and the Rust/Python grammar
fixture sources are available offline, and the editor, context acceptance helper,
documentation checker and User Guide projector build with the approved resource
profile. Rust/Cargo 1.98, rust-analyzer, Bash, Git, Spore, cc, Python 3 and the
required Linux/PTY utilities are available. Pi regression tests use their owned
peer fixture; no external executable override is present.

An owned temporary HOME/XDG fixture exercised the existing PTY wrapper with
`/bin/true`, verified terminal-attribute restoration and was removed afterward.
No editor acceptance journey or combined gate was executed in preparation.
The tracker records the admitted prerequisites. The controller subsequently
passed the unchanged gate below on the clean integrated candidate recorded in
the closure section. No gate was rerun for documentation closure.

## Frozen final gate

The controller alone runs this ordered command on the exact clean integrated
dev candidate. Prepare target/script/tool admission before returning
ready_for_gate. Do not change the command after execution starts.

    export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 && \
    cargo fmt --all -- --check && \
    cargo clippy --locked --all-targets -- -D warnings && \
    cargo test --locked --lib --test contexts --test configuration --test commands --test modal --test display --test files --test tree --test recovery --test pointer -- --test-threads=1 && \
    cargo test --locked --test git_integration source -- --test-threads=1 && \
    cargo test --locked --test integrated_terminal lifecycle -- --test-threads=1 && \
    cargo test --locked --test shell_commands lifecycle -- --test-threads=1 && \
    cargo test --locked --test pi --test lsp -- --test-threads=1 && \
    bash scripts/check-projects-workspaces.sh && \
    cargo run --locked --quiet --example documentation-check && \
    bash scripts/build-user-guide.sh && \
    git diff --check

## Delivery-set closure

All eight canonical slices (#298-#305) are integrated into dev and every report
is delivered. The controller passed the exact frozen gate at `6bbc6138ac9227beca51e6b86f759bbf39e36c94`
in receipt `ZS6AIYOX66WRNHE5ICPVA46GIP` on 2026-11-07. The 817.18-second run passed
1,036 selected Rust tests, native capacity/resource acceptance, both-profile PTYs,
618-command/289-setting documentation checks and the 91-page local Spore build.
[Slice 8 closure](../report/slices/rev-projects-workspaces/slice-8-guidance.md#delivery-set-closure) records the exact command, suite timings, actual resource
measurements, cleanup and evidence limits. Post-gate changes are restricted to
the thirteen documentation paths below; tested source and assertions are unchanged.
The core specification/model and tracker are Completed. P19 and unrelated pending
work retain their existing separate boundaries.

## Closure paths

Only these thirteen Markdown paths may differ after the tested runtime candidate.
Complete the plan/tracker, specification/model and index together; every report
must be delivered. Preserve immutable report introduction links, the real gate
candidate/receipt, exact commands, measurements and evidence limits.

- `ops/default/plan/rev-projects-workspaces.md`
- `ops/default/tracker/rev-projects-workspaces.md`
- `ops/default/spec/editor-projects-workspaces.md`
- `ops/default/spec/editor-projects-workspaces-model.md`
- `ops/default/README.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-1-state.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-2-directories.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-3-projects.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-5-management.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-6-startup.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-7-acceptance.md`
- `ops/default/report/slices/rev-projects-workspaces/slice-8-guidance.md`

Runtime, tests, guides, changelog, assets and acceptance assertions remain
unchanged after the gate. Reconcile NAV01/NAV02 and interaction/reference wording
before the gate, preserving their historical delivery evidence. Do not change
an unrelated draft's status or absorb its work during closure.
