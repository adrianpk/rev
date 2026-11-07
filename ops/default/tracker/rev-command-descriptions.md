# Command Descriptions Tracker

Status: Reviewing
Date: 2026-11-07
Delivery set: rev-command-descriptions
Approved: product owner; complete the catalog in one implementation slice
Plan: [Command Descriptions Delivery](../plan/rev-command-descriptions.md)
Specification: [Command Descriptions](../spec/editor-command-descriptions.md)
Ticket: [TKT-20261106202027](../ticket/ready/20261106202027-complete-command-catalog-descriptions.md)
Active slice: Slice 1 - Complete descriptions
Progress owner: retained Looper checkpoint and canonical validation evidence

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Complete descriptions | `feat/rev-command-descriptions-01-catalog` | `feat(commands): describe every built-in command` | `ops/default/report/slices/rev-command-descriptions/slice-1-catalog.md` |

## Slice 1: Complete descriptions

State: reviewing
Dependency: approved committed preparation on clean dev
Requirements: CDS01-CDS05
Branch: `feat/rev-command-descriptions-01-catalog`
PR: `#307`
Report: `ops/default/report/slices/rev-command-descriptions/slice-1-catalog.md`
Validation: the nine exact ordered commands in the plan and local configuration

- [x] T1.1: Populate canonical command/family metadata with every specified
  built-in summary, retaining aliases and optional user-command metadata.
- [x] T1.2: Verify all names/texts against the spec and add bounded native
  description coverage in both input profiles; preserve existing selector rules.
- [x] T1.3: Complete reference, User Guide, Unreleased and Spore guidance;
  introduce the slice report and record the actual commit/PR/check evidence.

## Final gate and closure

- [ ] Verify the single canonical PR integrated and its report recorded.
- [ ] Run the unchanged focused gate on the exact clean dev candidate.
- [ ] Record successful evidence and its actual scope.
- [ ] Move the ticket to solved with resolution fixed and update all links.
- [ ] Complete the report, plan, tracker, specification and index within the
  configured exact closure paths.

The slice implementation and nine ordered local checks passed. The canonical
report records task/commit correspondence, six selected tests, documentation
checks and the 91-page Spore build. Exact clean-HEAD evidence follows the final
PR-reference commit through the controller validator. Integration and the dev
gate remain pending; no delivered slice or completed set is claimed here.
