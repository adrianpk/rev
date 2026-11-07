# Slice 1: Context state and catalog

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-01-state`
PR: pending

## Purpose

Deliver the independently testable context/domain and atomic storage foundation
for PW01, PW09, PW15, PW17-PW19 and PW22. Public commands, live buffer integration
and startup presentation remain assigned to the later approved slices.

## Delivered Behavior

- Project, Workspace and root IDs are distinct opaque types. Session references
  carry a typed owner and checked positive revision. Display names preserve
  Unicode and case, trim their edges and remain independent from storage paths.
- Context descriptors and generation-checked prepared replacements are separate
  from the durable catalog and file snapshots. Dropping preparation publishes
  nothing and does not modify source files.
- Sessions preserve ordered root-relative and loose-file references, an active
  reference, logical Unicode-scalar cursor positions and optional source-anchored
  viewports. Overlap selects the most specific root; aliases cannot publish
  duplicate roots or canonical file entries. Position restoration clamps changed
  text through native source coordinates.
- The aggregate version-1 TOML catalog publishes all registrations and selected
  sessions through one atomic replacement. Expected catalog, metadata, membership
  and session revisions reject stale updates and overflow. A prepared conversion
  therefore publishes either the complete old Project or complete new Workspace.
- Missing reads create no directories, lock or state file. Unsupported, corrupt
  and oversized state remains intact. A damaged session with valid ownership
  headers remains associated with its registration and survives unrelated updates.
- Nonblocking ownership, descriptor-anchored state directories, private staging,
  source-root observations and final identity checks reject substitutions before
  replacement. Later sync/observation faults report publication with a durability
  warning, so callers do not retry creation under a new identity.

## Implementation Notes

T1.1 and T1.2 share the cohesive `context` module; T1.3 exercises its public
domain/store boundary with the existing native test framework and isolated
temporary files. The aggregate candidate is immutable after preparation and
retains its allocated IDs. Cooperating writers reject a stale complete catalog;
automatic rebasing of independent additions is not needed to prevent lost writes.
The runtime foundation is introduced in
`834ee5bdf30d16d327b2da21a45e37c1281d7243`.

The codec uses the existing TOML parser and `toml_edit` escaping. Its path shape
matches bookmarks: exactly one UTF-8 `path` or lossless native `path_bytes`
inside a location table. Project/Workspace `session_revision` selects one included
snapshot whose owner and membership revision must match. On disk, `active_file`
is an index into the ordered session entries; memory uses a validated file
reference. Malformed session payloads retain their TOML table for unrelated writes.

The store reuses the source-save `Output` fault boundary and private `Temporary`
files. A single `contexts.toml.lock` protects reread, conflict checks and atomic
replacement. State belongs to the existing XDG state root; no project-local
registry, dependency, worker thread, periodic writer or inactive setting is added.
No changelog entry is added for this internal foundation.

## Contracts Added or Changed

- Limits: 256 Projects, 128 Workspaces, 64 roots per Workspace, 512 files per
  session, 256 UTF-8 name bytes, 4096 native path bytes, 2 MiB encoded session
  and 8 MiB complete catalog. Counts and encoded sizes are checked together;
  overflow rejects the complete candidate rather than truncating saved state.
- Root membership revisions advance only for root/path membership changes.
  Renames preserve root IDs, membership revisions and selected sessions.
- New or relocated roots must remain available directories at publication.
  Previously retained unavailable roots and their file references remain stored.
- `Store::publish` returns a published catalog plus an optional durability
  warning. An error from `Output::replace` means replacement did not occur.
- The later live owner must enforce the approved single prepared context/job
  and cooperative 16-file admission pass. This slice defines the generation and
  admission contracts without claiming UI scheduling or restoration integration.

## Files of Interest

- `src/context.rs`, `src/context/types.rs`: identities, records, errors and limits.
- `src/context/catalog.rs`: admission, linked revisions and immutable preparation.
- `src/context/path.rs`, `src/context/position.rs`: native identity and coordinates.
- `src/context/codec.rs`, `src/context/store.rs`: bounded TOML and atomic storage.
- `tests/contexts.rs`, `tests/contexts/domain.rs`, `tests/contexts/storage.rs`:
  native domain, concurrency, substitution, capacity and publication-fault cases.

## Validation

All five implementation checks passed locally. The final clean branch HEAD
requires the same ordered commands through `looper validate`; its exact
candidate and receipt remain in controller-owned evidence outside the repository.

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The combined delivery-set gate belongs to the controller after integration.
No real user catalog, source repository, remote service or terminal fixture is
used by these storage/domain cases.

The native target passed 18 tests with no failures or ignored cases; its most
recent implementation run took 0.88 seconds after compilation. The documentation
check passed 596 commands, 289 configuration entries, 85 reachable pages and 578
local links/anchors. The existing vendored crossterm warning remains unchanged.

## Risks and Follow-ups

The filesystem fault cases establish both sides of the replacement boundary;
they do not simulate power loss. Actual durability depends on the filesystem.
Later slices must invoke storage from the approved owned I/O path, integrate
source decisions and live lifetimes, and measure the combined ordinary/capacity
profile. The store preserves usable metadata when a session payload is damaged;
invalid catalog or ownership headers reject the catalog as a whole.
