# Documentation and v0.1.0 Tracker

Status: In progress
Date: 2026-10-30
Delivery set: rev-documentation
Slice strategy: behavior-first
Planning baseline: `0301d34d9f2d43e3ad5247fa0671a7dbaf65fe7b`
Target version: `0.1.0`
Approval: approved by the product owner, 2026-10-30; three-slice plan and execution through PRs to dev
Active slice: Slice 1 - Everyday editor guide; reviewing
Plan: [Documentation Plan](../plan/rev-documentation.md)
Gate: focused documentation/reference gate defined in the plan

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Everyday editor guide | `docs/rev-documentation-01-editor` | `docs(guide): separate the everyday editor learning path` | `ops/default/report/slices/rev-documentation/slice-1-editor-guide.md` |
| Slice 2 | Independent Org documentation | `docs/rev-documentation-02-org` | `docs(org): separate Org guides and references` | `ops/default/report/slices/rev-documentation/slice-2-org-documentation.md` |
| Slice 3 | v0.1.0 documentation closure | `docs/rev-documentation-03-v0.1.0` | `docs(release): prepare v0.1.0 documentation and changelog` | `ops/default/report/slices/rev-documentation/slice-3-v0.1.0-documentation.md` |

## Tasks

| Task | Outcome | Depends on | Status | Commit | Validation |
| --- | --- | --- | --- | --- | --- |
| T1.1 | Complete the six-chapter editor path and independent Org tutorial path. | Approved and committed planning artifacts. | completed | `4f8c8dc13d169fbbcb13f2477b5f828b8067ddfb` | Passed both navigation walks, 262 local targets and 53-page reachability. |
| T1.2 | Align README/indexes and record Slice 1 evidence. | T1.1 | completed | record on integration | Passed entry review, 253 local targets, 53-page reachability and whitespace. |
| T2.1 | Separate Org commands, bindings and settings, with focused task/reference entries. | Slice 1 integrated. | pending | pending | Complete feature inventory and compatible destinations. |
| T2.2 | Adapt split-catalog guards, validate both reference areas and record Slice 2 evidence. | T2.1 | pending | pending | Native registry/context/bounds tests and Python reference/link checks. |
| T3.1 | Reconcile the delivered product and prepare balanced 0.1.0 release notes. | Slice 2 integrated. | pending | pending | Source/guide/changelog audit; version/date consistency. |
| T3.2 | Finish ergonomic review, run the selected gate and record documentation closure. | T3.1 | pending | pending | Exact-candidate documentation gate; reports and tracker consistency. |

## Slice integration

| Slice | State | PR | Integrated commit | Report state |
| --- | --- | --- | --- | --- |
| Slice 1 | reviewing | pending | pending | reviewing |
| Slice 2 | queued | pending | pending | not created |
| Slice 3 | queued | pending | pending | not created |

## Delivery acceptance

- The User Guide teaches everyday editing without a mandatory Org step.
- Org has an independent learning entry, task grouping and complete references.
- Extracted catalogs retain all commands, bindings, contexts and settings.
- Existing examples remain simple, correct and linked from their proper quadrant.
- The initial-version changelog covers delivered user outcomes without future
  ticket claims or development-stage repetition.
- The exact final documentation candidate passes the plan's focused gate.
- Each integrated slice has its canonical report and traceable PR/commit evidence.

## Release handoff

No tag, main promotion, binary release, website or mirror publication belongs
to this tracker. The completed documentation can support a separately authorized
0.1.0 release with exact-candidate full-gate evidence. A documentation-gate pass
must never be reported as a completed product release.

## Planning checks

Local checks passed for the three identical plan/tracker slice mappings, six
task IDs, existing links, draft/active state, private-context exclusion and
new-file whitespace. No implementation or release validation has run for this
new delivery set.
