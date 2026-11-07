# Slice 5: Artifact lifecycle and conversion

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-05-management`
PR: `#302`
Canonical PR: [#302](https://forge.adrianpk.com/adrianpk/rev/pulls/302)
Report introduction: [immutable report](https://forge.adrianpk.com/adrianpk/rev/src/commit/5813d3f9dfaba049598fac448f48defef434c1b9/ops/default/report/slices/rev-projects-workspaces/slice-5-management.md)

## Purpose

Deliver the lifecycle operations for PW07-PW08, PW12-PW14, PW17-PW19 and PW22.
Close verified Slice 4 PR #301 and activate only the approved Slice 5 branch.
Integration, successor startup and the combined gate remain controller-owned.

## Delivered Behavior

- Eight named commands select typed Project/Workspace identities before editing
  metadata: Project and Workspace rename/forget, Project root relocation,
  Workspace root-path relocation, Project conversion and independent duplication.
  Names remain trimmed, case-sensitive and unique within their artifact type.
- Rename changes only the display name and record revision. Forget removes only
  registration and its selected session. Active Project forget retains live dirty
  buffers, source positions and views as temporary directory work. Inactive edits
  retain the current context. Active Workspace forget resolves native departure
  and closes to standalone editing with no selected replacement root.
- Explicit relocation validates an existing replacement directory, preserves
  artifact/root identities and publishes matching root/session revisions.
  Root-relative files and the active reference follow the new path; loose files
  retain absolute references. Missing files remain diagnosed saved entries.
  Active relocation stages restored files after source decisions and before
  publication. Other artifacts keep their own paths; no source directory moves.
- Project conversion allocates fresh Workspace/root identities, remaps all saved
  root and active-file references, and consumes the Project in the same aggregate
  replacement that registers its Workspace/session. Active conversion retains
  dirty text, buffer IDs, history, cursor/view positions and editing focus.
  Inactive conversion uses its saved snapshot without switching current work.
- Duplication retains the source Project and its selected snapshot, initializes
  an independent Workspace session from the live or saved source and opens it.
  Native source/clock and owned-job decisions precede destination publication;
  cancellation creates nothing. Discard restores disk content, while approved
  saves retain their ordinary filesystem effects. Source files remain shared.
- Stale/busy publication, malformed input, damaged selected sessions, name/root
  conflicts and substituted replacement directories retain source work and
  reject publication. Rename/forget can manage a damaged session registration
  without interpreting or resetting its preserved payload. Published state is
  adopted before any later activation/durability diagnostic.

## Implementation Notes

T5.1/T5.2 and the T5.3 native cases/guidance are delivered in
[`71ec2ee2`](https://forge.adrianpk.com/adrianpk/rev/commit/71ec2ee2b563e1a547b3f5e02edf49567a617e3c).
Slice 4 closure and Slice 5 activation are recorded in `dc6d15020a13`.

The existing single owned IO job handles lifecycle reads, preparation, file
loading and publication. File capture/restoration retains the sixteen-file
cooperative pass limit. Native source continuations and terminal/Git ownership
remain responsible for departure; no new shutdown or transaction framework is
introduced.

A checked lifecycle plan retains the selected record, expected catalog,
preallocated destination identities, source generation/stamps and observed root
identities. Conversion/duplication candidates use the same session capture as
Project and Workspace checkpoints. Destination file admission completes before
publication. The aggregate store then checks expected state, references,
revisions, capacities and root observations under its existing nonblocking lock
and single atomic replacement. No independent delete follows conversion.

Active conversion adopts the new typed owner around the existing buffers.
Metadata-only rename and Project forget leave source owners in place. A forgotten
Workspace uses a rootless standalone tree; a later explicit standalone tree
request retains the ordinary browsing contract. Active relocation/duplication
use the existing prepared replacement and final native activation path.

## Contracts Added or Changed

- Eight canonical lifecycle commands; no new bindings, configuration keys,
  dependencies or persistent schema.
- Stable artifact/root identities for rename and relocation, fresh identities
  for conversion/duplication, independent snapshots and non-authoritative copied
  Project provenance.
- Checked root/session revisions and one conversion commit point, including
  explicit reconciliation of published state before later diagnostics.
- Native source-decision cancellation and passive metadata/source preservation.
- Command inventories, lifecycle reference, User Guide and the existing coherent
  Unreleased outcome describe only delivered commands.

## Files of Interest

- `src/session/contexts/management.rs`: typed selection, source departure,
  candidate preparation, publication and live owner reconciliation.
- `src/session/contexts/management/change.rs`: checked lifecycle catalog changes
  and fresh-root file/active-reference remapping.
- `src/session/contexts/checkpoint.rs`, `projects.rs`: shared bounded capture,
  staged file restoration and owned worker results.
- `src/session/quit.rs`, `src/tree/root.rs`: native departure continuation and
  rootless closure after active Workspace forget.
- `tests/contexts/management.rs`: ten native lifecycle cases with real isolated
  catalogs and source files.
- `docs/reference/contexts.md`: lifecycle command, session, relocation and
  cancellation contracts, linked from command inventories and the User Guide.

## Validation

All five required implementation commands passed:

1. `cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test files --test recovery -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
5. `git diff --check`

The required regression run passed 55 context, 75 file and 25 recovery cases:
155 total. Ten new native lifecycle cases cover both profiles, live dirty
conversion and retained focus, inactive conversion, independent duplication and
cancellation, active/inactive relocation, missing files, label/forget behavior,
busy/stale/corrupt publication, conflicts, root substitution during departure
and damaged-session management. Existing aggregate conversion fault cases retain
coverage before replacement and after directory-sync/final-observation warnings,
including restart state and source-byte preservation. This slice reuses that
store boundary rather than adding another fault mechanism.

The focused lifecycle command is
`env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts management -- --test-threads=1`.
Documentation checks passed 617 commands, 289 configuration entries, 86 reachable
pages and 587 local links/anchors. The pre-existing vendored Crossterm
unused-parentheses warning remains outside this slice.

Final clean-HEAD evidence is recorded after report/PR references are committed
and pushed. The delivery-set full gate remains controller-owned.

## Risks and Follow-ups

Startup-screen presentation after context closure remains in approved Slice 6;
this slice leaves no arbitrary directory root when forgetting an active
Workspace. Representative terminal journeys, capacity/resource measurements and
final guidance remain in their recorded successor slices. No combined-gate or
measured-capacity acceptance is claimed here.
