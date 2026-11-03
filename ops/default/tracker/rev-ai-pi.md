# Pi Integration and External Reload Tracker

Status: Completed
Completed: 2026-11-03
Date: 2026-11-02
Delivery set: rev-ai-pi
Slice strategy: behavior-first
Approved: product owner, 2026-11-02; proceed with the explained Pi/native-result and configurable external-reload scope through the established autonomous Looper workflow
Planning baseline: `8c6dcb2018cc0f4084c35f2ea4cb34a14de33979`
Spec: [Pi Integration](../spec/editor-ai-pi-integration.md)
Model: [Pi Integration Model](../spec/editor-ai-pi-integration-model.md)
Spec: [External File Changes](../spec/editor-external-file-changes.md)
Model: [External File Changes Model](../spec/editor-external-file-changes-model.md)
Concern: [Pending Work Index](../README.md)
Full gate: `cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked --lib --test files --test configuration --test external_changes --test ai --test pi -- --test-threads=1 && cargo test --locked --test completion --test lsp --test bookmarks -- --test-threads=1 && cargo test --locked --test modal contexts:: -- --test-threads=1 && cargo test --locked --test commands palette::catalog_reference -- --exact && bash scripts/check-ai-pi.sh && cargo run --locked --quiet --example documentation-check && bash scripts/build-user-guide.sh && git diff --check`
Active slice: None — all eight slices delivered; full gate passed
Plan: [Delivery Plan](../plan/rev-ai-pi.md)

## Execution ownership

The owner approved proceeding on 2026-11-02. Looper owns the eight-slice
execution, canonical PR integration, configured gate, bounded recovery and
closure. All eight slices are integrated at their validated heads. Slice entries
and tickets are delivered; the frozen integrated gate passed and the delivery
set is complete. Ready tickets are actionable inputs, not completion evidence.

## Canonical slice map

| Slice | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- |
| Slice 1 | `feat/rev-ai-pi-01-reload` | `feat(files): add checked buffer reload and external change policy` | `ops/default/report/slices/rev-ai-pi/slice-1-reload.md` |
| Slice 2 | `feat/rev-ai-pi-02-observation` | `feat(files): observe external changes and reload clean buffers` | `ops/default/report/slices/rev-ai-pi/slice-2-observation.md` |
| Slice 3 | `feat/rev-ai-pi-03-rpc` | `feat(ai): connect Pi RPC for selection explanations` | `ops/default/report/slices/rev-ai-pi/slice-3-rpc.md` |
| Slice 4 | `feat/rev-ai-pi-04-results` | `feat(ai): apply generated results through native editing` | `ops/default/report/slices/rev-ai-pi/slice-4-results.md` |
| Slice 5 | `feat/rev-ai-pi-05-discovery` | `feat(ai): delegate bounded project discovery to Pi` | `ops/default/report/slices/rev-ai-pi/slice-5-discovery.md` |
| Slice 6 | `feat/rev-ai-pi-06-sessions` | `feat(ai): retain isolated Pi sessions and model selection` | `ops/default/report/slices/rev-ai-pi/slice-6-sessions.md` |
| Slice 7 | `feat/rev-ai-pi-07-definitions` | `feat(ai): save declarative commands in user configuration` | `ops/default/report/slices/rev-ai-pi/slice-7-definitions.md` |
| Slice 8 | `feat/rev-ai-pi-08-acceptance` | `docs(ai): document and verify Pi integration and file reload` | `ops/default/report/slices/rev-ai-pi/slice-8-acceptance.md` |

## Tasks

### Slice 1: Checked file reload

