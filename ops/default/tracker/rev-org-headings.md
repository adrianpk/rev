# Org Heading Presentation Tracker

Status: Approved
Date: 2026-11-03
Delivery set: rev-org-headings
Slice strategy: behavior-first
Approved: product owner, 2026-11-03; approved heading-presentation scope and explicit autonomous Looper startup; the agent derives this three-slice map under that authority
Planning baseline: `2324b522009266c331e528293aa80b1414b8765d`
Spec: [Org Heading Presentation](../spec/editor-org-heading-presentation.md)
Model: [Heading Presentation Model](../spec/editor-org-heading-presentation-model.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --all-targets -- -D warnings && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_headings --test configuration --test commands --test display -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test org_editing visibility -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test modal contexts:: -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test external_changes -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --lib session::ai::results -- --test-threads=1 && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-org-headings.sh && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check && env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh && git diff --check`
Active slice: Slice 3 - Joint acceptance and guidance
Plan: [Heading Presentation Plan](../plan/rev-org-headings.md)
Execution owner: Looper; checkpoint, canonical PR checks and exact-candidate gate receipts own progress

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Configured native markers | `feat/rev-org-headings-01-presentation` | `feat(org): display configurable native heading markers` | `ops/default/report/slices/rev-org-headings/slice-1-presentation.md` |
| Slice 2 | Independent view controls | `feat/rev-org-headings-02-view-controls` | `feat(org): control heading presentation per view` | `ops/default/report/slices/rev-org-headings/slice-2-view-controls.md` |
| Slice 3 | Joint acceptance and guidance | `feat/rev-org-headings-03-acceptance` | `test(org): verify heading presentation and document controls` | `ops/default/report/slices/rev-org-headings/slice-3-acceptance.md` |

## Slice 1: Configured native markers

State: delivered; canonical PR #238 merged into dev at
`5e662a9691b7ab08c7c89dd37a1f7d3d1dce217a`.
Requirements: OGP01-OGP05, OGP07; native observation/projection part of OGP08.
Dependency: approved preparation on clean dev.
Branch: `feat/rev-org-headings-01-presentation`
Expected PR title: `feat(org): display configurable native heading markers`
Report: `ops/default/report/slices/rev-org-headings/slice-1-presentation.md`

- [x] T1.1: Deliver the four complete bounded startup preferences, samples,
  template, provenance and configuration references.
- [x] T1.2: Render current native heading prefixes with configured markers and
  unchanged source/cell/wrap geometry; preserve literal/partial source.
- [x] T1.3: Verify levels, overrides, invalid glyphs, native frames, disabled
  coloring and source/dirty/history fidelity in tests/org_headings.rs.
Task commits: T1.1 `ac4ff6648919f72b1da4bb0f6f5fee343289afc6`;
T1.2 `12d47edd7a9736888cec508567b336ceb11c8967`;
T1.3 `4d1630aca2929d2569412cbf03498352213f5e01`. Focused development checks passed: two
configuration, three display, seven heading and three Org projection tests.
The controller receipt for the final canonical PR head owns exact validation.

Canonical PR: [#238](https://forge.adrianpk.com/adrianpk/rev/pulls/238).
Report introduction: `4d1630aca2929d2569412cbf03498352213f5e01`.
Final validation: receipt `DT6X246MDOZQJMVKQPBPAAAEQF` passed all five
configured commands at the merged head: format, strict Clippy, 195 native tests,
documentation and whitespace. The passive-reveal correction preserved the two
existing geometry assertions and four native result/drawer cases. Slice 1 of 3
is delivered; Acta records controller-owned integration.

## Slice 2: Independent view controls

State: delivered; canonical PR #239 merged into dev at
`9f80318352921105f11b34226b1b39d85fba293b`.
Requirements: OGP02-OGP04, OGP06, OGP08.
Dependency: Slice 1 integrated.
Branch: `feat/rev-org-headings-02-view-controls`
Expected PR title: `feat(org): control heading presentation per view`
Report: `ops/default/report/slices/rev-org-headings/slice-2-view-controls.md`

- [x] T2.1: Deliver the shared native pretty toggle/reset with view-local
  override, existing selectors/keymaps and no new factory binding.
- [x] T2.2: Preserve split/rebind/reset/edit/reload behavior and exact source
  cursor, pointer, selection, wrap and gutter meanings.
- [x] T2.3: Verify both profiles, readonly Org results, source copy/undo, native
  AI acceptance, cancellation and independent view choices.
Task commits: T2.1 `85464c398fdf7916b766e9c1cff2ecdf299b3252`;
T2.2 `59f02ca119e7e94a38f61b84e7feaffed77fe1bb`.
T2.3 `4a4b3f623077f961c243c99957b57c9ea041ae86`.
Development checks passed: 16 heading, 11 visibility and 5 native result tests.

Canonical PR: [#239](https://forge.adrianpk.com/adrianpk/rev/pulls/239).
Report introduction: `4a4b3f623077f961c243c99957b57c9ea041ae86`.
Final validation: receipt `GRKJCN7LNUOYNQCGJHBRINYNOT` passed all six
configured commands at the merged head: format, strict Clippy, 255 native tests,
documentation and whitespace. The exact Org name count includes the two
approved commands. Slice 2 of 3 is delivered; Acta records integration.

## Slice 3: Joint acceptance and guidance

State: reviewing; canonical PR and final clean-head receipt pending.
Requirements: OGP01-OGP08 together.
Dependency: Slice 2 integrated.
Branch: `feat/rev-org-headings-03-acceptance`
Expected PR title: `test(org): verify heading presentation and document controls`
Report: `ops/default/report/slices/rev-org-headings/slice-3-acceptance.md`

- [x] T3.1: Deliver the bounded native/resource/real-PTY example and Bash driver;
  verify both profiles, widths, resize, source/pretty behavior and terminal exit.
- [x] T3.2: Update current Org Markdown guidance, generated Spore, catalogs and
  Unreleased without public deployment or font/package installation.
- [x] T3.3: Prepare integrated dev for the unchanged controller full gate and
  preserve its exact receipt and representative measurements.
- [ ] Record successful exact-HEAD validation, immutable report and canonical PR.

Task commits: T3.1 `3124e9da400b4c2e775f7666cdbf314b337e960c`;
T3.2 `26f182284dbd491d90832c65a895acf5a9920568`. Native/resource/four-PTY
development acceptance and the existing shared terminal kernel-wake case pass.
T3.3 prepares the report and local gate prerequisites; the unchanged integrated
gate and delivery closure remain pending controller-owned integration.

## Required validation prefixes

Use the exact ordered command lists under the plan's Required validation prefixes
and in the frozen local configuration. The four low-footprint Cargo environment
settings are part of the tested build policy. Slice tests do not replace the
controller-owned integrated gate.

## Final gate and closure

- [ ] Confirm all three canonical slices integrated in dev.
- [ ] Pass the unchanged configured full gate on its exact clean candidate.
- [ ] Record gate receipt, selected checks, actual build footprint and bounded
  native/PTY measurements in the existing reports.
- [ ] Mark all reports delivered, this tracker and plan Completed, and the paired
  heading specification/model Completed; reconcile the pending index.
- [ ] Commit/push only admitted documentation closure paths and verify the
  controller's complete checkpoint.

The general drawers, quit, M-x hints and tree highlight tasks remain outside
this set. The prior AI-presentation ticket is solved; its capture is not a new
implementation task. Looper owns execution after startup.
