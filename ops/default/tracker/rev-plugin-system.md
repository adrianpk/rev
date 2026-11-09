# Plugin System Tracker

Status: In progress
Date: 2026-11-09
Delivery set: rev-plugin-system
Approved: owner approved host/API implementation and delegated remaining experimental engineering choices
Planning baseline: a398dcb77dd0ce036d989f038068ac942fc68ebd
Specification: [Plugin System](../spec/editor-plugin-system.md)
Model: [Plugin Identity, Protocol and State](../spec/editor-plugin-system-model.md)
Concern: [Pending Editor Work](../README.md)
Active slice: None; all thirteen slices delivered; controller gate pending
Execution authorized: one thirteen-slice Looper run with automatic integration into dev
Plan: [Plugin System Delivery](../plan/rev-plugin-system.md)
Progress owner: selected Looper run; no second runtime tracker

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

## Delivery tasks

## Slice 1: Protocol and Rust SDK

State: delivered
Dependency: approved preparation and controller preflight
Branch: feat/rev-plugin-system-01-protocol
PR: #1 - https://forge.adrianpk.com/rev/rev/pulls/1
Report: ops/default/report/slices/rev-plugin-system/slice-1-protocol.md
Implementation: bdaef1028c743fab1b2180f9d2a0bed9216df80d
Integrated: 924db7e317729b8fde1f8fd72e3038b1bfae10c6
Validation: all seven frozen commands passed through the controller validator on the clean integrated commit; canonical PR contract and merge verified.

- [x] T1.1: Publish complete JSON-RPC/NDJSON envelopes, handshake directions, method/error schemas, qualified IDs, scalar units and finite admission types in an independent public SDK crate.
- [x] T1.2: Add the Cargo workspace and SDK version/license without changing editor release identity; implement bounded client transport/correlation using existing dependencies and normal Rust ownership.
- [x] T1.3: Create native protocol fixtures and SDK roundtrips for malformed, fragmented, oversize, Unicode and unsupported-version traffic; advertise no unimplemented host capability.

## Slice 2: Passive declarations and configuration

State: delivered
Dependency: Slice 1 integrated
Branch: feat/rev-plugin-system-02-declarations
PR: #2 - https://forge.adrianpk.com/rev/rev/pulls/2
Report: ops/default/report/slices/rev-plugin-system/slice-2-declarations.md
Implementation: 3914b057d19dee39bcaf7dfaed7702d89d0ee90c
Integrated: e5404541b619b4233260fd25267fea56d79d6d4e
Validation: all seven frozen commands passed through the controller validator on the clean integrated commit; canonical PR contract and merge verified. Eleven focused modal regressions also passed locally.

- [x] T2.1: Add explicit opt-in plugins configuration, local manifest resolution, full passive declaration validation and complete reference/sample defaults without spawning unused programs.
- [x] T2.2: Expose plugin-list/activate/deactivate/restart metadata, qualified contributed commands and descriptions in native discovery; unavailable handlers report their state until activation exists.
- [x] T2.3: Exercise namespaces, atomic declaration admission, unknown fields, source-relative paths, profile isolation and dormant startup through real config/catalog boundaries.

## Slice 3: Owned process lifecycle

State: delivered
Dependency: Slice 2 integrated
Branch: feat/rev-plugin-system-03-lifecycle
PR: #3 - https://forge.adrianpk.com/rev/rev/pulls/3
Report: ops/default/report/slices/rev-plugin-system/slice-3-lifecycle.md
Implementation: 71857f4a05786909e1d7beb58d8845b2f0cccc05
Integrated: a3e86c211a33bfe3bfe7298848120abe7dddcc33 through canonical PR #3
Validation: all seven frozen ordered commands passed locally, plus 22 focused protocol/declaration/lifecycle tests and standalone fixture formatting; controller validation passed the exact clean integrated HEAD; see the report.

