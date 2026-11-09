# Slice 4: Configuration and persistent concerns

Status: reviewing
Delivery set: rev-architecture-quality
Plan: [Architecture Quality Delivery](../../../plan/rev-architecture-quality.md)
Tracker: [Architecture Quality Tracker](../../../tracker/rev-architecture-quality.md)
Branch: `refactor/rev-architecture-quality-04-storage`
PR: #18 - https://forge.adrianpk.com/rev/rev/pulls/18
Implementation: 9417c9276874476408cc9d698b9287d903995ec7

## Purpose

Resolve the approved ownership, representation and source-bound findings in
configuration, file IO/watch, contexts, locations and bookmarks.

## Delivered Behavior

Configuration validates services, presentation, file policy and Org editing in
separate steps while retaining diagnostic order and partial-value semantics.
Validated numeric projection uses checked conversions.

Catalog decoding, owner/session validation and revision checks have explicit
steps. Publication retains descriptor-anchored private directories, the OS lock,
expected catalog bytes, repeated root identities and the existing replacement
commit point. Post-publication durability failures still return the published
catalog with a warning.

Bounded reload separates open-file identity from complete-read evidence. Private
workspace cleanup checks identity and enumerates every admitted output before
removing files. File-watch batches retain overflow, retirement and rescan evidence
before scheduling observations.

Context capture owns one captured-session candidate. Root observations, bounded
file capture, membership reconciliation, management prompts and departure
retirement have distinct steps. Bookmark observations release collection/cache
guards before returning shared row indexes.

## Implementation Notes

All 49 initial source-bound diagnostics in the 81-file selection are resolved
without changing the five-parameter, three-level or ten-branch limits. Helpers
remain under the frozen concern roots.

Configuration, data, state and cache ownership remain independent. Catalog/session
byte charging, file-admission steps, watch queue/batch limits, deadlines, private
permissions, compare-and-swap checks and uncertain-write evidence are retained.
No generic storage framework is added.

Metadata/error conversion borrows its inputs. The existing buffer, plugin and Org
callers only adapt those internal borrowing signatures. Shared watch, bookmark
and context handles use explicit Arc clones. The workspace departure continuation
retains owned checked-source authority with one local reasoned Clippy expectation;
rejection consumes the same authority as successful publication.

## Contracts Added or Changed

Public APIs, configuration and state formats, resource limits and runtime
behavior remain unchanged.

## Files of Interest

- `src/config/`: configuration validation and publication.
- `src/file/`: native file IO and watch ownership.
- `src/context/`, `src/session/contexts/`: durable records and native transitions.
- `src/location/`, `src/bookmark/`: checked locations and bookmark persistence.

## Validation

The four frozen commands passed during implementation checks. Final ordered
evidence must be recorded through Looper on the clean published PR-reference
commit before worker handoff.

1. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-quality.sh --scope src/config.rs --scope src/config --scope src/file.rs --scope src/file --scope src/context.rs --scope src/context --scope src/location.rs --scope src/location --scope src/bookmark.rs --scope src/bookmark --scope src/session/contexts.rs --scope src/session/contexts`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --test configuration --test files --test external_changes --test contexts --test bookmarks -- --test-threads=1`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet -p rev --example documentation-check`
4. `git diff --check`

The quality driver passes formatting, global first-party compiler/all-group
admission, scoped Clippy and source bounds for all 81 selected files. Inventory
covers 1,799 first-party Rust files, two packages and 73 Cargo roots; no Cargo root
is in this concern selection. The 1,169 selected-profile diagnostics outside these
owners remain explicitly deferred to their recorded slices.

All 378 native integration tests passed: configuration 162, files 75, external
changes 21, contexts 62 and bookmarks 58. Existing fixtures exercise invalid and
readonly sources, partial/uncertain writes, concurrent/stale publishers, private
path substitution, damaged sessions, Project/Workspace boundaries and watch
retirement. Test inputs, assertions, deadlines and resource limits are retained.
Documentation passes for 106 reachable pages and 743 local links/anchors.

An additional implementation check passed all 185 library tests, including the
private workspace-identity, bookmark-index and bounded-rescan regressions:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p rev --lib -- --test-threads=1`

These implementation checks precede the final PR-reference validation.

## Risks and Follow-ups

Later domain diagnostics remain assigned to their recorded slices. The
controller owns merges and the complete delivery-set gate.
