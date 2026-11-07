# Slice 2: File and directory contexts

Status: reviewing
Delivery set: rev-projects-workspaces
Plan: [Projects and Workspaces Delivery](../../../plan/rev-projects-workspaces.md)
Tracker: [Projects and Workspaces Tracker](../../../tracker/rev-projects-workspaces.md)
Branch: `feat/rev-projects-workspaces-02-directories`
PR: pending

## Purpose

Deliver temporary standalone/directory contexts for PW01, PW04, PW06,
PW18-PW20 and PW22. Close the verified Slice 1 report/tracker and activate this
recorded slice without changing delivery identities or the controller gate.

## Delivered Behavior

- File visits retain standalone or explicitly chosen directory intent. Showing
  or hiding the tree does not register an artifact or change context kind.
  In-context file aliases reuse canonical source buffers; outside files remain
  loose files without replacing the directory root.
- `generate-new-buffer` adds an unnamed buffer in the active context.
  `open-directory` uses native path completion and accepts existing directories.
  Directory acceptance through `find-file` reaches the same checked operation.
  Direct CLI/library file and directory targets remain available.
- Opening a directory prepares its tree and fresh source owners before departure.
  Modified and unnamed buffers reuse save/discard/cancel, including ordinary
  write-file and overwrite decisions. Cancellation retains text, source stamps,
  undo history and windows. Approved saves retain their normal filesystem effects.
- `context-close` resolves the active sources and owned work, then continues in
  one empty standalone buffer with a hidden tree. It does not return application
  exit. Old source/result buffers leave the destination inventory.
- Context departure retains bookmark ownership, command/shell histories and
  kill-ring text, while retiring source-qualified views, requests and wake state.
  Runtime LSP enable/disable intent survives without granting new analysis.
  Existing terminal/Git confirmations and bounded cleanup precede activation.
- Temporary-directory use does not discover/register a Project or checkpoint a
  named session. Directory identity is rechecked after source decisions; failure
  retains the live context. Native root renames keep the descriptor, tree and file
  associations consistent without creating a new context generation.

## Implementation Notes

The session owns one typed context, one checked generation and at most one
prepared replacement. A replacement contains a fresh native session with no
source activation. A directory observation records its device/inode. The
existing quit continuation carries an explicit exit-or-context goal through
source saves, bookmark decisions, clock departure and terminal/Git cleanup.
Only its accepted context goal can activate the matching prepared generation.
Canceled or failed continuations reclaim the candidate.

Activation prepares bookmark closures on a bounded copy, retires old request
owners and moves the admitted application histories into the fresh session.
Each context gets a fresh notification-ready flag while inheriting native host
signal permission. Retired observations cannot cause an empty context to poll
continuously. Existing process owners perform cancellation and reaping; no
second process supervisor, state writer, new dependency or service is added.

T2.1/T2.2 share the native session and departure integration. T2.3 adds domain/UI
integration cases and updates the existing file/tree assertions for checked
directory replacement, retaining their source/history/geometry checks. Public
commands and the complete inventories are documented at delivery; Unreleased
contains one combined reader-facing outcome through changelog-normalize.
The implementation, native fixtures and public guidance are introduced in
`5032eb9f5161c718a7dd820eb374376263e54bb8`.

## Contracts Added or Changed

- `Session::context` and `context_generation` expose intent and live lifetime
  independently of the tree's presentation root.
- `Session::load` can admit a directory request that awaits source decisions;
  the live context changes only when that continuation succeeds.
- New commands share M-x and modal colon dispatch and remain callable from
  supported tree/terminal/Git focus. Source drafts keep their existing guards.
- Ordinary file visits retain their context and selected directory. Directory
  acceptance now replaces the context after decisions instead of merely
  retargeting a tree while exposing the old buffer inventory.
- Post-save cancellation does not roll back a completed source save. Failed
  destination revalidation never discards the still-usable current sources.
- Named Project/Workspace registration, session capture and startup presentation
  remain assigned to the later approved slices.

## Files of Interest

- `src/session/contexts.rs`, `src/session.rs`: context admission and activation.
- `src/session/quit.rs`, bookmark/clock/terminal/Git continuations: checked goals.
- `src/session/file.rs`, `src/session/path.rs`, `src/input/path.rs`: native targets.
- `src/buffer/lifecycle.rs`, `src/input/usage.rs`, `src/file/watch.rs`: retained
  application state and retired source lifetimes.
- `src/command.rs`: three implemented commands in the shared public catalog.
- `tests/contexts/editor.rs`, existing file/directory/tree cases: native evidence.
- Command/binding/prompt/tree/path references, editing how-to, User Guide and
  `CHANGELOG.md`: the delivered operations and cancellation behavior.

## Validation

All five implementation checks passed locally. The final clean PR head requires these
ordered commands through `looper validate`; exact candidate/receipt evidence
is controller-owned outside the repository.

- `cargo fmt --all -- --check`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo clippy --locked --lib --bin rev -- -D warnings`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked --test contexts --test files --test tree -- --test-threads=1`
- `env CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked --quiet --example documentation-check`
- `git diff --check`

The required native target passed 28 context, 75 file and 71 tree tests, with
zero failures or ignored cases. The latest implementation run took 5.18, 11.45
and 12.38 seconds respectively after compilation. Documentation passed 599
commands, 289 configuration entries, 85 reachable pages and 578 local links and
anchors. Cargo emits the unchanged vendored crossterm warning.

Native cases exercise both editing profiles, real isolated files, a retained
catalog, source decisions, canonical aliases, typed generations, root replacement
and renaming, bookmark/history/kill-ring preservation and a runtime analysis
choice. Owned real shell/PTY children establish cleanup and stale-source
rejection; their process identities are observed gone after accepted cleanup.
The combined delivery gate and representative whole-terminal acceptance remain
controller/later-slice work.

## Risks and Follow-ups

The startup screen is delivered in Slice 6; this slice's context-close result is
an immediately usable standalone editing buffer. Named persistence stays in
later slices. Native context/process cases establish these boundaries, not the
combined delivery gate or every service interaction. Required checks must remain
green on the final clean PR head before handoff.
