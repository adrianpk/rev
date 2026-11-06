# Better Gutter Tracker

Status: In progress
Date: 2026-11-06
Delivery set: rev-better-gutter
Approved: product owner; specification, three-slice preparation, push and one autonomous Looper delivery through dev
Plan: [Better Gutter Delivery](../plan/rev-better-gutter.md)
Specification: [Better Gutter](../spec/editor-better-gutter.md)
Model: [Better Gutter Model](../spec/editor-better-gutter-model.md)
Active slice: None; all three slices and repair 1 integrated, controller-owned final gate pending
Progress owner: retained Looper checkpoint and canonical validation receipts

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Outside marker layout | `feat/rev-better-gutter-01-layout` | `feat(gutter): place bookmark markers before line numbers` | `ops/default/report/slices/rev-better-gutter/slice-1-layout.md` |
| Slice 2 | Fixed symbols and font flag | `feat/rev-better-gutter-02-markers` | `feat(gutter): select built-in bookmark symbols with a font flag` | `ops/default/report/slices/rev-better-gutter/slice-2-markers.md` |
| Slice 3 | Joint acceptance and guidance | `test/rev-better-gutter-03-acceptance` | `test(gutter): verify marker geometry and document presentation` | `ops/default/report/slices/rev-better-gutter/slice-3-acceptance.md` |

## Slice 1: Outside marker layout

State: delivered
Dependency: approved committed preparation on clean dev
Requirements: BMK03, BTG02, BTG04
Implementation: `f4698fc7b506a610cbbef6aa12f6e5bd459be019` (T1.1-T1.3)
PR: `#282`
Report: `ops/default/report/slices/rev-better-gutter/slice-1-layout.md`
Integrated: `8af7f4e02e77e31ca286d82b6651791b53e3eec8`
Validation: Looper receipt `LE6UT6QCD7YDRHULYWKTSCXVIZ` passed the four required
commands on that clean PR-reference HEAD, including 223 approved suite tests.
Focused completion and documentation checks also passed locally.

- [x] T1.1: Allocate the marker outside numbers; preserve mirrored placement,
  narrow-pane safety, visibility and zero folding-column allocation.
- [x] T1.2: Share the checked geometry across paint, input, wrapping and overlays
  while retaining prepared bookmark observations and canonical source identity.
- [x] T1.3: Verify affected native geometry, source fidelity and presentation;
  update the layout reference and one reader-facing Unreleased outcome.

## Slice 2: Fixed symbols and font flag

State: delivered
Dependency: Slice 1 integrated
Requirements: BMK03, BTG01, BTG04 and configuration model
Implementation: `c59fea87a73273887736a3a362f0cd19db18875f` (T2.1-T2.3)
PR: `#283`
Report: `ops/default/report/slices/rev-better-gutter/slice-2-markers.md`
Integrated: `d35f78b3fbb39c501f10e36ac91cf13736a75429`
Validation: Looper receipt `3MSR4YDCLRLALDOSOS6NTQE22V` passed all five required
commands on that clean PR-reference HEAD, including all-target Clippy,
250 suite tests and documentation catalogs.

- [x] T2.1: Deliver the startup boolean and its fixed mapping, with typed merge,
  provenance and ignored legacy marker-string compatibility.
- [x] T2.2: Apply the compiled square/bookmark and blue role through existing
  owners; retain palette overrides and view/source lifetimes.
- [x] T2.3: Verify defaults, both choices, malformed input and old configuration;
  update the complete sample, catalog, migration guidance and Unreleased outcome.

## Slice 3: Joint acceptance and guidance

State: delivered
Dependency: Slice 2 integrated
Requirements: BMK03, BTG01-BTG04 together
Implementation: `0d1219ed67b779c65d75fddf2d8f0672ac9a8778` (T3.1),
`da090457b897ffb0eedd3bc97691baab3aede4f9` (T3.2)
PR: `#284`
Report: `ops/default/report/slices/rev-better-gutter/slice-3-acceptance.md`
Report introduction: `cafc0f0a78197de885221e0581b311f067b21835` (T3.3 reporting)
Integrated: `09b88ac85eee353dfb9649d43f0e696bb02d7fa9`
Validation: Looper receipt `KZYA354UGVNE62JZKDAKX5QPLZ` passed all six required
commands against that clean PR-reference HEAD, including all-target Clippy,
94 suite tests and documentation catalogs. Focused native acceptance, two bounded
profile PTYs and the Markdown-derived Spore build also passed locally.

- [x] T3.1: Deliver the bounded native Rust/Bash acceptance entrypoint and one
  representative owned PTY journey for each interaction profile.
- [x] T3.2: Complete Diataxis guidance and Markdown-derived Spore content;
  reconcile affected baseline contracts without replacing historical evidence.
- [ ] T3.3: Record real reports and task/commit/PR evidence, hand off the clean
  integrated candidate for the frozen final gate and close only after it passes.

All three reports record delivered implementation and slice evidence.
Slice handoff and traceability are complete; T3.3 remains open only for the
controller-owned integrated final gate and delivery-set closure.

## Final-gate repair 1

State: delivered
Branch: `fix/rev-better-gutter-validation-1`
Expected PR title: `fix(gutter): correct better gutter acceptance`
PR: `#285`
Integrated: `5473ef7c28b2ff978821faa37e0c9c9e3518c8e8`
Report: `ops/default/report/slices/rev-better-gutter/slice-3-acceptance.md`
Failed candidate: `10054d1fece9557d37923fa8be0d825727232ac7`
Failed validation: `VIR36T3PHGW77O2D4MNMV64DGX`
Diagnosis: `org_indentation::gutter_and_bookmark_identity` retained two old
factory-dot expectations. Updated both visible-marker and folded-marker-absence
assertions to the approved square while retaining the geometry/source checks.
Implementation: `4f5d6da28d93d780d8b4a905f5d53100424f2203`
Validation: Looper receipt `S4UHXCYEFUXAHMB6ASP7PIB6PS` passed all four required
commands against that clean PR-reference HEAD, including all-target Clippy and
250 suite tests. The focused Org indentation regression also passed all 18 tests.
Canonical PR #285 is merged; the unchanged full-gate retry remains controller-owned.

## Final gate and closure

The exact ordered final commands and Cargo environment are defined in the
plan's Final gate section and copied into the local controller configuration.
No gate, acceptance, publication or performance result is claimed at planning.

Gate prerequisites are available: Rust/Cargo with formatting and Clippy, Bash,
the existing PTY utilities and Spore. Locked offline Cargo metadata recognizes
all approved gate targets; both gate scripts passed Bash syntax validation.
The acceptance entrypoint and reused helpers are tracked. These prerequisite
checks do not execute the full gate.

- [x] Verify the three canonical PRs integrated and three reports delivered.
- [ ] Pass the frozen focused final gate on the exact clean candidate.
- [ ] Record the validation receipt, exact commands, immutable report commits,
  actual timing, terminal cleanup and evidence limits.
- [ ] Reconcile completed plan/tracker/specification/model and pending index.
- [ ] Preserve future folding and debugger feature boundaries.

Any retained technical recovery or final-gate repair stays within the plan's
delegation and the controller's existing ownership, identity and budgets.
Main promotion, release, installation and mirrors remain separately requested.
