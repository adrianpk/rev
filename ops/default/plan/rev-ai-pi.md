# Pi Integration and External Reload Delivery

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
Tracker: [Delivery Tracker](../tracker/rev-ai-pi.md)
Supporting contract: [AI Commands](../spec/editor-ai-commands.md), [shared model](../spec/editor-ai-commands-model.md)

## Outcome and authority

Deliver configurable external-file reload and optional Pi-backed editing in one
eight-slice Looper set. External observation is general editor behavior. Pi
uses its real RPC interface for models, authentication, project discovery and
context/history. rev retains source identities, explicit result acceptance and
one native undoable application. Direct API construction and direct Pi source
writes remain outside this set.

The owner approved proceeding after reviewing the architecture on 2026-11-02.
The exact eight-slice decomposition is preparation under that approved scope,
not a claim that the owner separately reviewed each task. This authority covers
one canonical PR per slice, Rebase + Fast-forward into dev, the unchanged final
gate, bounded repair PRs and documentation closure. Use one retained controller
through the entire set. No main promotion, release, mirror/site publication or
global editor installation is part of this instruction.

The parent AI contract/model retain their broader draft status. Complete only
their selected shared Pi behavior, not the deferred native API. Do not close or
rewrite the later native API design as if it shipped. Keep unrelated tickets,
especially quit, M-x binding hints and tree highlights, untouched.

## Strategy and prerequisites

Eight behavior-first increments first deliver a useful explicit reload, then
general observation, then a minimal selection explanation over real RPC. The
initial explanation context is explicitly input-only until Slice 5 completes
directory discovery. Subsequent increments add checked generation, guarded
discovery, retained sessions and saved command definitions. Keep every slice
compilable and valid; expose only working commands with truthful current scope.

Delivered prerequisites are native file/buffer/history/coordinate owners,
configuration and atomic settings publication, M-x/colon input, native panels,
Org/anchors, bookmarks, completion and the optional LSP client. No other
pending feature or external extension ecosystem is required.

Reuse src/file, src/buffer, src/session, src/input, src/command, src/config,
src/lsp and their actual private module boundaries. Add a focused src/ai owner
rather than another editor session, configuration loader, history engine,
selection store or background daemon. The optional transport may share bounded
process/IO techniques, but another feature's shell/write tools do not become
AI capabilities.

## Technical admission before startup

On 2026-11-02 a disposable offline probe exercised the installed Pi 1.3.0
compiled executable through real RPC, with no provider/model request:

- Explicit loading of a small internal JavaScript tool resource succeeded
  while automatic extensions/MCP/skills/templates/context loading was disabled.
- The admitted tool read the expected owned UTF-8 file.
- Parent traversal and an escaping symlink were both rejected.
- The active tool set contained only the named read probe, not native shell,
  edit or write tools.
- RPC notification and handled-response records were consumed independently,
  and closing stdin completed the owned child.
- The corrected profile used an explicit named tool allowlist.
  `--no-tools` keeps even the explicitly activated probe unavailable; it is not
  the launch flag for a usable custom read profile.

The temporary fixture was removed. This establishes actual extension/RPC
admission, not complete production path confinement, model correctness or
race coverage. Slice 5 must extend the evidence to opened-handle/root changes,
all admitted discovery operations, snapshots, budgets and effective resources.
Never treat a handled command response as proof that its independent work
completed. Ordinary model operations require settled/result checks.

Selected implementation choices use existing dependencies:

- Enable the `fs` and `event` features of already locked rustix 1.1.5 for
  Linux notification/poll ownership. Its safe inotify API is present. Do not
  add a watcher crate or first-party unsafe code.
- Reuse std threads/process/IO and the existing JSON/TOML/format-preserving
  publisher. Do not add a generic async runtime, HTTP client or agent SDK.
- Embed a minimal private JavaScript resource with the Rust executable and
  extract it to checked owned XDG cache for Pi to load. Pi executes that
  resource using its installed extension runtime. No separate Node/npm setup,
  application plugin installation or new editor scripting API is introduced.
- Implement only guarded read/list/literal-search/path matching needed for
  context. Use actual path/handle admission and finite directory traversal;
  do not delegate an unchecked whole-root search to a tool that follows links
  outside the grant. No project index or relevance ranking lives in rev.
