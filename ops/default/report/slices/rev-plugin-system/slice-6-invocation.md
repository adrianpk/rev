# Slice 6: Native invocation and file lifecycle

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-06-invocation`
PR: pending

## Purpose

Deliver T6.1-T6.3: typed noninteractive command admission, qualified package
calls and explicit revision/disk-qualified saving and checked native closing.

## Delivered Behavior

- commands.list exposes bounded, paginated native and qualified metadata with
  typed arguments, explicit noninteractive admission and handler availability.
- commands.invoke admits the eight existing native text motions against a
  captured text stamp, pane and scalar point. It preserves another pane's focus
  and reuses native grapheme and wrapped-line motion. Unsupported classes fail
  explicitly; save/write/kill/quit classes require native confirmation.
- Qualified calls validate manifest arguments, activate lazily and use admitted
  generation-qualified handlers. Correlated results and typed failures return
  to the original caller. Every nested invocation derives its causal identity
  and depth from a live parent callback; forged parents and depth resets fail.
- buffers.save requires the exact expected source revision and native disk
  observation. Associated saves retain the original observation; unsaved text
  can explicitly save to an absent destination. Occupied targets, read-only
  resources, stale captures and external changes cannot grant overwrite consent.
- The supervisor stages native file bytes with the existing temporary-output
  owner. Native publication rechecks revision, association, permissions and disk
  evidence before replacement and retains native uncertain-write reconciliation.
- buffers.close requires a current stamp and clean ordinary text. Modified text
  and active clocks require native consent. Native removal updates all shared
  views, source references, bookmarks, completion and LSP owners; files survive.
- Prompt/context departure, capture/source ownership, macro work, retired process
  generations and expired request authority refuse the relevant operations.

## Implementation Notes

The implementation commit introduces this report and delivers T6.1-T6.3.
Canonical PR and immutable implementation references follow in the PR-reference
commit.

The command owner defines a typed Motion/Consent/Unsupported admission policy.
The package adapter validates captured native owners and typed argument objects;
it does not run unrestricted user dispatch or synthesize keyboard input. Package
calls preserve their parent operation and receive a new invocation/generation ID.
The caller request permit follows the callee until completion; callee retirement
reports disconnection. Existing sixteen-per-package and sixty-four-per-editor
combined permits can refuse recursion before the causal ceiling of eight.

The native file owner separates private staging from its existing guarded
replacement/completion path. Normal native saves share the same implementation.
The plugin path reserves its source snapshot against the shared sixteen-MiB
retained-ingress budget before capture. The supervisor writes, flushes and fsyncs
temporary bytes, then sends the staged file back for serialized native admission.
No source/history/modified state is acknowledged before native publication.

Retargeting an already associated buffer retains the native write-file and
bookmark loading/consent workflow. Plugin saves never manufacture overwrite
consent or replace a caller's stale disk token with a fresh observation. Native
metadata checks, rename and directory synchronization remain synchronous; the
cooperative service ceiling is not a filesystem latency guarantee.

## Contracts Added or Changed

- Advertised implemented capabilities now include invocation and file_lifecycle;
  commands includes commands.list as well as host-to-package callbacks.
- Native motions require empty arguments, no prefix, a live captured text stamp,
  pane and scalar point. Selection bounds and pane/source membership are checked.
- Qualified handler arguments retain their declared inert JSON types. Current
  prefix and macro-active contexts remain unsupported until Slice 8.
- commands.invoke accepts only a live callback's generation/invocation/origin;
  the host derives nested depth and rejects depth above eight.
- Save and close preconditions never fall back to active focus. A successful
  save returns its source stamp and new native expected_disk observation.
- Explicit save snapshots admit at most sixteen MiB per source and share the
  sixteen-MiB aggregate retained-ingress budget with file opens.
- Uncertain file completion returns state_conflict while preserving native
  published-file association/warning; no failed response claims rollback.

## Files of Interest

- `src/command/plugins.rs`, `src/session/plugins/invocation.rs`: explicit native
  policy, command metadata, captured target validation and qualified dispatch.
- `src/plugins/lifecycle.rs`, `src/plugins/lifecycle/worker.rs`: typed arguments,
  live causal admission, result relay and retirement of nested request permits.
- `src/session/plugins/files.rs`, `src/buffer/save.rs`, `src/file/save.rs`:
  revision/disk-qualified capture, off-input staging and native guarded save/close.
- `tests/plugins/invocation.rs`, `src/buffer/save/tests.rs`: real child RPC and
  post-staging native source/disk/association refusal evidence.
- `docs/reference/plugins.md`, `CHANGELOG.md`: current experimental capabilities
  and explicit invocation/file lifecycle boundaries.

## Validation

All eight frozen ordered commands passed locally:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- invocation --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test commands -- prefix_arguments palette ranking context_isolation transient_isolation prompt_reset workspace_reset --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test editing -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
- `git diff --check`

The SDK passed 12 tests; command regressions passed 52 tests in 240.88 seconds
(300.04 seconds including compilation), and editing passed 114 tests in 58.66
seconds (60.86 seconds including compilation). Documentation validation passed
684 commands per combined inventory, 299 configuration entries, 100 reachable
pages and 684 local links/anchors.

Combined plugin regression passed all 46 protocol/declaration/lifecycle/buffer/
transaction/invocation tests in 38.99 seconds. After final native motion cleanup,
fresh clippy passed and the seven invocation tests passed in 3.43 seconds after
compilation. Those RPC tests cover both startup profiles, native encoding,
shared-view close, focus changes, typed nested results, forged/stale captures,
recursive admission bounds, modified-close consent, C-g and unchanged disk bytes.

Focused native validation passed the staged-save test's six variants (revision,
read-only, disk change, changed association, new-file appearance and closed owner)
in 0.43 seconds, and 28 existing save/consent tests in 7.23 seconds. Standalone
fixture formatting passed. Exact focused commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- invocation transactions buffers lifecycle declarations protocol --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- buffer::save::tests --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test files -- saving decisions --test-threads=1`
- `rustfmt --edition 2024 --check tests/plugins/peer.rs`

The final PR-reference commit receives exact clean-HEAD controller evidence for
all eight frozen commands before handoff. The integrated full gate belongs to
the controller.

## Risks and Follow-ups

Native final metadata/replacement/directory-sync operations retain their existing
synchronous filesystem behavior. Filesystem stalls can exceed the cooperative
service budget. Retargeting an associated buffer requires the native write-file
workflow. Prefix/macro integration, bindings, event subscriptions and state remain
in their recorded later slices. Main, release, mirrors and guide publication are
outside this slice.
