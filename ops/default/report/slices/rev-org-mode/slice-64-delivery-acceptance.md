# Slice 64: prepare complete native Org delivery acceptance

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `test/rev-org-64-delivery-acceptance`
PR: pending

## Purpose

Prepare ORG01, ORG18 and ORG31 acceptance under the approved
[plan](../../../plan/rev-org-mode.md) and [tracker](../../../tracker/rev-org-mode.md).

## Delivered Behavior

The existing acceptance collector admits all 64 approved slice identities and
reports, 192 completed tracker tasks and actual ORG01-ORG31 scenario execution.
Readiness remains prerequisite resolution, without a full-gate success claim.

## Implementation Notes

The additive Org delivery manifest pins all 63 earlier canonical PR integration
commits and 189 task commits. Forgejo closed-PR state was verified before capture;
Git ancestry places each task in its owning slice integration range. Established
report heading and PR formats remain readable without rewriting earlier records.
The final slice requires its canonical PR, delivered report, integration ancestry
and three task commits before runtime admission. Prerequisites permit its reviewing
report and pending integration only. Provenance does not replace semantic checks:
all 34 native assertions must occur in actual Cargo results, alongside the existing
same-candidate terminal stage assertions and release resource observations.

The eleven frozen gate commands and their order are explicitly admitted, with
no change to `scripts/check-all.sh`. Final integration must point to the reviewed
PR-reference report stored at that commit, not a later closure commit. Native,
terminal and resource records must still share that exact clean HEAD and tree.

The controller pins 91 exact documentation closure paths; 88 currently exist.
The absent `docs/reference/panels.md`, `docs/reference/org-time.md` and
`docs/how-to/org-mode.md` are permissions, not required new documents. Closure
must retain the actual tested SHA, validation identifier, ordered commands and
runtime observations separately from its later documentation SHA. Existing
historical closure metadata stays unchanged. No final runtime result is claimed
by fixture preparation or this slice's prerequisite checks.

The approved 192 task descriptions must match between plan and tracker, with
unchanged identities and spec/model ownership links. All 18 historical closure
records retain their original candidate and gate identifiers. Negative fixtures
reject changed canonical records, unrelated task commits, incomplete reports,
altered gate order, omitted native assertions, stale HEAD/tree evidence and
premature use of a later closure commit. Fixture Git history tests the reviewed
snapshot boundary independently of runtime execution.

## Contracts Added or Changed

- ORG01: complete source-preserving native scenario admission.
- ORG18: all 34 agreed scenarios require actual execution with zero skips.
- ORG31: canonical delivery provenance and same-candidate evidence stay distinct.

## Files of Interest

- `scripts/configuration_ui/org_delivery.py`: Org integration and task admission.
- `scripts/configuration_ui/acceptance.json`: additive immutable delivery records.
- `scripts/configuration_ui/evidence.py`: current runtime and prerequisite admission.
- `scripts/test_acceptance.py`: negative acceptance fixtures.

## Validation

Development check `python3 -m unittest discover -s scripts -p test_acceptance.py`
passed all 26 checker tests. A read-only admission check resolved all 63 pinned
canonical reports and 189 task commits in their owning Git ranges.
Required exact-head controller checks are pending. The full gate belongs to the
controller after canonical integration.

## Risks and Follow-ups

Canonical merge, the unchanged full gate and exact-path documentation closure
remain controller-owned. A later documentation head cannot inherit the tested
candidate identity.