- Pin the actual installed Pi 1.3.0 executable for acceptance. Resolve a binary
  without invoking installation/update wrappers or rewriting global settings.

These are scope-bounded preparation choices. Another package/runtime/service,
broader tools or source-write authority requires a material decision rather
than a silent fallback.

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

## Slice tasks, acceptance and required checks

### Slice 1: Checked file reload

Ticket: [Checked file reload](../ticket/solved/20261102211106-reload-visiting-files-safely.md)
Requirements: EFC01, EFC03-EFC07.
Dependencies: delivered native file, buffer and configuration owners.

- T1.1: Add the proposed file-policy settings and explicit revert-buffer command through the native file/buffer owners, with complete startup schema/default/sample coverage.
- T1.2: Capture and validate fresh disk bytes asynchronously; preserve unsaved text through revision-bound discard consent and publish one native undoable reload with checked clean-state evidence.
- T1.3: Cover invalid/missing/readonly sources, disk and typing races, undo/redo, all shared views and unchanged-byte observations without a second text or history owner.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test files --test configuration --test external_changes -- --test-threads=1
git diff --check
```

### Slice 2: External change observation

Ticket: [External change observation](../ticket/solved/20261102211107-observe-external-file-changes.md)
Requirements: EFC01-EFC08.
Dependencies: Slice 1 integrated; preserve all earlier contracts.

- T2.1: Use the already locked rustix filesystem/event APIs for bounded parent-directory notifications and native session wake integration; no recursive project watcher or periodic content reads.
- T2.2: Apply reload/ask policies, coalesced per-buffer notices, dirty conflict protection and explicit reload; retain subscriptions through atomic rename, deletion/recreation and shared/hidden buffers.
- T2.3: Verify native notifications, overflow/registration limits, cleanup, idle behavior and canonical invalidation of completion, bookmarks, Org projections and LSP state.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test external_changes --test files --test lsp -- --test-threads=1
git diff --check
```

### Slice 3: Pi RPC explanation

Ticket: [Pi RPC explanation](../ticket/in_progress/20261102211108-connect-pi-rpc-for-explanation.md)
Requirements: PI01-PI04, PI07; AI01, AI05, AI07, AI16.
Dependencies: Slice 2 integrated; preserve all earlier contracts.

- T3.1: Admit typed AI/Pi configuration and own one lazy version-checked RPC child with bounded LF framing, independent streams, continuous consumption and explicit operation generations.
- T3.2: Deliver ai-explain for a captured selection through a fresh input-only session and a native read-only result buffer; keep the limited intermediate context visible until directory discovery lands.
- T3.3: Handle prompt acceptance versus settled completion, errors, unavailable authentication, C-g/modal Escape cancellation and process shutdown while ordinary editing remains responsive.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test ai --test pi --test configuration -- --test-threads=1
git diff --check
```

### Slice 4: Checked AI editing

Ticket: [Checked AI editing](../ticket/ready/20261102211109-apply-ai-results-through-native-editing.md)
Requirements: PI03, PI06, PI07; AI03, AI05-AI09, AI14-AI17.
Dependencies: Slice 3 integrated; preserve all earlier contracts.

- T4.1: Add ai-prompt and ai-implement with explicit point/selection capture and the bounded version-1 generated-text contract; display proposals through the native transient-buffer owner.
- T4.2: Deliver explicit ai-apply-result and ai-discard-result with source revision/read-only admission, one native undo step and no implicit source save or model-selected target.
- T4.3: Fence partial, stale, cancelled, duplicated and oversized results, including local edits, external reloads and focus changes during generation.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test ai --test external_changes -- --test-threads=1
git diff --check
```

### Slice 5: Guarded directory context

Ticket: [Guarded directory context](../ticket/ready/20261102211110-delegate-guarded-project-discovery.md)
Requirements: PI02-PI05, PI07; AI06, AI09, AI11-AI13, AI17, AI20.
Dependencies: Slice 4 integrated; preserve all earlier contracts.