- [x] T3.1: Implement one lazy owned process group per package/editor, coalesced triggers, explicit startup and bidirectional nonblocking RPC with generation-qualified requests.
- [x] T3.2: Deliver cancel/deactivate/restart/shutdown, bounded stderr, failed-spawn descriptor cleanup and owned group escalation/reaping; preserve created buffers and report uncertain descendants.
- [x] T3.3: Verify activation triggers, concurrent first use, crash/EOF/malformed peers, cancellation and both-profile native input responsiveness through real Rust child fixtures.

## Slice 4: Independent text buffers

State: delivered
Dependency: Slice 3 integrated
Branch: feat/rev-plugin-system-04-buffers
PR: #4 - https://forge.adrianpk.com/rev/rev/pulls/4
Report: ops/default/report/slices/rev-plugin-system/slice-4-buffers.md
Implementation: abac32019cec6e09e6c3abe5ebc12ad377e36157
Integrated: c3315cd934500cfbbb134f23371c767de9638f21 through canonical PR #4
Validation: all eight frozen commands passed locally; fresh focused validation passed 31 plugin tests and standalone fixture formatting. Controller validation passed all eight commands on the exact clean integrated HEAD.

- [x] T4.1: Implement list/open/create/describe/read/show using native opaque handles and source stamps, including hidden and unsaved text independently of focus.
- [x] T4.2: Capture invocation point/selection/pane/revision and reuse native path identity; create ordinary unsaved buffers with package provenance and reject unsupported native resource kinds.
- [x] T4.3: Verify Unicode ranges, display placement, foreign/stale/closed handles, revision observations and focus changes with the real Session/buffer/view owners.

## Slice 5: Atomic edits and history

State: delivered
Dependency: Slice 4 integrated
Branch: feat/rev-plugin-system-05-transactions
PR: #5 - https://forge.adrianpk.com/rev/rev/pulls/5
Report: ops/default/report/slices/rev-plugin-system/slice-5-transactions.md
Implementation: 0d8ec0cf5e3d2447c8e245cc3901599a3a12caa7
Integrated: 9df6c5beb7dd2df7eece84a4f90b1c87671cc724 through canonical PR #5
Validation: all eight frozen commands passed through the controller validator on the exact clean integrated HEAD; canonical PR contract and merge verified. Local focused checks also passed 39 plugin tests, eight native edit/coordinator tests and fixture formatting.

- [x] T5.1: Implement prepare-all/publish-all native coordination, including every history/view reservation, complete stamp rechecks and infallible final publication.
- [x] T5.2: Apply bounded scalar replacements across explicitly captured targets with one native undo unit per buffer and no implicit saving or cross-buffer undo.
- [x] T5.3: Prove no partial source/revision/history/view changes for invalid, stale, closed, read-only, overlapping and late-canceled batches; exercise real undo/redo and completion invalidation.

## Slice 6: Native invocation and file lifecycle

State: delivered
Dependency: Slice 5 integrated
Branch: feat/rev-plugin-system-06-invocation
PR: #6 - https://forge.adrianpk.com/rev/rev/pulls/6
Report: ops/default/report/slices/rev-plugin-system/slice-6-invocation.md
Implementation: 912a6dbe11bf273b50c88add88e93e0b024542be
Integrated: 82e8605eb1535b490862387b7baf9e528153ca31 through canonical PR #6
Validation: all eight frozen commands passed through the controller validator on the exact clean integrated HEAD; canonical PR contract and merge verified. Focused checks also passed 46 combined plugin tests, seven invocation tests, a six-variant staged-save test and 28 native save/consent regressions.

- [x] T6.1: Publish a typed noninteractive native-command admission policy and qualified plugin dispatch; validate captured targets/arguments and reject prompting/reentrant unsupported classes explicitly.
- [x] T6.2: Implement revision-and-disk-qualified save and checked close through existing native conflict/read-only/path/consent policies; no force or implicit discard.
- [x] T6.3: Exercise stale disk/source observations, new-file absence, changed focus, modified close, nested calls, native cancellation and unchanged files after refused operations.

## Slice 7: Contextual keybindings

