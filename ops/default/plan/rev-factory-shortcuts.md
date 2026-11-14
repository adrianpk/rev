# Factory Shortcut Alignment Delivery

Status: Approved for audit
Date: 2026-11-14
Delivery set: rev-factory-shortcuts
Slice strategy: layered
Planning authority: product-owner request for slicing, 2026-11-14
Approved: product owner, 2026-11-14; prepare, commit and push the delivery records, execute audit slices 1-2 through autonomous Looper PR integration into dev, and retain the explicit mapping decision checkpoint before runtime changes
Execution authority: audit slices 1-2; runtime slices remain gated by accepted mapping decisions
Planning baseline: `197bf98be615ceca06a3bab0ef5cd24b3bee2ffb`
Spec: [Factory Shortcut Alignment](../spec/editor-factory-shortcut-alignment.md)
Tracker: [Factory Shortcut Alignment Tracker](../tracker/rev-factory-shortcuts.md)
Active slice: Slice 1, Core and profile audit
Readiness: audit execution approved; mapping freeze and runtime acceptance remain pending
Integration target: `dev`

## Outcome and strategy

Align existing native operations with familiar shortcut families according to
their effects and contexts. Preserve compiled defaults, explicit user overrides,
canonical command identities and current focus/profile ownership.

The layered strategy puts two reviewable contract increments before four
behavior increments and terminal acceptance. This ordering is necessary because
the complete catalog inventory is available but the semantic review and
Agenda/Git decisions are incomplete. A name-only comparison is insufficient to
freeze a safe replacement map.

The initial P01-P08 proposals are not the full review boundary. Every command
and configurable input action must receive an evidence-backed disposition.
Additional assignments require explicit specification and manifest entries.
Missing command behavior is recorded separately; it is not implemented through
an approximate binding or an undeclared adapter.

## Readiness and decision boundaries

1. **Preparation:** review and commit the specification's audit scope, this
   plan and the matching tracker before any formal slice branch is created.
   The continuation approves preparation publication and audit execution.
   Runtime approval remains subject to the mapping checkpoint below.
2. **Contract slices:** Slice 1 and Slice 2 may run only under approved audit
   authority. They change review artifacts, not factory defaults. Record any
   remaining product decision and return its concrete alternatives for review.
3. **Mapping freeze:** before closing Slice 2 for implementation, settle spec
   decisions D01-D03 and obtain acceptance of the final specification and
   before/after mapping contract. Reconcile exact runtime tasks, slice count,
   PR titles, validation selection and expected cost, then commit the matching
   plan/tracker. Do not continue into Slice 3 with placeholders.
4. **Runtime delivery:** Slice 3 onward uses only that accepted manifest. A
   material new mapping or behavior requires an explicit plan/spec revision.
   An execution owner must not spend retries trying to resolve a product choice.
5. **Integration:** one branch and one PR to `dev` per approved slice. Each
   slice starts only after the preceding slice is integrated and closed.
   The approved continuation delegates controller-owned Rebase + Fast-forward
   integration of the two audit PRs into dev. It does not approve unresolved
   runtime mappings or delegate product decisions.

Slice 6 is provisional until D02 is settled. If the accepted Git contract has
no meaningful implementation or regression increment, remove that slice and
renumber the remaining map before approval; do not create an empty feature PR.
The proposed count is not a requirement to manufacture seven changes.

## Prerequisites and inspection outputs

| Prerequisite | Current state | Entry condition |
| --- | --- | --- |
| Native catalog and 28 context maps | Available at the inspection baseline | Re-extract at the actual review candidate and account for catalog changes. |
| Compiled defaults, overrides, selector hints and native operations | Existing implementation | Preserve their owning contracts; assess actual dispatch rather than descriptions alone. |
| Core and profile semantic coverage | Partial | Complete Slice 1 matrix rows and cross-name/indirect evidence. |
| Org, Markdown, Agenda and Git coverage | Partial | Complete Slice 2, including D01-D03 and approved exceptions. |
| Specification and final mapping manifest | Draft / not yet produced | Accept before changing runtime defaults. |
| Plan, tracker and controller authority | Slicing proposed | Commit approved preparation and match the executing owner to that scope. |