- T5.1: Package a small internal Pi JavaScript resource with rev and load it explicitly through the admitted Pi extension interface; expose only bounded read/list/search operations without a shell or user plugin installation.
- T5.2: Enforce canonical-root and opened-handle path checks, per-operation budgets and fresh unsaved-source precedence while Pi retains all relevance selection and model/context iteration.
- T5.3: Validate traversal, symlink/root replacement races, missing/binary/large sources, disabled external resources and actual Pi tool admission; native API transport/discovery remains deferred.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test pi --test ai -- --test-threads=1
git diff --check
```

### Slice 6: Private Pi sessions

Ticket: [Private Pi sessions](../ticket/ready/20261102211111-retain-private-pi-sessions.md)
Requirements: PI01-PI04, PI07, PI08; AI16-AI20.
Dependencies: Slice 5 integrated; preserve all earlier contracts.

- T6.1: Persist bounded schema-1 private session bindings in XDG state while Pi owns its JSONL history; reuse only matching root/model/grant observations.
- T6.2: Implement ai-set-directory, ai-select-model, ai-reset-session and ai-disconnect, with owned idle shutdown, safe restart and fresh ephemeral input-only sessions.
- T6.3: Cover history isolation, changed roots/providers, compaction and session limits, corrupt records, concurrent instances and actual child exit without exporting credentials.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test pi --test ai -- --test-threads=1
git diff --check
```

### Slice 7: Reusable AI commands

Ticket: [Reusable AI commands](../ticket/ready/20261102211112-save-and-bind-ai-commands.md)
Requirements: PI01, PI06, PI08; AI03, AI04, AI14, AI18.
Dependencies: Slice 6 integrated; preserve all earlier contracts.

- T7.1: Admit bounded literal ai.commands definitions and dynamic ai-name resolution in the shared M-x/colon catalog, preserving typed input/context/output combinations and name collision rules.
- T7.2: Implement ai-save-command through the existing format-preserving conflict-checked root publisher; activate the validated definition while retaining ordinary startup keybinding semantics.
- T7.3: Verify include/source conflicts, user comments, profile keymaps, narrow input context after broad directory history and reuse with a fresh native target and optional Pi model.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test ai --test configuration --test commands -- --test-threads=1
git diff --check
```

### Slice 8: Joint acceptance and guidance

Ticket: [Joint acceptance and guidance](../ticket/ready/20261102211113-accept-and-document-pi-and-reload.md)
Requirements: EFC01-EFC08, PI01-PI08; selected shared AI behavior.
Dependencies: Slice 7 integrated; preserve all earlier contracts.

- T8.1: Deliver representative native and PTY journeys for both startup profiles, real kernel file notifications and the installed Pi RPC/tool/session lifecycle, with measured idle/RSS/startup limits.
- T8.2: Exercise one bounded real-provider operation using available existing Pi authentication; report unavailable authentication variants distinctly and never invent live evidence from fixtures.
- T8.3: Document ordinary Pi and reload workflows, complete references/configuration/command catalogs, canonical Markdown and Spore guidance and Unreleased; finish ticket transitions before the unchanged integrated gate and controller closure.

Required ordered command sequence:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --test external_changes --test ai --test pi -- --test-threads=1
bash scripts/check-ai-pi.sh
cargo run --locked --quiet --example documentation-check
bash scripts/build-user-guide.sh
git diff --check
```


## Cross-contract acceptance matrix

| Contract | Owning slice | Specific preserved behavior |
| --- | --- | --- |
| EFC01-EFC07 checked reload | 1 | Reload/ask TOML defaults, explicit command, fresh consent, source races, one undo, clean-state evidence and valid view positions. |
| EFC02/EFC08 observation lifetime | 2 | Real parent-directory notifications, rename-over saves, hidden/shared buffers, overflow, finite reads and idle event waiting. |
| PI01/PI02 protocol ownership | 3 | Real subprocess interface, LF-only framing/backpressure, accepted versus settled completion, unavailable provider and responsive editing. |
| PI06 checked result application | 4 | Fixed target/revision, explicit preview/accept/discard, no direct disk write, no stale or repeated publication and normal history. |
| PI03/PI05 context and authority | 5 | Multi-file discovery including a non-bundled language, root/link/handle admission, unsaved precedence, finite data and no shell/write/resources escalation. |
| PI04/PI08 authentication/history | 6 | Pi-owned credentials, private root/model/grant sessions, fresh input-only history, restart/reset, limits and owned idle/exit shutdown. |
| Shared AI03/AI04/AI18 definitions | 7 | Validated declarative names/targets/contexts/output, shared command/binding lookup, source-preserving persistence and fresh invocation state. |
| Joint interaction and resources | 8 | Both startup profiles, external reload fences an AI result, missing Pi preserves ordinary editing, representative PTY and actual installed-Pi evidence. |

