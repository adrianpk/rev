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
and invokes every earlier named Org journey. Focused selectors remain available.
An execution receipt identifies the clean commit/tree, observed session counts,
isolated roots, successful terminal restoration, reaped children and closed PTYs.

## Implementation Notes

The existing collector, PTY owner and acceptance manifest remain authoritative.
Session observation wraps their real construction, finish and close boundaries.
Success labels follow completed assertions and cleanup; a footer or candidate
identity alone is not semantic proof. Earlier historical map rows and stage
identities remain unchanged. Reference admission now expects the complete
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

Focused evidence-checker tests will run through controller validation. The full
delivery gate remains controller-owned.

## Risks and Follow-ups

PTY evidence checks native output, real source/file outcomes and lifecycle
restoration. It does not substitute for physical-terminal pixel rendering.
