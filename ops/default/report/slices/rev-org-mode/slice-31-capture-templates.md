# Slice 31: define native Org capture targets and templates

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-31-capture-templates`
PR: `#132`

## Purpose

Deliver typed capture admission and non-writing target discovery under the
[approved plan](../../../plan/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates)
and [tracker](../../../tracker/rev-org-mode.md#slice-31-define-native-org-capture-targets-and-templates).

## Delivered Behavior

T31.1 admits typed native entry/item/checkitem/table-line/plain templates and
file/headline/outline/date-tree target data. Empty arrays select the reference
Task fallback, with an explicit missing destination. Separate assigning-source
roots qualify template files and default files without IO or path creation.

## Implementation Notes

Selection retains literal text and bounded data. No template language is
executed. The public capture choice is distinct from an editable source or
future draft. Exact keys, array replacement and per-leaf provenance follow the
existing configuration owner.

T31.2 reuses the native bounded reader and structural parser for one explicit
capture file. Actual canonical snapshots take precedence through path/inode
aliases. Missing files, outline/date headings and list/table containers stay
non-writing plans. Qualified body/subtree ranges exclude child headings for
container lookup. Ambiguity, malformed/incomplete structure and excessive work
reject. Date-tree plans retain the supplied date and chronological sibling anchor.

T31.3 verifies exact/over template counts and field limits, explicit file entry,
byte-limited BOM/CRLF reads, missing parents, invalid bytes, malformed structure,
chronological date plans, retained old revision evidence and owned cancellation.
Native kind defaults follow the reference; plain capture requires explicit text.
The exhaustive sample remains equal to actual defaults, with 149 catalog leaves,
54 RGB roles and unchanged bindings. This preparatory layer adds no command
placeholder or release-note claim for the later complete capture lifecycle.

## Contracts Added or Changed

- ORG10: typed template/target selection and explicit destination entry.
- ORG17: admitted capture leaves and exhaustive defaults/sample coherence.

## Files of Interest

- `src/config/capture.rs`: native template kinds and admission.
- `src/org/capture.rs`: exact selection and explicit file entry.
- `src/org/capture/target/`: checked heading/container plans.
- `src/org/io/capture.rs`: real canonical snapshots and bounded source reads.
- `tests/configuration/org_capture.rs`: defaults, rejection and source roots.
- `tests/org_io/capture_bounds.rs`: real-file failures, bounds and job ownership.
- `docs/reference/org-capture.md`: native data contract.

## Validation

T31.1: `39f9929bc439ca4b062810dcaafbc5bc2a9f6968` (report introduction).
T31.3: `3a35303cc0e14693be7d4f4f641c49919f814a3d`.
T31.2: reader move `cf1810e5ee255621a18973f2329800fbc613b6ac` and implementation `768d2a9fd6956ec0f1cddb607b4d81ee0b09b885`.


- `cargo test --test configuration org_capture -- --nocapture`: three cases
  passed with zero failures or ignored tests.
- `cargo clippy --lib --test configuration -- -D warnings`: passed.
- Changed-source quality passes at unchanged parameter/nesting/complexity limits.
- `python3 scripts/check-config-reference.py`: passed after adding capture's
  typed leaves to the existing exhaustive checker; no schema assertion weakened.
- `cargo test --test org_io capture -- --nocapture`: four file-backed workflows
  passed, zero failures or ignored tests, including aliases, readonly canonical
  text, BOM/CRLF, missing targets, ambiguity and literal-block ownership.
- `cargo clippy --lib --test org_io -- -D warnings`: passed.
- Initial literal-block fixture used a raw heading, which the approved structural
  contract correctly rejects. The valid fixture now uses comma quoting; source
  exclusion and malformed rejection remain separate observable assertions.
- `cargo test --test configuration --test org_io capture -- --nocapture`:
  four configuration and nine file-backed capture workflows pass, zero failures
  or ignored tests.
- `cargo clippy --lib --test org_io --test configuration -- -D warnings`: passed.
- Final required controller order binds the clean pushed PR head:
  `cargo fmt --all -- --check`;
  `cargo clippy --all-targets --all-features -- -D warnings`;
  `cargo test --test org_agenda --test configuration --test org_io`;
  `python3 scripts/check-source.py`;
  `git diff --check`.
- Focused final controller checks cover the existing source/parser and command
  regressions plus the exhaustive configuration reference.

## Risks and Follow-ups

Editable expansion and publication belong to the
recorded successor slices. The delivery-set full gate has not run.

[Canonical PR #132](https://forge.adrianpk.com/adrianpk/rev/pulls/132) targets dev. All task commits and this PR-reference follow-up are pushed before exact-head controller validation.
