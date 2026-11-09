# Plugin System Delivery

Status: In progress
Date: 2026-11-09
Delivery set: rev-plugin-system
Approved: owner approved host/API implementation and delegated remaining experimental engineering choices
Planning baseline: a398dcb77dd0ce036d989f038068ac942fc68ebd
Specification: [Plugin System](../spec/editor-plugin-system.md)
Model: [Plugin Identity, Protocol and State](../spec/editor-plugin-system-model.md)
Concern: [Pending Editor Work](../README.md)
Active slice: Slice 13 - Independent client and complete acceptance
Tracker: [Plugin System Tracker](../tracker/rev-plugin-system.md)
Slice strategy: behavior-first

## Outcome, prerequisites and authority

Deliver PLG01-PLG12 and the selected model through thirteen independently valid
increments in one Looper run. Protocol/SDK contracts precede executable clients;
each subsequent slice adds real native behavior and advertises only implemented
capabilities. This order follows distinct ownership and acceptance boundaries,
not a fixed slice quota. Distribution/catalog and the development-workflow plugin
are separate future deliveries, not hidden tasks in this set.

The current native buffer/history/view, contextual command/keymap, prefix/macro,
external-file-change and XDG persistence owners are the integration foundation.
Require clean synchronized dev, public canonical rev/rev origin, working provider
and forge access, and sufficient private per-worktree build capacity. Do not
retarget legacy Looper configurations from the old repository identity.

The owner authorizes these exact branches and canonical slice PRs, controller
review and Rebase + Fast-forward integration into dev, the unchanged selected
integrated gate and documentation-only closure. No per-slice owner PR approval
is required. Main promotion, tags/releases, mirrors, public guide deployment and
global binary installation retain separate boundaries. Do not change Looper.

Use Conventional Commits without task-ID suffixes. Looper is the only live
progress owner; this tracker preserves durable tasks/evidence, not a second
controller. Keep current run/conversation/counters and frozen commands on recovery.
Do not invent provider access or claim a started run until acknowledged.

## Canonical slice map

| Slice | Short name | Exact branch | Expected PR title | Report |
| --- | --- | --- | --- | --- |
| Slice 1 | Protocol and Rust SDK | feat/rev-plugin-system-01-protocol | feat(plugins): define versioned RPC and Rust SDK | ops/default/report/slices/rev-plugin-system/slice-1-protocol.md |
| Slice 2 | Passive declarations and configuration | feat/rev-plugin-system-02-declarations | feat(plugins): admit local declarations and native command metadata | ops/default/report/slices/rev-plugin-system/slice-2-declarations.md |
| Slice 3 | Owned process lifecycle | feat/rev-plugin-system-03-lifecycle | feat(plugins): activate and supervise owned package processes | ops/default/report/slices/rev-plugin-system/slice-3-lifecycle.md |
| Slice 4 | Independent text buffers | feat/rev-plugin-system-04-buffers | feat(plugins): expose captured multi-buffer read and display operations | ops/default/report/slices/rev-plugin-system/slice-4-buffers.md |
| Slice 5 | Atomic edits and history | feat/rev-plugin-system-05-transactions | feat(plugins): publish checked multi-buffer edit batches atomically | ops/default/report/slices/rev-plugin-system/slice-5-transactions.md |
| Slice 6 | Native invocation and file lifecycle | feat/rev-plugin-system-06-invocation | feat(plugins): guard native invocation and explicit save or close | ops/default/report/slices/rev-plugin-system/slice-6-invocation.md |
| Slice 7 | Contextual keybindings | feat/rev-plugin-system-07-keymaps | feat(plugins): resolve owned contextual bindings with user precedence | ops/default/report/slices/rev-plugin-system/slice-7-keymaps.md |
| Slice 8 | Prefix arguments and macro calls | feat/rev-plugin-system-08-recording | feat(plugins): preserve counted and recorded qualified command calls | ops/default/report/slices/rev-plugin-system/slice-8-recording.md |
| Slice 9 | Revision events and backpressure | feat/rev-plugin-system-09-events | feat(plugins): deliver bounded revision events and fair callbacks | ops/default/report/slices/rev-plugin-system/slice-9-events.md |
| Slice 10 | Package-owned persistent state | feat/rev-plugin-system-10-state | feat(plugins): coordinate versioned package state across editor instances | ops/default/report/slices/rev-plugin-system/slice-10-state.md |
| Slice 11 | Fault and resource acceptance | test/rev-plugin-system-11-faults | test(plugins): verify lifecycle isolation and bounded resource behavior | ops/default/report/slices/rev-plugin-system/slice-11-faults.md |
| Slice 12 | Authoring and user guidance | docs/rev-plugin-system-12-guidance | docs(plugins): document experimental authoring and editor workflows | ops/default/report/slices/rev-plugin-system/slice-12-guidance.md |
| Slice 13 | Independent client and complete acceptance | test/rev-plugin-system-13-acceptance | test(plugins): verify independent Go interoperability and delivery acceptance | ops/default/report/slices/rev-plugin-system/slice-13-acceptance.md |

