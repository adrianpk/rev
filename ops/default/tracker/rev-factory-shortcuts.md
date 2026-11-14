# Factory Shortcut Alignment Tracker

Status: Slice 2 proposal awaiting owner decision
Date: 2026-11-14
Delivery set: rev-factory-shortcuts
Slice strategy: layered
Planning authority: product-owner request for slicing, 2026-11-14
Approved: product owner, 2026-11-14; prepare, commit and push the delivery records, execute audit slices 1-2 through autonomous Looper PR integration into dev, and retain the explicit mapping decision checkpoint before runtime changes
Execution authority: audit slices 1-2; mapping decision checkpoint before runtime changes
Plan: [Factory Shortcut Alignment Delivery](../plan/rev-factory-shortcuts.md)
Spec: [Factory Shortcut Alignment](../spec/editor-factory-shortcut-alignment.md)
Active slice: Slice 2, Mode mapping contract
Progress: 1/7 slices delivered; 4/21 tasks completed
Readiness: complete Slice 2 proposal; D01-D03 acceptance required before PR submission and runtime freeze

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Core and profile audit | `docs/rev-factory-shortcuts-01-core-audit` | `docs(input): audit core shortcut equivalence` | `ops/default/report/slices/rev-factory-shortcuts/slice-1-core-audit.md` |
| Slice 2 | Mode mapping contract | `docs/rev-factory-shortcuts-02-mode-contract` | `docs(input): specify contextual shortcut mappings` | `ops/default/report/slices/rev-factory-shortcuts/slice-2-mode-contract.md` |
| Slice 3 | Editor defaults | `feat/rev-factory-shortcuts-03-editor` | `feat(input): align core factory shortcuts` | `ops/default/report/slices/rev-factory-shortcuts/slice-3-editor.md` |
| Slice 4 | Org document defaults | `feat/rev-factory-shortcuts-04-org` | `feat(org): align document factory shortcuts` | `ops/default/report/slices/rev-factory-shortcuts/slice-4-org.md` |
| Slice 5 | Agenda defaults | `feat/rev-factory-shortcuts-05-agenda` | `feat(org): align agenda shortcut families` | `ops/default/report/slices/rev-factory-shortcuts/slice-5-agenda.md` |
| Slice 6 | Git defaults | `feat/rev-factory-shortcuts-06-git` | `feat(git): align reviewed shortcut families` | `ops/default/report/slices/rev-factory-shortcuts/slice-6-git.md` |
| Slice 7 | Terminal acceptance and guidance | `test/rev-factory-shortcuts-07-acceptance` | `test(input): verify factory shortcut compatibility` | `ops/default/report/slices/rev-factory-shortcuts/slice-7-acceptance.md` |

## Entry and mapping gates

| Gate | Required evidence | State |
| --- | --- | --- |
| Audit entry | Product-owner continuation accepted; publish the preparation before controller startup. | approved; preparation published at `2f5cfa485fb0169c0f4dcdea475a33eb4a80eabd` |
| Core review | Slice 1 complete core/profile matrix with actual probe evidence and pending mode rows identified. | integrated through PR #47 at `c279fd03ccc383a9ae606665bf0cd7ae2cfccc49` |
| Mapping freeze | Slice 2 complete matrix; D01-D03 settled; accepted exact mapping contract and reconciled spec/plan/tracker committed. | pending |
| Runtime entry | Slice 3 onward uses only that accepted manifest; concrete task and validation selection contains no placeholders. | pending |
| Delivery closure | All accepted slices integrated; selected gate passes for the exact dev candidate; reports, guide and ticket reconciled. | pending |

The continuation approves the two audit slices and their autonomous integration.
The exact mapping checkpoint remains pending. Before the Slice 2 PR, publish
the clean proposal and return recovery_requested with kind decision through
the ordinary controller mechanism. Do not self-approve D01-D03 or enter Slice 3.
Any changed frozen identity or command needs explicit configuration resolution;
preserve the same checkpoint and consumed budgets.

## Slice 1: Core and profile audit

