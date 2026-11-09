# Slice 13: Independent client and complete acceptance

Status: delivered
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `test/rev-plugin-system-13-acceptance`
PR: #13 - https://forge.adrianpk.com/rev/rev/pulls/13
Implementation: 99316fbb9054540a8fed7ee05f6e9fcb0f395cd8
Fixture correction: a15fb9c7e8eb2a2b85795dbc1d1094cb5485c4af
Integrated: fd682a89b95ee0d6ad7846ce29e6ff6c8ef371f9 through canonical PR #13
Independent client: 3365b84649919492ade7e80c63139b459b5ad923

## Purpose

Exercise the native host with an independent Go implementation and reconcile
PLG01-PLG12 evidence before controller integration and final delivery validation.

## Delivered Behavior

The separate [Go collector](https://forge.adrianpk.com/rev/plugin-examples/src/commit/3365b84649919492ade7e80c63139b459b5ad923/README.md)
uses only the public rev.plugins/0.1 contract and standard library. It lazily
collects current ordinary text, including hidden unsaved sources, creates an
unsaved native result, shows it in the captured pane and binds F8 locally in
both profiles. Earlier results are excluded. Source edits, save and close are
outside this command. Results retain native navigation and undo after retirement.

The Rust bridge checks both native selectors, unchanged source revisions/files,
an unsaved visiting source and an unnamed source, scalar chunks split between a
base character/combining mark/astral character, local-only dispatch, one active
generation and retained result text/undo. The external runner builds/tests Go
outside the editor tree and prints both clean source commits with the exact API.

All thirteen canonical slice PRs are integrated in dev. This matrix records
implementation evidence. The exact integrated controller gate and approved
Markdown closure remain pending.

| Requirement | Native/public evidence | Detailed delivery record |
| --- | --- | --- |
| PLG01 Optional and identifiable | disabled_isolation, dormant_catalog, lazy_profiles, activation_triggers; independent passive discovery in both profiles | [Slices 2](slice-2-declarations.md) and [3](slice-3-lifecycle.md) |
| PLG02 Package namespaces | atomic_catalog, manifest_rejections, qualified_calls and registration admission | [Protocol](slice-1-protocol.md) and [declarations](slice-2-declarations.md) |
| PLG03 Commands and contextual keys | local_scope, global_precedence, user_selectors, dynamic_conflicts, prompt_ownership; semantic prefix/recording tests; independent local F8 | [Keymaps](slice-7-keymaps.md) and [recording](slice-8-recording.md) |
| PLG04 Independent multi-buffer access | hidden_sources, canonical_open, captured_context and both Go/native journeys | [Buffers](slice-4-buffers.md) |
| PLG05 Stamps and coordinates | handle_and_range_errors, refused_targets, captured_native_motion, save_preconditions; independent scalar reads | [Buffers](slice-4-buffers.md), [transactions](slice-5-transactions.md), [invocation](slice-6-invocation.md) |
| PLG06 Atomic batches | refused_targets, shared_views, retired_batch, finite_batches; reviewed prepare/check/non-fallible commit coordinator | [Transactions](slice-5-transactions.md) |
| PLG07 Native history and explicit files | local_history, no_implicit_save, explicit_save_close, associated_save; Go retains source files and native undo | [Transactions](slice-5-transactions.md) and [invocation](slice-6-invocation.md) |
| PLG08 Revision events | publications, resnapshot, slow_consumers, causal_chain, invalidation_timeout and reconnect | [Events](slice-9-events.md) |
| PLG09 Cancellation and bounded service | cancellation_restart, cooperative_cancel, startup_deadline, pipe_backpressure, resource_profiles and two real PTY journeys; Go cancellation wire test | [Lifecycle](slice-3-lifecycle.md), [recording](slice-8-recording.md), [faults](slice-11-faults.md) |
| PLG10 Retirement and retained text | retired_creation, admission_and_retirement, terminal_journeys and independent Go result retention | [Buffers](slice-4-buffers.md), [keymaps](slice-7-keymaps.md), [faults](slice-11-faults.md) |
| PLG11 Explicit experimental agreement | SDK protocol/correlation tests, handshake_rejections, unimplemented_methods, guidance::protocol_examples and independent Go offer refusals | [Protocol](slice-1-protocol.md) and [guidance](slice-12-guidance.md) |
| PLG12 Separate coordinated state | persistence, concurrent_instances, deletion_and_pages, contention, namespaces, invalid_stores and bounds | [State](slice-10-state.md) |

## Implementation Notes

Go code, module, tests, binary output and check-rev.sh belong exclusively to
rev/plugin-examples. Only its dev commit was pushed. The editor adds one
explicitly invoked Rust example; ordinary Cargo builds/tests require no Go.
The client admits 64 sources and 256 KiB of collected text, uses at most one
outgoing request within one callback, bounds frames and shares one finite
command deadline. Its owned close-on-exec nonblocking descriptor adapter gives
Go readiness/deadlines on real inherited exec pipes.

Initial native execution exposed that inherited Go stdin wrappers did not
support deadlines. Duplicating/registering the actual pipes corrected that
adapter; the same native assertions then passed. The first combined Rust run
passed 85 tests and failed slow_consumers because a fixture reader parsed an
append before its final LF. It now observes only complete framed records and
still strictly parses every complete line. Five consecutive focused runs passed
with the original event, queue, invalidation, input and deadline assertions.

The first formal check of a4f1916576b3f7d7241aceb6458cb80dd9cf75e6
passed 85 plugin tests but causal_chain timed out with an active child and zero
pending requests. Repeated isolated and combined checks did not reproduce the
timeout naturally. Inspection found that the event fixture exposed request-N
before writing its JSON, while the child read a request as soon as it existed.
A controlled 20 ms gap between creation and writing reproduced the same timeout
and captured the child request thread failing to parse the empty file. Requests
now use a temporary file and atomic rename, matching the existing response
publication. The same delayed-publication probe then passed, followed by all
nine event tests with the original eight-second deadline and causal assertions.
The initial timeout did not retain child stderr, so its specific occurrence
cannot be attributed conclusively. Temporary delays, repeated-loop probes and
diagnostic instrumentation are absent from the final fixture.

Formal candidate c1004ffbf0cad6ab2cbf105b004fcd19b8825392 passed causal_chain
but slow_consumers exceeded the real invalidation-delivery deadline. Its pressure
fixture performed twelve full expiry turns after every paste, including idle
turns while the child stayed paused. A focused measurement spent 3.25/3.65 seconds
on that loop for one/eight consumers. It now waits until native ready work is
drained after each paste; measured pressure time was 0.88/0.93 seconds. All 1,024
pastes, both consumer counts, source-size assertions, per-package RPC/active
checks, eight-second fixture waits and five-second invalidation deadline remain.
All nine event tests and two additional slow-consumer runs passed. The separate
invalidation_timeout case still proves retirement on the real deadline. Timing
probes are removed and native runtime/limits remain unchanged.

All twelve prior canonical PRs were verified merged into dev, and their
integrations are ancestors of this slice. Missing integration headers for
Slices 7-9 are reconciled with those verified commits. Earlier reports keep
their historical commands, stage-specific capabilities and measured candidates.

## Contracts Added or Changed

Public API rev.plugins/0.1, editor 0.6.0, SDK 0.1.0, native runtime and finite
admission remain unchanged. The Go package has its own 0.1.0 identity. Its
collection is a set of per-source revision-qualified reads, not a simultaneous
read transaction. A later bind/show refusal can leave an ordinary created buffer.
The native host retains every resource, keymap, source and publication decision.

The SDK/reference still distinguish unsupported schema-only registration and
message methods from implemented capabilities, and state from unattached
library hosts. Current command/default/sample and Spore navigation consistency
use the existing strict checks. Source review confirms the atomic coordinator
reserves all candidates/history/views before checks and has no recoverable
partial failure in commit; observers run after all targets and views publish.

## Files of Interest

- [Native bridge](../../../../../examples/plugin-interop.rs).
- [Event fixture framing](../../../../../tests/plugins/events.rs).
- [Public client and acceptance runner](https://forge.adrianpk.com/rev/plugin-examples/src/commit/3365b84649919492ade7e80c63139b459b5ad923/README.md).
- [Authoring guide](../../../../../docs/tutorials/write-plugin.md), [behavior reference](../../../../../docs/reference/plugins.md), tracker and reconciled reports.

## Validation

Local ordered checks passed on editor a15fb9c7e8eb2a2b85795dbc1d1094cb5485c4af
with clean pushed client 3365b84649919492ade7e80c63139b459b5ad923. All thirteen
commands also passed through the controller validator on final clean PR HEAD
fd682a89b95ee0d6ad7846ce29e6ff6c8ef371f9, receipt
7YT4SLZOBA44SGUR3XZCZ34YU4. Canonical PR #13 is merged into dev at that same
commit. This is slice-validation evidence; the controller's integrated gate
remains pending after documentation preparation. Exact ordered commands:

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

Results: twelve SDK protocol tests and one compiled doctest; 86 plugin,
52 command, 114 editing, 110 macro, 45 modal and four completion regressions
passed. The documentation checker passed 684 commands, 299 configuration
entries, 106 reachable pages and 743 local links/anchors. Strict Spore check and
render passed. The corrected ordered local run took 605.59 seconds; the plugin
and command test binaries took 97.24 and 269.20 seconds respectively. Initial
compilation and focused correction checks are separate from that duration.

The independent runner passed both native profiles and printed both source
commits/API. Go passed three tests covering framed envelopes, incompatible
offers and cancellation during a pending request. Standalone Go formatting,
Bash syntax and the client build passed. Five corrected slow-consumer runs
passed before the combined suite.

The existing five resource profiles and two terminal journeys run in the plugin
selection. Quantitative historical samples remain pinned in the Slice 11 report;
no numbers are relabeled as measurements of this PR. Native runtime source is
unchanged since its integrated dbbdd0b4 candidate; the SDK change since then is
the compiled documentation example. Local build output remains ignored and
the guide build does not publish a site.


### Gate repair 1: Coherent peer artifacts

Repair status: reviewing
Repair branch: `fix/rev-plugin-system-gate-1`
Repair PR: pending
Failed integrated candidate: dee4c94dac5e5afa87cdfa1575e995290332acaf
Retained diagnostic: G5JH7N2T4RYIIO3MN6WJLM6PWX

The controller's combined gate stopped in the plugin target after formatting,
required Clippy and SDK checks had passed. The lifecycle peer helper independently
selected the newest SDK and serde_json rlibs from a mixed build directory. Those
artifacts used incompatible crate identities despite matching source versions.
The original selection reproduced the compiler's Value/Serialize/Deserialize
mismatches against the retained build artifacts without running the full gate.

Both native-peer and downloadable-example helpers now obtain their library paths
from Cargo artifact messages for one locked SDK build. A worktree-local
`target/plugin-peer-sdk` cache separates fixture compilation from the parent
Cargo target lock and reuses the SDK build across test processes and runs.
Missing or duplicate required artifacts fail explicitly. The peer source,
public download, runtime, dependencies, test assertions and deadlines are unchanged.

The hidden-source and exact downloadable-example workflows passed on the new
cache, then passed again with newer incompatible rlib lookalikes in both build
directories. All 86 plugin tests also passed with those owned lookalikes present.
The initial worktree build took 132 seconds; subsequent complete plugin testing
took 86.28 seconds. The SDK-only cache has its own initial compilation cost,
then Cargo reuses its artifacts. Child activation deadlines start after fixture
compilation. No gate command or limit was changed.

All six required repair commands passed locally. Fresh exact clean-HEAD evidence
is recorded through the controller validator after the repair PR references are
committed. Required repair commands:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- --test-threads=1`
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
6. `git diff --check`

Additional focused command passed twice, including the mixed-artifact case:
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- buffers::hidden_sources guidance::hello_workflow --test-threads=1`.
An optional `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked -p rev --test plugins -- -D warnings`
reported the pre-existing nested-format warning in the unchanged configuration
fixture. The required workspace library/binary Clippy selection passed.
Controller repair integration, the unchanged full gate and delivery closure
remain pending.

## Risks and Follow-ups

The Go client is a focused Linux example, not a general SDK or cross-platform
runtime. Experimental consumers must pin matching sources. Process supervision
does not establish a sandbox or prove escaped-descendant settlement. The
controller owns the unchanged integrated gate and permitted Markdown closure.
Stable API adoption, distribution/catalog, releases, public guide deployment and
the development-workflow package retain separate authority.
