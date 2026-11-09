# Slice 13: Independent client and complete acceptance

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `test/rev-plugin-system-13-acceptance`
PR: pending
Implementation: 99316fbb9054540a8fed7ee05f6e9fcb0f395cd8
Fixture correction: a15fb9c7e8eb2a2b85795dbc1d1094cb5485c4af
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

This matrix records implementation evidence. Controller integration, the exact
integrated dev gate and approved Markdown closure remain pending.

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
with clean pushed client 3365b84649919492ade7e80c63139b459b5ad923. The final clean PR
HEAD receives fresh controller evidence for the same exact ordered commands:

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

## Risks and Follow-ups

The Go client is a focused Linux example, not a general SDK or cross-platform
runtime. Experimental consumers must pin matching sources. Process supervision
does not establish a sandbox or prove escaped-descendant settlement. The
controller owns final integration, the unchanged integrated gate and permitted
Markdown closure. Stable API adoption, distribution/catalog, releases, public
guide deployment and the development-workflow package retain separate authority.