State: delivered
Dependency: Slice 6 integrated
Branch: feat/rev-plugin-system-07-keymaps
PR: #7 - https://forge.adrianpk.com/rev/rev/pulls/7
Report: ops/default/report/slices/rev-plugin-system/slice-7-keymaps.md
Implementation: ef082253d8093bf1af5041349b943be1c99ae378
Integrated: 92646729268ccbc15ea7bf432c45b60d5870eb4a through canonical PR #7
Validation: all eight frozen commands passed locally; fresh nine-test keymap selection and 55 combined plugin tests passed, including native dispatch/hints, both profiles, hidden targets, precedence/conflicts, selectors, retirement and finite ownership. All eight commands passed exact clean-HEAD controller validation before handoff; canonical merge verified.

- [x] T7.1: Integrate passive and dynamic global/buffer contributions into existing native contexts, normalization, prefix admission and recovery ownership in both startup profiles.
- [x] T7.2: Implement explicit user selectors and deterministic effective/shadowed/conflicted inspection; recompute current underlying mappings when a generation retires.
- [x] T7.3: Verify local-before-global, user-before-default, competing defaults in both orders, unreachable prefixes, transient ownership and effective key hints using real native dispatch.

## Slice 8: Prefix arguments and macro calls

State: delivered
Dependency: Slice 7 integrated
Branch: feat/rev-plugin-system-08-recording
PR: #8 - https://forge.adrianpk.com/rev/rev/pulls/8
Report: ops/default/report/slices/rev-plugin-system/slice-8-recording.md
Implementation: 557f2358bcf530636242b8be25bcc1a3a3d10ffb
Integrated: 7e8d5c7475ba614e6d0940ff2a79ce5f33de2418 through canonical PR #8
Validation: all nine frozen ordered commands passed locally; final focused validation passed 65 combined plugin tests including ten recording journeys, five native library tests and fixture formatting. All nine commands passed exact clean-HEAD controller validation before handoff; canonical merge verified.

- [x] T8.1: Forward one captured integer prefix under declared policy and admit semantic plugin-call recording with qualified identity/validated inert arguments exactly once.
- [x] T8.2: Add cooperative cancellable macro replay and explicit compatible schema-2 saves for plugin steps; keep native MacroName, schema-1 reads, old record identities and no implicit migration.
- [x] T8.3: Verify repeated calls, package absence/changed declarations, namespaces, generated-edit double-recording, cancellation, nested causal bounds and legacy/new library roundtrips.

## Slice 9: Revision events and backpressure

State: delivered
Dependency: Slice 8 integrated
Branch: feat/rev-plugin-system-09-events
PR: https://forge.adrianpk.com/rev/rev/pulls/9
Implementation: f738e46cb79aa7b3c854f74e24e242dc4cc00da7
Report: ops/default/report/slices/rev-plugin-system/slice-9-events.md
Integrated: 95ef40a9fbda4e5e32bd582b9b14eeeeaad01670
Validation: all eight frozen ordered commands passed exact clean-HEAD controller validation before handoff; 74 combined plugin regressions passed; canonical merge verified

- [x] T9.1: Implement subscriptions, post-publication source/lifecycle events, sequence/revision/origin propagation and retired-generation rejection.
- [x] T9.2: Enforce per-package/aggregate queues, explicit resnapshot invalidation on overflow, causal depth and 32-item/2-ms cooperative service passes without idle polling.
- [x] T9.3: Exercise event-induced edits across competing packages, slow consumers, reconnect/resnapshot, late callbacks and ordinary input under load with controlled native fixtures.

## Slice 10: Package-owned persistent state

State: delivered
Dependency: Slice 9 integrated
Branch: feat/rev-plugin-system-10-state
PR: #10 - https://forge.adrianpk.com/rev/rev/pulls/10
Implementation: 0efa209d4e93e69cb6386f117f10f8cb708acaaa
Report: ops/default/report/slices/rev-plugin-system/slice-10-state.md
Integrated: 2be4e880daf567c6aa52a127ccab4b851dd48953
Validation: all seven frozen ordered commands passed locally; focused checks passed 81 combined plugin regressions, three native publication-fault tests and standalone peer formatting. All seven commands passed exact clean-HEAD controller validation before handoff; canonical merge verified.

