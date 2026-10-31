# Slice 3: Persistent collection

Status: awaiting integration
Delivery set: rev-bookmarks
Plan: [Native Bookmarks Delivery](../../../plan/rev-bookmarks.md)
Tracker: [Native Bookmarks Tracker](../../../tracker/rev-bookmarks.md)
Branch: `feat/rev-bookmarks-03-persistence`
PR: [#187](https://forge.adrianpk.com/adrianpk/rev/pulls/187)

## Purpose

Deliver BMK05 and the storage/limit aspects of BMK07 through the existing
configuration, file observation, staged publication and session owners.

## Delivered Behavior

First bookmark use lazily admits one strict version-1 TOML collection. Reads
allocate at most 16 MiB of store bytes. Names and context validate as a whole;
native paths round trip losslessly, including non-UTF-8 Unix paths. Loaded
records receive fresh session identities and remain unresolved until visited.
Malformed, unsupported and over-limit stores remain unchanged.

The store compares exact loaded identity and bytes under a nonblocking standard
library file lock. It stages, syncs and atomically replaces in the destination
directory through the existing concrete Output boundary. Publication failures
retain memory and old disk; failures after rename retain an unresolved outcome
that can only reconcile matching actual candidate bytes. Newly created data
directories and files use user-only permissions. Existing store modes are
preserved, and a read-only target cannot be bypassed through inode replacement.

`bookmark-save` checkpoints in every mode without saving source text.
`bookmark-reload` validates a complete fresh candidate, confirms replacement of
dirty memory and rechecks disk after confirmation. Cancel, malformed input and
concurrent changes retain both states. Default on-exit saves follow all source
save/discard decisions. On-change saves explicit mutations and source-save or
rename checkpoints; text edits only dirty evidence. Manual quit leaves pending
changes unsaved. Quit-time failure offers retry, discard or cancel while
retaining all live unsaved source and bookmark state.

## Implementation Notes

The canonical collection owns its dirty evidence and configured admission and
context limits. Text transactions mark evidence dirty without performing IO.
Executable startup supplies the existing XDG data destination; embedded sessions
perform user data IO only when their host configures an explicit destination.
Typed startup settings merge by leaf and reject invalid paths, modes or limits.

## Validation

Development check: `cargo test --locked --lib bookmark::store -- --nocapture`
passed five store tests, including two real writer processes, a separate held
interprocess lock, strict admission, native path round trips and staged versus
post-publication faults. Later development checks added permission and output
size coverage, twelve native persistence journeys and two configuration tests.
The documentation checker passed 311 commands, 185 configuration entries,
58 reachable pages and 331 local links/anchors.

The controller records final receipts outside the repository for this exact
ordered required list after the PR-reference follow-up commits:

1. `cargo fmt --all -- --check`
2. `cargo test --locked --test bookmarks --test files --test recovery --test configuration --test commands`
3. `git diff --check`

A separate controller batch checks `cargo test --locked --lib`,
`cargo clippy --locked --all-targets -- -D warnings` and
`cargo run --locked --quiet --example documentation-check` on that same final
candidate before the required batch. Handoff requires both batches to pass.

The first full library batch at the PR-reference candidate failed the unchanged
process-lifetime test's immediate descendant-stop assertion. The identical
89-test retry passed; no assertions or time/resource limits were changed.
The first required batch exposed a Slice 1 catalog test that still classified
save/reload as unimplemented. Its expectations now admit the delivered native
commands while retaining rejection of the later commands. Both controller
batches must be regenerated after this follow-up commit.

## Task Commits

T3.1: `88dc724e94768d26e5c7819bb9fedfe4ce47481b` introduces checked storage,
lazy admission, typed settings and real writer/failure tests. T3.2 supplies the
native policy continuations, commands, configuration coverage and reference
catalog/sample updates in `e642f451f6938722cc35b82c7cc6b5754f36997b`. Full task references are recorded in the tracker.

## Risks and Follow-ups

The file lock serializes cooperating instances; arbitrary external writers can
still race and are handled by checked observations and explicit uncertainty.
Source fragments stored as context may describe unsaved text. Checked reopening
never recreates discarded source content or claims historical identity from a
context match. Listing and marker projection remain assigned to their recorded
successor slices.
