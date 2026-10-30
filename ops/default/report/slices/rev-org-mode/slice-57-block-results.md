# Slice 57: insert checked Org block and call results

Status: reviewing
Delivery set: rev-org-mode
Plan: `ops/default/plan/rev-org-mode.md`
Tracker: `ops/default/tracker/rev-org-mode.md`
Branch: `feat/rev-org-57-block-results`
PR: `#158`

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

Task commits: T57.1 `1128069a7db97ea487e8de2c282d94cc5261b702`,
T57.2 `54250e4481c0155a7811e4c5261f6d1f3ae99112`,
T57.3 `e19238ffd4602b32d058d75d48f506f5eb01376b`.
The terminal correction is `ba9f585ffe4895e6c2fca4c2bf62649d98c7d568`.

Canonical PR: [#158](https://forge.adrianpk.com/adrianpk/rev/pulls/158), targeting `dev`.

## Contracts Added or Changed

- ORG28: selected named calls, declared libraries and checked result publication.
- ORG14: one source transaction, exact revision admission and retained prior results.

## Files of Interest

- `src/org/execution/input.rs`: native call and library observations.
- `src/org/execution/results.rs`: checked result sites and formatting policy.
- `src/session/execution.rs`: confirmation, owned execution and source publication.
- `tests/org_io/block_results.rs`: real process, call and source transactions.

## Validation

Required pre-PR receipt `QXUOLVUPGFOZ3LDB4XAVHPMD5W` passed on
`ba9f585ffe4895e6c2fca4c2bf62649d98c7d568`:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test org_io --test org_editing --test org_source`
- `python3 scripts/check-source.py`
- `git diff --check`

All 482 tests passed: 226 editing, 178 I/O and 78 source; zero failures,
ignored or filtered tests. The unchanged source checker retains safe Rust,
five parameters, three nesting levels and cyclomatic complexity ten.

Terminal receipt `QDDKAM7NKY3KMHNV52YEPAH6DC` passed on the same head:

- `python3 scripts/check-configuration-ui.py --suite terminal --case org-execution`
- `git diff --check`

Both owned RGB and NO_COLOR sessions passed, zero skips. They check physical
block/call execution, confirmation and decline, canonical result replacement,
output inspection, undo, exact unsaved BOM/CRLF source and absence of execution
during visit/export. The terminal case exposed an overlong confirmation prompt;
the correction restores the concise executable label while retaining full source
provenance in the execution outcome.

Eleven result cases cover output/value/table adapters, inline replacement,
literal argv boundaries, final-stage output, structural result boundaries,
shared-view anchors, unsaved canonical library precedence and stale publication.
Existing real-process cases retain cancellation, timeout, child/group cleanup,
read-only refusal, output limits and source fidelity. The boundary driver sends
consent only after successful preparation, preserving the unchanged-source
assertion on refusal.

Development configuration checks passed 127 cases, including library layer
origins, replacement and finite file/path admission. Three shared process cases
also passed. References agree on 172 schema/catalog entries, 54 active RGB roles
and 259 default bindings. The PR-reference follow-up receives fresh focused and
required controller receipts on its exact clean head; final receipts are recorded
in Acta. Controller-owned `dev` integration remains pending.

## Risks and Follow-ups

Confirmed programs have normal host authority. Source/library observations do
not sandbox code. Native result formatting requires the declared external adapter
contract and retains unsupported source instead of evaluating expressions.