Notify without stealing input, keep dirty text under both file policies, and
distinguish absent/invalid/unstable disk data from an empty document. No source
save is implied by AI invocation or result acceptance. A disk reload advances
the same source revision observed by completion, LSP, bookmarks and native Org;
a projection or an agent result never owns competing text.

For Pi, root/model/grant identity is checked at reuse. A directory session's
history cannot enter an input-only operation. Saved history is not a live
filesystem index. Refresh the invocation snapshot each time, fence old results,
and keep observation/accounting across compaction/retry. Use the exact current
Pi 1.3 contract, not undocumented latest API behavior.

## Provider, tooling and resource evidence

Ordinary Rust tests use owned protocol/file fixtures for deterministic failures.
The new Bash acceptance script batches representative native and PTY scenarios
and real installed-Pi offline admission/session checks. It must not install
Pi, invoke update wrappers, change user authentication or require Python/Node/
npm as contributor build tools. The internal JavaScript is exercised by Pi.

Slice 8 must additionally record at least one short real-provider operation
using existing admitted Pi authentication and the configured Pi model.
Use a new owned toy project and generated source snapshots, not maintainer
working files or unrelated private conversations.
Limit that live probe to one operation, a one-minute deadline, at most four
model turns and eight context tool calls; do not run paid matrices across
models/providers. The approved implementation includes this bounded acceptance
operation, not new accounts, subscription changes or credential export.
If no usable credentials exist, report the external blocker rather than
manufacturing live success. The mandatory full gate itself remains offline
with respect to model requests; do not repeat a provider call on each retry.
Report which authentication form was actually exercised and distinguish
unavailable variants from controlled fixtures.

Record actual startup latency, external Pi RSS and editor RSS separately,
idle CPU/wakeups, file-notification/reload latency and owned child shutdown.
No performance figure is presumed. Existing source/runtime identity must
support any reused evidence. Retain normal idle behavior while AI is unused;
external notification uses finite event waiting, not a content-polling loop.

## Gate scope, expected cost and recovery

The exact Full gate in metadata is frozen into the local controller config.
It covers the changed file/configuration/AI owners, affected completion/LSP/
bookmark revisions, shared command catalog and modal local contexts, plus one
joint native/PTY/real-Pi-offline script and canonical docs generation. It is
not the exhaustive suite. Do not append every old Org, modal, wrap or installer
journey or require all historical package-manager fixtures.

Expected warm cost is roughly 10-15 minutes plus a cold compile. This is a
planning estimate; record actual timing and justify material expansion.
Per-slice checks remain narrow. Batch variants within native fixtures rather
than launching one full Pi process/provider request per case.

Pin bounded-v1 recovery before startup: two diagnostics per family, eight per
run, three failed repairs per family, three environmental retries and three
final-gate repair PRs. Use repair prefix `fix/rev-ai-pi-acceptance`, title
`fix(ai): repair approved Pi and reload acceptance`, and existing Slice 8 report.
Retain the original conversation, immutable config/gate and default 200 worker
turns/five stalls. No Looper source, services, user credential/config changes,
new authority or weakened checks are delegated by technical recovery.

## Reports, tickets and closure

Use one report per mapped path with the required sections and actual evidence.
PR bodies contain exactly Summary, Changes and Validation; the last Changes
bullet is the immutable report link and whitespace is the last check. Commit
subjects have no task-number suffixes. Update Unreleased when each meaningful
user behavior actually works; use the changelog skill, without selecting a
release version.

Finish each ticket's ready -> in_progress -> reviewing -> solved transitions
and moved links before the full gate. Looper owns integration; the documents
are not another runtime tracker. Capture actual commits/PRs/timestamps only.

After the exact integrated candidate passes, documentation closure may change
only the plan/tracker, eight reports, the two capability spec/model pairs,
pending index and eight exact solved tickets. The broader AI API spec/model
are supporting draft documents and are not closure paths: do not mark the
deferred API complete. Do not modify application code after the gate as closure.
