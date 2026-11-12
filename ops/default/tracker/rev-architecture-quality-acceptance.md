# Standalone Acceptance Tracker

Status: In progress
Date: 2026-11-12
Delivery set: rev-architecture-quality-acceptance
Parent delivery set: rev-architecture-quality
Slice strategy: behavior-first
Approved: owner explicitly requires all recorded acceptance debt to be corrected before closing the parent set and requested Looper visibility
Active slice: Slice 1, Standalone acceptance debt
Specification: [Architecture Quality and Commit Checks](../spec/editor-architecture-quality.md)
Concern: [Pending Editor Work](../README.md)
Plan: [Standalone Acceptance Correction](../plan/rev-architecture-quality-acceptance.md)
Parent tracker: [Architecture Quality Tracker](rev-architecture-quality.md)
Progress owner: selected correction Looper run

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Standalone acceptance debt | `fix/rev-architecture-quality-acceptance` | `fix(acceptance): resolve standalone editor journeys` | `ops/default/report/slices/rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md` |

## Slice 1: Standalone acceptance debt

Status: delivered
Branch: `fix/rev-architecture-quality-acceptance`
PR: `#28`
Integrated: `74896fd18e90403c0961c16bc5ae86ea4932e633` through canonical PR #28.
Report: `ops/default/report/slices/rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md`

- [x] T1.1: Complete file-tree native/color/both-profile PTY acceptance passes.
- [x] T1.2: Complete ranking native/restart/paired-publisher acceptance passes.
- [x] T1.3: Complete keybinding-hint native/both-profile PTY acceptance passes.
- [x] T1.4: Complete wrapping native/PTY/performance acceptance passes.
- [x] T1.5: Complete Projects/Workspaces native/both-profile PTY acceptance passes.
- [ ] T1.6: Corrective PR integrated, expanded exact-candidate gate passed, and parent debt closure committed/pushed.

## Acceptance

All seven focused checks in the plan and both parts of the expanded integrated
gate are required. The parent set remains reopened until all five entrypoints
pass without the previously deferred failures.

## Gate preparation

Slice 1 is integrated and its report is delivered. The correction set and T1.6
remain open until the controller passes the expanded gate and closes the parent
delivery. The seven ordered focused commands passed on clean published and now
integrated HEAD `74896fd18e90403c0961c16bc5ae86ea4932e633`.

The existing executable staged hook and repository-local
`core.hooksPath=.githooks` are verified without reactivation. Cargo, Rust, Go,
Spore and the existing PTY tools are available. The external plugin acceptance
checkout is clean at `3365b84649919492ade7e80c63139b459b5ad923`; its original
runner resolves from the frozen gate command. No gate commands, source files,
assertions, resource bounds or build-cache ownership change during preparation.
The controller runs all five complete acceptance scripts followed by the parent
plan's unchanged eight-stage integrated gate on the exact clean candidate.

## Required gate repair

T1.6 remains open. The first retry retained one namespace init/reaping failure;
its corrected environment then exposed a premature exact-source assertion in
`integration::presentation::native_merge`. The plan pins the fixture-only
repair branch `fix/rev-architecture-quality-acceptance-reload-1`, exact PR title,
existing Slice 1 report and unchanged five-second settlement budget. Both
profiles and every existing source/HEAD/dispatch/status assertion remain required.
The complete Git target must pass before another expanded gate attempt.

Repair state: delivered
Branch: `fix/rev-architecture-quality-acceptance-reload-1`
PR: `#29`
Implementation: `38b76c17ee14aeb165f52c725e77b6965f8a1096`
Integrated: `ed9f8dedea2a7633d529220be0bcbc7cb579cd90` through canonical PR #29.
Report: `ops/default/report/slices/rev-architecture-quality-acceptance/slice-1-standalone-acceptance.md`

The exact native-merge case reproduced the premature source assertion in the
approved reaping namespace. The fixture now latches observed integration
verification and requires the exact reloaded source within the same existing
five-second wait. Both profiles, submit aliases and later HEAD, dismissal,
source and sequence-status assertions remain. Scoped quality passes all 55 Git
fixture files with zero blocking diagnostics. The focused native case passes in
both profiles; the complete Git target passes all 242 cases with no ignores in
400.03 seconds. All four ordered repair checks passed through the controller's
validator on clean published HEAD `ed9f8dedea2a7633d529220be0bcbc7cb579cd90`.
Forgejo and Acta confirm its integration into `dev`. Gate preparation verifies
the retained reaper checksum and its UID 1000 / child PID 2 execution without
changing it. The expanded gate and T1.6 closure remain controller-owned and pending.
