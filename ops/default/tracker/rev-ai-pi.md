# Pi Integration and External Reload Tracker

Status: Approved
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
Active slice: Slice 3 — Pi RPC explanation
Plan: [Delivery Plan](../plan/rev-ai-pi.md)

## Execution ownership

The owner approved proceeding on 2026-11-02. Looper owns the eight-slice
execution, canonical PR integration, configured gate, bounded recovery and
closure. Slices 1 and 2 are integrated at their validated heads. Slice 3 is active;
later slices remain pending. Ready tickets are actionable inputs, not completion evidence.

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

State: in_progress.
Ticket: [Pi RPC explanation](../ticket/in_progress/20261102211108-connect-pi-rpc-for-explanation.md)
Requirements: PI01-PI04, PI07; AI01, AI05, AI07, AI16.
Dependencies: Slice 2 integrated.
Branch: `feat/rev-ai-pi-03-rpc`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-3-rpc.md`

- [ ] T3.1: Admit typed AI/Pi configuration and own one lazy version-checked RPC child with bounded LF framing, independent streams, continuous consumption and explicit operation generations.
- [ ] T3.2: Deliver ai-explain for a captured selection through a fresh input-only session and a native read-only result buffer; keep the limited intermediate context visible until directory discovery lands.
- [ ] T3.3: Handle prompt acceptance versus settled completion, errors, unavailable authentication, C-g/modal Escape cancellation and process shutdown while ordinary editing remains responsive.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.

### Slice 4: Checked AI editing

State: ready.
Ticket: [Checked AI editing](../ticket/ready/20261102211109-apply-ai-results-through-native-editing.md)
Requirements: PI03, PI06, PI07; AI03, AI05-AI09, AI14-AI17.
Dependencies: Slice 3 integrated.
Branch: `feat/rev-ai-pi-04-results`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-4-results.md`

- [ ] T4.1: Add ai-prompt and ai-implement with explicit point/selection capture and the bounded version-1 generated-text contract; display proposals through the native transient-buffer owner.
- [ ] T4.2: Deliver explicit ai-apply-result and ai-discard-result with source revision/read-only admission, one native undo step and no implicit source save or model-selected target.
- [ ] T4.3: Fence partial, stale, cancelled, duplicated and oversized results, including local edits, external reloads and focus changes during generation.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.

### Slice 5: Guarded directory context

State: ready.
Ticket: [Guarded directory context](../ticket/ready/20261102211110-delegate-guarded-project-discovery.md)
Requirements: PI02-PI05, PI07; AI06, AI09, AI11-AI13, AI17, AI20.
Dependencies: Slice 4 integrated.
Branch: `feat/rev-ai-pi-05-discovery`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-5-discovery.md`

- [ ] T5.1: Package a small internal Pi JavaScript resource with rev and load it explicitly through the admitted Pi extension interface; expose only bounded read/list/search operations without a shell or user plugin installation.
- [ ] T5.2: Enforce canonical-root and opened-handle path checks, per-operation budgets and fresh unsaved-source precedence while Pi retains all relevance selection and model/context iteration.
- [ ] T5.3: Validate traversal, symlink/root replacement races, missing/binary/large sources, disabled external resources and actual Pi tool admission; native API transport/discovery remains deferred.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.

### Slice 6: Private Pi sessions

State: ready.
Ticket: [Private Pi sessions](../ticket/ready/20261102211111-retain-private-pi-sessions.md)
Requirements: PI01-PI04, PI07, PI08; AI16-AI20.
Dependencies: Slice 5 integrated.
Branch: `feat/rev-ai-pi-06-sessions`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-6-sessions.md`

- [ ] T6.1: Persist bounded schema-1 private session bindings in XDG state while Pi owns its JSONL history; reuse only matching root/model/grant observations.
- [ ] T6.2: Implement ai-set-directory, ai-select-model, ai-reset-session and ai-disconnect, with owned idle shutdown, safe restart and fresh ephemeral input-only sessions.
- [ ] T6.3: Cover history isolation, changed roots/providers, compaction and session limits, corrupt records, concurrent instances and actual child exit without exporting credentials.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.

### Slice 7: Reusable AI commands

State: ready.
Ticket: [Reusable AI commands](../ticket/ready/20261102211112-save-and-bind-ai-commands.md)
Requirements: PI01, PI06, PI08; AI03, AI04, AI14, AI18.
Dependencies: Slice 6 integrated.
Branch: `feat/rev-ai-pi-07-definitions`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-7-definitions.md`

- [ ] T7.1: Admit bounded literal ai.commands definitions and dynamic ai-name resolution in the shared M-x/colon catalog, preserving typed input/context/output combinations and name collision rules.
- [ ] T7.2: Implement ai-save-command through the existing format-preserving conflict-checked root publisher; activate the validated definition while retaining ordinary startup keybinding semantics.
- [ ] T7.3: Verify include/source conflicts, user comments, profile keymaps, narrow input context after broad directory history and reuse with a fresh native target and optional Pi model.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.

### Slice 8: Joint acceptance and guidance

State: ready.
Ticket: [Joint acceptance and guidance](../ticket/ready/20261102211113-accept-and-document-pi-and-reload.md)
Requirements: EFC01-EFC08, PI01-PI08; selected shared AI behavior.
Dependencies: Slice 7 integrated.
Branch: `feat/rev-ai-pi-08-acceptance`
PR: pending.
Report: `ops/default/report/slices/rev-ai-pi/slice-8-acceptance.md`

- [ ] T8.1: Deliver representative native and PTY journeys for both startup profiles, real kernel file notifications and the installed Pi RPC/tool/session lifecycle, with measured idle/RSS/startup limits.
- [ ] T8.2: Exercise one bounded real-provider operation using available existing Pi authentication; report unavailable authentication variants distinctly and never invent live evidence from fixtures.
- [ ] T8.3: Document ordinary Pi and reload workflows, complete references/configuration/command catalogs, canonical Markdown and Spore guidance and Unreleased; finish ticket transitions before the unchanged integrated gate and controller closure.

Validation and actual commit/PR evidence: pending. Use the plan and configured
ordered commands; do not claim the integrated gate from focused checks.


## Preparation evidence

A disposable installed-Pi 1.3.0 offline RPC admission probe passed on
2026-11-02: owned read, parent traversal and escaping-link rejection, explicit
read-only tool selection, independent response/notification handling and child
exit on stdin closure. No model/provider request was made. This is preparation
evidence, not source implementation or full path-race/model acceptance.
The plan records its exact boundary and corrected named-tool allowlist.

## Final gate and closure

Pending. The Full gate metadata must match the local frozen configuration.
Its affected native regressions and joint offline native/PTY/Pi flow are the
selected integrated gate, not an exhaustive repository run. The single bounded
real-provider acceptance observation belongs to Slice 8 and is not repeated
automatically on gate retries.

Finish ticket transitions and all implementation before gate admission.
Closure preserves actual candidate/receipt and historical identities, changes
only preconfigured documents, and never marks the deferred native API complete.
