# Slice 63: reconcile native Org contracts and references

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `docs/rev-org-63-baseline-reconciliation`
PR: [#164](https://forge.adrianpk.com/adrianpk/rev/pulls/164)
Integrated: `6e53e29bde7218bed721b2b90776475605fca956`

## Purpose

Reconcile ORG01, ORG17 and ORG31 with the delivered editor/configuration/view
contracts and native references. The [plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) retain the approved slice boundary.

## Delivered Behavior

The editor baseline admits native Org and explicit, consented operation adapters
while preserving canonical text, coordinate domains, ordinary history, safe saves
and independent views. Configuration and panel models now link the actual native
control owners. References expose clock recovery/report/timer sections and the
previously unindexed capture, columns, source-block and feed pages.

## Implementation Notes

Earlier completed baseline metadata and validated candidates remain unchanged.
Org source/model additions do not claim final delivery acceptance. Clock and
cross-document draft language now points to the delivered policies instead of
unresolved implementation choices. All source/runtime behavior remains governed
by the existing native owners.

The capability/evidence document preserves all three earlier tested baselines and
records the distinct terminal/resource candidates. It lists native formats and
every optional program boundary without treating installed tools as successful
execution. The model now distinguishes semantic fixed policy from actual TOML
leaves and documents the delivered opt-in version-1 clock recovery record; fixed
values, native limits, source authority and parser rejection remain unchanged.

The existing acceptance manifest now maps ORG01-ORG31 and all 34 scenario names
to existing native assertions, actual owned terminal selectors and dedicated or
explicitly shared resource records. The prerequisite stage resolves the compiled
native catalog and links, rather than treating that listing as execution. Earlier
manifest cases, integration identities, terminal counts and resource ceilings are
unchanged. Final same-candidate runtime admission remains the owning delivery
stage's work.

The reference checker now covers every Org feature page and reconciled baseline.
It skips literal Markdown code spans/fences when extracting links: Org call
syntax such as `[headers](args)` is source text, while real file links and anchors
remain checked. Negative fixtures cover missing/reordered/renamed requirements
and scenarios, unknown native/terminal/resource identities, omitted compiled
cases and invalid real links/anchors. All 22 checker tests passed during development.
A bounded comparison also confirmed all 63 editor, 28 configuration, four
interaction and 31 Org requirement blocks retain their original text, together
with every earlier validated-candidate/gate field.

Canonical PR: [#164](https://forge.adrianpk.com/adrianpk/rev/pulls/164), targeting `dev`.

## Contracts Added or Changed

- ORG01: reconcile native activation with the base runtime boundary.
- ORG17: connect shipped controls, source contexts and exact startup references.
- ORG31: separate historical validation from current Org evidence.

## Files of Interest

- `ops/default/spec/rev-editor.md` and `rev-editor-model.md`: native Org amendments.
- `ops/default/spec/editor-configuration.md` and model: shared startup owner.
- `ops/default/spec/editor-interaction.md`: compatible retained projections.
- `docs/reference/org-mode.md`, `org-controls.md`, `bindings.md`, `index.md`: complete reference navigation.
- `ops/default/spec/editor-org-mode-evidence.md`: capabilities, boundaries, candidates and scenario links.
- `scripts/configuration_ui/org_contracts.py`: scenario/native/terminal/resource prerequisite resolution.
- `scripts/configuration_ui/acceptance.json`: additive Org scenario identities.
- `scripts/check-config-reference.py`, `scripts/test_acceptance.py`: reference links and negative evidence fixtures.

## Validation

Required ordered checks passed on clean implementation candidate
`4184c6ccae7898398769589a4442a57e38d2ef5f`, controller receipt `LXWSBQR6G3PWDECFSIGCVZ4KY6`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_source --test org_editing --test org_time --test org_agenda --test org_tables --test org_io --test configuration --test commands`
- `python3 scripts/check-config-reference.py`
- `python3 scripts/check-configuration-ui.py --suite prerequisites`
- `python3 scripts/check-source.py`
- `git diff --check`

All 1042 selected native tests passed: 161 commands, 133 configuration, 97 agenda,
226 editing, 192 IO, 81 source, 73 tables and 79 time; zero failures, ignored or
filtered tests. Reference checks confirm 178 schema/catalog entries, 54 RGB roles,
263 bindings, nine optional examples and the 24545-byte packaged template.
All baseline/Org reference links and anchors resolve. The existing prerequisite
collector compiled/resolved every named native assertion for all 31 Org requirements
and 34 scenarios, with known owned terminal/resource selectors. A listing remains
prerequisite evidence; it is not a terminal/resource runtime pass on this candidate.

Focused controller receipt `NVWGVBUGEOE4A2I2BFFFRBOR3B` passed on the same candidate:

- `python3 scripts/test_acceptance.py`
- `git diff --check`

All 22 checker tests passed, including negative scenario/native/journey/resource
mapping and literal-code versus real-link cases. Existing accepted requirement
blocks and all earlier validated-candidate fields remain unchanged. Current native
tests, historical terminal execution and historical release resource measurements
retain their separate candidate meanings; the final full gate remains pending.
Source quality retains safe Rust, five parameters, nesting three and cyclomatic ten.

Task commits: T63.1 `491d2b5322e0`, T63.2 `5b083a95fd3a`, T63.3 `4184c6ccae78`.
Final PR-reference candidate `6e53e29bde7218bed721b2b90776475605fca956` passed
the same ordered required commands under receipt `ZB6JLIOSZKCIWMB7H43F4VVU2L`
(1042 native tests) and focused commands under `CQ5BRJ6IRONYAU2K5YZYB5FMDY`
(22 checker tests). All 34 native scenario references resolve against the final
execution log. Canonical PR #164 is verified merged into dev at that exact head.
The controller retains final delivery-gate ownership.

## Risks and Follow-ups

The owning controller still must integrate the remaining approved slice and run
the unchanged final gate before delivery closure.
