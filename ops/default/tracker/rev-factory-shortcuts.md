# Factory Shortcut Alignment Tracker

Status: Completed
Completed: 2026-11-15
Date: 2026-11-14
Delivery set: rev-factory-shortcuts
Slice strategy: layered
Planning authority: product-owner request for slicing, 2026-11-14
Approved: product owner, 2026-11-14; retained P01-P19 proposal plus exact History c a/c f/+; current slices 3-7 and controller-owned dev integration authorized; core-first functional follow-through preparation requested
Execution authority: current slices 1-7 under accepted manifest and predecessor order; separate functional follow-through preparation
Plan: [Factory Shortcut Alignment Delivery](../plan/rev-factory-shortcuts.md)
Spec: [Factory Shortcut Alignment](../spec/editor-factory-shortcut-alignment.md)
Follow-through: [Emacs Interaction Parity](../spec/editor-emacs-interaction-parity.md)
Active slice: None; all seven slices integrated
Progress: 7/7 slices delivered; 21/21 tasks completed
Readiness: all seven slices integrated; controller full gate PASS at `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`; documentation closure complete

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
| Mapping freeze | Slice 2 complete matrix; D01-D03 settled; accepted exact mapping contract and reconciled spec/plan/tracker committed. | accepted and integrated through PR #48 |
| Runtime entry | Slice 3 onward uses only the accepted manifest with unchanged identities/checks. | all seven slices integrated; exact accepted manifest preserved |
| Delivery closure | All accepted slices integrated; selected gate passes for the exact dev candidate; reports, guide and ticket reconciled. | completed; full gate PASS at `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e` |

The accepted owner decision settles the exact retained proposal plus History
c a/c f/+ (decision SHA-256 `cc915b2d4af14a6ad0da34a517724e9664f59436b40d1004877e8a539af21939`).
It authorizes the current remaining slices 3-7 and controller-owned dev
integration. D03 findings have a requested [core-first functional owner](../spec/editor-emacs-interaction-parity.md)
and ordered subsequent preparation; they are not permanent arbitrary exceptions.
No second stop applies to this precise amendment. Identities, commands, budgets,
consent, user overrides and predecessor order remain unchanged.

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