State: delivered; PR #47 integrated into dev at `c279fd03ccc383a9ae606665bf0cd7ae2cfccc49`
Requirements: FSA01, FSA02, FSA03, FSA07, FSA08; core portion of D03.
Dependency: approved and committed audit preparation; no runtime authorization is required by this slice's scope.
Branch: `docs/rev-factory-shortcuts-01-core-audit`
Expected PR title: `docs(input): audit core shortcut equivalence`
PR: `#47` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/47)); merged into `dev`
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-1-core-audit.md`
Task commits: T1.1 `45d247e0a46893f81888fd09cbfac684d5aab743`; T1.2 `efa39f79a55d5a609781744a6ae8ae71f9d396f0`; T1.3/report introduction `13366e525c7e3703ccea1b48bb70b9aeccc244a2`.
Validation: live source-compiled native/reference audit and configured coverage/document consistency check PASS; `git diff --check` PASS. The exact clean-HEAD controller receipt passed for the merged candidate `c279fd03ccc383a9ae606665bf0cd7ae2cfccc49`.

- [x] T1.1: Re-extract the complete public command/action inventory and context maps at the actual candidate. Persist sanitized catalog.json beside the semantic matrix and record explicit coverage ownership; distinguish duplicate inherited entries, aliases and indirect dispatcher access. Preserve reproducible inspection commands and version evidence without personal configuration or machine identifiers.
- [x] T1.2: Review core editing, navigation, selection, arguments/history, panes, projects, prompts/completion, tree/bookmarks, macros, terminal controls and other native command families, including six modal states. Compare operations under different names and reject misleading approximations. Record each operation's disposition and references; mark mode-owned rows pending Slice 2 rather than claiming global completion.
- [x] T1.3: Reproduce high-value effects and boundaries for P01-P04 and related primitives with owned native/reference probes. Verify first-text placement, join direction, paragraph/delimiter differences, profile separation, protected keys and terminal aliases. Produce the slice report; retain factory and personal configuration bytes unchanged.

Completed focused checks: coverage reconciliation against the live catalog; source/reference evidence review; owned behavioral probes with exact commands and limits; local-link/identifier checks; `git diff --check`. No broad Cargo or historical feature gate is required for documentation-only audit outputs.

## Slice 2: Mode mapping contract

State: proposal prepared; owner decision required before PR submission
Requirements: FSA01, FSA02, FSA05, FSA06, FSA07, FSA09; D01-D03.
Dependency: Slice 1 integrated; mode reference sources and native fixtures available.
Branch: `docs/rev-factory-shortcuts-02-mode-contract`
Expected PR title: `docs(input): specify contextual shortcut mappings`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-2-mode-contract.md`
Task commits: none
Validation: focused native/reference comparison, 92 normalized checks and 21 active language/profile sessions PASS; proposal structural checks PASS. Configured approval-dependent clean-HEAD validation awaits owner acceptance; no delivery gate ran.

- [x] T2.1: Complete semantic review of Org headings/lists/tables/source blocks, Agenda/capture/calendar/source editing, Markdown plain/list/table dispatch and every native Git panel/menu. Compare active contexts and effects, including source targets behind projections; finish all pending matrix rows and evidence-backed named-only or adaptation dispositions.
- [ ] T2.2: Present and settle D01 Agenda placements and D02 Git family decisions, including P06-P08 and displaced command access. Produce the exact before/after mapping contract, preserve explicit tree/Markdown conventions and identify each selected supersession of legacy tables. Keep unresolved product choices visible and obtain their acceptance before closing the mapping freeze.
- [ ] T2.3: Reconcile the specification, mapping contract, remaining slice tasks, coverage/check selection and expected cost. Remove an empty provisional Git slice if appropriate through a reviewed map revision. Record approval evidence and the slice report; keep runtime defaults unchanged. Runtime slices remain ineligible until accepted artifacts are committed.

Completed focused checks: catalog/context coverage, normalized sequence/prefix checks for both profiles, native/reference effects, baseline/document consistency and `git diff --check`. Required approval-dependent clean-HEAD validation remains pending; focused probes are not terminal or full behavioral acceptance.

T2.2 has a complete exact proposal and concrete D01/D02 alternatives; acceptance
is pending. T2.3 has reconciled P01-P19 runtime tasks, unchanged identities/gates
and the measured-cost estimate in the mapping contract; its approval record and
PR remain pending. The report is the decision handoff, not Slice 2 delivery.

## Slice 3: Editor defaults