State: delivered.
Ticket: [Checked file reload](../ticket/solved/20261102211106-reload-visiting-files-safely.md)
Requirements: EFC01, EFC03-EFC07.
Dependencies: delivered native owners.
Branch: `feat/rev-ai-pi-01-reload`
PR: [#229](https://forge.adrianpk.com/adrianpk/rev/pulls/229).
Report: `ops/default/report/slices/rev-ai-pi/slice-1-reload.md`

- [x] T1.1: Add the proposed file-policy settings and explicit revert-buffer command through the native file/buffer owners, with complete startup schema/default/sample coverage.
- [x] T1.2: Capture and validate fresh disk bytes asynchronously; preserve unsaved text through revision-bound discard consent and publish one native undoable reload with checked clean-state evidence.
- [x] T1.3: Cover invalid/missing/readonly sources, disk and typing races, undo/redo, all shared views and unchanged-byte observations without a second text or history owner.

Implementation commits: `256c17b2b360bb72b62ed42fa79581afcb7d1898`
(configuration) and `1dbe66439e2273fda440996e08bbd399751925af` (native reload), plus
`7232bd69a5f58aebda0407b9df6bea5fd7fbb7b4` (regressions/report).
Focused validation passed: configuration reload-policy composition/startup (2),
external reload/native command cases (11), all-target compilation and documentation
catalogs (418 commands, 223 configuration entries; 68 pages/414 links).
PR #229 merged at `2f87ae673641d72abac12a985ec1630b2d542213` on
2026-11-02T21:38:02Z. Final clean-head receipt `RM7KIWHX3TU37HLMBDT57F3D22`
passed all four configured commands and 227 tests before controller integration. This is
Slice 1 evidence, not the integrated delivery-set gate.

### Slice 2: External change observation

State: delivered.
Ticket: [External change observation](../ticket/solved/20261102211107-observe-external-file-changes.md)
Requirements: EFC01-EFC08.
Dependencies: Slice 1 integrated.
Branch: `feat/rev-ai-pi-02-observation`
PR: [#230](https://forge.adrianpk.com/adrianpk/rev/pulls/230).
Report: `ops/default/report/slices/rev-ai-pi/slice-2-observation.md`

- [x] T2.1: Use the already locked rustix filesystem/event APIs for bounded parent-directory notifications and native session wake integration; no recursive project watcher or periodic content reads.
- [x] T2.2: Apply reload/ask policies, coalesced per-buffer notices, dirty conflict protection and explicit reload; retain subscriptions through atomic rename, deletion/recreation and shared/hidden buffers.
- [x] T2.3: Verify native notifications, overflow/registration limits, cleanup, idle behavior and canonical invalidation of completion, bookmarks, Org projections and LSP state.

Implementation: `16fd877abf0196cb934edf3859b6c0aa81c1c67e` lands T2.1/T2.2
together to keep the notification/reload owner coherent.
T2.3 tests/report: `da61e4da9c6974d1e57cecf99fdd2f083e7fc512`. Focused checks passed:
nine real kernel/session observation cases, two controlled worker ownership/bounds
cases, native signal typing continuity, both-profile real Linux PTY wake/idle
checks and LSP reload/completion invalidation. Documentation catalogs passed.
PR #230 merged at `93e22753752d8304e946833b7a2f9c8b9d302518` on
2026-11-02T22:28:14Z. Final clean-head receipt `FL7VQPPICASDP37MYGALPWKXXT`
passed all four configured commands and 142 tests. The focused late-notice
regression also passed. This is Slice 2 evidence, not the integrated set gate.

### Slice 3: Pi RPC explanation

State: delivered.
Ticket: [Pi RPC explanation](../ticket/solved/20261102211108-connect-pi-rpc-for-explanation.md)
Requirements: PI01-PI04, PI07; AI01, AI05, AI07, AI16.
Dependencies: Slice 2 integrated.
Branch: `feat/rev-ai-pi-03-rpc`
PR: [#231](https://forge.adrianpk.com/adrianpk/rev/pulls/231).
Report: `ops/default/report/slices/rev-ai-pi/slice-3-rpc.md`

- [x] T3.1: Admit typed AI/Pi configuration and own one lazy version-checked RPC child with bounded LF framing, independent streams, continuous consumption and explicit operation generations.
- [x] T3.2: Deliver ai-explain for a captured selection through a fresh input-only session and a native read-only result buffer; keep the limited intermediate context visible until directory discovery lands.
- [x] T3.3: Handle prompt acceptance versus settled completion, errors, unavailable authentication, C-g/modal Escape cancellation and process shutdown while ordinary editing remains responsive.

Implementation: `8725da90411ada30b14fb5371c9be4dd06f7d8f7` (configuration),
`9cc5443b89a5bb11f89e3b5cb769b099f2af3049` (RPC/lifecycle) and
`0e81da925428984974e1f23251d1972566020a69` (native selection/result integration).
Focused protocol/configuration/ownership checks and real offline Pi 1.3.0
get_state/shutdown probes passed. Catalogs passed 419 commands and 230 settings.
The Slice 3 report retains measured process observations and exact focused
commands. PR #231 merged at `eb2b45803856e7707ffc122ad697ef02d84a9fc7` on
2026-11-02T23:02:57Z. Receipt `N77GD6JFCVJQGKPXJ7QCEPDOZA` passed all
four configured checks and 155 tests (7 AI, 143 configuration, 5 Pi). This is
Slice 3 evidence, not the integrated delivery-set gate.

Report introduction and remaining lifecycle regressions: `18dfaa6cc4c0ef0875b9330dc27bc325733e911f`.

### Slice 4: Checked AI editing

State: delivered.
Ticket: [Checked AI editing](../ticket/solved/20261102211109-apply-ai-results-through-native-editing.md)
Requirements: PI03, PI06, PI07; AI03, AI05-AI09, AI14-AI17.
Dependencies: Slice 3 integrated.
Branch: `feat/rev-ai-pi-04-results`
PR: [#232](https://forge.adrianpk.com/adrianpk/rev/pulls/232).
Report: `ops/default/report/slices/rev-ai-pi/slice-4-results.md`

- [x] T4.1: Add ai-prompt and ai-implement with explicit point/selection capture and the bounded version-1 generated-text contract; display proposals through the native transient-buffer owner.
- [x] T4.2: Deliver explicit ai-apply-result and ai-discard-result with source revision/read-only admission, one native undo step and no implicit source save or model-selected target.
- [x] T4.3: Fence partial, stale, cancelled, duplicated and oversized results, including local edits, external reloads and focus changes during generation.

Implementation: `b9dbf29d6b89f49d1735c15e3104470fba9de1c2` delivers T4.1/T4.2
as one capture/proposal/native-publication transition. Ten new focused integration
cases and two private live-admission tests passed. They cover both profiles,
source/result guards, ordinary undo/shared views and real external-file changes.
Catalogs passed 423 commands and 230 settings. The Slice 4 report retains exact
commands and observed scope. Canonical PR #232 merged into dev at
`c3468516c904edcd0ea35c94322ea1aa7b57f35a` on 2026-11-02T23:35:02Z.
Receipt `D6ZTJX2FOGYYOPQNEAPR3TGO5N` passed all four configured checks and
38 tests; this is Slice 4 evidence, not the integrated gate.

Report introduction and T4.3 coverage: `4a4fb82f913f1e2a0bf376ffc3932324c34f920c`.

### Slice 5: Guarded directory context

State: delivered.
Ticket: [Guarded directory context](../ticket/solved/20261102211110-delegate-guarded-project-discovery.md)
Requirements: PI02-PI05, PI07; AI06, AI09, AI11-AI13, AI17, AI20.
Dependencies: Slice 4 integrated.
Branch: `feat/rev-ai-pi-05-discovery`
PR: [#233](https://forge.adrianpk.com/adrianpk/rev/pulls/233).
Report: `ops/default/report/slices/rev-ai-pi/slice-5-discovery.md`

- [x] T5.1: Package a small internal Pi JavaScript resource with rev and load it explicitly through the admitted Pi extension interface; expose only bounded read/list/search operations without a shell or user plugin installation.
- [x] T5.2: Enforce canonical-root and opened-handle path checks, per-operation budgets and fresh unsaved-source precedence while Pi retains all relevance selection and model/context iteration.
- [x] T5.3: Validate traversal, symlink/root replacement races, missing/binary/large sources, disabled external resources and actual Pi tool admission; native API transport/discovery remains deferred.

Implementation: `bee7889f191cd961de5f275b24ca7cc26d375aa7` delivers the
coherent resource/profile/native-observation transition. Six focused native
cases and 27 actual offline Pi checks passed, including real tool execution with
controlled model events, root/file races, unsaved precedence and finite budgets.
The Slice 5 report records exact commands and distinguishes provider evidence.
Canonical PR #233 merged into dev at `9d16dc77b4371c161b1ea6716f31fea767b5393a` on
2026-11-03T00:11:56Z. Receipt `OLNWBRUOJRSR4SWXPZISRJSPLN` passed all four configured
checks and 28 native tests; this is Slice 5 evidence, not the integrated gate.

Report introduction and T5.3 actual-runtime evidence: `984889b23b2c0885ccf8c67accd14b5af9d03467`.

### Slice 6: Private Pi sessions

State: delivered.
Ticket: [Private Pi sessions](../ticket/solved/20261102211111-retain-private-pi-sessions.md)
Requirements: PI01-PI04, PI07, PI08; AI16-AI20.
Dependencies: Slice 5 integrated.
Branch: `feat/rev-ai-pi-06-sessions`
PR: [#234](https://forge.adrianpk.com/adrianpk/rev/pulls/234).
Report: `ops/default/report/slices/rev-ai-pi/slice-6-sessions.md`

- [x] T6.1: Persist bounded schema-1 private session bindings in XDG state while Pi owns its JSONL history; reuse only matching root/model/grant observations.
- [x] T6.2: Implement ai-set-directory, ai-select-model, ai-reset-session and ai-disconnect, with owned idle shutdown, safe restart and fresh ephemeral input-only sessions.
- [x] T6.3: Cover history isolation, changed roots/providers, compaction and session limits, corrupt records, concurrent instances and actual child exit without exporting credentials.

Implementation commits: `ad97bb9e569c5858eb0f3e33a3ef36fd1618d581` (T6.1/T6.2) and
`7ef7b03396602642ca99a408101e867d2b017c78` (T6.3). Nine focused session cases and
27 actual offline Pi resource checks passed, including actual history restoration
and owned exit with a controlled local provider and zero external requests.
Strict library lint and documentation catalogs passed (428 commands, 230
configuration entries; 68 pages/414 links). PR #234 merged at `d4a5c4c90dda580fdad612c91478189ae14ba644`
on 2026-11-03T01:01:25Z. Receipt `Y5YWQGRNOAKJ3ADM3Q6VBHY532` passed all four configured
commands and 37 native tests; the integrated gate was pending at that slice's integration.

Report introduction: `31b7d41f182361cb9c7de84f8a5a66b5d881f881`.

### Slice 7: Reusable AI commands

State: delivered.
Ticket: [Reusable AI commands](../ticket/solved/20261102211112-save-and-bind-ai-commands.md)
Requirements: PI01, PI06, PI08; AI03, AI04, AI14, AI18.
Dependencies: Slice 6 integrated.
Branch: `feat/rev-ai-pi-07-definitions`
PR: [#235](https://forge.adrianpk.com/adrianpk/rev/pulls/235).
Report: `ops/default/report/slices/rev-ai-pi/slice-7-definitions.md`

- [x] T7.1: Admit bounded literal ai.commands definitions and dynamic ai-name resolution in the shared M-x/colon catalog, preserving typed input/context/output combinations and name collision rules.
- [x] T7.2: Implement ai-save-command through the existing format-preserving conflict-checked root publisher; activate the validated definition while retaining ordinary startup keybinding semantics.
- [x] T7.3: Verify include/source conflicts, user comments, profile keymaps, narrow input context after broad directory history and reuse with a fresh native target and optional Pi model.

Implementation commits: `8f349aca77a480575a4738c4b5a43d05922e809a` (T7.1/T7.2) and
`9509b8d606a1933dc641e0fccd0c0ac3583a2b84` (T7.3). Seven focused definition cases,
strict library lint and all-test compilation passed. Documentation catalogs
passed 429 commands, 231 configuration entries, 68 pages and 415 links/anchors.
Native journeys cover fresh targets, narrow context, real scoped publication,
source conflicts and restart-bound keymaps with a controlled peer. Two older
empty-query fixtures now assert the current first compiled AI command while
retaining acceptance, row-limit and source-immutability checks; both focused
cases passed. PR #235 merged at `af6dd10a18846b4ac5e7b6a5787e3b145b3f4012` on
2026-11-03T01:32:11Z. Receipt `C7EIUNEWCMOYKJ6Q4W4FPA6I7G` passed all four configured
checks and 333 suite cases; the integrated gate was pending at that slice's integration.

Report introduction: `0c074ee5a9ee191ffed3280b3d1bf44966877c1d`.

### Slice 8: Joint acceptance and guidance

State: delivered.
Ticket: [Joint acceptance and guidance](../ticket/solved/20261102211113-accept-and-document-pi-and-reload.md)
Requirements: EFC01-EFC08, PI01-PI08; selected shared AI behavior.
Dependencies: Slice 7 integrated.
Branch: `feat/rev-ai-pi-08-acceptance`
PR: [#236](https://forge.adrianpk.com/adrianpk/rev/pulls/236).
Report: `ops/default/report/slices/rev-ai-pi/slice-8-acceptance.md`

- [x] T8.1: Deliver representative native and PTY journeys for both startup profiles, real kernel file notifications and the installed Pi RPC/tool/session lifecycle, with measured idle/RSS/startup limits.
- [x] T8.2: Exercise one bounded real-provider operation using available existing Pi authentication; report unavailable authentication variants distinctly and never invent live evidence from fixtures.
- [x] T8.3: Document ordinary Pi and reload workflows, complete references/configuration/command catalogs, canonical Markdown and Spore guidance and Unreleased; finish ticket transitions before the unchanged integrated gate and controller closure.

T8.1 focused native/PTY journeys passed in both profiles with installed Pi 1.3.0
and a controlled local provider (zero HTTP requests). Six focused session cases
and strict driver lint passed. Initial-history creation admission:
`47928e8b376385049bf249ff76fd6e8299a6447d`.
T8.1 driver/report introduction: `b8fadd21cef5f9b8c34f3737965cc0ca908f289a`.
The initial T8.2 real-provider attempt failed completion under the existing
configured model and Pi-owned OAuth (one turn/zero tools). Retained incident 1
preserves that failure and the operator's bounded additional-operation decision.
After renewed OAuth, the one additional operation passed at the integrated head
in 2873 ms with the same model, one turn, zero tools, fresh input-only history and
unchanged generated toy source. Acta `01M6G9M4ZBQFX4G7C93FD0VXXJ` records it.
T8.2 is complete; API-key variants were not exercised and no further live
operation was performed.

Canonical PR [#236](https://forge.adrianpk.com/adrianpk/rev/pulls/236) merged at
`dbdb11b78c76f3d39c7cb3df6542b064f90a83ba` on 2026-11-03T07:45:06Z. Exact-head receipt
`RJLI2OQ42YTAU7K373I7PFAIKG` passed all seven configured commands, 61 suite cases,
the nested save case, both-profile release native/PTY/Pi acceptance, documentation
(70 pages/434 links) and the User Guide build. The report preserves separate
editor/Pi resource measurements. Slice 8 is delivered; the integrated gate below
passed for the complete delivery.


## Preparation evidence

A disposable installed-Pi 1.3.0 offline RPC admission probe passed on
2026-11-02: owned read, parent traversal and escaping-link rejection, explicit
read-only tool selection, independent response/notification handling and child
exit on stdin closure. No model/provider request was made. This is preparation
evidence, not source implementation or full path-race/model acceptance.
The plan records its exact boundary and corrected named-tool allowlist.

## Final gate and closure

Completed on 2026-11-03. The unchanged Full gate metadata above matches the
controller evidence for immutable candidate `ecd02abcbd1f08530d8fad1395672dc6d00ee085`.
Receipt `HBFDUZX54JK3RJUBGSQYIJANGO` passed from
2026-11-03T08:58:36Z to 2026-11-03T09:04:09Z (approximately 333 seconds).

All 578 selected top-level tests passed with zero failures/ignored cases, plus
six nested child cases. Strict format/lint, both-profile release native/PTY
installed-Pi journeys, documentation (429 commands, 231 settings, 70 pages,
434 links/anchors), local Spore User Guide generation and whitespace passed.
The [Slice 8 closure](../report/slices/rev-ai-pi/slice-8-acceptance.md#delivery-set-closure)
records exact commands, measured resources and owned cleanup.

Real-provider observations were separate from this offline gate. The initial
failed attempt and retained incident 1 remain recorded; its one operator-authorized
OAuth continuation passed in 2873 ms with one turn/zero tools and unchanged toy
source. No further provider request was made by the gate or closure.

All eight reports are delivered and tickets solved. EFC01-EFC08, PI01-PI08 and
selected shared AI behavior are complete. The broader native API specification
and model retain draft status. Only configured documentation closure paths
change after the immutable tested candidate. No successor is activated.
