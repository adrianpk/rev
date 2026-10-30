# Slice 61: verify complete native Org terminal workflows

Status: drafting
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `test/rev-org-61-combined-terminal`
PR: pending

## Purpose

Extend existing terminal admission for the delivered native Org workflows under
ORG01, ORG02, ORG17 and ORG18. [Plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) retain the approved task boundaries.

## Delivered Behavior

The default terminal stage retains all ten historical configuration/UI sessions
and invokes every earlier named Org journey. Two connected RGB/NO_COLOR sessions
use one process each for canonical task metadata/history, clocks, capture/refile,
habit and multi-source agenda views, table publication and real missing-tool
consent/failure. Focused selectors remain available.
An execution receipt identifies the clean commit/tree, observed session counts,
isolated roots, successful terminal restoration, reaped children and closed PTYs.

## Implementation Notes

The existing collector, PTY owner and acceptance manifest remain authoritative.
Session observation wraps their real construction, finish and close boundaries.
Success labels follow completed assertions and cleanup; a footer or candidate
identity alone is not semantic proof. Earlier historical map rows and stage
identities remain unchanged. NO_COLOR output is checked throughout every observed PTY read, rather than only
its final frame. PTYs without an explicit cwd receive their own fixture work/root
directory. Cleanup also closes only owned unfinished fixtures after a failure;
that cleanup cannot convert the failed assertion into a passing receipt.

New connected fixtures preserve the existing paste-refusal behavior of retained
views and assert the visible scrolled yes/no question plus missing-runner identity.
They do not require an off-screen prompt prefix. The original eight-second PTY
deadline, source byte assertions, consent steps and resource limits are unchanged.

Reference admission now expects the complete
shipped catalog rather than its earlier preparatory counts.

The full terminal stage is broader than a focused selector. Existing journeys
remain batched through one native build. The approved complete-gate target is
three to six warm minutes; actual terminal cost will be recorded without raising
timeouts or resource budgets.

## Contracts Added or Changed

- ORG01/ORG02: native mode and canonical source behavior through owned terminals.
- ORG17: focused keys and compiled commands across connected workflows.
- ORG18: source/history guarantees and actual owned terminal/process cleanup.

## Files of Interest

- `scripts/check-configuration-ui.py`: default and focused journey selection.
- `scripts/configuration_ui/org_combined.py`: connected native workflows and independent file expectations.
- `scripts/configuration_ui/evidence.py`: historical and Org evidence admission.
- `scripts/configuration_ui/terminal_evidence.py`: actual owned PTY lifetime observations.
- `scripts/configuration_ui/acceptance.json`: additive Org terminal identities/counts.
- `scripts/test_acceptance.py`: rejection of missing, stale or incomplete evidence.

## Validation

Required ordered checks remain pending:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test commands --test display`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

Focused controller receipt `UPR5RKQ45WBOTS6XOSOTFU6YE2` passed on
`83b53cbfffae77865abec8b6b0d7d5c520139b39`:

- `python3 scripts/test_acceptance.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-combined`
- `git diff --check`

All 19 evidence-checker tests and both connected native sessions passed, with
all eight stages per session and zero skips. Required checks will include the
stronger continuous NO_COLOR observation on the subsequent clean candidate. The full
delivery gate remains controller-owned.

## Risks and Follow-ups

PTY evidence checks native output, real source/file outcomes and lifecycle
restoration. It does not substitute for physical-terminal pixel rendering.
