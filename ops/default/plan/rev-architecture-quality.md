# Architecture Quality Delivery

Status: In progress
Date: 2026-11-09
Delivery set: rev-architecture-quality
Slice strategy: behavior-first
Approved: owner requested one architecture remediation set with strict pre-commit Clippy/linter enforcement
Planning baseline: abbf0c21eeae1680c02c7a477fa0894d7996a393
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Review: [Architecture and Ownership Nit Review](../report/rev-architecture-nit-review.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 6 - External process workflows
Tracker: [Architecture Quality Tracker](../tracker/rev-architecture-quality.md)

## Outcome and authority

Deliver AQL01-AQL06, verified F1-F7 remediation and the complete native source
policy in ten ordered increments under one Looper controller. Separate package,
source/history, persistence, Git, process, document and interaction boundaries
make corrections reviewable. Tests/examples and commit enforcement have their own
acceptance; this count follows those owners, not a fixed quota.

The owner explicitly requested implementation, setup and commit-time checks in
one set. These exact branches and canonical PRs, controller review and Rebase +
Fast-forward integration into dev are authorized, with no separate slice-PR
approval. Looper owns progress, merges, the full gate and closure. Preserve its
run/conversation/counters/frozen checks during recovery. Main/release/tag, mirrors,
site publication and global editor installation stay separate. No changes to
Coredrift, Hatmax, external examples or global machine/agent configuration.

Use Conventional Commits without task-ID suffixes. Follow the existing three-
section PR body and canonical per-slice report, with successful exact ordered
validation through looper validate. Preserve unrelated work and other controllers.

## Baseline evidence and staged adoption

The reviewed baseline is clean and synchronized. Complete first-party inventory:
1,787 Rust files / 340,877 lines. Complete baseline Clippy fails duplicate_mod;
the existing native source metric reports 608 diagnostics. These are transparent
pre-existing findings, not knowingly broken new tests or 608 behavioral defects.
The last plugin delivery is complete and its wire/history/resource assertions
remain protected, including the independent external Go client's acceptance.

First establish correct discovery, working scoped guards and shared test support.
Each domain then clears the selected profile and source diagnostics in its owner
roots; scope is explicit, never a line/commit baseline. Final full-workspace
adoption occurs only after every domain is clean. Do not publish a global success
from a focused run or add broad suppressions to make an intermediate stage green.
No runtime feature, migration, dependency or generic framework is introduced.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Quality foundation | chore/rev-architecture-quality-01-foundation | chore(lint): establish complete native workspace quality checks | ops/default/report/slices/rev-architecture-quality/slice-1-foundation.md |
| Slice 2 | Plugin and SDK ownership | refactor/rev-architecture-quality-02-plugins | refactor(plugins): clarify request ownership and avoid redundant payload copies | ops/default/report/slices/rev-architecture-quality/slice-2-plugins.md |
| Slice 3 | Core editing and projection | refactor/rev-architecture-quality-03-core | refactor(core): enforce ownership and complexity in native editing | ops/default/report/slices/rev-architecture-quality/slice-3-core.md |
| Slice 4 | Configuration and persistent concerns | refactor/rev-architecture-quality-04-storage | refactor(storage): preserve explicit ownership across native state concerns | ops/default/report/slices/rev-architecture-quality/slice-4-storage.md |
| Slice 5 | Git operation boundaries | refactor/rev-architecture-quality-05-git | refactor(git): separate operation-specific preparation and observation | ops/default/report/slices/rev-architecture-quality/slice-5-git.md |
| Slice 6 | External process workflows | refactor/rev-architecture-quality-06-processes | refactor(process): clarify IO ownership and bounded cleanup paths | ops/default/report/slices/rev-architecture-quality/slice-6-processes.md |
| Slice 7 | Org workflow decomposition | refactor/rev-architecture-quality-07-org | refactor(org): keep native document workflows within explicit owners | ops/default/report/slices/rev-architecture-quality/slice-7-org.md |
| Slice 8 | Session interaction and completion owner | refactor/rev-architecture-quality-08-interaction | refactor(session): own completion transitions and simplify native dispatch | ops/default/report/slices/rev-architecture-quality/slice-8-interaction.md |
| Slice 9 | Fixtures and developer examples | refactor/rev-architecture-quality-09-fixtures | refactor(test): keep acceptance fixtures within native quality contracts | ops/default/report/slices/rev-architecture-quality/slice-9-fixtures.md |
| Slice 10 | Staged enforcement and acceptance | chore/rev-architecture-quality-10-enforcement | chore(lint): enforce strict staged commit checks and document architecture rules | ops/default/report/slices/rev-architecture-quality/slice-10-enforcement.md |

## Validation and tooling contract

Extend examples/source-check.rs rather than introducing another source policy.
scripts/check-quality.sh is the canonical Bash entry point. With --scope PATH it
checks every admitted Rust source under the explicitly named workspace roots and
selected Clippy diagnostics for those owners, printing scoped coverage honestly.
Compiler/build failures always fail. Use the existing serde/serde_json support
for native diagnostic parsing if needed; no Python jq/parser dependency is added.
The driver runs formatting, compiler/Clippy and source policy with explicit component results; avoid duplicate Clippy builds for equivalent checks. Baseline all-group/compiler failures remain blocking globally during scoped adoption. Before final global lint inheritance is adopted, other owners' selected advisory
diagnostics remain visible as deferred work, not suppressed or counted as passed.
Without scopes, check the complete first-party workspace and reject every selected
violation. Empty/unresolvable selections fail. Never narrow ordinary final commit
checks to changed lines; compile complete applicable workspace crate/target owners.

The staged hook executes staged source/manifests/policy/tooling in an immutable
index export, not the worktree. Use private per-worktree cache ownership and safe
stable paths to avoid gratuitous rebuilds; preserve untracked/ignored user data,
index/worktree bytes and active builds. No source-format auto-fix, stash, forced
Git operation or cache-clean workaround belongs to a hook. Docs-only commits use
applicable fast checks. Full lint remains independently runnable.

Keep package/wire/config/state/source-coordinate APIs compatible. A public owned
parameter or guarded conversion may legitimately retain its shape under a narrow
reasoned expect. No blanket panic/Arc/Rc ban, arbitrary helper extraction, tuple
parameter bags or closure/macro hiding replaces ownership design. Tests retain
names, assertions, varied inputs, timeouts and resource ceilings. Do not add a
new assertion for every refactor or test implementation details unnecessarily.

The paths in each focused source selection must remain covered after extraction:
keep helpers under their selected concern root or extend the invoked explicit
selection to its new owned path without dropping prior coverage. Do not alter
frozen command strings after checkpoint creation. Source move outside an owner
that changes the contract requires review rather than hiding a diagnostic.

## Slice tasks and frozen focused commands

### Slice 1: Quality foundation

Dependency: committed/pushed preparation, available tools/capacity and controller preflight.

Findings: F1, F2, F3, F7.

- T1.1: Correct Cargo workspace/member/target discovery in the existing Rust source checker; place cohesive discovery/diagnostic helpers under examples/source_check, retain metrics/legacy arguments, and add nonempty scoped checks plus bounded Clippy admission.
- T1.2: Introduce scripts/check-quality.sh with the selected ownership/representation profile, truthful scoped coverage and no Python requirement; repair duplicate macros test support and keep complete baseline all-target Clippy green.
- T1.3: Add real parser/discovery/exception/scope fixtures and an evidence-backed criteria map; cover new tooling itself without presenting unreviewed domains as passing full-profile lint.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope examples/source-check.rs --scope examples/source_check --scope tests/macros/acceptance.rs --scope tests/macros/pty.rs`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --example source-check`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- capture large_cancellation --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

### Slice 2: Plugin and SDK ownership

Dependency: Slice 1 canonically integrated.

Findings: F3, F4, F5.

- T2.1: Resolve F4 by separating encoded request payload ownership from pending correlation while preserving permits, deadlines, captured context, causal propagation and exactly-once completion.
- T2.2: Resolve F5 without duplicating keymap observations on unchanged cache hits; complete SDK error/panic contracts, selected ownership/conversion diagnostics and existing source bounds in SDK/plugin owners.
- T2.3: Verify public protocol, typed client, callbacks, cancellation/retirement, keymap precedence and atomic publication using the existing SDK and native plugin fixtures; retain wire/API/state versions.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope crates/rev-plugin-sdk --scope src/plugins.rs --scope src/plugins --scope src/session/plugins.rs --scope src/session/plugins`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

### Slice 3: Core editing and projection

Dependency: Slice 2 canonically integrated.

Findings: F3.

- T3.1: Correct selected ownership/representation diagnostics and existing source bounds in buffers, source coordinates, views, windows, display, syntax, gutter/tree and terminal-emulation owners.
- T3.2: Extract coherent decision/preparation/projection functions while preserving source units, revision checks, local history, shared views, caches and existing work/resource ceilings.
- T3.3: Validate checked editing, Unicode/grapheme/visual projection, selection, syntax and view restoration with the actual native owners; record any legitimate invariant exceptions narrowly.

Maintainer-approved prerequisite correction: repair the four display/gutter
failures reproduced before Slice 3. Prepare the initial Org view before capturing
calendar authority, locate configuration entries independently of parent rows,
and distinguish active-file tree highlighting from editing-pane overlays.
Retain all test names, meaningful assertions, varied geometry inputs, time/resource
bounds and frozen commands. This approval adds no slice and permits no general
test cleanup or weakened acceptance. Fixture corrections need no runtime change.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/lib.rs --scope src/main.rs --scope src/search.rs --scope src/search --scope src/rectangle.rs --scope src/rectangle --scope src/pointer.rs --scope src/pointer --scope src/panel.rs --scope src/panel --scope src/kill.rs --scope src/jump.rs --scope src/buffer.rs --scope src/buffer --scope src/coordinates.rs --scope src/coordinates --scope src/view.rs --scope src/view --scope src/window.rs --scope src/window --scope src/display.rs --scope src/display --scope src/syntax.rs --scope src/syntax --scope src/gutter.rs --scope src/gutter --scope src/tree.rs --scope src/tree --scope src/term.rs --scope src/term`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing --test coordinates --test display --test windows --test syntax --test pointer --test gutter --test tree --test core -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 4: Configuration and persistent concerns

Dependency: Slice 3 canonically integrated.

Findings: F3.

- T4.1: Correct selected borrowing/conversion/source-policy diagnostics in configuration, file IO/watch, contexts, locations, bookmarks and their native persistence adapters.
- T4.2: Keep configuration/data/state/cache separation, private namespace paths, locks, CAS and uncertain-write evidence; preserve coordinated publication and avoid a generic store framework.
- T4.3: Exercise invalid/stale/readonly sources, partial writes, concurrent records, project/workspace boundaries and existing file-watch retirement through native fixture paths.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/config.rs --scope src/config --scope src/file.rs --scope src/file --scope src/context.rs --scope src/context --scope src/location.rs --scope src/location --scope src/bookmark.rs --scope src/bookmark --scope src/session/contexts.rs --scope src/session/contexts`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test configuration --test files --test external_changes --test contexts --test bookmarks -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 5: Git operation boundaries

Dependency: Slice 4 canonically integrated.

Findings: F3.

- T5.1: Split the verified oversized native Git poll/dispatch/operation handlers into cohesive operation-family and observation owners under the existing modules.
- T5.2: Resolve selected ownership/conversion and five/three/ten diagnostics in native and session Git code; retain immutable observations, process ownership, authority checks, hook/signing policy and mutation preconditions.
- T5.3: Validate status/index/commit/history/remote/sequence/conflict paths through existing disposable repositories, preserving all negative assertions and exact native command behavior.

Maintainer-approved compatibility exception: replace the existing six-parameter
`Repositories::prepare_family_output` signature with a cohesive native
`FamilyOutput { family, args, output }` request and adapt its session call and
three existing test calls. Retain the five-parameter source limit, exact command
arguments, resource bounds, assertions, user commands and plugin/SDK protocols.
This exception is limited to this Rust signature; it does not authorize general
API changes or moving failing slice checks to the final gate. Finish the remaining
Git source corrections and integration repairs before this slice is integrated.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/git.rs --scope src/git --scope src/session/git.rs --scope src/session/git`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test git_integration -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 6: External process workflows

Dependency: Slice 5 canonically integrated.

Findings: F3.

- T6.1: Correct selected ownership, narrowing, guard-lifetime and source-policy findings in AI/Pi, LSP, shell, integrated-terminal and native process owners plus their session adapters.
- T6.2: Keep worker IO away from input/rendering, shared mailboxes intentional, owned process cleanup/cancellation finite and generation-bound; preserve existing environment/credential redaction and terminal restoration.
- T6.3: Exercise real owned child fixtures and native request/reload/fallback/terminal relocation boundaries without changing protocol capabilities, external configuration or unrelated host processes.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/ai.rs --scope src/ai --scope src/lsp.rs --scope src/lsp --scope src/process.rs --scope src/process --scope src/shell.rs --scope src/shell --scope src/terminal.rs --scope src/terminal --scope src/session/ai.rs --scope src/session/ai --scope src/session/lsp.rs --scope src/session/lsp --scope src/session/shell.rs --scope src/session/shell --scope src/session/terminals.rs --scope src/session/terminals`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test ai --test lsp --test lsp_installation --test shell_commands --test integrated_terminal --test pi -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 7: Org workflow decomposition

Dependency: Slice 6 canonically integrated.

Findings: F3.

- T7.1: Correct selected lint/source findings in native Org and its session document/agenda/capture/table/export/execution adapters, keeping parsing and publication responsibilities distinct.
- T7.2: Extract cohesive validation, preparation and outcome handlers while preserving Org source text, navigation, folding/indentation, native consent, state ownership and document limits.
- T7.3: Run relevant existing native Org integration cases, preserving public command names, keybindings, timestamps, agenda/capture targets, formulas and code-block behavior.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/org.rs --scope src/org --scope src/session/agenda --scope src/session/agenda.rs --scope src/session/agenda_actions --scope src/session/agenda_actions.rs --scope src/session/agenda_custom --scope src/session/agenda_custom.rs --scope src/session/agenda_queries --scope src/session/agenda_queries.rs --scope src/session/agenda_sources.rs --scope src/session/capture --scope src/session/capture.rs --scope src/session/refile --scope src/session/refile.rs --scope src/session/attachment --scope src/session/attachment.rs --scope src/session/document.rs --scope src/session/outline --scope src/session/outline.rs --scope src/session/tables --scope src/session/tables.rs --scope src/session/tangling --scope src/session/tangling.rs --scope src/session/execution --scope src/session/execution.rs --scope src/session/feeds --scope src/session/feeds.rs --scope src/session/previews.rs --scope src/session/exports --scope src/session/exports.rs --scope src/session/publishing --scope src/session/publishing.rs --scope src/session/source_edit --scope src/session/source_edit.rs --scope src/session/links --scope src/session/links.rs --scope src/session/habits --scope src/session/habits.rs --scope src/session/org --scope src/session/org.rs`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test org_agenda --test org_editing --test org_headings --test org_indentation --test org_io --test org_source --test org_tables --test org_time -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 8: Session interaction and completion owner

Dependency: Slice 7 canonically integrated.

Findings: F3, F6.

- T8.1: Resolve F6 with a cohesive completion concern owner and explicit admission/retirement transitions; keep public completion observations and native editing/context semantics intact.
- T8.2: Correct remaining command/input/macro/session selected diagnostics and source bounds, including retained macro body identity without raw-pointer-type casts or identity-spelling workarounds.
- T8.3: Verify both interaction profiles, prefix arguments, semantic macro replay/history, stale completion, transient ownership and source/view/context switches using native regression suites.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/command.rs --scope src/command --scope src/input.rs --scope src/input --scope src/macros.rs --scope src/macros --scope src/session.rs --scope src/session`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands --test modal --test completion --test macros -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

### Slice 9: Fixtures and developer examples

Dependency: Slice 8 canonically integrated.

Findings: F2, F3.

- T9.1: Resolve remaining selected diagnostics and existing source bounds in native test and example code, including every Cargo root and shared fixture ownership.
- T9.2: Keep existing case identities, semantic/resource/PTY assertions and varied Unicode/numeric inputs; extract cohesive setup/check steps instead of hiding branches in macros, closures or unrelated parameter bundles.
- T9.3: Reconcile all-target compilation, native source coverage and developer/publication helper behavior; verify representative macro/plugin/command journeys and SDK documentation before final global adoption.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope tests --scope examples --scope crates/rev-plugin-sdk/tests`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros --test commands --test plugins -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
5. `git diff --check`

### Slice 10: Staged enforcement and acceptance

Dependency: Slice 9 canonically integrated.

Findings: F1, F2, F3, F4, F5, F6, F7.

- T10.1: Adopt the final inherited workspace Clippy/source policy, exact method restrictions and reasoned local expectations; add canonical Makefile commands and concise repository AGENTS/local concern guidance.
- T10.2: Implement the nonmutating index-snapshot hook, exclusively owned per-worktree reusable cache and explicit install procedure; verify real Git partial staging, manifests/policy changes, deletes/renames, conflicts, concurrency and unchanged user content.
- T10.3: Recheck F1-F7 and the criteria map against final code, measure cold/warm checks, document use, run focused enforcement acceptance and prepare the unchanged controller gate/Markdown closure.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make test-hooks`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
5. `git diff --check`

## Frozen integrated gate

This broad source/ownership change affects every native owner, so one complete
workspace native test pass is justified at the delivery boundary. Earlier slices
use relevant targets rather than repeating that full pass. Do not chain all old
Python/PTY/performance delivery scripts; preserve their meaningful native cases.
The external Go runner is an existing separate compatibility acceptance, never
a dependency of normal editor/SDK builds or commit-time checks. It is read-only
with respect to its source repository and pins both tested source commits.

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make lint`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 make test-hooks`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --all-targets -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --workspace --doc`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash "$(git rev-parse --path-format=absolute --git-common-dir)/../../rev-plugin-examples/scripts/check-rev.sh" "$PWD"`
8. `git diff --check`

All compiler/check commands use one build job, disabled incremental output and
no dev/test debuginfo. Tooling must preserve checkout-local targets. Cold builds
are the main expected cost; measure cold/warm lint/hook elapsed time and report
native-suite timing. Keep readiness/cancel tests finite; do not raise limits or
repeat a passing complete gate without changed code or a demonstrated concern.

## Preparation, hook activation and closure

Require clean synchronized dev and no competing repository owner before starting.
Review only adds this specification, the report, plan/tracker and concern links;
all code/hook/config implementation belongs to the recorded worktrees.

After Slice 10 integration, prepare_gate may run the integrated strict lint and
hook acceptance checks and activate make install-hooks in this checkout. Verify
repository-local core.hooksPath and a real linked-worktree fixture without changing
another repository's configuration. This is an explicitly requested setup action;
if conflicting pre-existing hooks appear, preserve them and report the conflict.
The controller then owns the unchanged full gate. Source commits cannot be repaired
by disabling the hook or weakening its checks.

A final-gate defect within this set may use up to three bounded repair PRs under
the installed Looper authorization mechanism, preserving all assertions/gates and
worker budgets. No changes to Looper itself, unapproved dependencies, broad cache
cleanup, force-push or run resets are authorized.

After successful exact-candidate gate evidence and verified hook activation,
closure changes only this plan/tracker/specification, the review report, concern
index and the ten slice report Markdown paths. Mark F1-F7 resolved with evidence,
record any genuine review limits, deliver all reports and mark artifacts Completed.
Do not edit runtime, lint/hook policy or command strings after the tested candidate.
