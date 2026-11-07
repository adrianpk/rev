# Slice 3: Project registration and sessions

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-03-projects`
PR: pending

## Purpose

Deliver explicit one-root Projects and basic file sessions for PW05-PW06,
PW15-PW16, PW18-PW20 and PW22. Close verified Slice 2 PR #299 and activate only
this recorded slice; integration and the delivery-set gate remain controller-owned.

## Delivered Behavior

- `project-create` chooses an existing directory and proposes its basename.
  `project-promote` names the active temporary directory while preserving live
  buffer identities, unsaved text, cursor positions, views and active clocks.
  Unique names/canonical roots are checked before durable registration; an
  existing root offers its typed Project identity without replacing its session.
- `project-switch-project` and `project-list` share the native name/path picker.
  Selection carries a typed ID and expected record, checks state before opening
  and after source decisions, and treats the active ID as a no-op. Path/name/list
  cancellation retains the source. Ordinary directory/file use stays temporary.
- Sessions retain native file-buffer order, active file, logical cursor and
  source-anchored viewport positions, including absolute loose files. Unnamed
  text, terminal state and transient results are excluded. Restoration uses
  native ingress and clamps changed Unicode coordinates to valid graphemes.
  Unavailable files are skipped/reported and their saved entries retained;
  missing roots and damaged state preserve registrations and original bytes.
- Accepted departure and clean quit resolve native source/clock/terminal/Git
  ownership, then checkpoint before activation or Exit. Storage busy/conflict
  and root/source changes retain usable live work. Confirmed ordinary saves
  retain their effects. Publication is distinguished from later durability or
  activation failure; published identities are adopted before reporting it.
- Executable startup supplies the passive XDG state path. Embedded sessions opt
  into a host-owned `contexts.toml` path explicitly. Empty listing writes nothing.
  Existing runtime LSP intent and per-file service authority remain effective.

## Implementation Notes

T3.1/T3.2/T3.3 are delivered in [`d099a88b`](https://forge.adrianpk.com/adrianpk/rev/commit/d099a88b3562df9ce4ef3ad611ecf23f7a869782).
Slice 2 closure and Slice 3 activation are recorded in `5099980b3a9a`.

One session-owned existing `org::Job` slot handles catalog reads, native file
loading and publication. Restoration and source-coordinate capture admit at most
16 files per cooperative pass. Generation tokens, record comparisons and native
source stamps qualify results. Input stays with the state operation during final
confirmation/publication; pre-publication capture and reads can be canceled.
Completed operations retain no periodic Project poll. The aggregate store's
nonblocking lock, revision checks and single replacement remain authoritative.

Live source-position cache entries are bounded by live file buffers and removed
on buffer close. Prepared replacements retain their own buffers/views, and source
owners retire through the existing checked departure pipeline. Registered roots
cannot be renamed through the ordinary tree command; explicit metadata relocation
belongs to the recorded lifecycle slice.

## Contracts Added or Changed

- Four canonical commands are available through M-x/modal colon without new
  default bindings or startup restoration settings.
- `Session::set_context_storage` gives embedded hosts an explicit state boundary;
  ordinary library constructors discover no user catalog.
- Project sessions checkpoint on accepted departure and clean quit, with one
  persistent owner per selected Project. Same-ID selection does not checkpoint.
- No dependency, lockfile, vendor, state-limit or controller-gate changes.

## Files of Interest

- `src/session/contexts/projects.rs`: native prompts, typed selection, staged
  loading, final revision confirmation and owned IO scheduling.
- `src/session/contexts/checkpoint.rs`: bounded source capture, aggregate candidate
  publication and commit-point reconciliation.
- `src/session/contexts/positions.rs`: source/view anchors and Unicode restoration.
- `src/session/quit.rs`, `src/session/contexts.rs`, `src/session/events.rs`: shared
  source-departure ownership and activation.
- `tests/contexts/projects.rs`, `tests/contexts/editor.rs`: native Project journeys
  and temporary/persistent terminal cleanup.
- `docs/reference/contexts.md`: delivered commands, session policy and failure
  behavior; guide/reference links and the coherent Unreleased outcome are updated.

## Validation

All five required implementation commands passed:

1. `cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test recovery --test integrated_terminal -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
5. `git diff --check`

The required native run passed 37 context, 49 integrated-terminal and 25 recovery
cases. Nine new Project cases cover both entry profiles, promotion with dirty
sources, same-ID selection, ordered/loose-file restoration, non-UTF-8 roots,
missing/binary/changed Unicode sources, sixteen-file batching, nonzero source
viewports, canceled capture, stale picker/departure revisions, busy publication,
failed initial registration, clocks and clean quit. The terminal case covers both
temporary and persistent contexts with real owned children. Documentation checks
passed 603 commands, 289 configuration entries, 86 reachable pages and 582 links.
The existing vendored Crossterm unused-parentheses warning remains outside scope.

The controller's final clean-HEAD receipt is recorded after PR-reference metadata
is committed and pushed. No delivery-set full gate is run by this worker.

## Risks and Follow-ups

Workspace operations, lifecycle management, startup actions and combined resource
acceptance remain in their existing approved successor slices. This slice does
not claim the delivery-set gate or measured capacity acceptance.