## Engineering contracts and permitted implementation roots

Native ownership stays with existing buffers, source revisions, edit/history
publication, views, command dispatch and input contexts. Keep a small plugin
boundary under src/plugins.rs and src/plugins/, with narrow adapters in existing
owners. Do not create a parallel text store, dispatch engine, key grammar,
persistence framework or plugin language. The new Rust SDK crate is independent
of private editor types and shares only the public wire contract. Use existing
pinned dependencies and stdlib. Add no Go content/build step to this repository.

Normal implementation roots are src/, crates/rev-plugin-sdk/, tests/plugins.rs,
tests/plugins/, necessary existing command/editing/macros/modal/completion/config
regressions, examples/plugin-interop.rs, Cargo.toml/Cargo.lock, appropriate docs/
Diataxis pages, publication/userguide/ navigation, CHANGELOG.md and this set's
ops artifacts. Preserve unrelated native behavior and earlier acceptance.

### Explicit external example scope

The separate rev/plugin-examples repository is the only additional repository
admitted by this delivery. Its local checkout is the sibling rev-plugin-examples
of the primary editor checkout, derivable from Git's absolute common directory.
It is public and provisioned. Preparation may bootstrap README/LICENSE on main
with the required Initial commit, then switch to dev. Slice 13 owns the actual
Go client, module, tests, README and scripts/check-rev.sh there, using conventional
commits on its dev and pushing only that branch. Verify its independent origin
before mutation; never replace the editor origin or put Go files/binaries here.
No separate Looper or secondary progress tracker is created.

The external script receives the editor worktree path, builds/tests its own
client outside that tree, and runs the editor's Rust plugin-interop example
against the resulting program. Its execution must exercise the native host and
public NDJSON API, not reproduce host policy in a second implementation. Evidence
records the clean exact editor/API/example source commits and test commands.
The external client/gate runner is an explicitly approved prerequisite; it is
not a Go requirement for ordinary Cargo builds/tests. Verify the external dev
commit is pushed before the final slice's immutable acceptance.

## Slice tasks and focused acceptance

### Slice 1: Protocol and Rust SDK

Dependency: approved preparation committed/pushed and controller preflight.

Requirements: PLG02, PLG05, PLG09, PLG11.

- T1.1: Publish complete JSON-RPC/NDJSON envelopes, handshake directions, method/error schemas, qualified IDs, scalar units and finite admission types in an independent public SDK crate.
- T1.2: Add the Cargo workspace and SDK version/license without changing editor release identity; implement bounded client transport/correlation using existing dependencies and normal Rust ownership.
- T1.3: Create native protocol fixtures and SDK roundtrips for malformed, fragmented, oversize, Unicode and unsupported-version traffic; advertise no unimplemented host capability.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `git diff --check`

### Slice 2: Passive declarations and configuration

Dependency: Slice 1 canonically integrated.

Requirements: PLG01, PLG02, PLG03.

- T2.1: Add explicit opt-in plugins configuration, local manifest resolution, full passive declaration validation and complete reference/sample defaults without spawning unused programs.
- T2.2: Expose plugin-list/activate/deactivate/restart metadata, qualified contributed commands and descriptions in native discovery; unavailable handlers report their state until activation exists.
- T2.3: Exercise namespaces, atomic declaration admission, unknown fields, source-relative paths, profile isolation and dormant startup through real config/catalog boundaries.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- declarations --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `git diff --check`

### Slice 3: Owned process lifecycle

Dependency: Slice 2 canonically integrated.

Requirements: PLG01, PLG09, PLG10, PLG11.

- T3.1: Implement one lazy owned process group per package/editor, coalesced triggers, explicit startup and bidirectional nonblocking RPC with generation-qualified requests.
- T3.2: Deliver cancel/deactivate/restart/shutdown, bounded stderr, failed-spawn descriptor cleanup and owned group escalation/reaping; preserve created buffers and report uncertain descendants.
- T3.3: Verify activation triggers, concurrent first use, crash/EOF/malformed peers, cancellation and both-profile native input responsiveness through real Rust child fixtures.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- lifecycle --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `git diff --check`

### Slice 4: Independent text buffers

Dependency: Slice 3 canonically integrated.

Requirements: PLG04, PLG05, PLG10.

