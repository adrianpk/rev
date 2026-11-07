# Slice 6: Startup screen and shared consumers

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-06-startup`
PR: pending

## Purpose

Deliver the native startup entrypoint and shared-consumer boundaries for
PW01-PW04 and PW20-PW22, including accepted P01/P03/P18. Close verified
Slice 5 PR #302 and activate only the approved Slice 6 branch. Integration,
successor startup and the combined gate remain controller-owned.

## Delivered Behavior

- No-target executable startup presents New Buffer, Open File, Open Dir,
  Open Project and Open Workspace. Explicit file/directory targets retain their
  native meaning and resolve relative paths from the captured invocation cwd.
  Embedded constructors remain passive and can opt in through `dashboard-open`.
- The startup screen retains the current live context, buffers and views. Its
  five actions dispatch existing native commands, typed pickers and checked
  departure. New Buffer starts fresh standalone work after source decisions;
  Open File visits within retained work. Selecting the active artifact resumes
  its live state without replaying a snapshot or publishing a checkpoint.
- Both editing profiles use native selection and command input. Native ranking,
  name completion and configured effective hints remain available. Raw source
  typing and paste cannot alter the screen or retained source. C-g cancels a
  nested picker/command to the screen and then returns to retained work. Explicit
  named commands continue through their ordinary owners.
- Successful context-close and active Workspace forget return to the screen over
  fresh standalone work. Canceled source decisions retain live work and publish
  no replacement. Pointer activation requires matching press/release on a live
  action; banner, spacing, modified gestures and stale resize geometry cannot
  activate actions. Mouse support remains optional.
- The supplied 100x13 Unicode/RGB artwork renders as native styled cells on black.
  A compact centered group retains the five action identities across resize.
  Narrow/short terminals use a plain rev label and keep selection visible;
  zero-sized presentation remains valid. NO_COLOR uses the plain label, marker
  and ordinary monochrome selection. Source palettes remain unchanged.
- Workspace AI defaults, LSP attachment/save boundaries and default source
  terminals use the selected file's most specific root, or its parent for a loose
  file. Explicit AI directory selection retains precedence; tree-focused
  terminals use the chosen root and terminal-here keeps native cwd behavior.
  No consumer receives a union of peer roots or automatic service activation.
  Git/shell source directories, Org source ownership and global bookmark, usage
  and kill-ring owners retain their existing authority.

## Implementation Notes

T6.1/T6.2 and the T6.3 native cases/guidance are delivered in
[`79fe9a6b`](https://forge.adrianpk.com/adrianpk/rev/commit/79fe9a6b29412c2ca714b6becf4d58bef70d6f2f).
Slice 5 closure and Slice 6 activation are recorded in `e98587ef0a7f`.

The optional startup presentation lives within Session rather than a synthetic
buffer or persistent artifact. Its input admission precedes retained Git and
terminal input, while pending selectors keep native prompt/completion ownership.
The same command-availability predicate filters effective key hints and raw
screen dispatch. Accepted named commands retain their native handler semantics.
Presentation skips source projection and terminal resize while it owns the
screen; returning restores those owners without replacing live work.

The immutable asset remains 35,650 bytes with SHA-256
`0b427846a5a84cb525c812f25e4e28c08a2117254e21e21cd617b04e26628d62`.
A one-time bounded parser admits only its RGB SGR/reset controls and 1,300
single-cell glyphs. Rows contain native text and RGB face runs; raw escape bytes
never enter painted row text. Shared layout computes painting and pointer hits.
The ordinary source palette is neither replaced nor recolored.

File/root consumer selection uses the existing canonical paths and Workspace
record. Context capture, publication, source decisions and owned job retirement
continue through the existing single IO worker and cooperative limits. The
startup screen does not create registrations, enable LSP or start a terminal.

## Contracts Added or Changed

- One public `dashboard-open` command and a read-only `dashboard_visible`
  observer; no new default bindings, settings, dependencies or durable schema.
- No-target executable startup, all five context actions, nested cancellation
  and screen return after accepted context closure.
- Passive native RGB artwork, black startup content and compact/NO_COLOR
  fallback without source palette or artwork changes.
- Source-specific Workspace roots for existing AI/LSP/terminal consumers;
  existing explicit activation and directory overrides retain precedence.
- CLI, command/binding inventories, context reference, introductory guidance and
  the coherent Unreleased outcome describe the delivered entrypoints.

## Files of Interest

- `src/session/startup.rs`, `startup/screen.rs`: executable entry, retained screen,
  native dispatch/cancellation and pointer admission.
- `src/display/startup.rs`, `startup/artwork.rs`, `style.rs`: bounded artwork,
  fitting/fallback and native black/RGB cells.
- `src/input/context.rs`, `dispatch.rs`, `hints.rs`, `modal.rs`: profile input
  ownership, read-only admission and configured effective hints.
- `src/session/contexts.rs`, `lsp.rs`, `ai/controls.rs`, `terminals/entry.rs`:
  accepted screen closure and source-specific consumer boundaries.
- `tests/contexts/startup.rs`, `tests/display/ranking.rs`: native actions,
  retention, rendering, isolated startup/NO_COLOR and command-origin cases.
- `docs/reference/contexts.md`, `cli.md`: delivered native entrypoint and consumer
  contracts linked from the User Guide and inventories.

## Validation

All five required implementation commands passed:

1. `cargo fmt --all -- --check`
2. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
3. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test commands --test modal --test display -- --test-threads=1`
4. `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
5. `git diff --check`

The required regression run passed 172 command, 62 context, 45 display and 120
modal cases: 399 total.

Seven new native startup cases cover both profiles, all actions, retained dirty
source stamps/views/history, source decisions, same-artifact selection, RGB cell
geometry, compact/zero-sized projections, one-row black content, pointer admission
and resize. Child
processes isolate executable targets and NO_COLOR from real user configuration.
A real owned terminal verifies the second Workspace root cwd, survives the
retained screen and is reaped after accepted context closure. Consumer cases
also cover overlapping/loose-file AI roots and disabled LSP retention. The new
command-origin display case checks effective configured hints and ranking.

Eleven configuration startup cases pass after entering New Buffer explicitly
where they exercise source editing; startup diagnostics, configuration lifetime,
publication and source-preservation assertions remain intact. Existing directory
command fixtures now assert the admitted fresh context and exact source bytes,
including repeat suppression. The catalog parity case compares every row and
status while allowing only process-wide generated buffer-name identities to
differ between independent sessions.

Focused successful commands:

- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts startup -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts artwork_and_resize -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test display startup_command_origin -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test configuration startup -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands path_ -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test commands palette::catalog_dispatch -- --test-threads=1`

Documentation checks passed 618 commands, 289 configuration entries, 86 reachable
pages and 588 local links/anchors.
The pre-existing vendored Crossterm unused-parentheses warning remains outside
this slice. Final clean-HEAD evidence is recorded after report/PR references are
committed and pushed. The delivery-set full gate remains controller-owned.

## Risks and Follow-ups

Representative real terminal journeys and the capacity/resource profile
remain in approved Slice 7. Final guidance reconciliation and the combined gate
remain in Slice 8 and controller ownership. This slice claims native and isolated
process evidence only; it does not claim full-gate or measured-capacity acceptance.
