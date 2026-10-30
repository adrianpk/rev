# Slice 57: insert checked Org block and call results

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-57-block-results`
PR: pending

## Purpose

Deliver checked block/call results under the approved plan's Slice 57, contracts
ORG28 and ORG14.

## Delivered Behavior

Selected block execution and named local/library calls prepare bounded source
observations, retain confirmation and update recognized RESULTS through one
checked source transaction. Declared adapters return output, scalar value or
rectangular tab-delimited data. Failed jobs retain prior content.

## Implementation Notes

Execution preparation reuses native source scans, canonical file observations,
header resolution and the owned program runner. Result publication uses the
existing source edit/history owner. Library files remain explicit configuration
data and are never ingested on startup. Literal variables use quoted shell
assignments or whole argv tokens. End headers select result publication.

See [calls and results](../../../../../docs/reference/org-source-blocks.md#calls-and-results)
for syntax, adapter contracts, finite bounds and supported result options.

## Contracts Added or Changed

- ORG28: selected named calls, declared libraries and checked result publication.
- ORG14: one source transaction, exact revision admission and retained prior results.

## Files of Interest

- `src/org/execution/input.rs`: native call and library observations.
- `src/org/execution/results.rs`: checked result sites and formatting policy.
- `src/session/execution.rs`: confirmation, owned execution and source publication.
- `tests/org_io/block_results.rs`: real process, call and source transactions.

## Validation

Development checks passed:

- `cargo check --lib`
- `cargo test --test org_io block_results -- --test-threads=4`

Seven integration cases passed, including BOM/CRLF undo, literal call arguments,
explicit library reads, canonical unsaved library precedence, call/target eval
restrictions, and changed-library refusal both before launch and after a real
job. `cargo test --test configuration source_libraries` also passed, validating
layer origin, replacement and finite file/path admission. Development Clippy and
the unchanged source checker pass. The eleven result cases and nine existing execution cases now pass. They cover
output/value/table results, inline replacement, typed argv literals, final-stage
output, structural result boundaries, shared-view anchors, read-only refusal and
ordinary history. The boundary driver supplies consent only after successful
preparation, so refusal itself cannot paste text into source. The real terminal
case now checks block and named-call publication and physical undo; its required
controller validation is pending. Full required exact-head validation is pending.

## Risks and Follow-ups

Confirmed programs have normal host authority. Source/library observations do
not sandbox code. Native result formatting requires the declared external adapter
contract and retains unsupported source instead of evaluating expressions.