- T4.1: Implement list/open/create/describe/read/show using native opaque handles and source stamps, including hidden and unsaved text independently of focus.
- T4.2: Capture invocation point/selection/pane/revision and reuse native path identity; create ordinary unsaved buffers with package provenance and reject unsupported native resource kinds.
- T4.3: Verify Unicode ranges, display placement, foreign/stale/closed handles, revision observations and focus changes with the real Session/buffer/view owners.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- buffers --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
8. `git diff --check`

### Slice 5: Atomic edits and history

Dependency: Slice 4 canonically integrated.

Requirements: PLG05, PLG06, PLG07, PLG09.

- T5.1: Implement prepare-all/publish-all native coordination, including every history/view reservation, complete stamp rechecks and infallible final publication.
- T5.2: Apply bounded scalar replacements across explicitly captured targets with one native undo unit per buffer and no implicit saving or cross-buffer undo.
- T5.3: Prove no partial source/revision/history/view changes for invalid, stale, closed, read-only, overlapping and late-canceled batches; exercise real undo/redo and completion invalidation.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- transactions --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
8. `git diff --check`

### Slice 6: Native invocation and file lifecycle

Dependency: Slice 5 canonically integrated.

Requirements: PLG03, PLG05, PLG07, PLG09.

- T6.1: Publish a typed noninteractive native-command admission policy and qualified plugin dispatch; validate captured targets/arguments and reject prompting/reentrant unsupported classes explicitly.
- T6.2: Implement revision-and-disk-qualified save and checked close through existing native conflict/read-only/path/consent policies; no force or implicit discard.
- T6.3: Exercise stale disk/source observations, new-file absence, changed focus, modified close, nested calls, native cancellation and unchanged files after refused operations.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- invocation --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
8. `git diff --check`

### Slice 7: Contextual keybindings

Dependency: Slice 6 canonically integrated.

Requirements: PLG02, PLG03, PLG10.

- T7.1: Integrate passive and dynamic global/buffer contributions into existing native contexts, normalization, prefix admission and recovery ownership in both startup profiles.
- T7.2: Implement explicit user selectors and deterministic effective/shadowed/conflicted inspection; recompute current underlying mappings when a generation retires.
- T7.3: Verify local-before-global, user-before-default, competing defaults in both orders, unreachable prefixes, transient ownership and effective key hints using real native dispatch.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- keymaps --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
8. `git diff --check`

### Slice 8: Prefix arguments and macro calls

Dependency: Slice 7 canonically integrated.

Requirements: PLG02, PLG03, PLG07, PLG09.

- T8.1: Forward one captured integer prefix under declared policy and admit semantic plugin-call recording with qualified identity/validated inert arguments exactly once.
- T8.2: Add cooperative cancellable macro replay and explicit compatible schema-2 saves for plugin steps; keep native MacroName, schema-1 reads, old record identities and no implicit migration.
- T8.3: Verify repeated calls, package absence/changed declarations, namespaces, generated-edit double-recording, cancellation, nested causal bounds and legacy/new library roundtrips.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- recording --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
8. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
9. `git diff --check`

### Slice 9: Revision events and backpressure

Dependency: Slice 8 canonically integrated.

Requirements: PLG05, PLG08, PLG09, PLG10.

- T9.1: Implement subscriptions, post-publication source/lifecycle events, sequence/revision/origin propagation and retired-generation rejection.
- T9.2: Enforce per-package/aggregate queues, explicit resnapshot invalidation on overflow, causal depth and 32-item/2-ms cooperative service passes without idle polling.
- T9.3: Exercise event-induced edits across competing packages, slow consumers, reconnect/resnapshot, late callbacks and ordinary input under load with controlled native fixtures.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- events --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
8. `git diff --check`

### Slice 10: Package-owned persistent state

Dependency: Slice 9 canonically integrated.

Requirements: PLG09, PLG12.

- T10.1: Implement state.list/get/put/delete with own-package concern/record identities, bounded JSON values in schema-1 TOML and opaque CAS versions under the native XDG state root.
- T10.2: Use existing nonblocking lock and checked atomic-publication patterns outside native input; preserve unrelated records, uncertain-write evidence and neighboring native stores.
- T10.3: Verify restart, simultaneous instances, stale same-record edits, independent records, lock contention, invalid/symlink/oversized stores and no configuration-directory writes.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- state --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `git diff --check`

### Slice 11: Fault and resource acceptance

Dependency: Slice 10 canonically integrated.

Requirements: PLG01, PLG08, PLG09, PLG10, PLG11.

- T11.1: Consolidate real child-process lifecycle tests for descriptor ownership, handshake limits, EOF/crash, flooding, stalled peers and owned descendants without interrupting unrelated processes.
- T11.2: Measure ordinary startup, installed-unused, active-idle, large-read and bounded event-load profiles; report actual latency/RSS/descriptor observations and native service responsiveness.
- T11.3: Add representative real PTY journeys for non-modal/modal command use, result buffers, local bindings, cancellation and plugin failure; reuse lower-level variants instead of multiplying journeys.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- faults --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
8. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
9. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
10. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
11. `git diff --check`

