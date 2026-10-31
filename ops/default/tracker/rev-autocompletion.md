# Native Autocompletion Tracker

Status: In progress
Date: 2026-10-31
Delivery set: rev-autocompletion
Slice strategy: behavior-first
Approved: product owner, 2026-10-31; approved capability and explicit commit, push and Looper startup request; this breakdown elaborates that authorized scope
Planning baseline: `6e38bb95fdf0ff81cec6d48a6e8b3bdaa9130fda`
Spec: [Native Autocompletion](../spec/editor-autocompletion.md)
Model: [Native Autocompletion Model](../spec/editor-autocompletion-model.md)
Concern: [Pending Work Index](../README.md)
Plan: [Native Autocompletion Plan](../plan/rev-autocompletion.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --all-targets && bash scripts/check-completion.sh && cargo run --locked --quiet --example documentation-check && git diff --check`
Active slice: Slice 8 — Native acceptance, footprint and guidance; in progress
Execution owner: Looper; checkpoint and canonical merge evidence own progress

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Checked manual completion | `feat/rev-autocompletion-01-session` | `feat(completion): add checked manual Rust completion` | `ops/default/report/slices/rev-autocompletion/slice-1-session.md` |
| Slice 2 | Cursor popup and automatic invocation | `feat/rev-autocompletion-02-interaction` | `feat(completion): show automatic cursor suggestions` | `ops/default/report/slices/rev-autocompletion/slice-2-interaction.md` |
| Slice 3 | Rust, C and Go structural sources | `feat/rev-autocompletion-03-systems` | `feat(completion): cover Rust C and Go declarations` | `ops/default/report/slices/rev-autocompletion/slice-3-systems.md` |
| Slice 4 | HTML, CSS, JavaScript and TypeScript | `feat/rev-autocompletion-04-web` | `feat(completion): cover web languages and TSX` | `ops/default/report/slices/rev-autocompletion/slice-4-web.md` |
| Slice 5 | JSON and TOML key completion | `feat/rev-autocompletion-05-formats` | `feat(completion): complete JSON and TOML keys` | `ops/default/report/slices/rev-autocompletion/slice-5-formats.md` |
| Slice 6 | Native Org context completion | `feat/rev-autocompletion-06-org` | `feat(org): complete native document contexts` | `ops/default/report/slices/rev-autocompletion/slice-6-org.md` |
| Slice 7 | Explicit external grammar admission | `feat/rev-autocompletion-07-external` | `feat(syntax): load registered external grammar assets` | `ops/default/report/slices/rev-autocompletion/slice-7-external.md` |
| Slice 8 | Native acceptance, footprint and guidance | `feat/rev-autocompletion-08-acceptance` | `test(completion): validate native editing and resource bounds` | `ops/default/report/slices/rev-autocompletion/slice-8-acceptance.md` |

## Slice 1: Checked manual completion

Acceptance: CMP01, CMP03-CMP04 and the manual part of CMP05.

- [x] T1.1: Implement a revision-qualified completion session and a real manual Rust path using explicit declaration queries on the current buffer. Keep candidate display/insertion identity separate; no fake provider registry or public extension interface.
- [x] T1.2: Add completion-at-point with its compact cursor popup, preselection and manual navigation/accept/cancel; checked literal replacement, deterministic filtering and one-transaction undo. Create tests/completion.rs and cover read-only admission, Unicode/range errors, stale revision/view/session results and source immutability on failure.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#195](https://forge.adrianpk.com/adrianpk/rev/pulls/195) merged
through Rebase + Fast-forward at `15bd4321a5bfbbe36b4831fc0f310ab21b081516`.
Receipt `TCDTYFCOTRHYOQNLEVRIOXDZGJ` passed its three configured checks and
353 tests at that exact candidate. Slice 1 of 8 completed.

## Slice 2: Cursor popup and automatic invocation

Acceptance: CMP01, CMP04-CMP06.

- [x] T2.1: Integrate the cursor popup with automatic requests, single-border/full-row styling, above/below placement and source focus. Implement active-popup navigation/accept/cancel while retaining normal Tab/Enter and Org behavior outside that context.
- [x] T2.2: Add debounced automatic identifier/separator triggers, smart-case matching, suppression/cancellation/paste policy, finite settings and runtime controls. Update the complete sample, keymap/command catalogs and actual behavior references in this slice.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#196](https://forge.adrianpk.com/adrianpk/rev/pulls/196) merged
through Rebase + Fast-forward at `1fc87e42f92a64d467e935343df41d06543bf2ad`.
Receipt `OQMJASNUJJFUKHIDXZMAIPRC27` passed its three configured checks and
397 tests at that exact candidate. Slice 2 of 8 completed.

## Slice 3: Rust, C and Go structural sources

Acceptance: CMP03, CMP05-CMP07; systems-language subset.

- [x] T3.1: Generalize the existing syntax owner without a second parser/index owner. Add the measured C and Go grammars alongside Rust, explicit extraction/highlight queries, language identity and bounded same-language open-buffer reuse.
- [x] T3.2: Implement declaration/scope coverage and deterministic locality/ranking under the specified snapshot/symbol budgets. Invalidate on edits, rename, grammar changes, buffer close and observation eviction; never scan unopened files.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#197](https://forge.adrianpk.com/adrianpk/rev/pulls/197) merged
through Rebase + Fast-forward at `1469a344ebb18d067119951774e2447ab6e941b9`.
Receipt `YY3FXCH6SEAT74Q3OKMFYPD2UQ` passed its three configured checks and
151 tests at that exact candidate. Slice 3 of 8 completed.

## Slice 4: HTML, CSS, JavaScript and TypeScript

Acceptance: CMP03, CMP05-CMP07; web-language subset.

- [x] T4.1: Add the measured HTML, CSS, JavaScript and TypeScript/TSX grammar/query payloads. Distinguish TSX parser selection while preserving existing language preference resolution; include the needed HTML/CSS associations.
- [x] T4.2: Provide the specified declarations, observed tags/attributes/selectors/custom properties and finite standard HTML/CSS catalogs. Admit separator/identifier contexts and preserve graceful plain editing around unsupported syntax.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#198](https://forge.adrianpk.com/adrianpk/rev/pulls/198) merged
through Rebase + Fast-forward at `b93665235c4b29954d2c53f824904dd232e6e1a3`.
Receipt `5GMST5QOZS5KBPNTHKKH4PAPQP` passed its three configured checks and
270 tests at that exact candidate. Slice 4 of 8 completed.

## Slice 5: JSON and TOML key completion

Acceptance: CMP03, CMP05-CMP07; structured-format subset.

- [x] T5.1: Add the measured JSON and TOML grammars, explicit highlight/extraction queries and structural key/table contexts to the existing language catalog.
- [x] T5.2: Complete observed keys/tables through the common interaction, including quoted structural-key admission without ordinary value-string suggestions. Cover nested/duplicate keys, escaped names, invalid documents and unsaved edits.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#199](https://forge.adrianpk.com/adrianpk/rev/pulls/199) merged
through Rebase + Fast-forward at `843b4259a89ce05ea452adbd878b9c434240f110`.
Receipt `5IYDBO5SS52SD334S6BUXLVBGL` passed its three configured checks and
202 tests at that exact candidate. Slice 5 of 8 completed.

## Slice 6: Native Org context completion

Acceptance: CMP08 with CMP01 and CMP04-CMP07.

- [x] T6.1: Adapt current Org observations and effective configuration for TODO positions, tags, property keys, supported directives/link types and current-document heading targets. Reuse the existing parser and document model.
- [x] T6.2: Integrate structural-prefix triggering, checked replacement and unsaved target invalidation. Preserve literal blocks, ordinary prose, Org Tab actions outside an active popup and declared-language completion in source-edit buffers.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#200](https://forge.adrianpk.com/adrianpk/rev/pulls/200) merged
through Rebase + Fast-forward at `28a43ed88c143b0ef131806bdb48a159ec3fbf57`.
Receipt `QEPJ6DEWDQJQVN5OYGUKUWZBWC` passed its three configured checks and
646 tests at that exact candidate. Slice 6 of 8 completed.

## Slice 7: Explicit external grammar admission

Acceptance: CMP07, CMP09 and the R31 delta; preserve R35.

- [x] T7.1: Add the pinned safe tree-sitter-loader API with default features disabled. Implement explicit versioned parser/symbol/highlight/completion registration, lazy one-time bounded admission and bundled/plain fallback. Define and document the finite query capture profile.
- [x] T7.2: Test a real precompiled grammar replacement and an unbundled language through the actual safe loader. Reject missing/wrong-symbol/ABI/query assets without losing fallback; prove startup registration and process-lifetime retention. Reconcile the named R31 baseline restriction while preserving historical gate evidence and R35.
- [x] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

Canonical PR [#201](https://forge.adrianpk.com/adrianpk/rev/pulls/201) merged
through Rebase + Fast-forward at `706590012c848231b0b0ff6cb9ba94e9eb4a2233`.
Receipt `YGFKP3HPVAFOMYLF3YIW3XSRGF` passed its three configured checks and
291 tests at that exact candidate. Slice 7 of 8 completed.

## Slice 8: Native acceptance, footprint and guidance

Acceptance: CMP01 and CMP03-CMP09 jointly; CMP02 semantic switching reserved for the LSP successor.

- [x] T8.1: Add examples/completion-acceptance.rs and scripts/check-completion.sh using the existing Rust/Bash PTY approach. Exercise manual/automatic selection, edit/undo/cancel, all ten parsers, Org contexts, external admission and native fallback. Measure integrated release bytes against the recorded baseline, idle CPU/work, bounded indexing and burst responsiveness.
- [x] T8.2: Complete concise completion tutorial/how-to/reference pages with small visual feedback examples, complete sample/command catalogs and Unreleased notes through the established skills. Record actual capability limits and distinguish syntax candidates from semantic LSP. Keep Org supplementary to ordinary editing.
- [ ] Verify the plan's ordered exact-HEAD checks, canonical PR integration and delivered report.

The [Slice 8 report](../report/slices/rev-autocompletion/slice-8-acceptance.md)
records implementation, focused receipts and actual normal-release measurements.
Canonical [PR #202](https://forge.adrianpk.com/adrianpk/rev/pulls/202) is open;
final candidate validation and controller integration remain in progress.

## Final gate and closure

- [ ] Verify all 8 canonical slice merges and clean synchronized dev.
- [ ] Run the pinned Full gate on that exact integrated candidate.
- [ ] Record candidate, command, outcome/receipt, actual resource evidence and any authorized repair.
- [ ] Close only configured operational documentation and preserve predecessor/historical evidence.

- [ ] Record native acceptance separately from semantic switching, then permit the already approved LSP successor.

Native completion uses no server, directory scan or persistent index. Before
prepare_gate, compile the owned grammar fixtures and establish Rust/Bash/PTY
prerequisites. Keep temporary HOME/XDG/source trees isolated from user data.

The controller checkpoint and canonical merge evidence own delivery progress.
The full delivery-set gate and closure remain pending. Full acceptance and remaining
boundaries follow the linked plan/specification; no additional ticket is queued.
