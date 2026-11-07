# Slice 4: Workspace roots and trees

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-04-workspaces`
PR: `#301`
Canonical PR: [#301](https://forge.adrianpk.com/adrianpk/rev/pulls/301)
Report introduction: [immutable report](https://forge.adrianpk.com/adrianpk/rev/src/commit/2b9b7916024932d59cf6463d2bbb85d3f29090f8/ops/default/report/slices/rev-projects-workspaces/slice-4-workspaces.md)

## Purpose

Deliver independent Workspace roots and basic sessions for PW09-PW11, PW15-PW16
and PW18-PW22. Close verified Slice 3 PR #300 and activate only the recorded Slice
4 branch; integration and the combined gate remain controller-owned.

## Delivered Behavior

- `workspace-create` opens a named empty Workspace with an add-root hint.
  `workspace-open` and `workspace-list` share typed name/path selection. Names
  remain unique within their own artifact type; same-ID reopening retains live
  work. Workspace sessions reuse checked native loading, cursor/view anchors,
  ordered file buffers, loose files and departure/clean-quit checkpoints.
- `workspace-add-directory` appends a checked directory.
  `workspace-add-project` adds only a selected Project's directory and optional
  provenance under a fresh Root ID. It leaves Project registration and sessions
  intact and imports no Project buffers. Canonical/physical duplicate roots are
  rejected; distinct ancestor/descendant roots remain valid.
- The native tree renders ordered peer roots, initially collapsed. Each route
  retains its root identity, loaded preorder and expansion. Decorated/minimal
  labels, Nerd Fonts, keyboard navigation, pointer activation and resized panels
  use the existing owners. Overlapping routes reuse canonical source buffers.
- Missing roots remain visible and retain their saved entries. Partial and
  all-unavailable Workspaces retain their actual identity and membership, with
  available files/loose files usable and no substituted cwd or replacement files.
  An empty Workspace also stays empty through tree focus and refresh.
- `workspace-remove-root` resolves only files that lose coverage. Overlap remaps
  through surviving roots without closing their buffers. Save/discard/cancel,
  affected clock departure, source validation and close admission precede the
  atomic membership/session publication. Loose files, unrelated dirty sources
  and terminal ownership survive; removing the last root remains valid. Source
  directories/files and Project registrations are retained.
- Busy/stale publication and canceled source decisions preserve live buffers and
  membership. The last-source scratch replacement and bookmark/location closure
  are prepared before publication. Published state is adopted before any later
  diagnostic. Guide/reference links and one coherent Unreleased outcome cover
  the delivered commands and session behavior.

## Implementation Notes

T4.1/T4.2/T4.3 are delivered in [`8a356b7d`](https://forge.adrianpk.com/adrianpk/rev/commit/8a356b7dd14b33ac7c6e2cb3760f3dde1725362a).
Slice 3 closure and Slice 4 activation are recorded in `f32904201a65`.

A typed persistent record unifies Project and Workspace session ownership while
keeping their records and sessions independent. The existing single owned IO job
and sixteen-file cooperative capture/restoration limit remain authoritative.
Membership advances the record, root and snapshot revisions in the aggregate
store's single replacement; unrelated Project records remain unchanged.

Tree entries and hits carry Workspace/root identity plus path and viewport
position. Refresh rebuilds only retained expansions in stored root order;
painting performs no directory scan. Membership changes retain unchanged root
preorders and retire removed routes. Empty/unavailable membership never asks the
ordinary tree owner to supply cwd.

Root removal adds a scoped goal to the native source continuation. Only covered
files losing coverage enter save/discard/cancel, and only an affected clock needs
its close/keep decision. A prepared batch close captures source stamps, checked
locations/bookmarks and any final-source replacement. Successful publication
then retires affected native buffer/view/analysis references without global
terminal/Git shutdown. Full context departure keeps its existing cleanup path.

Two existing pointer fixtures now reopen their source inside the fresh directory
context introduced by Slice 2. Their text, history, geometry, alias and disk-byte
assertions are retained. Modal tests use the existing C-g return from tree focus
before source colon entry; no tree binding is added.

## Contracts Added or Changed

- Six canonical Workspace commands, without new bindings or configuration keys.
- Workspace-owned root/session revisions, retained unavailable entries and
  most-specific membership remapping on root removal.
- Native peer-root tree rows/hits distinguish identical paths reached through
  different memberships and retain existing geometry/palette policies.
- Native scoped source closure stages fallible admission before durable effects.
- No dependencies, lockfiles, vendor code, state limits or controller gate change.

## Files of Interest

- `src/session/contexts/workspaces.rs`: typed Workspace/root operations and scoped
  membership preparation/publication.
- `src/session/contexts/record.rs`, `checkpoint.rs`, `projects.rs`: independent
  persistent owners and shared checked capture/restoration/IO.
- `src/tree/workspace.rs`, `entry.rs`, `hit.rs`: ordered lazy routes and retained
  pointer identity through overlap and unavailable roots.
- `src/buffer/lifecycle.rs`, `src/session/quit.rs`: staged batch closure and native
  scoped source/clock continuation.
- `tests/contexts/workspaces.rs`: native Workspace/session, tree/pointer,
  publication, clock, terminal and last-source cases.
- `docs/reference/contexts.md`, `file-tree.md`: delivered root/session policy and
  command behavior, linked from the User Guide and command inventories.

## Validation

All five required implementation commands passed:

1. `cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test tree --test pointer -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
5. `git diff --check`

The required run passed 45 context, 67 pointer and 71 tree cases: 183 total.
Eight new native Workspace cases cover both profiles, independent sessions,
Project-derived roots, overlapping canonical buffers and remapped references,
partial/all-unavailable membership, empty/last-root behavior, retained source
state on cancellation/busy/stale writes, actual pointer routing, decorated/minimal
and Nerd Fonts rows, clock ownership, live terminal retention and final-source
closure. The focused pointer regression passed all eight selected cases after the
fixture correction. Documentation checks passed 609 commands, 289 configuration
entries, 86 reachable pages and 586 local links/anchors. The pre-existing vendored
Crossterm unused-parentheses warning remains outside this slice.

Final clean-HEAD evidence is recorded after report/PR references are committed
and pushed. The delivery-set full gate remains controller-owned.

## Risks and Follow-ups

Lifecycle rename/forget/relocation/conversion, startup actions, representative
terminal acceptance and resource measurements remain in their existing successor
slices. This report does not claim combined gate or measured-capacity acceptance.
