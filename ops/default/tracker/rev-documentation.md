# Documentation and v0.1.0 Tracker

Status: In progress
Date: 2026-10-30
Delivery set: rev-documentation
Slice strategy: behavior-first
Planning baseline: `0301d34d9f2d43e3ad5247fa0671a7dbaf65fe7b`
Target version: `0.1.0`
Approved: product owner, 2026-10-30; one autonomous Looper cycle for all three slices, including controller-owned Rebase + Fast-forward merges into dev
Active slice: Slice 1 - Everyday editor guide; reviewing in PR #168
Plan: [Documentation Plan](../plan/rev-documentation.md)
Gate: `scripts/check-documentation.sh && git diff --check`
Execution: one autonomous Looper cycle; the controller owns PR merges and tracker progression

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Everyday editor guide | `docs/rev-documentation-01-editor` | `docs(guide): separate the everyday editor learning path` | `ops/default/report/slices/rev-documentation/slice-1-editor-guide.md` |
| Slice 2 | Independent Org documentation | `docs/rev-documentation-02-org` | `docs(org): separate Org guides and references` | `ops/default/report/slices/rev-documentation/slice-2-org-documentation.md` |
| Slice 3 | v0.1.0 documentation closure | `docs/rev-documentation-03-v0.1.0` | `docs(release): prepare v0.1.0 documentation and changelog` | `ops/default/report/slices/rev-documentation/slice-3-v0.1.0-documentation.md` |

## Tasks

| Task | Outcome | Depends on | Status | Commit | Validation |
| --- | --- | --- | --- | --- | --- |
| T1.1 | Complete the six-chapter editor path and independent Org tutorial path. | Approved and committed planning artifacts. | reviewing; integration pending | `4f8c8dc13d169fbbcb13f2477b5f828b8067ddfb` | Native checker verifies both ordered indexes, Next/Previous sequences and guide returns. |
| T1.2 | Align README/indexes, add native Rust/Bash checks and record Slice 1 evidence. | T1.1 | reviewing; integration pending | entries `0c207ece72388bac167fed080351a65890b34a4a`; native checks `53641811317a5545ee306f147f8bf5d6a9429aaf` | Four checker regression tests and focused source-quality check passed; exact-HEAD Looper receipt required for handoff. |
| T2.1 | Separate Org commands, bindings and settings, with focused task/reference entries. | Slice 1 integrated. | pending | pending | Complete feature inventory and compatible destinations. |
| T2.2 | Adapt split-catalog guards, validate both reference areas and record Slice 2 evidence. | T2.1 | pending | pending | Native registry/context/bounds tests and Rust documentation checks. |
| T3.1 | Reconcile the delivered product and prepare balanced 0.1.0 release notes. | Slice 2 integrated. | pending | pending | Source/guide/changelog audit; version/date consistency. |
| T3.2 | Finish ergonomic review, run the selected gate and record documentation closure. | T3.1 | pending | pending | Exact-candidate documentation gate; reports and tracker consistency. |

## Slice integration

| Slice | State | PR | Integrated commit | Report state |
| --- | --- | --- | --- | --- |
| Slice 1 | reviewing; controller integration pending | `#168` | pending | reviewing; PR reference recorded |
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
new-file whitespace. Slice 1 implementation checks are recorded in its report;
the exact-HEAD controller receipt owns the handoff validation outcome.

## Controller handoff

The existing Slice 1 branch and PR #168 retain both original implementation
commits and now include the approved plan, native checker/driver and PR-reference
follow-up. Integration remains pending. The PR lists the configured native
documentation command first; Looper records the final clean-HEAD validation
before its normal controller-owned integration path.
No manual human PR approval is a prerequisite for any of these three slices.
The current approved plan owns the updated validation and execution contract.

The Slice 1 integration repair includes dev baseline
`34d8dba32dffcc78b0189fea33dd734f3458111f` without rewriting its task history.
Plan/tracker conflicts retain the approved policy and current review state;
the controller must verify fresh exact-HEAD evidence before PR integration.