State: planned; accepted mapping freeze required
Requirements: FSA03, FSA04, FSA07-FSA09; P01-P04 and accepted core manifest rows.
Dependency: Slice 2 integrated and mapping freeze accepted/committed.
Branch: `feat/rev-factory-shortcuts-03-editor`
Expected PR title: `feat(input): align core factory shortcuts`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-3-editor.md`
Task commits: none
Validation: not run; this record contains planned checks only.

- [ ] T3.1: Apply only accepted editor/profile factory assignments for first text, pane balance, redo and registered Project switching. Preserve canonical handlers, context transitions, modal state and existing search/operator keys. Keep factories and the exhaustive configuration reference consistent.
- [ ] T3.2: Extend existing command/keymap tests with a cohesive `factory_shortcuts` module for actual input-to-effect regressions. Cover Unicode/blank lines, selection/count policy, redo groups, pane focus, cancelled Project restoration, overrides, local overlays and all affected modal states; avoid a second handwritten production binding catalog.
- [ ] T3.3: Update the affected binding reference and core User Guide guidance; use the changelog-normalize skill for one concise user-facing outcome when behavior is usable. Run the core slice checks and record the canonical report and task evidence.

Planned checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `cargo test --locked --test modal keymaps -- --test-threads=1`; `git diff --check`.

## Slice 4: Org document defaults

State: planned; accepted mapping freeze required
Requirements: FSA03-FSA09; P05-P07 and accepted Org document manifest rows.
Dependency: Slice 3 integrated; Org replacement rows accepted.
Branch: `feat/rev-factory-shortcuts-04-org`
Expected PR title: `feat(org): align document factory shortcuts`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-4-org.md`
Task commits: none
Validation: not run; this record contains planned checks only.

- [ ] T4.1: Apply accepted effort, property-column and Babel header-argument factory assignments through native owners. Retain named clock reports, explicit old user assignments and unaffected aliases. Preserve unsupported-help and radio-button boundaries rather than reusing their familiar chords.
- [ ] T4.2: Extend the shared factory-shortcut regressions with Org source-context effects: effort choices/errors, read-only column projection without a clocktable edit, header insertion/cancellation and undo, inherited profile maps and retained explicit overrides. Guard existing list/heading/table dispatch and partial M-S-RET equivalence.
- [ ] T4.3: Reconcile only selected Org contract rows, binding/configuration references and relevant guide text. Preserve historical delivery evidence, use a concise changelog outcome for the delivered behavior and produce the slice report.

Planned checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 5: Agenda defaults

State: planned; accepted mapping freeze required
Requirements: FSA03, FSA05-FSA09; P08 and accepted D01 manifest rows.
Dependency: Slice 4 integrated; every implemented Agenda placement explicitly accepted.
Branch: `feat/rev-factory-shortcuts-05-agenda`
Expected PR title: `feat(org): align agenda shortcut families`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-5-agenda.md`
Task commits: none
Validation: not run; this record contains planned checks only.

- [ ] T5.1: Apply proposed P08-P13 only after acceptance: daily d, tags v M, search v /, show-all habits v A, graph window v G and sort v S; remove their displaced single keys through the existing native owners. Preserve named access for displaced habit/view actions, current prompt ownership and all unselected defaults.
- [ ] T5.2: Extend the factory-shortcut suite with selected-row target identity, daily view versus habit-date behavior, each approved family relocation, named and overridden old access, cancellation, source immutability for view-only actions and protection against suffix keys granting consent.
- [ ] T5.3: Reconcile the selected Agenda reference/specification rows and task/agenda guide text; record concise user-visible changes and the slice report. Do not add missing reference features to occupy keys.

Planned checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 6: Git defaults

State: planned; accepted mapping freeze required
Requirements: FSA02, FSA03, FSA05, FSA07-FSA09; accepted D02 manifest rows.
Dependency: Slice 5 integrated; a meaningful Git increment and its exact menu/direct-action boundary accepted at mapping freeze.
Branch: `feat/rev-factory-shortcuts-06-git`
Expected PR title: `feat(git): align reviewed shortcut families`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-6-git.md`
Task commits: none
Validation: not run; this record contains planned checks only.

