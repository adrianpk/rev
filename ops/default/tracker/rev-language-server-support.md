# Optional Language Server Support Tracker

Status: In progress
Date: 2026-10-31
Delivery set: rev-language-server-support
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; approved capability and explicit commit, push and Looper startup request; this breakdown elaborates that authorized scope
Planning baseline: `6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda`
Spec: [Optional Language Server Support](../spec/editor-language-server-support.md)
Model: [Optional Language Server Support Model](../spec/editor-language-server-support-model.md)
Concern: [Pending Work Index](../README.md)
Plan: [Optional Language Server Support Plan](../plan/rev-language-server-support.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-lsp.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 5 — Shared lifecycle and real Rust interoperability
Execution owner: Looper; checkpoint and canonical merge evidence own progress

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Optional owned server transport | `feat/rev-language-server-support-01-transport` | `feat(lsp): add bounded optional server connections` | `ops/default/report/slices/rev-language-server-support/slice-1-transport.md` |
| Slice 2 | Versioned documents and buffer eligibility | `feat/rev-language-server-support-02-documents` | `feat(lsp): synchronize canonical unsaved buffers` | `ops/default/report/slices/rev-language-server-support/slice-2-documents.md` |
| Slice 3 | Semantic candidates and native fallback | `feat/rev-language-server-support-03-candidates` | `feat(lsp): feed semantic results into native completion` | `ops/default/report/slices/rev-language-server-support/slice-3-candidates.md` |
| Slice 4 | Resolved completion and atomic additional edits | `feat/rev-language-server-support-04-edits` | `feat(lsp): apply resolved same-document completion edits` | `ops/default/report/slices/rev-language-server-support/slice-4-edits.md` |
| Slice 5 | Shared lifecycle and real Rust interoperability | `feat/rev-language-server-support-05-lifetime` | `feat(lsp): retain and recover owned server lifetimes` | `ops/default/report/slices/rev-language-server-support/slice-5-lifetime.md` |
| Slice 6 | Joint source acceptance and guidance | `feat/rev-language-server-support-06-acceptance` | `test(lsp): validate semantic completion and native fallback` | `ops/default/report/slices/rev-language-server-support/slice-6-acceptance.md` |

## Slice 1: Optional owned server transport

Acceptance: LSP01-LSP02 transport/launch layer and LSP04-LSP05 configuration/root contracts.

- [x] T1.1: Implement the disabled-by-default stdio client, typed JSON framing/initialization and finite worker queues/deadlines. Launch argv without a shell and share connections by server configuration plus canonical root; no server process or polling when disabled.
- [x] T1.2: Add the declared server/root/lifetime settings, installed Rust default, root-marker policy and truthful native status/activation controls. Create tests/lsp.rs with a bounded local process fixture for framing, initialization and unavailable prerequisites; expose only working commands.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Implementation: T1.1 `dc8b3428da348d5f259c69134b7a742739fe8785`; T1.2 `37dff9caf62315049ee65fb98348548a52dd956c` introduces the Slice 1 report. Canonical PR: [#203](https://forge.adrianpk.com/adrianpk/rev/pulls/203), merged at `6306455453c3e45a6ee5dd7a236d5615f239b62d`. Exact configured receipt `Z2RUGLJVPJIWU4JMTREAUSC3DN` passed all three commands and 382 tests; report delivered.

## Slice 2: Versioned documents and buffer eligibility

Acceptance: LSP02, LSP05-LSP06 and document ownership parts of LSP01.

- [x] T2.1: Implement didOpen/didChange/didClose and negotiated save notifications for full/incremental sync. Map UTF-8/UTF-16 using the original canonical transaction, synchronize unsaved text before requests and qualify protocol versions by connection/document generation.
- [x] T2.2: Implement current-buffer disconnect/connect across views, exclusion generation, reactivation/close behavior and normal automatic attachment. Retire document work without disrupting other attachments; preserve native completion when excluded or over limits.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Implementation: T2.1 `7049f12fb921e55d832fc1d266138189c0274c5c`; T2.2 `8d3369ca1d73a9fe1d64467f539f659a8675cb44` introduces the Slice 2 report. Canonical PR: [#204](https://forge.adrianpk.com/adrianpk/rev/pulls/204). Merged at `1fe415317613496a1e90c33a06757173c0cd99b5`; exact configured receipt `XXWQOCGBQ2UDSAMYPZW4YFGD4V` passed all three commands and 297 tests; report delivered.

## Slice 3: Semantic candidates and native fallback

Acceptance: LSP03 and CMP02 shared-source selection.

- [x] T3.1: Request semantic completion for eligible current revisions, translate plain insertion/TextEdit/InsertReplaceEdit and supported list defaults, and preserve server order within common match classes. Handle trigger characters, incomplete lists and source readiness without a separate UI.
- [x] T3.2: Implement native fallback for valid empty, no admitted items, absent/unsupported capability, timeout, exclusion and unavailable connection. Preserve distinct outcomes and reject stale session/revision/view/activation/connection replies.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Implementation: T3.1 `1f97405488ae3213e8226fbb2337f9cc00e236e0`; T3.2 `6ebbfbf69dbbfac6775394108fb70aaacd488b72` introduces the Slice 3 report with source outcomes, admission/policy regressions and completion guidance. Canonical PR: [#205](https://forge.adrianpk.com/adrianpk/rev/pulls/205). Merged at `df2cef8db7e2065bb6c5ddf888abdf86b2d1e63a`; receipt `X2QWAAQE6DLNMYYRXWEUTR3HSK` passed the exact three configured commands and 402 tests; report delivered.

## Slice 4: Resolved completion and atomic additional edits

Acceptance: LSP02-LSP03 complete edit contract and CMP04.

- [x] T4.1: Resolve an eligible selected item at most once for supported fields and admit bounded current-document additionalTextEdits. Prepare the primary and all non-overlapping additional edits against one originating revision before publishing one undo transaction.
- [x] T4.2: Reject unsupported snippets/commands/cross-document forms, overlapping or malformed ranges, oversized payloads and obsolete resolve responses. Keep ordinary editing usable during resolve and restore native behavior on cancel/failure.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Implementation: T4.1 `07df3dc83553cc0c49131203971a92742966f767`; T4.2 `0857783862901aeef22c3b9d73c99027cfeff6b4` introduces the Slice 4 report with atomic edit boundaries, source fallback and obsolete-generation regressions. Canonical PR: [#206](https://forge.adrianpk.com/adrianpk/rev/pulls/206). Merged at `d94a5f3b474f084a37f8d084b6d435c0e550b8bc`; receipt `G56H6AFFQXGEPLHIYN7UHRLWVG` passed the exact three configured commands and 325 tests; report delivered.

## Slice 5: Shared lifecycle and real Rust interoperability

Acceptance: LSP01, LSP04-LSP06 and bounded LSP02 failure handling.

- [x] T5.1: Complete healthy idle retention, optional last-buffer shutdown, bounded disable/exit cleanup, explicit restart and failed-state suppression. Handle server requests/notifications under the selected capability profile without granting edit or command authority.
- [x] T5.2: Add real installed rust-analyzer integration on an isolated minimal Rust fixture, plus controlled process failures for cancellation, queue/backpressure, EOF and shutdown. Verify other configured/unbundled language entries remain admissible without a grammar dependency.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Implementation: T5.1 `8081de017dc5a2f5d4bb2e5d3e56a6725f57f2c7`; T5.2 `846071a8e318814a6062a065c092e267e6b8cf5e` introduces the Slice 5 report with installed rust-analyzer unsaved completion, undo/disk/cleanup proof and explicit-restart cancellation. Canonical PR: [#207](https://forge.adrianpk.com/adrianpk/rev/pulls/207). Final exact-HEAD validation is recorded through the controller after this PR-reference follow-up.

## Slice 6: Joint source acceptance and guidance

Acceptance: LSP01-LSP06 and CMP01-CMP09 across both delivered sets.

- [ ] T6.1: Add examples/lsp-acceptance.rs and scripts/check-lsp.sh. Exercise disabled startup, enable/attach, unsaved semantic completion, resolved import/undo, buffer exception/restore, valid-empty fallback, server failure, shared retention, disable and exit. Run real rust-analyzer and bounded fixture cases with finite deadlines.
- [ ] T6.2: Update optional-LSP how-to/reference and the existing completion guide, full config/catalog and Unreleased notes. Record integrated executable size and separate server resources; reconcile native completion acceptance after source switching passes without overwriting predecessor gate evidence.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

## Final gate and closure

- [ ] Verify all 6 canonical slice merges and clean synchronized dev.
- [ ] Run the pinned Full gate on that exact integrated candidate.
- [ ] Record candidate, command, outcome/receipt, actual resource evidence and any authorized repair.
- [ ] Close only configured operational documentation and preserve predecessor/historical evidence.

- [ ] Establish CMP02 source switching with real Rust completion and controlled empty/unavailable responses; close the joint native/LSP acceptance.

Native completion must close before this set activates. Real rust-analyzer is
required for the final successful-server journey; a fixture alone cannot satisfy
it. Verify installed server and Rust toolchain in prepare_gate. Keep each child,
PTY and temporary HOME/XDG/source tree owned and bounded; no user stores are used.

No implementation, gate receipt or merge is claimed by this initial tracker.
Looper alone updates progress after startup. Full acceptance and remaining
boundaries follow the linked plan/specification; no additional ticket is queued.
