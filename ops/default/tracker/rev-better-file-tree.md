# Better File Tree Tracker

Status: Approved
Date: 2026-11-06
Delivery set: rev-better-file-tree
Approved: product owner; specification/model, preparation commits, three-slice plan, push and one autonomous Looper delivery through dev
Plan: [Better File Tree Delivery](../plan/rev-better-file-tree.md)
Specification: [Better File Tree](../spec/editor-better-file-tree.md)
Model: [Better File Tree Model](../spec/editor-better-file-tree-model.md)
Concern: [Pending Work Index](../README.md)
Active slice: Slice 1, Decorated presentation
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Decorated presentation | `feat/rev-better-file-tree-01-decoration` | `feat(tree): add decorated and minimal presentations` | `ops/default/report/slices/rev-better-file-tree/slice-1-decoration.md` |
| Slice 2 | Font icons and geometry | `feat/rev-better-file-tree-02-icons` | `feat(tree): add fixed Nerd Fonts icons and cell-safe labels` | `ops/default/report/slices/rev-better-file-tree/slice-2-icons.md` |
| Slice 3 | Acceptance and guidance | `test/rev-better-file-tree-03-acceptance` | `test(tree): verify decorated presentation and document modes` | `ops/default/report/slices/rev-better-file-tree/slice-3-acceptance.md` |

## Slice 1: Decorated presentation

State: ready
Dependency: approved committed preparation on clean dev
Requirements: BFT01, BFT03 and ordinary-symbol BFT02/BFT04
Implementation: pending
PR: pending
Report: `ops/default/report/slices/rev-better-file-tree/slice-1-decoration.md`
Validation: pending; exact ordered commands are in the plan and local configuration

- [ ] T1.1: Deliver the startup boolean and entry palette with typed composition,
  provenance, defaults, sample and existing background/selection compatibility.
- [ ] T1.2: Deliver ordinary-symbol decoration and compiled file-kind inference;
  preserve minimal labels, cached observations and all tree interactions.
- [ ] T1.3: Verify configuration/defaults/classification/colors and minimal mode;
  update the current reference/sample/catalog and one Unreleased product outcome.

## Slice 2: Font icons and geometry

State: pending
Dependency: Slice 1 integrated
Requirements: BFT01-BFT04 and configuration model
Implementation: pending
PR: pending
Report: `ops/default/report/slices/rev-better-file-tree/slice-2-icons.md`
Validation: pending; exact ordered commands are in the plan and local configuration

- [ ] T2.1: Reuse `gutter.nerd_fonts` for fixed icons with one checked prefix
  geometry, unchanged nesting and complete directory/diagnostic distinctions.
- [ ] T2.2: Preserve canonical full-row hits, clipping, current focused
  selection, color precedence and NO_COLOR behavior under both presentations.
- [ ] T2.3: Verify the four combinations, Unicode/narrow geometry, input,
  scroll/resize and shared gutter behavior; update reference and Unreleased.

## Slice 3: Acceptance and guidance

State: pending
Dependency: Slice 2 integrated
Requirements: BFT01-BFT04 together
Implementation: pending
PR: pending
Report: `ops/default/report/slices/rev-better-file-tree/slice-3-acceptance.md`
Validation: pending; exact ordered commands are in the plan and local configuration

- [ ] T3.1: Deliver bounded Rust/Bash native matrix and one owned PTY journey
  per interaction profile using existing helpers and explicit cleanup checks.
- [ ] T3.2: Complete Diataxis and Markdown-derived Spore guidance; reconcile
  directly affected descriptions without replacing historical delivery evidence.
- [ ] T3.3: Record real reports and immutable commit/PR evidence, prepare the
  clean candidate for the controller gate and close only after that gate passes.

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