State: delivered; PR #48 integrated into dev at `2ae7f75ff1a067a8b553dfc665b6022b66610e57`
Requirements: FSA01, FSA02, FSA05, FSA06, FSA07, FSA09; D01-D03.
Dependency: Slice 1 integrated; mode reference sources and native fixtures available.
Branch: `docs/rev-factory-shortcuts-02-mode-contract`
Expected PR title: `docs(input): specify contextual shortcut mappings`
PR: `#48` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/48)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-2-mode-contract.md`
Task commits: T2.1/report introduction `da5056f6e6feebc52d9114f88036097ea8e132a7`; accepted T2.2/T2.3 reconciliation and functional preparation `2fbc7ca375b666f7c4670bdef64d68f5b130af46`; this record includes canonical PR references.
Validation: focused native/reference comparison, 98 normalized checks and 21 active language/profile sessions PASS; proposal structural checks PASS. Both frozen commands passed through controller validation at `2ae7f75ff1a067a8b553dfc665b6022b66610e57` before integration; no delivery-set gate ran.

- [x] T2.1: Complete semantic review of Org headings/lists/tables/source blocks, Agenda/capture/calendar/source editing, Markdown plain/list/table dispatch and every native Git panel/menu. Compare active contexts and effects, including source targets behind projections; finish all pending matrix rows and evidence-backed named-only or adaptation dispositions.
- [x] T2.2: Present and settle D01 Agenda placements and D02 Git family decisions, including P06-P08 and displaced command access. Produce the exact before/after mapping contract, preserve explicit tree/Markdown conventions and identify each selected supersession of legacy tables. Keep unresolved product choices visible and obtain their acceptance before closing the mapping freeze.
- [x] T2.3: Reconcile the specification, mapping contract, remaining slice tasks, coverage/check selection and expected cost. Remove an empty provisional Git slice if appropriate through a reviewed map revision. Record the accepted precise amendment, slice report and source-verified core-first functional parity specification with subsequent slicing preparation; keep current runtime defaults unchanged. Runtime slices require committed accepted artifacts and predecessor integration.

Completed focused checks: catalog/context coverage, normalized sequence/prefix checks for both profiles, native/reference effects, baseline/document consistency and `git diff --check`. The controller receipt records final clean-HEAD validation after the PR-reference commit; focused probes are not terminal or full behavioral acceptance.

T2.2 records accepted D01/D02 plus the precise c a/c f/+ amendment. T2.3
reconciles the 29-record/49-edit manifest, specification, runtime tasks, unchanged
identities/gates and cost estimate, and prepares the requested source-verified
functional parity requirements and subsequent slicing. Runtime behavior remains
unchanged. Canonical PR #48 and its immutable report link are recorded; the final controller receipt owns exact clean-HEAD handoff evidence.

## Slice 3: Editor defaults

State: delivered; PR #49 integrated into dev at `c1c0aeaf217f63beb64bd25733efe661c09d2c4e`
Requirements: FSA03, FSA04, FSA07-FSA09; P01-P04 and accepted core manifest rows.
Dependency: Slice 2 integrated and mapping freeze accepted/committed.
Branch: `feat/rev-factory-shortcuts-03-editor`
Expected PR title: `feat(input): align core factory shortcuts`
PR: `#49` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/49)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-3-editor.md`
Task commits: T3.1 `94a1f69eae233e28308d158bb273947001c7ba61`; T3.2 `aad088fea37eaa2cefa79a2ae9b119b47253ce4c`; T3.3/report introduction `756ec6b1e5c497c996e89eb7273d5e4a3ec4b4e0`; this record includes canonical PR references.
Validation: all five required local commands PASS; 10 factory-shortcut, 11 configuration keymap and six modal keymap tests; documentation check PASS. All five required commands and canonical PR validation passed at clean HEAD `c1c0aeaf217f63beb64bd25733efe661c09d2c4e` before integration.

- [x] T3.1: Apply only accepted editor/profile factory assignments for first text, pane balance, redo and registered Project switching. Preserve canonical handlers, context transitions, modal state and existing search/operator keys. Keep factories and the exhaustive configuration reference consistent.
- [x] T3.2: Extend existing command/keymap tests with a cohesive `factory_shortcuts` module for actual input-to-effect regressions. Cover Unicode/blank lines, selection/count policy, redo groups, pane focus, cancelled Project restoration, overrides, local overlays and all affected modal states; avoid a second handwritten production binding catalog.
- [x] T3.3: Update the affected binding reference and core User Guide guidance; use the changelog-normalize skill for one concise user-facing outcome when behavior is usable. Run the core slice checks and record the canonical report and task evidence.

Completed local checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `cargo test --locked --test modal keymaps -- --test-threads=1`; `git diff --check`.

## Slice 4: Org document defaults

