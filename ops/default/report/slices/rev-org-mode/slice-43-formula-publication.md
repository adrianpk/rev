# Slice 43: publish Org recalculation as one transaction

Status: delivered
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-43-formula-publication`
PR: [#144](https://forge.adrianpk.com/adrianpk/rev/pulls/144)

## Purpose

Deliver explicit checked recalculation under the
[approved plan](../../../plan/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction)
and [tracker](../../../tracker/rev-org-mode.md#slice-43-publish-org-recalculation-as-one-transaction).

## Delivered Behavior

T43.1 publishes explicit whole-table formula results through one canonical
history transaction. All numerical values and formatted results are prepared
before source changes. Only changed result fields are replaced; formula text,
unrelated values, separators and padding remain intact. Identical results add
no history. Ordinary undo/redo restores the complete selected operation.

T43.2 exposes table, cell and column recalculation through the compiled M-x
catalog. C-c * runs the native whole-table operation. C-c C-c on a TBLFM line
applies that program; table-body C-c C-c remains alignment. Ordinary table
recalculation uses the first formula line. Cell/column selection evaluates only
its transitive assigned dependencies and publishes only the selected outputs.
Unselected dependency result fields and unrelated failing formulas remain intact.

T43.3 extends the existing two owned terminal sessions with physical C-c *,
named recalculation, successful results, unsupported/numeric failure preservation
and complete undo/redo. Canonical native workflows verify BOM/CRLF/EOF saves,
unrelated Unicode source, peer points, inactive marks, cancellation, stale
source/view requests and rejected precise destinations.

## Implementation Notes

Task commits: T43.1 `bf412f7dd09ec93ed767fc6430a59ba1f496142e`, T43.2 `7b952916a66f05bfdf861475fd65ea5c335a416f`, T43.3 `060cee2e12457b034ca260bc4077b9b3e74c5407`.

The existing table snapshot captures the canonical stamp and immutable source.
Its owned cancellable worker parses the selected program, projects actual
table values, computes private numerical results and validates every output
destination. Publication rechecks the source/view request and delegates source,
read-only, point/mark affinity and history authority to the existing buffer
transaction. No numerical observation supplies a new buffer identity.

Recalculation uses existing source fields only. Ragged missing input is zero,
but a missing output field rejects before publication. Proposed source/output
retains the native 8 MiB ceiling. Header detection uses the first separator
with data above and below. A later numeric failure or unsupported program
cannot publish earlier successful calculations. No idle recalculation runs.

The source owner captures validated numeric preferences with the initiating
configuration. Significant digits, grid cells, syntax/numeric work steps,
syntax depth and decimal precision have typed defaults, bounded parser
admission, per-leaf overlay/provenance and live operation consumers. Empty input
and half-away rounding retain the fixed native profile. The complete sample,
binding catalog and configuration reference describe the actual accepted leaves.

## Contracts Added or Changed

- ORG13: explicit recalculation publishes prepared results as one undo unit.
- ORG14: failed calculations preserve existing results and formula source.
- ORG17: compiled native recalculation command.

## Files of Interest

- `src/org/table/calculate.rs`: numerical preparation and checked field plans.
- `src/session/tables.rs`: existing publication owner and natural point affinity.
- `tests/org_tables/recalculation.rs`: real canonical history/failure workflows.
- `docs/reference/org-tables.md`: explicit recalculation semantics.
- `src/org/formula/eval/graph.rs`: selected dependency closure without recursive traversal.
- `src/config.rs` and `src/config/`: typed numerical settings and source-owned projection.
- `tests/org_tables/calculation_lifecycle.rs`: real file, shared-view and request guards.
- `scripts/configuration_ui/org_tables.py`: existing RGB/NO_COLOR terminal journey.

## Validation

- `cargo test --test org_tables recalculation -- --nocapture`: two transaction and rejection workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed.
- Changed-source quality passed with unchanged parameter, nesting and cyclomatic limits.
- `cargo test --test org_tables --test commands -- recalculation visibility_keys`: seven native workflows passed for publication, selection, formula context, settings and compiled bindings; zero failed or ignored.
- `cargo clippy --lib --test org_tables --test commands -- -D warnings`: passed; changed-source quality passed after moving formula-selection logic into its owner.
- `python3 scripts/check-config-reference.py`: passed after updating the documented Org map to 71 entries; 160 schema/catalog entries and 250 default bindings agree with the packaged reference.
- `cargo test --test org_tables -- calculation recalculation`: eight native workflows passed, zero failed or ignored.
- `cargo clippy --lib --test org_tables -- -D warnings`: passed after lifecycle coverage; changed-source quality passed.
- `python3 scripts/check-configuration-ui.py --suite terminal --case org-tables`: two owned RGB/NO_COLOR sessions passed, zero skipped. Existing exact-canvas, persisted-byte, cleanup and resource assertions remain in place; no limits were increased.
- Required batch `7MCSHW4DQONBX6V75VLWQQ6ICY` at `33891d3c23a240a6363daba19abba4269da00c55` stopped at the command regression check because the public command catalog omitted the three new identities. The catalog now includes all three, and the binding catalog retains one C-c C-c identity with its expanded behavior.
- `cargo test --test commands palette::catalog_reference -- --nocapture`: passed after the catalog correction. Final evidence is regenerated for the subsequent clean PR head.

Final clean candidate `9698833f2061afda469515ee7f1fd06bb406f309` passed
required receipt `PVVHBQWSO3JDXA4CAUZX75RHJU`: six ordered checks, 445 tests and
two owned terminal sessions. Focused receipt `WJYTUOCOFJ3XUC6KGT6FZHZEOB`
passed 231 library/configuration/source tests and the configuration reference.
Both final receipts have zero failures, ignored tests or skipped sessions.
An earlier focused run at the same candidate failed the existing process-lifetime
cleanup assertion; its isolated rerun and the complete fresh focused batch
passed without code, assertion or resource-limit changes. The cause remains
unconfirmed; Acta note `01M645S0GK91F15H9QQ8S0J8TF` retains the diagnostic.

Canonical PR #144 merged into dev at that candidate, verified against Forgejo
and origin/dev. Slice 43 is delivered.

## Risks and Follow-ups

None recorded.
