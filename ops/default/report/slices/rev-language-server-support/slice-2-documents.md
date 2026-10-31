# Slice 2: Versioned documents and buffer eligibility

Status: delivered
Delivery set: rev-language-server-support
Plan: [Optional LSP Delivery](../../../plan/rev-language-server-support.md)
Tracker: [Optional LSP Tracker](../../../tracker/rev-language-server-support.md)
Branch: `feat/rev-language-server-support-02-documents`
PR: [#204](https://forge.adrianpk.com/adrianpk/rev/pulls/204)

## Purpose

Deliver the document and buffer-activation contracts of LSP02, LSP05-LSP06
through the existing canonical publication and optional connection owners.

## Delivered Behavior

Eligible visiting buffers send their current unsaved text through didOpen.
Canonical publications send didChange in monotonically increasing protocol
versions; undo/redo also receive new revisions and versions. Incremental ranges
come from the original transaction, use the negotiated UTF-8/UTF-16 units and
apply in descending base order. Full synchronization sends the complete snapshot.
Successful ordinary saves send only the negotiated didSave form, including text
when requested; synchronization never writes the source file.

Aliases and split views share one canonical document. `lsp-disconnect` excludes
that buffer across its views until `lsp-connect` restores eligibility or it closes.
Another attached buffer keeps its connection and synchronization evidence.
Restore never enables globally disabled LSP. Reopening or restoring creates a
fresh document generation; replacing a connection also changes its identity.
Retired pending initialization work cannot publish an earlier snapshot.

Closing retires document work independently of the connection. Healthy servers
remain by default; the selected last-buffer policy shuts down and reaps the owned
child. Oversized documents retain ordinary editing and native completion without
sending a truncated snapshot or launching an unnecessary server.

## Implementation Notes

The canonical edit owner captures immutable pre/post stamps, ropes and the exact
published changes before history grouping. A private post-commit observer queues
them only after all canonical text, points and anchors have published. Inactive
or closed observations perform no snapshot work. JSON conversion, framing and
pipe work stay with the existing connection worker, outside terminal dispatch.
This introduces no second authoritative text or history owner.

One permit follows each observation into the encoded writer queue. Both stages
share the connection's 64-message/16 MiB limit; transition cannot multiply its
capacity. Document snapshots admit 8 MiB each and 32 MiB in total. Failed growth
preserves the source and retires only the limited document. Shutdown/failure
clears pending observations, and activation retirement releases snapshot capacity.

Synchronization evidence includes connection ID, activation generation, canonical
buffer/revision and protocol version. Exact-revision evidence is unavailable
while changes are pending, after exclusion or for an unattached buffer, even if
a shared transport is ready. Server contracts must admit open/close plus full or
incremental changes before receiving unsaved documents.

Position mapping uses the rope's byte/UTF-16 indexes rather than repeatedly
scanning long lines. Checked conversion rejects split UTF-8 bytes, surrogate
halves, CRLF interiors and out-of-line positions, including empty final lines.

## Contracts Added or Changed

The native registry adds working `lsp-connect` and `lsp-disconnect` commands.
Connection status and exact document-version evidence remain separate. The
existing `lsp.shutdown_on_last_buffer` setting now controls attachment cleanup.
The complete configuration sample and command/binding/default catalogs reflect
these delivered contracts; no new preference or dependency is introduced.

## Files of Interest

- `src/buffer/edit/observe.rs`, `src/buffer/edit.rs`: canonical publication evidence.
- `src/lsp/documents.rs`: versioned synchronization, activation and snapshot bounds.
- `src/lsp/attachment.rs`: canonical attachment, exclusion and lifecycle ownership.
- `src/lsp/budget.rs`, `src/lsp/inbox.rs`, `src/lsp/io.rs`: shared bounded delivery.
- `src/lsp/position.rs`, `src/lsp/protocol.rs`: checked units and negotiated sync/save.
- `src/session/lsp.rs`, `src/session/save.rs`, `src/session/buffers.rs`: native integration.
- `tests/lsp/documents.rs`, `tests/lsp/documents.sh`: owned protocol/process fixtures.

## Validation

Successful development checks:

- `cargo check --locked --offline`: passed during document-owner integration.
- `cargo test --locked --test lsp`: all 17 LSP integration tests passed.
- `cargo test --locked --test lsp --lib lsp::`: both resource-accounting unit tests
  passed; that name filter selected no integration cases.
- `cargo test --locked --test lsp checked_positions`: checked conversion passed.
- `cargo clippy --locked --test lsp -- -D warnings`: passed.
- `cargo run --locked --quiet --example documentation-check`: 330-command
  inventories, 211 configuration entries and 63 reachable pages passed.

Changed production owners pass the existing source-quality bounds. The
unattached-buffer regression failed before its correction and passed afterward;
the final-line conversion regression was also corrected without weakening its
round-trip assertions. Final ordered slice evidence is recorded through the
controller after the canonical PR-reference follow-up.

Coverage includes CRLF, non-BMP/combining text, full and incremental changes,
multiple ranges, undo/redo, unsaved opening, negotiated save forms, canonical
aliases, split views, exclusion/restoration, close/reopen, pending-generation
retirement, last-buffer shutdown, snapshot/queue limits and source immutability.
Owned fixture children finish and are reaped after their checks. The protocol
fixture establishes deterministic local behavior; real-server acceptance remains
with its planned slice.

T2.1 is commit `7049f12fb921e55d832fc1d266138189c0274c5c`.
T2.2 is commit `8d3369ca1d73a9fe1d64467f539f659a8675cb44`, which introduces this report with the completed buffer-local status boundary and shared-connection exclusion regressions.

## Risks and Follow-ups

Semantic candidates, request freshness and native fallback enter Slice 3.
Resolved edits and complete interoperability/acceptance remain in Slices 4-6.

## Integration

Canonical PR [#204](https://forge.adrianpk.com/adrianpk/rev/pulls/204) merged
through Rebase + Fast-forward into dev at
`1fe415317613496a1e90c33a06757173c0cd99b5`. Exact configured validation
receipt `XXWQOCGBQ2UDSAMYPZW4YFGD4V` passed the three ordered commands
on that clean candidate, including 297 integration tests. The controller recorded
the merge in Acta.