### Slice 12: Authoring and user guidance

Dependency: Slice 11 canonically integrated.

Requirements: PLG01, PLG02, PLG03, PLG04, PLG11, PLG12.

- T12.1: Add concise opt-in authoring and end-user tutorials/how-tos, complete protocol/configuration/command/error references and an explanation of ownership, transactions and experimental compatibility.
- T12.2: Keep core User Guide approachable with a separate optional plugin entry, short accurate examples and useful text feedback; derive Spore from Markdown and preserve navigation.
- T12.3: Verify SDK documentation examples, complete command descriptions/sample defaults, public links, Unreleased product outcomes and strict rendered-guide consistency without publishing the site.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- guidance --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
8. `git diff --check`

### Slice 13: Independent client and complete acceptance

Dependency: Slice 12 canonically integrated.

Requirements: PLG01, PLG02, PLG03, PLG04, PLG05, PLG06, PLG07, PLG08, PLG09, PLG10, PLG11, PLG12.

- T13.1: Implement the small Go multi-buffer client and its build/test/check-rev.sh only in the separate rev/plugin-examples checkout; push its conventional dev commit and pin both commits/API in evidence.
- T13.2: Add examples/plugin-interop.rs as the native Rust integration bridge and prove hidden unsaved reads, created visible result, native navigation and public-only protocol compatibility with the independent client.
- T13.3: Complete PLG01-PLG12 joint acceptance, reconcile SDK/reference/catalog/Spore evidence and all reports, then prepare the unchanged selected controller gate and documentation-only closure.

Frozen ordered focused checks:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
8. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
9. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
10. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
11. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
12. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash "$(git rev-parse --path-format=absolute --git-common-dir)/../../rev-plugin-examples/scripts/check-rev.sh" "$PWD"`
13. `git diff --check`

## Frozen ordered integrated gate

The following exact commands, in order, form the full delivery-set gate. They
protect the new API/SDK and affected shared owners; they do not append every
historical acceptance suite. Use the same command strings in configuration.

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
7. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1`
8. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test modal -- prefix_arguments repeat count operator navigation --test-threads=1`
9. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test completion -- manual_edit cancel stale --test-threads=1`
10. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
11. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/build-user-guide.sh`
12. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash "$(git rev-parse --path-format=absolute --git-common-dir)/../../rev-plugin-examples/scripts/check-rev.sh" "$PWD"`
13. `git diff --check`

All Cargo/build commands use one build job, disabled incremental output and no
dev/test debuginfo. Keep targets private to each checkout; do not share mutable
build directories or remove active build output. Initial compilation is the main
expected cost; native tests and representative PTY journeys must have finite
readiness/operation deadlines. Record actual durations at acceptance rather than
promising a fixed run time. Do not run the full gate for every early slice.

The plugins integration target uses native Rust tests, real Session/source/view
owners and owned local Rust peer executables. SDK transport boundary tests are
not a substitute for native host integration. Filters must select real tests;
zero-test success does not establish a slice. Real PTY tests use the Cargo-built
binary and existing rustix/terminal fixture patterns, isolated HOME/XDG paths and
finite observable readiness. No Python dependency or custom test framework is
introduced. Shell scripts remain Bash. SDK Rustdoc examples run with its tests.

## Corrections, evidence and closure

Every slice supplies its one report, exact PR identity and successful ordered
clean-HEAD evidence through looper validate. Report actual observations, not
anticipated outcomes. Catalog/sample consistency is kept current when commands
or configuration are introduced, rather than deferred into an unrelated repair.

If an integrated gate exposes a defect in this approved feature, the owner
already authorizes bounded corrections through the ordinary repair PR route,
with the same assertions, contracts and gate. Use the installed Looper's explicit
authorize-gate-repairs mechanism outside worker turns when its retained inactive
checkpoint permits it. At most three repairs are delegated; allowed roots are
limited to this feature's runtime, SDK, focused regression, guide/config and
existing report paths. This is not authority to alter Looper, widen scope,
weaken limits, discard tests or reset a run. An unavailable external toolchain or
provider is reported precisely. No repair policy is fabricated in a checkpoint.

Closure follows the successful exact integrated controller gate. Change only
this plan, tracker, specification/model, concern index and the thirteen report
Markdown paths. Mark reports delivered and artifacts Completed with references
to the controller evidence. Preserve requirements, wire/coordinate/persistence
contracts, ordered commands, branch/title/report identities and the tested runtime
candidate. Close no distribution or development-workflow artifact.