- [ ] T6.1: Apply proposed P14-P19 only after acceptance: History A cherry-pick; remove History P/F/f; add Refs f/P for direct fetch/push, retaining existing Refs aliases. Update History panel guidance with the changed routes. Keep menu-versus-direct-action behavior explicit, operation-specific review/confirmation intact, selection/range identities stable and displaced native actions reachable through the accepted routes.
- [ ] T6.2: Add owned local-repository factory-shortcut regressions for every changed Git family, including history selection, complete prefixes, cancelled menus and commands, explicit overrides and absence of unintended mutation. Do not use credentials, external remotes or broad transport journeys to establish key routing.
- [ ] T6.3: Reconcile the exact superseded Binding review matrix rows, current binding reference and Git guide, preserving historical validation. Record delivered behavior and the canonical slice report; do not present intentional retained exceptions as missing implementations.

Planned checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`. Add a narrow existing Git regression only if the accepted implementation changes its owner beyond input routing; record that selection before execution.

## Slice 7: Terminal acceptance and guidance

State: planned; accepted mapping freeze required
Requirements: FSA01-FSA10 together; accepted manifest and retained exceptions.
Dependency: all accepted runtime slices integrated; manifest identity unchanged.
Branch: `test/rev-factory-shortcuts-07-acceptance`
Expected PR title: `test(input): verify factory shortcut compatibility`
PR: pending
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-7-acceptance.md`
Task commits: none
Validation: not run; this record contains planned checks only.

- [ ] T7.1: Add a narrow `scripts/check-factory-shortcuts.sh` entrypoint and `examples/factory-shortcuts-acceptance.rs`, reusing existing owned native/surface/PTY infrastructure. Cross-check inventory coverage, accepted normalized routes, actual effects, overrides, profile/focus transitions and selector hints. Compare executable results with the separately accepted mapping contract; do not rewrite expectations from current defaults.
- [ ] T7.2: Exercise bounded physical-input journeys in nonmodal and modal editing plus one local read-only Git journey. Cover representative Meta/control and multi-chord routes, prefix cancellation, editing/profile/language transitions and terminal restoration. Keep variants native, use finite observable conditions and clean only owned fixtures/processes. Verify expected source/history effects rather than treating a successful exit as compatibility.
- [ ] T7.3: Complete reference/guide consistency and Spore projection; produce the final slice report and prepare the exact integrated candidate for the selected delivery-set gate. Close the originating ticket and delivery metadata only after that gate passes. Record measured check costs and remove integrated inactive clean worktrees.

Planned checks: `make lint`; `bash scripts/check-factory-shortcuts.sh`; `bash scripts/check-documentation.sh`; `bash scripts/build-user-guide.sh`; `git diff --check`. This prepares the slice; the integrated candidate still requires the final gate below.

## Decision log

| Decision | Scope | State |
| --- | --- | --- |
| D01 | P08-P13 daily `d` and native `v M`, `v /`, `v A`, `v G`, `v S`; retaining current keys is an explicit alternative. | proposal complete; owner acceptance pending |
| D02 | P14-P19 History A, remove P/F/f; Refs f/P aliases, native direct-action exceptions and exact legacy supersessions. | proposal complete; owner acceptance pending |
| D03 | All 707 rows and six actions reviewed; source Save/heading gaps, indirect routes and deliberate native conventions recorded. | classification complete; owner acceptance pending |

Do not mark a decision settled from successful parsing, a same-name match or
the existence of a current factory key. Record the reviewed operation contract
and product acceptance in the mapping artifact and owning specification.

## Delivery-set gate

State: not run. No delivery evidence, receipt or tested candidate exists.
Environment: one Cargo job, incremental disabled, dev/test debuginfo disabled.

Proposed canonical order:

1. `make lint`
2. `make test-hooks`
3. `bash scripts/check-factory-shortcuts.sh`
4. `bash scripts/check-documentation.sh`
5. `bash scripts/build-user-guide.sh`
6. `git diff --check`

The factory-shortcut script is planned in T7.1. Its native/input/profile checks,
three representative PTYs and cost selection are defined by the plan; freeze
the final command list at mapping approval. Historical feature gates are not
added implicitly. Failed checks retain their actual candidate and result.

## Closure

- [ ] Every native operation/action has a disposition and accepted mapping changes match the frozen contract.
- [ ] Every accepted slice has its task commits, integrated PR and canonical report with actual validation.
- [ ] The selected integrated gate passes against the exact dev candidate.
- [ ] Binding/configuration references, guide projection and originating ticket reflect delivered behavior.
- [ ] Integrated inactive clean worktrees are removed non-forcibly; unrelated or active work remains untouched.
