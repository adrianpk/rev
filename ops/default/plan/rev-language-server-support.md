# Optional Language Server Support Delivery

Status: In progress
Date: 2026-10-31
Delivery set: rev-language-server-support
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; approved capability and explicit commit, push and Looper startup request; this breakdown elaborates that authorized scope
Planning baseline: `6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda`
Spec: [Optional Language Server Support](../spec/editor-language-server-support.md)
Model: [Optional Language Server Support Model](../spec/editor-language-server-support-model.md)
Concern: [Pending Work Index](../README.md)
Tracker: [Optional Language Server Support Tracker](../tracker/rev-language-server-support.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-lsp.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 3 — Semantic candidates and native fallback; PR #205 awaiting controller integration

## Outcome and authority

Deliver optional semantic completion using ordinary installed LSP servers after
[Native Autocompletion](rev-autocompletion.md) completes its controller gate and
closure. LSP is disabled by default. All interaction and checked edits use the
predecessor's native completion owner; no second popup, history or text model.
This set closes the combined source-precedence/fallback acceptance.

The owner's approval and startup request authorize planning commits/push to dev,
autonomous task implementation, one canonical PR per slice and Looper-owned
Rebase + Fast-forward merges into dev. Use one ordered Looper sequence: native
completion, exact-candidate gate and closure, then LSP. Do not launch the queued
successor independently. The controller owns task progress, validation receipts,
merge reconciliation and final closure; do not create a parallel tracker.

Main promotion, tags/releases, mirrors, installation, website publication,
server acquisition and unrelated presentation tickets are separate scope.
Do not add Python to required project checks. Use ordinary Conventional Commits
without task identifiers in subjects. Keep repository/configuration English.

## Strategy and prerequisites

6 behavior-first slices keep usable behavior and bounded owner layers
reviewable. Each slice compiles and passes its focused checks; no intentionally
broken contracts, fake working commands or deferred assertions.
Prerequisite: the predecessor's delivered session, popup, source choices,
grammar/Org owners and checked edit contract. Its plan and tracker must be
Completed with passing gate evidence before this set activates. External
rust-analyzer must be present for the real-server gate; the managed installer
is not a prerequisite. Initial protocol subset, roots, encodings, server lifetime,
per-buffer exclusions and finite limits are fixed in the approved specification.
Use serde_json for the typed JSON subset; do not introduce a general LSP
framework, async runtime, bundled server or new workspace model.

Complete sample/default/schema/reference coverage in the slice introducing each
setting/command. Update Unreleased through changelog-normalize once a usable
outcome exists. Use the established User Guide/Diataxis skills for product docs;
keep ordinary editing as the core path and Org as supplementary guidance.

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

Prerequisite: completed predecessor and approved preparation.
Acceptance: LSP01-LSP02 transport/launch layer and LSP04-LSP05 configuration/root contracts.
Owners: typed configuration, owned child process/streams, bounded protocol framing and canonical root selection.

- T1.1: Implement the disabled-by-default stdio client, typed JSON framing/initialization and finite worker queues/deadlines. Launch argv without a shell and share connections by server configuration plus canonical root; no server process or polling when disabled.
- T1.2: Add the declared server/root/lifetime settings, installed Rust default, root-marker policy and truthful native status/activation controls. Create tests/lsp.rs with a bounded local process fixture for framing, initialization and unavailable prerequisites; expose only working commands.

Cover fragmented/combined frames, UTF-8 framing length, malformed/oversized headers/body, EOF/stderr bursts, failed initialization, missing executables, root boundaries and alias reuse. No downloads, installer or language allowlist.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test configuration --test files --test commands`
- `git diff --check`

## Slice 2: Versioned documents and buffer eligibility

Prerequisite: Slice 1 integrated.
Acceptance: LSP02, LSP05-LSP06 and document ownership parts of LSP01.
Owners: canonical transactions/revisions, file URI/encoding conversion, session buffer lifecycle and per-buffer activation.

- T2.1: Implement didOpen/didChange/didClose and negotiated save notifications for full/incremental sync. Map UTF-8/UTF-16 using the original canonical transaction, synchronize unsaved text before requests and qualify protocol versions by connection/document generation.
- T2.2: Implement current-buffer disconnect/connect across views, exclusion generation, reactivation/close behavior and normal automatic attachment. Retire document work without disrupting other attachments; preserve native completion when excluded or over limits.

Cover CRLF, non-BMP and combining text, multiple edits, undo, unsaved documents, split views, canonical aliases, excluded buffer reactivation, close/reopen and connection reuse. Never save to synchronize or assume wire offsets are display cells.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test coordinates --test editing --test files --test windows`
- `git diff --check`

## Slice 3: Semantic candidates and native fallback

Prerequisite: Slice 2 integrated.
Acceptance: LSP03 and CMP02 shared-source selection.
Owners: LSP completion request mapping, shared session/filter/popup and canonical literal edit admission.

- T3.1: Request semantic completion for eligible current revisions, translate plain insertion/TextEdit/InsertReplaceEdit and supported list defaults, and preserve server order within common match classes. Handle trigger characters, incomplete lists and source readiness without a separate UI.
- T3.2: Implement native fallback for valid empty, no admitted items, absent/unsupported capability, timeout, exclusion and unavailable connection. Preserve distinct outcomes and reject stale session/revision/view/activation/connection replies.

Use both current-buffer syntax and native Org fallback. Confirm preselection alone never edits, false snippet capability is advertised, unsupported items are omitted and no stale response revives a cancelled popup. Candidate source changes do not duplicate undo or replace user text implicitly.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test commands --test configuration --test syntax`
- `git diff --check`

## Slice 4: Resolved completion and atomic additional edits

Prerequisite: Slice 3 integrated.
Acceptance: LSP02-LSP03 complete edit contract and CMP04.
Owners: selected-item resolve lifecycle, range/encoding validation and canonical prepared multi-edit history.

- T4.1: Resolve an eligible selected item at most once for supported fields and admit bounded current-document additionalTextEdits. Prepare the primary and all non-overlapping additional edits against one originating revision before publishing one undo transaction.
- T4.2: Reject unsupported snippets/commands/cross-document forms, overlapping or malformed ranges, oversized payloads and obsolete resolve responses. Keep ordinary editing usable during resolve and restore native behavior on cancel/failure.

Cover automatic imports with primary insertion, insert/replace variants, Unicode ranges, read-only documents, in-flight user edits, same bytes with a newer generation, cancel/exclusion during resolve, one undo/redo and total edit budgets.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test editing --test coordinates --test rectangles --test org_source`
- `git diff --check`

## Slice 5: Shared lifecycle and real Rust interoperability

Prerequisite: Slice 4 integrated.
Acceptance: LSP01, LSP04-LSP06 and bounded LSP02 failure handling.
Owners: connection attachment sets, native process cleanup, runtime status/restart and existing completion source state.

- T5.1: Complete healthy idle retention, optional last-buffer shutdown, bounded disable/exit cleanup, explicit restart and failed-state suppression. Handle server requests/notifications under the selected capability profile without granting edit or command authority.
- T5.2: Add real installed rust-analyzer integration on an isolated minimal Rust fixture, plus controlled process failures for cancellation, queue/backpressure, EOF and shutdown. Verify other configured/unbundled language entries remain admissible without a grammar dependency.

Verify one server per config/root across buffers/views, no automatic respawn loop, no leaked owned child, no polling while idle, no hanging editor exit and native fallback after failure. Record actual server/toolchain; do not treat a deterministic fixture as real-server proof.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test recovery --test files --test windows --test configuration`
- `git diff --check`

## Slice 6: Joint source acceptance and guidance

Prerequisite: Slice 5 integrated.
Acceptance: LSP01-LSP06 and CMP01-CMP09 across both delivered sets.
Owners: joint native/real-server acceptance, Rust/Bash PTY, resource evidence and concise optional-LSP guidance.

- T6.1: Add examples/lsp-acceptance.rs and scripts/check-lsp.sh. Exercise disabled startup, enable/attach, unsaved semantic completion, resolved import/undo, buffer exception/restore, valid-empty fallback, server failure, shared retention, disable and exit. Run real rust-analyzer and bounded fixture cases with finite deadlines.
- T6.2: Update optional-LSP how-to/reference and the existing completion guide, full config/catalog and Unreleased notes. Record integrated executable size and separate server resources; reconcile native completion acceptance after source switching passes without overwriting predecessor gate evidence.

Own temporary sources, HOME/XDG roots, PTYs and server children. No network is required for the gate: preflight must find the installed server, and the Rust fixture must work with a local toolchain/offline dependencies. No user installation or public promotion.

Required ordered checks:

- `cargo fmt --all -- --check`
- `cargo test --locked --test lsp --test completion --test language_assets --test coordinates --test editing --test commands --test configuration --test recovery`
- `bash -n scripts/check-lsp.sh`
- `bash scripts/check-lsp.sh`
- `cargo run --locked --quiet --example documentation-check`
- `git diff --check`

## Final gate and execution cost

The metadata Full gate is the exact immutable controller command. Run it once
after all 6 PRs integrate into clean dev. Shared canonical edits, process/IO ownership, coordinates and completion lifecycle
justify the all-target native suite here. During slices use only their recorded
focused checks and concrete additional regressions warranted by changed owners.
Do not append all historical Org, export, wrap or bookmark journey scripts.

Expected warm execution is a few minutes, dominated by Cargo compilation and
real Rust-server initialization. First builds may take longer for new dependencies.
Record actual timing/resources instead of treating this estimate as evidence.
Server fixtures and the minimal Rust source tree must be ready before prepare_gate;
ordinary server processes must be cleaned up even on failed assertions.
The acceptance wrapper batches the existing source-check example over all production Rust files, marks actual crate roots explicitly and checks introduced example roots. A checker invocation with no source arguments is not evidence. No checks have passed merely because they are listed here.

Use exact-HEAD controller validation and preserve unchanged gate definitions,
checkpoint ownership, budgets and conversation. A failed final gate follows
Looper's explicitly authorized bounded repair/retry path; do not patch primary
dev to bypass it or weaken acceptance. No unsolicited successor set is authorized.

## Closure and successor boundary

Commit the approved spec/model, plan/tracker and index before init/start.
Keep local Looper configuration outside the repository; pin these exact slice
identities and ordered checks before the first checkpoint. Reports are created
by their implementation slices at the canonical map paths, not predeclared as
delivered. Before startup, verify clean canonical dev, installed tool help and
actual detached preflight.

After the full gate passes, closure may change only this plan/tracker,
its 6 configured reports, its specification/model and the pending-work index, plus the native completion specification/model to record joint acceptance.
No source, test, script, dependency, product documentation or changelog belongs
to the post-gate closure commit. Preserve exact candidate/receipt and historical
delivery evidence. Mark combined native/LSP acceptance delivered only after the joint gate passes.
The controller can hand off only after verified closure; neither set promotes main.
