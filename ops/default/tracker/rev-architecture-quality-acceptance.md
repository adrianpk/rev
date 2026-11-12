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

Status: reviewing
Branch: `fix/rev-architecture-quality-acceptance`
PR: `#28`
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
