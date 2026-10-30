# Slice 61: verify complete native Org terminal workflows

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `test/rev-org-61-combined-terminal`
PR: `#162`

## Purpose

Extend existing terminal admission for the delivered native Org workflows under
ORG01, ORG02, ORG17 and ORG18. [Plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) retain the approved task boundaries.

## Delivered Behavior

The default terminal stage retains all ten historical configuration/UI sessions
and invokes all 17 earlier named Org journeys, observing 57 owned sessions. Two connected RGB/NO_COLOR sessions
use one process each for canonical task metadata/history, clocks, capture/refile,
habit and multi-source agenda views, table publication and real missing-tool
consent/failure. Focused selectors remain available.
The two connected sessions bring the terminal total to 69: ten historical plus
59 Org sessions. An execution receipt identifies the clean commit/tree, observed session counts,
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

The native process-lifetime driver originally checked descendant state in the
same instant as leader reap. Group signaling does not synchronize that kernel
state observation. It now polls the same stopped-state predicate within the
existing 150 ms response budget; immediate leader reap, private-directory cleanup,
source, timeout and cancellation assertions remain unchanged. Focused receipt
`BZU5XNRJVC5VSI6F6OYZZEI4HR` at `5662300fabe99c37d2a3884206c25b8a8c3944c5`
passed all four lifetime modes, observing stopped descendants in 5.941..6.261 us,
plus all 19 evidence-checker tests.

The complete collector also exposed a local capture-loop name that hid the
read-only task function; the corrected runner retains both scenarios. Focused
receipt `EQMKAQZPA5N7LCHIIP7W4BZ53Q` at
`70ad3be3b9d3df8e413d21f306d74793e0e43186` passed all eight owned task sessions
and all 19 checker tests. Neither correction changes product behavior.

The full terminal stage is broader than a focused selector. Existing journeys
remain batched through one native build. The approved complete-gate target is
three to six warm minutes; actual terminal cost will be recorded without raising
timeouts or resource budgets.

Task commits: T61.1 `05c8d8ef90e8`, T61.2 `25cd33a84bca`, T61.3 `7a01781ba04a`.
Follow-up corrections retain the same workflow and source/resource assertions.
The retained outline list probe now explicitly pauses a relative timer, bounds
its elapsed reading and checks that timed-list continuation inserts that exact
field under ORG22; undo and stop preserve the original bytes. Literal C-c C-c
checks the precise ORG17 document refusal instead of a generic checkbox fallback.

Canonical PR: [#162](https://forge.adrianpk.com/adrianpk/rev/pulls/162), targeting `dev`.

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
- `tests/org_io/execution_edges.rs`: bounded observation of actual descendant stop.
- `scripts/test_acceptance.py`: rejection of missing, stale or incomplete evidence.

## Validation

Required ordered checks passed on clean implementation candidate
`3d376ba39994668482cb5836dbf852a46dffb1c3`, controller receipt
`XDLFXLAJ6G6YWFXYY6IAKHRSGF`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test commands --test display`
- `python3 scripts/check-configuration-ui.py --suite terminal`
- `python3 scripts/check-source.py`
- `git diff --check`

All 948 native tests passed: 161 commands, 39 display, 97 agenda, 226 editing,
192 I/O, 81 source, 73 tables and 79 time; zero failures, ignored or filtered tests.
Source quality retains safe Rust, at most five parameters, three nesting levels
and cyclomatic ten. All 69 owned terminal sessions passed, zero skips: ten
historical and 59 Org, including all eight connected stages in both color modes.
The complete terminal stage measured 55.807 seconds excluding build. Its actual
log passes the extended historical/Org admission, including the clean commit/tree,
exact ordered cases/stages, observed counts, isolated roots and completed lifetime.

Focused controller receipt `J4R5J4UBGSRHJ3XUWSYMA4TNJ4` passed on the same candidate:

- `python3 scripts/test_acceptance.py`
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`
- `git diff --check`

All 19 evidence-checker tests and both retained outline sessions passed. The
focused connected-session receipt `UPR5RKQ45WBOTS6XOSOTFU6YE2` at
`83b53cbfffae77865abec8b6b0d7d5c520139b39` also passed all sixteen connected
stages; the complete required run above includes those stages with continuous
NO_COLOR checks. No scenario, source assertion or existing deadline was dropped.

Final readiness requires fresh ordered required checks and focused checker tests
on the clean PR-reference candidate. Controller receipts and Acta retain that
exact head and the final validation IDs. The full delivery gate remains controller-owned.

## Risks and Follow-ups

PTY evidence checks native output, real source/file outcomes and lifecycle
restoration. It does not substitute for physical-terminal pixel rendering.
