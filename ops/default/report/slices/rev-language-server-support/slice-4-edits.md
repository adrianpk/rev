# Slice 4: Resolved completion and atomic additional edits

Status: reviewing
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-04-edits`
PR: pending canonical creation

## Purpose

Complete LSP03's selected-item resolve and same-document edit contract, LSP02
request freshness and CMP04 canonical atomic history through the shared session.

## Delivered Behavior

Acceptance resolves only the selected item when the server advertises resolve.
Preselection/navigation send no resolve, and repeated acceptance while pending
cannot start another. Shared list-default data, insertion format and primary
edit ranges are materialized on the connection worker for that selected item.
The client advertises only `additionalTextEdits` as a mutable resolve property;
changes to the original label, matching/order or primary insertion contract are
rejected. Explicit snippet support remains false.

Additional edits supplied in the completion list or its resolve response must
use current-document UTF-8/UTF-16 positions. Valid imports and the primary
replacement prepare and publish as one canonical transaction against the same
originating revision. One undo/redo handles the whole batch. Cursor placement
accounts for additional insertions/deletions before the primary; other views,
anchors and document notifications retain canonical publication ownership.
Synchronization observes one version advance for the atomic batch. No file save
or separate text/history owner is introduced.

Admission permits at most 32 additional edits and 256 KiB total inserted text
per item, including the existing 64 KiB primary limit. All ranges must be valid
and disjoint, including rejection of equal-start insertions. Malformed, reversed,
out-of-line, split-unit and CRLF-interior ranges, cross-document forms, commands,
unsupported snippet/indentation formats and oversized batches reject the whole
item. A valid prefix of a rejected batch never publishes independently.

Resolve uses the existing asynchronous connection request owner, finite deadline,
per-buffer pending slot, connection request cap and bounded observation/writer
queue. The terminal retains the original invocation while waiting; ordinary
editing, movement, cancellation, exclusion and disable retire it. Native fallback
is offered for resolve errors, EOF, timeout or unsupported results when the
editing target remains current. Read-only or obsolete targets publish nothing.

## Implementation Notes

Candidates retain immutable proposal metadata by shared reference. Default data
is shared across items, and JSON materialization/merging remains on the worker.
The selected resolve consumes the same monotonic request IDs and permit ledger
as completion, with cancellation and late-response discard. Main-thread polling
exists only while a resolve is pending, using the existing completion deadline
mechanism and key handling; idle behavior is unchanged.

Arrival and publication recheck invocation, view, canonical revision, syntax
observations and synchronized connection/activation identity. Undo restoring the
same bytes still has a newer revision. A response waiting for terminal admission
cannot survive connection replacement. A retiring connection immediately reports
Stopping, preventing a rapid disable/enable from attaching a new generation to
the old worker while it still exposes its previous Ready status.

The EOF regression initially exposed premature invocation invalidation: server
failure removed synchronized readiness before the native fallback could run.
Target validity is now checked separately from server readiness. Successful
semantic publication still requires the full original connection/document
identity; a failed server can select fresh native observations without claiming
semantic readiness or applying a primary-only fragment.

The Unicode fixture retains its exact expected replacement assertion; its wire
end position was corrected to include the closing quote. This fixes fixture
range setup rather than weakening the required source/CRLF/undo outcome.

## Contracts Added or Changed

The advertised profile adds resolve of additionalTextEdits and CompletionList
data defaults. Existing completion, cancellation, LSP controls and TOML settings
remain the interface. The completion reference/how-to and existing Unreleased
entry now describe atomic same-document imports and asynchronous acceptance.

## Files of Interest

- `src/lsp/completion/proposal.rs`: immutable selected proposal and supported resolve merge.
- `src/lsp/completion/admit.rs`, `src/lsp/completion/edits.rs`: whole-item admission and budgets.
- `src/lsp/completion.rs`, `src/lsp/requests.rs`, `src/lsp/protocol.rs`: bounded resolve and capability contract.
- `src/completion/edits.rs`, `src/session/completion/resolve.rs`: atomic edits, cursor and deferred acceptance.
- `src/lsp.rs`: immediate retirement status for generation admission.
- `tests/lsp/edits.rs`, `tests/lsp/candidates.sh`: owned protocol, boundary and cancellation fixtures.
- `src/session/completion/tests.rs`: final read-only admission regression.

## Validation

Successful development checks:

- `cargo test --locked --test lsp edits:: -- --nocapture`: all 10 focused cases passed.
- `cargo test --locked --lib session::completion::tests::readonly_resolved -- --nocapture`: focused read-only rejection passed.
- `cargo clippy --locked --test lsp --test completion -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: 330-command
  inventories, 211 configuration entries and 63 reachable pages passed.

Changed production owners pass the existing safe-Rust source bounds: parameters
at most five, nesting at most three and cyclomatic complexity at most ten.
Final exact-HEAD evidence is recorded through the controller after the canonical
PR-reference follow-up, with precisely the configured order:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test editing --test coordinates --test rectangles --test org_source`
- `git diff --check`

Coverage includes immediate/resolved imports, default data and insert/replace
ranges, UTF-8/UTF-16/non-BMP/combining text and CRLF, exact edit/byte limits and
one excess, whole-item rejection, source/file/point/history preservation, one
undo/redo, no preselection resolve, repeated acceptance, EOF/error/timeout,
read-only admission and cancellation after edit/undo/view/invocation/exclusion/
restore/disable/connection replacement. Owned fixture children finish through
the existing bounded shutdown/reap owner.

T4.1 is commit `07df3dc83553cc0c49131203971a92742966f767`.
T4.2 introduces this report with complete rejection/freshness regressions and
source fallback/retirement boundary corrections.

## Risks and Follow-ups

Shared lifetime/restart and real installed Rust interoperability remain in
Slice 5. Joint executable acceptance and complete optional-LSP guidance remain
in Slice 6. Existing predecessor and integrated-slice evidence is preserved.
