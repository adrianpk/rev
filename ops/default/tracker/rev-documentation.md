# Documentation and v0.1.0 Tracker

Status: In progress
Date: 2026-10-30
Delivery set: rev-documentation
Slice strategy: behavior-first
Planning baseline: `0301d34d9f2d43e3ad5247fa0671a7dbaf65fe7b`
Target version: `0.1.0`
Approved: product owner, 2026-10-30; one autonomous Looper cycle for all three slices, including controller-owned Rebase + Fast-forward merges into dev
Active slice: Slice 2 - Independent Org documentation; reviewing
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
| T1.1 | Complete the six-chapter editor path and independent Org tutorial path. | Approved and committed planning artifacts. | completed | `4f8c8dc13d169fbbcb13f2477b5f828b8067ddfb` | Receipt `E5KZ2QKMPB33LEFR44KLTMDIUW`: ordered indexes, chapter sequences and guide returns passed. |
| T1.2 | Align README/indexes, add native Rust/Bash checks and record Slice 1 evidence. | T1.1 | completed | entries `0c207ece72388bac167fed080351a65890b34a4a`; native checks `53641811317a5545ee306f147f8bf5d6a9429aaf` | Receipt `E5KZ2QKMPB33LEFR44KLTMDIUW`: six ordered checks passed on the integrated candidate. |
| T2.1 | Separate Org commands, bindings and settings, with focused task/reference entries. | Slice 1 integrated. | reviewing; integration pending | `8a8dfd94b1fe6c2b740dc9fb35f8870784dfd3bf` | Complete inventories, 58 reachable pages, 320 local links/anchors and eleven legacy destinations. |
| T2.2 | Adapt split-catalog guards, validate both reference areas and record Slice 2 evidence. | T2.1 | reviewing; integration pending | `9c7d8d0deb7f2d1496027d97772dfcacca0d4b04` | Native checks passed during implementation; controller records the final clean-HEAD receipt before handoff. |
| T3.1 | Reconcile the delivered product and prepare balanced 0.1.0 release notes. | Slice 2 integrated. | pending | pending | Source/guide/changelog audit; version/date consistency. |
| T3.2 | Finish ergonomic review, run the selected gate and record documentation closure. | T3.1 | pending | pending | Exact-candidate documentation gate; reports and tracker consistency. |

## Slice integration

| Slice | State | PR | Integrated commit | Report state |
| --- | --- | --- | --- | --- |
| Slice 1 | delivered | `#168` | `01f03f00729f4465d3d853ee68fd9c30aa26c7cf` | delivered |
| Slice 2 | reviewing; controller integration pending | pending | pending | reviewing |
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

PR #168 is verified merged at `01f03f00729f4465d3d853ee68fd9c30aa26c7cf`;
its exact-candidate receipt passed all six ordered checks. Slice 2 starts from
that integration and this documentation closure. The configured Slice 2
validation prefix is `scripts/check-documentation.sh`. The controller owns
PR integration; the approved plan retains the execution and validation policy.