Slice 1 begins `ops/default/report/rev-factory-shortcuts/semantic-matrix.md`.
Slice 2 completes it and produces
`ops/default/report/rev-factory-shortcuts/mapping-contract.md`.
These are planned review artifacts, not completed evidence or editor data.
The catalog stores the native commands and context bindings; every command
gets a matrix row beginning with its backticked canonical name. Preserve
coverage identifiers even when aliases share an operation. No local paths or
personal bindings belong in the exported catalog.
Record reference versions, source owners, operation identity, point context,
profile, current direct/indirect route, proposed route, differences, validation
case and keep/add/relocate/indirect/named-only/adapt/unresolved disposition.

The mapping contract records every accepted addition, removal and alias by
normalized sequence and context/profile, displaced command access, deliberate
exceptions and affected legacy specification rows. Its identity is frozen at
mapping approval; subsequent implementation must not edit it to fit observed
output without an approved correction.

## Controller decision checkpoint

Use one registered run with the seven recorded identities and frozen commands.
The initial execution grant covers Slice 1 and Slice 2 audit work only. Before
submitting the Slice 2 PR, publish the complete proposal in a clean committed
slice branch and return `recovery_requested` with recovery kind `decision`.
Include all worker-schema fields, zero incident/action IDs, an empty probe and
validation ID, the observed candidate head, and the exact proposal paths.
Do not return `pr_ready`, mark mapping approval complete or enter Slice 3 until
the owner accepts D01-D03 and the exact proposal through the controller's
ordinary decision mechanism. Keep the same run, conversation and budgets.

The decision payload must summarize the proposed Agenda/Git placements,
intentional exceptions and any required adjustment to later slice scope. If
acceptance would change a frozen slice identity or gate command, return that
configuration issue for explicit resolution; never rewrite an active config.
Controller defaults remain 200 worker turns and five stalled turns. No extra
recovery, repair, budget or product-decision grant is implied.

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

## Slice 1: Core and profile audit

Requirements: FSA01, FSA02, FSA03, FSA07, FSA08; core portion of D03.
Dependency: approved and committed audit preparation; no runtime authorization is required by this slice's scope.

- T1.1: Re-extract the complete public command/action inventory and context maps at the actual candidate. Persist sanitized catalog.json beside the semantic matrix and record explicit coverage ownership; distinguish duplicate inherited entries, aliases and indirect dispatcher access. Preserve reproducible inspection commands and version evidence without personal configuration or machine identifiers.
- T1.2: Review core editing, navigation, selection, arguments/history, panes, projects, prompts/completion, tree/bookmarks, macros, terminal controls and other native command families, including six modal states. Compare operations under different names and reject misleading approximations. Record each operation's disposition and references; mark mode-owned rows pending Slice 2 rather than claiming global completion.
- T1.3: Reproduce high-value effects and boundaries for P01-P04 and related primitives with owned native/reference probes. Verify first-text placement, join direction, paragraph/delimiter differences, profile separation, protected keys and terminal aliases. Produce the slice report; retain factory and personal configuration bytes unchanged.

Checks: coverage reconciliation against the live catalog; source/reference evidence review; owned behavioral probes with exact commands and limits; local-link/identifier checks; `git diff --check`. No broad Cargo or historical feature gate is required for documentation-only audit outputs.

## Slice 2: Mode mapping contract

Requirements: FSA01, FSA02, FSA05, FSA06, FSA07, FSA09; D01-D03.
Dependency: Slice 1 integrated; mode reference sources and native fixtures available.

- T2.1: Complete semantic review of Org headings/lists/tables/source blocks, Agenda/capture/calendar/source editing, Markdown plain/list/table dispatch and every native Git panel/menu. Compare active contexts and effects, including source targets behind projections; finish all pending matrix rows and evidence-backed named-only or adaptation dispositions.
- T2.2: Present and settle D01 Agenda placements and D02 Git family decisions, including P06-P08 and displaced command access. Produce the exact before/after mapping contract, preserve explicit tree/Markdown conventions and identify each selected supersession of legacy tables. Keep unresolved product choices visible and obtain their acceptance before closing the mapping freeze.
- T2.3: Reconcile the specification, mapping contract, remaining slice tasks, coverage/check selection and expected cost. Remove an empty provisional Git slice if appropriate through a reviewed map revision. Record approval evidence and the slice report; keep runtime defaults unchanged. Runtime slices remain ineligible until accepted artifacts are committed.