State: delivered; PR #50 integrated into dev at `f1f186918dc24d1478f92bf47ee55f6a9bd5dea9`
Requirements: FSA03-FSA09; P05-P07 and accepted Org document manifest rows.
Dependency: Slice 3 integrated; Org replacement rows accepted.
Branch: `feat/rev-factory-shortcuts-04-org`
Expected PR title: `feat(org): align document factory shortcuts`
PR: `#50` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/50)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-4-org.md`
Task commits: T4.1 `2c79884984b64f3594bda3cd05ceb76f03d9b417`; T4.2 `cc39590b0d4b6513a809eda79c05e7ee96c5c927`; T4.3/report introduction `316a5e569288aa3ee6625b03392b740a75449ef0`; this record includes canonical PR references.
Validation: all five required local commands PASS; 20 shared factory-shortcut, four Org-context and 11 configuration keymap tests; documentation check PASS. All five required commands and canonical PR validation passed at clean HEAD `f1f186918dc24d1478f92bf47ee55f6a9bd5dea9` before integration.

- [x] T4.1: Apply accepted effort, property-column and Babel header-argument factory assignments through native owners. Retain named clock reports, explicit old user assignments and unaffected aliases. Preserve unsupported-help and radio-button boundaries rather than reusing their familiar chords.
- [x] T4.2: Extend the shared factory-shortcut regressions with Org source-context effects: effort choices/errors, read-only column projection without a clocktable edit, header insertion/cancellation and undo, inherited profile maps and retained explicit overrides. Guard existing list/heading/table dispatch and partial M-S-RET equivalence.
- [x] T4.3: Reconcile only selected Org contract rows, binding/configuration references and relevant guide text. Preserve historical delivery evidence, use a concise changelog outcome for the delivered behavior and produce the slice report.

Completed local checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 5: Agenda defaults

State: delivered; PR #51 integrated into dev at `9ca41e679978354e095bd09bfe9983aa735e8d5e`
Requirements: FSA03, FSA05-FSA09; P08 and accepted D01 manifest rows.
Dependency: Slice 4 integrated; every implemented Agenda placement explicitly accepted.
Branch: `feat/rev-factory-shortcuts-05-agenda`
Expected PR title: `feat(org): align agenda shortcut families`
PR: `#51` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/51)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-5-agenda.md`
Task commits: T5.1 `493deab7fff23a9d648b7ae5f83dae4aa1465153`; T5.2 `646ffd76c98efd0257e63a119bf162a8bee7e793`; T5.3/report introduction `4a112aa03fb29ece08edfe1daf3d35cbdb309066`; this record includes canonical PR references.
Validation: all five required local commands PASS; 28 shared factory-shortcut, four Org-context and 11 configuration keymap tests. Four focused native Agenda controls and documentation validation PASS. All five required commands and canonical PR validation passed at clean HEAD `9ca41e679978354e095bd09bfe9983aa735e8d5e` before integration.

- [x] T5.1: Apply accepted P08-P13: daily d, tags v M, search v /, show-all habits v A, graph window v G and sort v S; remove their displaced single keys through the existing native owners. Preserve named access for displaced habit/view actions, current prompt ownership and all unselected defaults.
- [x] T5.2: Extend the factory-shortcut suite with selected-row target identity, daily view versus habit-date behavior, each approved family relocation, named and overridden old access, cancellation, source immutability for view-only actions and protection against suffix keys granting consent.
- [x] T5.3: Reconcile the selected Agenda reference/specification rows and task/agenda guide text; record concise user-visible changes and the slice report. Do not add missing reference features to occupy keys.

Completed local checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 6: Git defaults

