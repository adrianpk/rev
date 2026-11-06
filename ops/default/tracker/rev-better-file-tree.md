# Better File Tree Tracker

Status: Approved
Date: 2026-11-06
Delivery set: rev-better-file-tree
Approved: product owner; specification/model, preparation commits, three-slice plan, push and one autonomous Looper delivery through dev
Plan: [Better File Tree Delivery](../plan/rev-better-file-tree.md)
Specification: [Better File Tree](../spec/editor-better-file-tree.md)
Model: [Better File Tree Model](../spec/editor-better-file-tree-model.md)
Concern: [Pending Work Index](../README.md)
Active slice: Slice 3, Acceptance and guidance
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Decorated presentation | `feat/rev-better-file-tree-01-decoration` | `feat(tree): add decorated and minimal presentations` | `ops/default/report/slices/rev-better-file-tree/slice-1-decoration.md` |
| Slice 2 | Font icons and geometry | `feat/rev-better-file-tree-02-icons` | `feat(tree): add fixed Nerd Fonts icons and cell-safe labels` | `ops/default/report/slices/rev-better-file-tree/slice-2-icons.md` |
| Slice 3 | Acceptance and guidance | `test/rev-better-file-tree-03-acceptance` | `test(tree): verify decorated presentation and document modes` | `ops/default/report/slices/rev-better-file-tree/slice-3-acceptance.md` |

## Slice 1: Decorated presentation

State: delivered
Dependency: approved committed preparation on clean dev
Requirements: BFT01, BFT03 and ordinary-symbol BFT02/BFT04
Implementation: T1.1 `0adbdac90ae076d3fb0d3ad3dff6b43c1fff2270`; T1.2 `a95c18e140eb6b0d78ac2e269af1c39accd3c28a`; T1.3 `8e42af2708051a5371d11cae49320565caa9c906`
Integrated: `219b8de4e1fd0957cccc0fe8fd9b5994b6063ed8`
PR: [#287](https://forge.adrianpk.com/adrianpk/rev/pulls/287)
Report: `ops/default/report/slices/rev-better-file-tree/slice-1-decoration.md`
Validation: five ordered commands passed locally; 271 tests plus two isolated NO_COLOR child checks. Looper validate passed all five commands on clean final HEAD `219b8de4e1fd0957cccc0fe8fd9b5994b6063ed8`; canonical PR #287 merged at that commit.

- [x] T1.1: Deliver the startup boolean and entry palette with typed composition,
  provenance, defaults, sample and existing background/selection compatibility.
- [x] T1.2: Deliver ordinary-symbol decoration and compiled file-kind inference;
  preserve minimal labels, cached observations and all tree interactions.
- [x] T1.3: Verify configuration/defaults/classification/colors and minimal mode;
  update the current reference/sample/catalog and one Unreleased product outcome.

## Slice 2: Font icons and geometry

State: delivered
Dependency: Slice 1 integrated
Requirements: BFT01-BFT04 and configuration model
Implementation: T2.1 `bc83d6a729972d9e079e200926ce720dfde5f41c`; T2.2/native T2.3 `b1cdf00fa429dad260fb1550f7590944e7d4660e`; T2.3 guidance `ac3dee145daf7bebb7dd1a21c6148997fca3b864`
Integrated: `e86542d3fde474cad006a69aeb22ee873c062355`
PR: [#288](https://forge.adrianpk.com/adrianpk/rev/pulls/288)
Report: `ops/default/report/slices/rev-better-file-tree/slice-2-icons.md`
Validation: five required commands passed in order; 235 selected tests and two isolated NO_COLOR child checks. Looper validate passed all five commands on clean final HEAD `e86542d3fde474cad006a69aeb22ee873c062355`; canonical PR #288 merged at that commit.

- [x] T2.1: Reuse `gutter.nerd_fonts` for fixed icons with one checked prefix
  geometry, unchanged nesting and complete directory/diagnostic distinctions.
- [x] T2.2: Preserve canonical full-row hits, clipping, current focused
  selection, color precedence and NO_COLOR behavior under both presentations.
- [x] T2.3: Verify the four combinations, Unicode/narrow geometry, input,
  scroll/resize and shared gutter behavior; update reference and Unreleased.

## Slice 3: Acceptance and guidance

State: reviewing
Dependency: Slice 2 integrated
Requirements: BFT01-BFT04 together
Implementation: T3.1 `cc2ae9dfc614a8e7956aef79c96ed542a3d093ae`; T3.2/T3.3 handoff `33411913f2e5d77191f61f27068b9d80ecbf8c39`
PR: [#289](https://forge.adrianpk.com/adrianpk/rev/pulls/289)
Report: `ops/default/report/slices/rev-better-file-tree/slice-3-acceptance.md`
Validation: six required commands passed locally in order; 231 tests, bounded native/NO_COLOR acceptance, two restored PTYs and local 85-page Spore build passed. Final clean-HEAD evidence is recorded separately by Looper after PR references are committed.

- [x] T3.1: Deliver bounded Rust/Bash native matrix and one owned PTY journey
  per interaction profile using existing helpers and explicit cleanup checks.
- [x] T3.2: Complete Diataxis and Markdown-derived Spore guidance; reconcile
  directly affected descriptions without replacing historical delivery evidence.
- [ ] T3.3: Record real reports and immutable commit/PR evidence, prepare the
  clean candidate for the controller gate and close only after that gate passes.

The worker handoff portion of T3.3 is prepared. Integration, the frozen gate and
final closure remain controller-owned and pending.

## Final gate and closure

The plan's Final gate section and local controller configuration own the exact
ordered commands and environment. This tracker records no implementation,
validation, performance or publication result at planning.

- [ ] Verify all three canonical PRs integrated and all reports delivered.
- [ ] Pass the frozen final gate on the exact clean integrated candidate.
- [ ] Record the receipt, commands, immutable report introductions, observed
  timing, cleanup and font-rendering evidence limits.
- [ ] Complete the plan/tracker/specification/model and pending-work index.
- [ ] Preserve separate selection-synchronization and main/release boundaries.

Any technical recovery or final-gate repair remains within the plan's bounded
delegation and the same retained controller, conversation and execution budget.