Checks: complete catalog/context coverage; no unresolved entry in the accepted implementation manifest; normalized sequence/prefix/conflict probes for both profiles; baseline/reference and document consistency; `git diff --check`. Do not label probes as terminal or full behavioral acceptance.

## Slice 3: Editor defaults

Requirements: FSA03, FSA04, FSA07-FSA09; P01-P04 and accepted core manifest rows.
Dependency: Slice 2 integrated and mapping freeze accepted/committed.

- T3.1: Apply only accepted editor/profile factory assignments for first text, pane balance, redo and registered Project switching. Preserve canonical handlers, context transitions, modal state and existing search/operator keys. Keep factories and the exhaustive configuration reference consistent.
- T3.2: Extend existing command/keymap tests with a cohesive `factory_shortcuts` module for actual input-to-effect regressions. Cover Unicode/blank lines, selection/count policy, redo groups, pane focus, cancelled Project restoration, overrides, local overlays and all affected modal states; avoid a second handwritten production binding catalog.
- T3.3: Update the affected binding reference and core User Guide guidance; use the changelog-normalize skill for one concise user-facing outcome when behavior is usable. Run the core slice checks and record the canonical report and task evidence.

Checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `cargo test --locked --test modal keymaps -- --test-threads=1`; `git diff --check`.

## Slice 4: Org document defaults

Requirements: FSA03-FSA09; P05-P07 and accepted Org document manifest rows.
Dependency: Slice 3 integrated; Org replacement rows accepted.

- T4.1: Apply accepted effort, property-column and Babel header-argument factory assignments through native owners. Retain named clock reports, explicit old user assignments and unaffected aliases. Preserve unsupported-help and radio-button boundaries rather than reusing their familiar chords.
- T4.2: Extend the shared factory-shortcut regressions with Org source-context effects: effort choices/errors, read-only column projection without a clocktable edit, header insertion/cancellation and undo, inherited profile maps and retained explicit overrides. Guard existing list/heading/table dispatch and partial M-S-RET equivalence.
- T4.3: Reconcile only selected Org contract rows, binding/configuration references and relevant guide text. Preserve historical delivery evidence, use a concise changelog outcome for the delivered behavior and produce the slice report.

Checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 5: Agenda defaults

Requirements: FSA03, FSA05-FSA09; P08 and accepted D01 manifest rows.
Dependency: Slice 4 integrated; every implemented Agenda placement explicitly accepted.

- T5.1: Apply the accepted Agenda family assignments and daily-view alias through the selected projection's existing native source owner. Preserve named access for displaced habit/view actions, current prompt ownership and all unselected defaults.
- T5.2: Extend the factory-shortcut suite with selected-row target identity, daily view versus habit-date behavior, each approved family relocation, named and overridden old access, cancellation, source immutability for view-only actions and protection against suffix keys granting consent.
- T5.3: Reconcile the selected Agenda reference/specification rows and task/agenda guide text; record concise user-visible changes and the slice report. Do not add missing reference features to occupy keys.

Checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test commands org_contexts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`.

## Slice 6: Git defaults

Requirements: FSA02, FSA03, FSA05, FSA07-FSA09; accepted D02 manifest rows.
Dependency: Slice 5 integrated; a meaningful Git increment and its exact menu/direct-action boundary accepted at mapping freeze.

- T6.1: Apply only approved Git context/family replacements or aliases. Keep menu-versus-direct-action behavior explicit, operation-specific review/confirmation intact, selection/range identities stable and displaced native actions reachable through the accepted routes.
- T6.2: Add owned local-repository factory-shortcut regressions for every changed Git family, including history selection, complete prefixes, cancelled menus and commands, explicit overrides and absence of unintended mutation. Do not use credentials, external remotes or broad transport journeys to establish key routing.
- T6.3: Reconcile the exact superseded Binding review matrix rows, current binding reference and Git guide, preserving historical validation. Record delivered behavior and the canonical slice report; do not present intentional retained exceptions as missing implementations.

Checks: `make lint`; `cargo test --locked --test commands factory_shortcuts -- --test-threads=1`; `cargo test --locked --test configuration keymaps -- --test-threads=1`; `git diff --check`. Add a narrow existing Git regression only if the accepted implementation changes its owner beyond input routing; record that selection before execution.

## Slice 7: Terminal acceptance and guidance

Requirements: FSA01-FSA10 together; accepted manifest and retained exceptions.
Dependency: all accepted runtime slices integrated; manifest identity unchanged.

- T7.1: Add a narrow `scripts/check-factory-shortcuts.sh` entrypoint and `examples/factory-shortcuts-acceptance.rs`, reusing existing owned native/surface/PTY infrastructure. Cross-check inventory coverage, accepted normalized routes, actual effects, overrides, profile/focus transitions and selector hints. Compare executable results with the separately accepted mapping contract; do not rewrite expectations from current defaults.
- T7.2: Exercise bounded physical-input journeys in nonmodal and modal editing plus one local read-only Git journey. Cover representative Meta/control and multi-chord routes, prefix cancellation, editing/profile/language transitions and terminal restoration. Keep variants native, use finite observable conditions and clean only owned fixtures/processes. Verify expected source/history effects rather than treating a successful exit as compatibility.
- T7.3: Complete reference/guide consistency and Spore projection; produce the final slice report and prepare the exact integrated candidate for the selected delivery-set gate. Close the originating ticket and delivery metadata only after that gate passes. Record measured check costs and remove integrated inactive clean worktrees.

Checks: `make lint`; `bash scripts/check-factory-shortcuts.sh`; `bash scripts/check-documentation.sh`; `bash scripts/build-user-guide.sh`; `git diff --check`. This prepares the slice; the integrated candidate still requires the final gate below.

## Validation selection and cost

Use one Cargo build job, disabled incremental output and no dev/test debuginfo
for runtime checks. Set `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` in the check
environment. Each checkout owns its default Cargo target and hook cache.

The proposed final gate, run once against the exact integrated candidate, is:

1. `make lint`
2. `make test-hooks`
3. `bash scripts/check-factory-shortcuts.sh`
4. `bash scripts/check-documentation.sh`
5. `bash scripts/build-user-guide.sh`
6. `git diff --check`

The new focused script is planned output of T7.1; it is not available or passing
at planning time. Its contract batches the new `commands::factory_shortcuts`
cases, existing command keymap and Org-context routing cases, configuration
keymap cases and modal keymap cases through their native test roots. It then
builds the editor/acceptance example once and runs the native effect matrix and
three representative PTYs. Extend selected prefix-argument regressions only
where accepted keys affect argument dispatch. Reference/sample consistency is
also covered by the existing documentation check.

Protection is limited to changed defaults and shared input/override/discovery
contracts. Do not chain historical Git, Org, Markdown, modal, LSP or presentation
full gates or select the entire accumulated integration suite automatically.
No baseline gate is run as part of this planning request.

Compilation and all-target lint are the significant costs. Native variants
share a build/fixture batch; physical journeys are deliberately representative.
The warm acceptance work should fit within minutes rather than historical
hours-long gates, but no measured runtime is claimed yet. At mapping freeze,
review existing timings and record a concrete cost estimate for the final
selection. T7.3 records actual per-phase durations and cold/warm conditions;
timing targets do not authorize deleting coverage or weakening assertions.

On a failure, reproduce and verify the correction at the narrowest affected
owner. Follow the repository/controller exact-candidate repair contract; do not
edit frozen commands or historical receipts to make acceptance pass.

## Worktree hygiene and closure

Implement each accepted slice in its recorded branch and a timestamped task
worktree under the canonical project worktree root. Keep the primary checkout
on `dev`. Work sequentially, without shared build directories or parallel
copies of all seven target trees.

After integration, verify the slice HEAD is an ancestor of refreshed `dev`,
the worktree is clean/unlocked and no process owns its files or build outputs.
Remove that integrated worktree without force in the same task. Preserve dirty,
active or doubtful worktrees; do not stop another build or editor to remove one.

Keep one PR and one canonical report per accepted slice, with exact task/commit
and validation evidence. Apply the established Summary/Changes/Validation PR
structure and immutable report link. Delivery closure requires the accepted
manifest, complete matrix, current documentation and a passing gate for the
exact `dev` candidate. This draft claims none of those execution results.