State: delivered; PR #52 integrated into dev at `37d98aa6e3a8e091f56c69262fab4dce87e15918`
Requirements: FSA02, FSA03, FSA05, FSA07-FSA09; accepted D02 manifest rows.
Dependency: Slice 5 integrated; a meaningful Git increment and its exact menu/direct-action boundary accepted at mapping freeze.
Branch: `feat/rev-factory-shortcuts-06-git`
Expected PR title: `feat(git): align reviewed shortcut families`
PR: `#52` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/52)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-6-git.md`
Task commits: T6.1 `3a1e010e9e61c4aa432824879472bdd446636256`; T6.2 `d0f8c3abd04b0300c0999a6d5f5955dbc7372293`; T6.3/report introduction `465c097161a86c5919c49806d1b3fc1ab7e5f287`; this record includes canonical PR references.
Validation: all four required local commands PASS; 34 shared factory-shortcut and 11 configuration keymap tests. Documentation validation PASS. All four required commands and canonical PR validation passed at clean HEAD `37d98aa6e3a8e091f56c69262fab4dce87e15918` before integration.

- [x] T6.1: Apply accepted P14-P19 and precise History amendment: History A cherry-pick; remove History P/F/f; add History c a amend, c f fixup and + paging; add Refs f/P for direct fetch/push, retaining existing Refs aliases. Update History panel guidance with the changed routes. Keep menu-versus-direct-action behavior explicit, operation-specific review/confirmation intact, selection/range identities stable and displaced native actions reachable through the accepted routes.
- [x] T6.2: Add owned local-repository factory-shortcut regressions for every changed Git family, including A/c a/c f/+ History routing and selected identities, Refs f/P aliases, removed P/F/f leaves, complete/cancelled prefixes, native review, explicit old overrides and absence of unintended mutation. Do not use credentials, external remotes or broad transport journeys to establish key routing.
- [x] T6.3: Reconcile the exact superseded Binding review matrix rows, current binding reference and Git guide, preserving historical validation. Record delivered behavior and the canonical slice report; do not present intentional retained exceptions as missing implementations.

Completed local checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`. Add a narrow existing Git regression only if the accepted implementation changes its owner beyond input routing; record that selection before execution.

## Slice 7: Terminal acceptance and guidance

