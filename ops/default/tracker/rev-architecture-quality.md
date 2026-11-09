# Architecture Quality Tracker

Status: In progress
Date: 2026-11-09
Delivery set: rev-architecture-quality
Slice strategy: behavior-first
Approved: owner requested one architecture remediation set with strict pre-commit Clippy/linter enforcement
Planning baseline: abbf0c21eeae1680c02c7a477fa0894d7996a393
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Review: [Architecture and Ownership Nit Review](../report/rev-architecture-nit-review.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 1 - Quality foundation
Plan: [Architecture Quality Delivery](../plan/rev-architecture-quality.md)
Execution authorized: one ten-slice Looper run, automatic canonical integration into dev and local hook setup
Progress owner: selected Looper run; no second runtime tracker

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

## Slice tasks

## Slice 1: Quality foundation

State: reviewing
Dependency: approved preparation and controller preflight
Branch: chore/rev-architecture-quality-01-foundation
PR: #15 - https://forge.adrianpk.com/rev/rev/pulls/15
Implementation: 3c76d32116e739f26c51abe1e171bd39bb69c877
Report: ops/default/report/slices/rev-architecture-quality/slice-1-foundation.md
Validation: all five frozen focused commands passed locally; final clean-HEAD evidence required through Looper before handoff

- [x] T1.1: Correct Cargo workspace/member/target discovery in the existing Rust source checker; place cohesive discovery/diagnostic helpers under examples/source_check, retain metrics/legacy arguments, and add nonempty scoped checks plus bounded Clippy admission.
- [x] T1.2: Introduce scripts/check-quality.sh with the selected ownership/representation profile, truthful scoped coverage and no Python requirement; repair duplicate macros test support and keep complete baseline all-target Clippy green.
- [x] T1.3: Add real parser/discovery/exception/scope fixtures and an evidence-backed criteria map; cover new tooling itself without presenting unreviewed domains as passing full-profile lint.

## Slice 2: Plugin and SDK ownership

State: planned
Dependency: Slice 1 integrated
Branch: refactor/rev-architecture-quality-02-plugins
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-2-plugins.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T2.1: Resolve F4 by separating encoded request payload ownership from pending correlation while preserving permits, deadlines, captured context, causal propagation and exactly-once completion.
- [ ] T2.2: Resolve F5 without duplicating keymap observations on unchanged cache hits; complete SDK error/panic contracts, selected ownership/conversion diagnostics and existing source bounds in SDK/plugin owners.
- [ ] T2.3: Verify public protocol, typed client, callbacks, cancellation/retirement, keymap precedence and atomic publication using the existing SDK and native plugin fixtures; retain wire/API/state versions.

## Slice 3: Core editing and projection

State: planned
Dependency: Slice 2 integrated
Branch: refactor/rev-architecture-quality-03-core
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-3-core.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T3.1: Correct selected ownership/representation diagnostics and existing source bounds in buffers, source coordinates, views, windows, display, syntax, gutter/tree and terminal-emulation owners.
- [ ] T3.2: Extract coherent decision/preparation/projection functions while preserving source units, revision checks, local history, shared views, caches and existing work/resource ceilings.
- [ ] T3.3: Validate checked editing, Unicode/grapheme/visual projection, selection, syntax and view restoration with the actual native owners; record any legitimate invariant exceptions narrowly.

## Slice 4: Configuration and persistent concerns

State: planned
Dependency: Slice 3 integrated
Branch: refactor/rev-architecture-quality-04-storage
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-4-storage.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T4.1: Correct selected borrowing/conversion/source-policy diagnostics in configuration, file IO/watch, contexts, locations, bookmarks and their native persistence adapters.
- [ ] T4.2: Keep configuration/data/state/cache separation, private namespace paths, locks, CAS and uncertain-write evidence; preserve coordinated publication and avoid a generic store framework.
- [ ] T4.3: Exercise invalid/stale/readonly sources, partial writes, concurrent records, project/workspace boundaries and existing file-watch retirement through native fixture paths.

## Slice 5: Git operation boundaries

State: planned
Dependency: Slice 4 integrated
Branch: refactor/rev-architecture-quality-05-git
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-5-git.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T5.1: Split the verified oversized native Git poll/dispatch/operation handlers into cohesive operation-family and observation owners under the existing modules.
- [ ] T5.2: Resolve selected ownership/conversion and five/three/ten diagnostics in native and session Git code; retain immutable observations, process ownership, authority checks, hook/signing policy and mutation preconditions.
- [ ] T5.3: Validate status/index/commit/history/remote/sequence/conflict paths through existing disposable repositories, preserving all negative assertions and exact native command behavior.

## Slice 6: External process workflows

State: planned
Dependency: Slice 5 integrated
Branch: refactor/rev-architecture-quality-06-processes
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-6-processes.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T6.1: Correct selected ownership, narrowing, guard-lifetime and source-policy findings in AI/Pi, LSP, shell, integrated-terminal and native process owners plus their session adapters.
- [ ] T6.2: Keep worker IO away from input/rendering, shared mailboxes intentional, owned process cleanup/cancellation finite and generation-bound; preserve existing environment/credential redaction and terminal restoration.
- [ ] T6.3: Exercise real owned child fixtures and native request/reload/fallback/terminal relocation boundaries without changing protocol capabilities, external configuration or unrelated host processes.

## Slice 7: Org workflow decomposition

State: planned
Dependency: Slice 6 integrated
Branch: refactor/rev-architecture-quality-07-org
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-7-org.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T7.1: Correct selected lint/source findings in native Org and its session document/agenda/capture/table/export/execution adapters, keeping parsing and publication responsibilities distinct.
- [ ] T7.2: Extract cohesive validation, preparation and outcome handlers while preserving Org source text, navigation, folding/indentation, native consent, state ownership and document limits.
- [ ] T7.3: Run relevant existing native Org integration cases, preserving public command names, keybindings, timestamps, agenda/capture targets, formulas and code-block behavior.

## Slice 8: Session interaction and completion owner

State: planned
Dependency: Slice 7 integrated
Branch: refactor/rev-architecture-quality-08-interaction
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-8-interaction.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T8.1: Resolve F6 with a cohesive completion concern owner and explicit admission/retirement transitions; keep public completion observations and native editing/context semantics intact.
- [ ] T8.2: Correct remaining command/input/macro/session selected diagnostics and source bounds, including retained macro body identity without raw-pointer-type casts or identity-spelling workarounds.
- [ ] T8.3: Verify both interaction profiles, prefix arguments, semantic macro replay/history, stale completion, transient ownership and source/view/context switches using native regression suites.

## Slice 9: Fixtures and developer examples

State: planned
Dependency: Slice 8 integrated
Branch: refactor/rev-architecture-quality-09-fixtures
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-9-fixtures.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T9.1: Resolve remaining selected diagnostics and existing source bounds in native test and example code, including every Cargo root and shared fixture ownership.
- [ ] T9.2: Keep existing case identities, semantic/resource/PTY assertions and varied Unicode/numeric inputs; extract cohesive setup/check steps instead of hiding branches in macros, closures or unrelated parameter bundles.
- [ ] T9.3: Reconcile all-target compilation, native source coverage and developer/publication helper behavior; verify representative macro/plugin/command journeys and SDK documentation before final global adoption.

## Slice 10: Staged enforcement and acceptance

State: planned
Dependency: Slice 9 integrated
Branch: chore/rev-architecture-quality-10-enforcement
PR: pending
Report: ops/default/report/slices/rev-architecture-quality/slice-10-enforcement.md
Validation: pending; frozen ordered commands in plan and controller configuration

- [ ] T10.1: Adopt the final inherited workspace Clippy/source policy, exact method restrictions and reasoned local expectations; add canonical Makefile commands and concise repository AGENTS/local concern guidance.
- [ ] T10.2: Implement the nonmutating index-snapshot hook, exclusively owned per-worktree reusable cache and explicit install procedure; verify real Git partial staging, manifests/policy changes, deletes/renames, conflicts, concurrency and unchanged user content.
- [ ] T10.3: Recheck F1-F7 and the criteria map against final code, measure cold/warm checks, document use, run focused enforcement acceptance and prepare the unchanged controller gate/Markdown closure.

## Final acceptance

- [ ] F1-F7 and complete selected source/lint findings corrected with evidence.
- [ ] Exact staged-index, partial-staging, conflict and concurrent-check journeys pass without content changes.
- [ ] Final workspace profile covers both packages/all applicable targets; no baselines or broad suppressions.
- [ ] Integrated strict lint/hook checks pass and local hook installation is verified.
- [ ] The unchanged full gate passed on the exact clean integrated candidate.
- [ ] Ten reports delivered and permitted Markdown closure committed/pushed.
