# Slice 10: Package-owned persistent state

Status: reviewing
Delivery set: rev-plugin-system
Plan: [Plugin System Delivery](../../../plan/rev-plugin-system.md)
Tracker: [Plugin System Tracker](../../../tracker/rev-plugin-system.md)
Branch: `feat/rev-plugin-system-10-state`
PR: #10 - https://forge.adrianpk.com/rev/rev/pulls/10
Implementation: 0efa209d4e93e69cb6386f117f10f8cb708acaaa

## Purpose

Coordinate bounded, package-owned JSON state across editor instances through
version-checked publication without performing storage IO on native input.

## Delivered Behavior

Packages with attached native state roots can list/get/put/delete records in
only their own namespace. Native application startup uses the existing XDG root
resolver. Library hosts attach an explicit Roots snapshot before activation;
isolated new/empty hosts do not discover personal persistence. Root attachment
performs no IO, and configured roots survive native context replacement.

Read-only access to a missing store creates nothing. JSON null is present data,
distinct from absence. A put requires an observed version or explicit null for
creation; delete requires the existing version. Each mutation locks and re-reads
current state, checks only its target and preserves unrelated records. Concurrent
same-version writers cannot both succeed. Independent records rebase safely.
Deletion retains the aggregate high-water mark, so recreation cannot admit an
old version. Cursor continuations reject an intervening store revision.

Invalid schemas, unknown fields, invalid identities, malformed JSON, duplicate
or out-of-order records, oversized input and symlink substitutions refuse
without rewriting the store. Configuration and neighboring native stores remain
separate. Lock contention returns busy without blocking ordinary input.

## Implementation Notes

The domain owner under src/plugins/state/ follows the existing native macro
library's descriptor-anchored NOFOLLOW walk, nonblocking OS lock and checked
atomic-publication pattern. It reuses native File Output and Temporary staging;
no dependency or general persistence framework is introduced.

Schema-1 TOML contains schema_version, aggregate revision and ordered records
with concern, record, version and value_json. JSON is stored as one inert string
to retain null and every JSON value kind. Put versions come from the monotonic
aggregate revision; unrelated record versions are preserved. Counters stop
before signed 64-bit TOML exhaustion. Empty envelopes retain their revision.

The profile admits 256 records/package, 256 KiB per JSON value and 4 MiB encoded
store input/output. Admission checks both stored JSON bytes and re-serialized
wire size, including compact numeric spellings that grow when encoded. The
bounded writer refuses encoded aggregate overflow before staging replacement.
Operations may retain prior file bytes, decoded records and bounded replacement
output; they cache no records between calls. Existing process/request/frame
limits bound parallel work. These are admission limits, not RSS measurements.

Each supervisor executes typed state operations directly under existing request
permits, generation retirement and deadline checks. Native dispatch contains no
state IO path. Reads share existing locks without creating them; mutations use
an exclusive lock. Before commit, parent, lock, destination and staging identities
are checked again. Staging cleanup removes only the still-owned ordinary file
and preserves unknown substituted entries.

Replacement is the commit point. Earlier failures preserve old bytes. A later
sync/observation failure returns state_conflict with the committed candidate
version and explicit durability uncertainty. It retains the on-disk candidate
and suspends this connection's mutations until a valid read re-observes state
and synchronizes the directory. No implicit rollback or blind retry is performed.
Cancellation before replacement prevents publication; later cancellation does
not reverse an already committed explicit mutation.

## Contracts Added or Changed

- State capability is advertised only with an attached native root. The existing
  rev.plugins/0.1 schemas and editor/SDK release identities remain unchanged.
- Session::attach_plugin_state binds an absolute root without parent traversal
  before any package activation. Library hosts retain explicit root ownership.
- State methods derive the package namespace from the admitted connection;
  concern and record IDs never select filesystem paths or another package.
- The only store is plugins/packages/<publisher.package>/state.toml under the
  existing native state root, separate from configuration/data/cache/native stores.
- CAS conflicts return state_conflict, contention returns busy, and malformed or
  oversized data preserves existing bytes. Uncertain completion is explicit.
- Pagination binds concern/revision/position; mutations invalidate old cursors.
  Present null and absent records remain distinct public results.

## Files of Interest

- src/plugins/state.rs and state/{codec,storage}.rs: bounded JSON/TOML records,
  CAS, pagination, anchored storage and uncertainty handling.
- src/plugins/lifecycle/{worker,editor}.rs and protocol.rs: typed supervisor-only
  execution, existing permit/liveness ownership and conditional capability offer.
- src/plugins/lifecycle.rs, src/session/plugins.rs and startup.rs: frozen native
  roots and explicit embedding attachment before activation.
- tests/plugins/state.rs and the existing Rust peer: real RPC, concurrent hosts,
  cross-process lock contention, restart, namespaces and invalid-store fixtures.
- src/plugins/state/tests.rs: deterministic native publication-boundary faults,
  substitutions, cancellation, version exhaustion and retained candidate evidence.
- docs/reference/plugins.md and CHANGELOG.md: current state API and user behavior.

## Validation

All seven frozen ordered commands passed locally:

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo fmt --all -- --check` — passed.
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --workspace --lib --bins -- -D warnings` — passed.
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev-plugin-sdk` — passed.
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- state --test-threads=1` — passed.
5. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test macros -- --test-threads=1` — passed.
6. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check` — passed.
7. `git diff --check` — passed.

The SDK ran 12 tests, the state target ran seven RPC/instance cases and the
native macro target ran 110 tests. Documentation validation checked 684 commands,
299 configuration entries, 100 pages and 685 links.

Focused local checks after the final storage changes also passed:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test plugins -- protocol declarations lifecycle buffers transactions invocation keymaps recording events state --test-threads=1` — passed; 81 plugin regressions across all ten implemented slices.
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- plugins::state::tests --test-threads=1` — passed; three native publication-fault, substitution and version-exhaustion tests.
- `rustfmt --edition 2024 --check tests/plugins/peer.rs` — passed; standalone Rust peer formatting.

Final clean-HEAD controller validation remains a handoff prerequisite and is
recorded outside this report. No integrated full gate, main promotion or
publication was performed by this worker.

## Risks and Follow-ups

Versions coordinate cooperating publishers. Trusted programs retain ordinary
account permissions; the state API is not a filesystem sandbox. Uncertain
completion requires a fresh read before further mutations. Byte/count limits
and supervisor isolation do not guarantee per-host disk latency. Resource
measurement, broader fault acceptance and authoring guidance remain in their
recorded slices. The controller owns merges, the integrated gate and successors.