- [x] T10.1: Implement state.list/get/put/delete with own-package concern/record identities, bounded JSON values in schema-1 TOML and opaque CAS versions under the native XDG state root.
- [x] T10.2: Use existing nonblocking lock and checked atomic-publication patterns outside native input; preserve unrelated records, uncertain-write evidence and neighboring native stores.
- [x] T10.3: Verify restart, simultaneous instances, stale same-record edits, independent records, lock contention, invalid/symlink/oversized stores and no configuration-directory writes.

## Slice 11: Fault and resource acceptance

State: delivered
Dependency: Slice 10 integrated
Branch: test/rev-plugin-system-11-faults
PR: #11 - https://forge.adrianpk.com/rev/rev/pulls/11
Implementation: 56d4e0b62ce5d32b539c799c7507f6e591655794
Report: ops/default/report/slices/rev-plugin-system/slice-11-faults.md
Integrated: dbbdd0b4a88b1537e829c985a2160f58c5e39836
Validation: all eleven frozen ordered commands passed locally; 84 combined plugin regressions and peer formatting passed. Five native resource profiles and two PTY journeys are measured in the report. Initial controller validation exposed a retired pending-count race; the correction passed five repeated backpressure runs and all 84 plugin regressions. All eleven commands passed fresh exact clean-HEAD controller validation on dbbdd0b4a88b1537e829c985a2160f58c5e39836 before handoff; canonical merge verified.

- [x] T11.1: Consolidate real child-process lifecycle tests for descriptor ownership, handshake limits, EOF/crash, flooding, stalled peers and owned descendants without interrupting unrelated processes.
- [x] T11.2: Measure ordinary startup, installed-unused, active-idle, large-read and bounded event-load profiles; report actual latency/RSS/descriptor observations and native service responsiveness.
- [x] T11.3: Add representative real PTY journeys for non-modal/modal command use, result buffers, local bindings, cancellation and plugin failure; reuse lower-level variants instead of multiplying journeys.

## Slice 12: Authoring and user guidance

State: delivered
Dependency: Slice 11 integrated
Branch: docs/rev-plugin-system-12-guidance
PR: #12 - https://forge.adrianpk.com/rev/rev/pulls/12
Implementation: 8a56835b74d63e8e8c5d0a50446a7cf2e08095bb
Report: ops/default/report/slices/rev-plugin-system/slice-12-guidance.md
Integrated: 9d308ce040d5559011218022652c5b7c74ecc078
Validation: all eight frozen commands passed locally; two guidance regressions cover the exact public Rust/TOML downloads in both profiles, twelve SDK tests and one doctest passed, and 52 command regressions passed. Documentation covers 106 pages/743 local links, 684 commands and 299 settings; strict Spore build and byte-identical downloads passed. All eight commands passed exact clean-HEAD controller validation on 9d308ce040d5559011218022652c5b7c74ecc078 before handoff; canonical merge verified.

- [x] T12.1: Add concise opt-in authoring and end-user tutorials/how-tos, complete protocol/configuration/command/error references and an explanation of ownership, transactions and experimental compatibility.
- [x] T12.2: Keep core User Guide approachable with a separate optional plugin entry, short accurate examples and useful text feedback; derive Spore from Markdown and preserve navigation.
- [x] T12.3: Verify SDK documentation examples, complete command descriptions/sample defaults, public links, Unreleased product outcomes and strict rendered-guide consistency without publishing the site.

## Slice 13: Independent client and complete acceptance

