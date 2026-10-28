# Slice 12: edit Org drawers and inherited properties

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-12-drawers-properties`
PR: [#113](https://forge.adrianpk.com/adrianpk/rev/pulls/113)

## Purpose

Deliver checked property edits and inspection with explicit source ownership.

## Delivered Behavior

Local property set/delete, additive storage and inspection use native prompts.
Selective `org.properties.inherit` defaults to empty and resolves nearest bases,
additions, nil barriers and file declarations without inserting inherited text.
Inspection reports source-owner provenance; explicit owner editing targets those
exact stored value ranges.
New drawers follow heading/planning metadata. Existing drawer folding retains
its independent view behavior and ordinary/property distinction.

## Implementation Notes

The existing temporary observation worker collects revision-bound property
owners and source ranges. Plans publish through the ordinary checked edit path.
Prompt stages recheck the originating view and source; C-g cancels without edits.
The derived model admits at most 4096 entries and 8192 property records; incomplete
observations reject property operations.

## Contracts Added or Changed

- ORG05: distinct drawers, local/additive stored properties and source owners.
- ORG17: compiled commands and the Org-local C-c C-x p binding.

## Files of Interest

- `src/org/property/`: native source ownership and edit plans.
- `src/session/outline/property.rs`: qualified prompt and publication lifecycle.
- `tests/org_editing/properties.rs`, `property_edges.rs`: exact source,
  source-owner edits, shared cursors, qualification, bounds and read-only checks.
- `tests/configuration/org_properties.rs`: typed bounds, includes and provenance.
- `scripts/configuration_ui/org_properties.py`: property steps inside the existing
  owned RGB/NO_COLOR outline journey.

## Validation

Development checks passed:

- `cargo check`
- `cargo test --test org_editing --test configuration propert`: five property
  tests and two configuration tests passed while implementing resolution.
- `cargo test --test org_editing propert`: all 13 property tests passed,
  including owner selection, shared-view replay, stale drafts, literal/deeper
  context rejection, ambiguous/malformed metadata and record-limit admission.
- `python3 scripts/check-config-reference.py`: schema/sample/template agree.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-outline`:
  two owned RGB/NO_COLOR sessions passed, zero skipped. Existing outline/list
  assertions remain intact; added property prompts, inherited provenance,
  explicit owner editing and cancellation restore exact BOM/CRLF/EOF bytes.

Focused source-quality checks passed for changed owners and tests. An initial
check caught a configuration dispatch complexity excess, corrected by keeping
property admission with the existing editing-policy validation. A file-preamble
fixture reproduced comment-context rejection; direct source comments are now
admitted without admitting nested or literal declarations.

Owned clean Emacs observations confirmed selective file defaults, additive
ordering, nil barriers and file-base resets. Reference semantics were checked
against [property syntax](https://orgmode.org/manual/Property-Syntax.html) and
[property inheritance](https://orgmode.org/manual/Property-Inheritance.html).
No runtime dependency was introduced.

After the PR-reference follow-up is pushed, the controller records these required
commands against its exact clean HEAD, in order:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_editing --test org_source --test configuration`
- `python3 scripts/check-source.py`
- `git diff --check`

A separate focused batch records `cargo test --test commands`,
`python3 scripts/check-config-reference.py`, the terminal command above and
`git diff --check` against the same HEAD. Canonical receipt IDs and exact outcomes
are retained in controller evidence and Acta.

## Risks and Follow-ups

Incomplete observations reject property operations at the documented native and
derived-model limits. Inspection displays at most 512 value characters and 16
contribution descriptions; explicit owner selection identifies its chosen source
before asking for a value. No other follow-up is recorded.
