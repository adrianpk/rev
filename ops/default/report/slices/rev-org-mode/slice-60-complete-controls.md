# Slice 60: complete the native Org control catalog

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-60-complete-controls`
PR: `#161`

## Purpose

Reconcile delivered native controls, the complete startup sample and embedded
template under ORG17 and ORG31. [Plan](../../../plan/rev-org-mode.md) and
[tracker](../../../tracker/rev-org-mode.md) own the approved task boundaries.

## Delivered Behavior

The control reference connects each shipped Org capability to its compiled
entry commands, focused context maps, typed preferences, palette roles and
feature references. It distinguishes accepted settings from fixed native policy.
The complete sample and embedded template include six commented, data-only Org
examples: agenda, capture, publishing, source execution, feeds and optional tools.
Active factory defaults remain identical. Agenda bulk commands invoked from
source now refuse with the required agenda context and preserve visibility.

## Implementation Notes

One compiled command catalog and the existing configuration loader remain the
owners. The reference sample is the executable's embedded template source.
The native catalog probe exercises all 228 Org operations against real read-only
Org source, including wrong-context diagnostics; edit suites supply successful
mutation coverage. All five Org roles use native input dispatch; calendar host
controls use its local map while inherited text editing belongs to an active
prompt. The probe exposed agenda bulk commands falling through to visibility
outside the agenda; the existing agenda owner now rejects that context.
Running/result ownership and source-action routing retain their original order.

The source checker initially measured that correction at complexity 11. A
separate agenda-entry admission helper restores the unchanged ceiling of 10.
The commented-record rejection probe uses sample bytes directly because TOML
serialization is intentionally absent from the existing dependency features.

Task commits: T60.1 `5b508307bdf6`, T60.2 `a2347cbd5712`, T60.3 `c479c16779a6`.
Follow-up corrections: `7dc2960233b3`, `3f8f0a74935b`, `9a6b1fdd3dee`.

Existing native suites establish feature behavior; catalog checks alone do not
prove terminal capability or optional tool success.

Canonical PR: [#161](https://forge.adrianpk.com/adrianpk/rev/pulls/161), targeting `dev`.

## Contracts Added or Changed

- ORG17: command, binding, preference and palette agreement.
- ORG31: complete documented controls and exact-candidate evidence.

## Files of Interest

- `docs/reference/org-controls.md`: capability and ownership map.
- `docs/reference/defaults.md`: exhaustive typed leaf and map catalog.
- `docs/reference/config.toml`: complete defaults and optional declarations.
- `scripts/check-config-reference.py`: packaged/reference consistency.
- `tests/commands/org_catalog.rs`: native routes, read-only catalog admission and supported reference names.
- `tests/configuration/reference/org_examples.rs`: real-loader examples, origins, reset and closed records.
- `src/session/agenda_actions.rs`: agenda context admission.

## Validation

Required ordered checks passed on clean final candidate
`ffccfe94175703d69242acebef956b7d83e7ff32`, verified merged through canonical
PR #161 into `dev`, controller receipt `NPEZJ4ZIMZKBOZI4NDGC6ZDLCH`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test configuration --test commands --test org_editing --test org_agenda`
- `python3 scripts/check-source.py`
- `git diff --check`

All 617 selected tests passed: 161 commands, 133 configuration, 97 agenda and
226 editing; zero failures, ignored or filtered tests. Source quality passed
with safe Rust, at most five parameters, three nesting levels and cyclomatic
ten. No required checks were changed.

Focused commands passed on `3f8f0a74935b2621e45a51b195c198695fb297e1`, receipt
`XMLDT2M76W27LM7JVMDLDSILF5`:

- `cargo test --test configuration reference::org_examples -- --test-threads=4`
- `cargo test --test commands org_catalog -- --test-threads=4`
- `python3 scripts/check-config-reference.py`
- `git diff --check`

All six selected new cases passed; other cases were filtered by those two
focused commands and subsequently included in the required suites. The reference
check agrees on 178 schema/catalog entries, 54 RGB roles, 263 bindings and nine
optional examples. The copied native template executable emits all 24,545 exact
reference bytes without runtime files or HOME/XDG creation. Real loader checks
retain record values and leaf origins, admit explicit empty-array replacement,
reject unknown record fields and create no declared paths or tool effects.

The final focused list also passed on the same merged candidate, receipt
`6KNDIFNLUDX2B7AOQRPAGWN6HH`:

- `python3 scripts/check-config-reference.py`
- `git diff --check`

Controller logs and Acta retain both final ordered checks. The full delivery
gate remains controller-owned.

## Risks and Follow-ups

Optional programs and paths remain explicit user declarations. Example admission
does not establish that an external tool is installed or that a destination is
writable; those outcomes belong to the selected native action.
