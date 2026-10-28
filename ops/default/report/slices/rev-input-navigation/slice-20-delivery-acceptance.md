# Slice 20: Reconcile combined delivery acceptance and references

Status: reviewing
Delivery set: rev-input-navigation
Plan: [Input and Navigation Delivery](../../../plan/rev-input-navigation.md)
Tracker: [Input and Navigation Tracker](../../../tracker/rev-input-navigation.md)
Branch: `test/rev-input-20-delivery-acceptance`
PR: [#100](https://forge.adrianpk.com/adrianpk/rev/pulls/100)

## Purpose

Reconcile the approved seven-topic contracts and their actual native, terminal,
resource, ticket and integration evidence for controller-owned final acceptance.

## Delivered Behavior

The editor baseline contains the complete R05 replacement and all 23 approved
GUT/PTR/NAV/WSP/STA additions with their scenarios. Responsive ordered vertical
motion and goto-line join the existing movement contract. Configuration adopts
the full CFG01/CFG14/CFG15 replacements and CFG23/CFG24 additions, including the
one absent-entry startup exception and preservation of pure loader behavior.
The configuration model includes the nine gutter/mouse leaves and four semantic
gutter palette roles already present in the native catalog and sample. View,
terminal and pointer ownership now describe the actual session-local state.

The existing configuration/UI acceptance map now covers 58 requirements. Its
original thirty entries are preserved exactly and independently pinned, with
named native and combined-journey references for all current additions. The
same acceptance owner checks the approved twenty-row map, nineteen earlier
canonical integration identities, delivered report/checkbox state and all seven
solved tickets. Inline and block commit arrays both resolve their assigned
primary task outcomes through actual Git ancestry and Conventional Commit task
identities. Listing prerequisites remain distinct from full runtime admission.

## Implementation Notes

Earlier validated candidate and gate identities remain unchanged as historical
facts. Current amendments link to this delivery and retain pending integrated
acceptance. Complete approved requirement blocks and scenarios are copied into
their actual owners; no new requirement identities or runtime capability are
invented. Existing command/default/gutter/status references already describe the
native controls. Terminal input now connects the bounded repeat behavior and
the measured decoder burst limitation to the actual declared workload.

All nineteen earlier canonical PRs (#81-#99) were checked live against their
configured exact title, branch, dev base and Rebase + Fast-forward head. Each
head is an ancestor of this worktree and Acta has the corresponding merge event.
All seven ticket outcomes are solved/fixed with their actual primary task
commits and report/PR identities. No ticket was moved or its earlier evidence
replaced. The tracker records that verified primary closure while leaving the
current PR, integrated gate and delivery documentation closure pending.

Task commits:

- T20.1: `6fbd2e67920d9a85ebd9f5d02be42a79a2b6ac32`
- T20.2: `6fff1b349a48b41e0b087b80d541037f14e837a2`
- T20.3: `253161872a1f2c067ea204f72bc458617dc5c339`

## Contracts Added or Changed

R05/R07/R16, CFG01/CFG14/CFG15/CFG23/CFG24 and the accepted GUT, PTR, NAV,
WSP and STA additions are reconciled with their existing state owners.

## Files of Interest

- `ops/default/spec/rev-editor.md`: current baseline and complete capability additions.
- `ops/default/spec/editor-configuration.md`: precise first-start creation exception.
- `ops/default/spec/editor-configuration-model.md`: native gutter/mouse catalog and state.
- `ops/default/spec/rev-editor-model.md`: view/capture/reporting ownership.
- `docs/reference/terminal-input.md`: ordered repeat and measured decoder limit.
- `scripts/configuration_ui/acceptance.json`: complete current mapping and pinned integration evidence.
- `scripts/configuration_ui/delivery.py`: current requirements, models, reports, tickets and ancestry admission.
- `scripts/test_acceptance.py`: rejection of incomplete and inconsistent delivery evidence.

## Validation

Focused block reconciliation checks compare every approved replacement and
addition, including scenarios, with its current target. Earlier full-gate
identities remain historical; this slice does not run the combined gate.
Development `python3 -m unittest discover -s scripts -p test_acceptance.py`
passed nineteen guard tests in 1.578 seconds. New mutations cover partial or
duplicate requirements/scenarios, incorrect model defaults, rewritten historical
provenance, changed approved maps/integrations, missing/altered report state,
current final-slice readiness versus delivery and mismatched ticket outcomes.
Both ticket commit-list representations are exercised with actual read-only Git
ancestry. These checks add approximately two seconds on a warm candidate.
The eleven full-gate stage identities/order, original runtime/native/terminal/
resource protections and exact-candidate log admission remain unchanged.
Final exact-head controller validation is recorded after PR-reference updates.

Required final ordered commands:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test pointer --test gutter --test commands --test display --test configuration --test panels --test windows --test editing`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-acceptance.py --prerequisites`
- `python3 -m unittest discover -s scripts -p test_acceptance.py`
- `python3 scripts/check-source.py`
- `git diff --check`

These focused/native/reference/prerequisite checks prepare the PR. They do not
run or replace `scripts/check-all.sh`. The controller integrates the final PR,
closes its integrated report state and runs the unchanged eleven-stage gate on
the exact clean pushed dev candidate. Only the configured documentation paths
may close afterward, retaining tested and closure identities as distinct facts.

## Risks and Follow-ups

The known instantaneous multi-kilobyte pointer-report decoder stall remains
recorded in Slice 19 and the terminal reference. It is outside the measured
120-report-per-second stream; no runtime/dependency correction is included here.