State: delivered
Dependency: Slice 12 integrated
Branch: test/rev-plugin-system-13-acceptance
PR: #13 - https://forge.adrianpk.com/rev/rev/pulls/13
Implementation: 99316fbb9054540a8fed7ee05f6e9fcb0f395cd8
Fixture correction: a15fb9c7e8eb2a2b85795dbc1d1094cb5485c4af
Report: ops/default/report/slices/rev-plugin-system/slice-13-acceptance.md
Integrated: fd682a89b95ee0d6ad7846ce29e6ff6c8ef371f9 through canonical PR #13
Independent client: 3365b84649919492ade7e80c63139b459b5ad923 - https://forge.adrianpk.com/rev/plugin-examples/src/commit/3365b84649919492ade7e80c63139b459b5ad923/README.md
Validation: all thirteen frozen commands passed locally on a15fb9c7e8eb2a2b85795dbc1d1094cb5485c4af with clean pushed Go client 3365b84649919492ade7e80c63139b459b5ad923. Twelve SDK tests/one doctest, 86 plugins, 52 commands, 114 editing, 110 macros, 45 modal and four completion tests passed. Strict documentation/Spore and both independent-client profiles passed. The initial 85/86 plugin run exposed an incomplete fixture-frame read; five corrected slow-consumer runs and all 86 plugins passed with unchanged assertions. A later formal run timed out in causal_chain; a controlled request-file exposure reproduced the active-child timeout and empty-JSON thread failure. Atomic request publication passed that probe and all nine event tests without changing deadlines or runtime behavior. The next formal run passed causal_chain but the fixed-spin pressure fixture exceeded the real invalidation deadline; draining ready native work retained all 1,024 pastes, both consumer counts and existing assertions/deadlines, then passed all nine events plus two more slow-consumer runs. All thirteen commands passed fresh exact clean-HEAD controller validation on fd682a89b95ee0d6ad7846ce29e6ff6c8ef371f9 before handoff, receipt 7YT4SLZOBA44SGUR3XZCZ34YU4; canonical merge verified. The integrated controller gate remains pending.

- [x] T13.1: Implement the small Go multi-buffer client and its build/test/check-rev.sh only in the separate rev/plugin-examples checkout; push its conventional dev commit and pin both commits/API in evidence.
- [x] T13.2: Add examples/plugin-interop.rs as the native Rust integration bridge and prove hidden unsaved reads, created visible result, native navigation and public-only protocol compatibility with the independent client.
- [x] T13.3: Complete PLG01-PLG12 joint acceptance, reconcile SDK/reference/catalog/Spore evidence and all reports, then prepare the unchanged selected controller gate and documentation-only closure.


## Gate repair 1: Coherent peer artifacts

Repair state: reviewing
Failed integrated candidate: dee4c94dac5e5afa87cdfa1575e995290332acaf
Branch: fix/rev-plugin-system-gate-1
Expected PR title: fix(plugins): select coherent artifacts for plugin test peers
PR: pending
Report: ops/default/report/slices/rev-plugin-system/slice-13-acceptance.md

The retained gate failure reproduces incompatible SDK/serde_json crate identities
from independent newest-rlib selection. Both fixture compilers now use one locked
Cargo SDK artifact graph in a private reusable cache. Fresh and mixed-artifact
focused native workflows passed, followed by all six required repair commands
and all 86 plugin tests. The report records the optional pre-existing test-target
Clippy warning. Exact final clean-HEAD evidence and canonical PR verification
precede handoff; controller integration and the unchanged full gate remain pending.

## Delivery-set acceptance and closure

- [x] All thirteen canonical slice PRs integrated in dev with exact-HEAD evidence.
- [x] PLG01-PLG12 native/SDK/representative PTY/resource acceptance recorded.
- [x] Independent Go client committed/pushed outside editor; both commits/API pinned.
- [ ] Ordered integrated controller gate passed on the exact clean dev candidate.
- [ ] All reports delivered and permitted Markdown closure committed/pushed.

Gate prerequisites verified: all thirteen canonical integrations, slice receipts
and delivered reports; the PLG01-PLG12 evidence matrix; clean pushed independent
client 3365b84649919492ade7e80c63139b459b5ad923 and its executable acceptance
runner; available Rust, Go, Bash and Spore tools. The frozen gate remains
unchanged. Its new integrated candidate and final Markdown closure belong to
the controller.

Catalog/distribution, stable API adoption, development-workflow package, main,
release/tag, mirrors, deployed guide and global installation are separate scopes.