State: delivered; PR #53 integrated into dev at `979ba5ebb64310d0a1d1e3801d53689fdf525cec`
Requirements: FSA01-FSA10 together; accepted manifest and retained exceptions.
Dependency: all accepted runtime slices integrated; manifest identity unchanged.
Branch: `test/rev-factory-shortcuts-07-acceptance`
Expected PR title: `test(input): verify factory shortcut compatibility`
PR: `#53` ([canonical PR](https://forge.adrianpk.com/rev/rev/pulls/53)); merged into dev
Report: `ops/default/report/slices/rev-factory-shortcuts/slice-7-acceptance.md`
Task commits: T7.1 `d2c27b13a53b2ec822b23436e79534e1600da09e`; T7.2 `601a67ec41b5b05fec312c78fa73d11842b9ef85`; T7.3 reference/guide consistency, Spore projection and report introduction `6f38eeb9417fb7719952acd0700dd11e941366da`; post-gate documentation and ticket closure completed on dev.
Validation: all five required local commands PASS; 89 selected command tests, 11 configuration keymap tests, six modal keymap tests, independent complete-map comparison, four native hint variants and three Linux PTYs. Documentation and Spore projection PASS. All five required commands and canonical PR validation passed at clean HEAD `979ba5ebb64310d0a1d1e3801d53689fdf525cec` after PR references, before integration. The slice receipt took 286.73 seconds. The later controller full gate passed at `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`; its separate evidence is recorded below.

- [x] T7.1: Add a narrow `scripts/check-factory-shortcuts.sh` entrypoint and `examples/factory-shortcuts-acceptance.rs`, reusing existing owned native/surface/PTY infrastructure. Cross-check inventory coverage, accepted normalized routes, actual effects, overrides, profile/focus transitions and selector hints. Compare executable results with the separately accepted mapping contract; do not rewrite expectations from current defaults.
- [x] T7.2: Exercise bounded physical-input journeys in nonmodal and modal editing plus one local read-only Git journey. Cover representative Meta/control and multi-chord routes, prefix cancellation, editing/profile/language transitions and terminal restoration. Keep variants native, use finite observable conditions and clean only owned fixtures/processes. Verify expected source/history effects rather than treating a successful exit as compatibility.
- [x] T7.3: Complete reference/guide consistency and Spore projection; produce the final slice report and prepare the exact integrated candidate for the selected delivery-set gate. Close the originating ticket and delivery metadata only after that gate passes. Record measured check costs and remove integrated inactive clean worktrees.

Completed local checks: `make lint`; `bash scripts/check-factory-shortcuts.sh`; `bash scripts/check-documentation.sh`; `bash scripts/build-user-guide.sh`; `git diff --check`. T7.3 reference/guide, report, cost, worktree, originating-ticket and delivery-metadata closure is complete after the successful controller-owned integrated gate below.

## Decision log

| Decision | Scope | State |
| --- | --- | --- |
| D01 | P08-P13 daily `d` and native `v M`, `v /`, `v A`, `v G`, `v S`; names and explicit old overrides retained. | accepted; exact manifest frozen |
| D02 | P14-P19 History A/c a/c f/+, remove P/F/f; Refs f/P aliases, current direct-action scope and exact legacy supersessions. | accepted; exact manifest frozen |
| D03 | All 707 rows and six actions reviewed; source Save/heading gaps, indirect routes and deliberate native conventions recorded. | accepted as current findings; requested core-first functional follow-through |

Do not mark a decision settled from successful parsing, a same-name match or
the existence of a current factory key. Record the reviewed operation contract
and product acceptance in the mapping artifact and owning specification.

## Delivery-set gate

State: PASS; controller-owned execution at immutable candidate `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e`.
Environment: one Cargo job, incremental disabled, dev/test debuginfo disabled.

Preparation: canonical PRs #47-#53 are merged into dev and all seven reports
are delivered. Required executables and Rust formatter/Clippy are available;
the five shell entrypoints pass `bash -n`. Locked offline Cargo metadata
confirms two workspace packages and the primary checkout's own target. Frozen
catalog/manifest identities and all selected gate commands remain unchanged.
The integrated Slice 7 worktree was clean, unlocked and inactive, then removed
without force; its branch and unrelated worktrees are preserved. Existing hooks
remain unchanged. No full-gate batch was executed during preparation.

Executed canonical order:

1. `make lint`
2. `make test-hooks`
3. `bash scripts/check-factory-shortcuts.sh`
4. `bash scripts/check-documentation.sh`
5. `bash scripts/build-user-guide.sh`
6. `git diff --check`

Exact successful controller command:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash -c 'set -euo pipefail; make lint; make test-hooks; bash scripts/check-factory-shortcuts.sh; bash scripts/check-documentation.sh; bash scripts/build-user-guide.sh; git diff --check'`: PASS.

Evidence log: `72b8d40d816ee1cf2eb1aec333658fcb6a9c503e.KNZHGNH5JIZCTDDOTFIVDFJ2PT.log`.
Log SHA-256: `5e2d7927bb242e0a5bdc94a2828e92acc88d41c5594ae6074e5b335c1e7a4d2b`.
The controller receipt is keyed by the tested candidate above. The later
closure commit changes only configured documentation paths and does not
replace or relabel that candidate's evidence.

Results: full-workspace lint and real hook acceptance PASS; 89 selected command
tests, 11 configuration keymap tests and six modal keymap tests PASS; independent
707-command / six-action / 28-map comparison, four native hint variants and
three bounded Linux PTYs PASS. Documentation covers 110 pages, 780 local
links/anchors, 707 names, 322 configuration entries and 10/12 reference tests.
Spore projects 111 Markdown pages and completes strict checks, rendering and
download finalization. Owned fixtures completed their cleanup; no process stop
was required during closure.

T7.1 implements the factory-shortcut script with the native/input/profile
selection defined by the plan and three representative PTYs. The accepted
command list and resource bounds remain unchanged. Historical feature gates
are not added implicitly. Failed checks retain their actual candidate and result.

## Closure

- [x] Every native operation/action has a disposition and accepted mapping changes match the frozen contract.
- [x] Every accepted slice has its task commits, integrated PR and canonical report with actual validation.
- [x] The selected integrated gate passes against the exact dev candidate.
- [x] Binding/configuration references, guide projection and originating ticket reflect delivered behavior.
- [x] Integrated inactive clean worktrees are removed non-forcibly; unrelated or active work remains untouched.
